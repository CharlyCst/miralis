use core::cmp::{min, PartialEq};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use spin::Mutex;

use crate::arch::{mie, Arch, Architecture, Csr};
use crate::config::PLATFORM_NB_HARTS;
use crate::device::{DeviceAccess, Width};
use crate::driver::clint::{
    ClintDriver, MSIP_OFFSET, MSIP_WIDTH, MTIMECMP_OFFSET, MTIMECMP_WIDTH, MTIME_OFFSET,
};
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::virt::VirtContext;
use crate::{debug, logger};

// ————————————————————————————— Virtual CLINT —————————————————————————————— //

// We use this structure to avoid false sharing in the virtual clint
// The typical size of a cache line is 64 bytes
#[repr(C, align(64))]
#[derive(Debug)]
struct TimestampEntry {
    counter_firmware: AtomicUsize,
    counter_payload: AtomicUsize,
    _padding: [u8; 56],
}

impl TimestampEntry {
    const fn max_value() -> Self {
        TimestampEntry {
            counter_firmware: AtomicUsize::new(usize::MAX),
            counter_payload: AtomicUsize::new(usize::MAX),
            _padding: [0; 56],
        }
    }
}

pub const CLINT_SIZE: usize = 0x10000;

#[derive(PartialEq)]
enum ClintTimer {
    Payload,
    Firmware,
}

/// Represents a virtual CLINT (Core Local Interruptor) device
#[derive(Debug)]
pub struct VirtClint {
    /// A driver for the physical CLINT
    driver: &'static Mutex<ClintDriver>,
    /// Virtual Machine Software Interrupt (MSI) map
    vmsi: [AtomicBool; PLATFORM_NB_HARTS],
    /// Policy Machine Software Interrupt (MSI) map
    policy_msi: [AtomicBool; PLATFORM_NB_HARTS],
    /// Buffer to remember when timer interrupt should be injected in the firmware and the payload
    next_timestamps: [TimestampEntry; PLATFORM_NB_HARTS],
}

impl DeviceAccess for VirtClint {
    fn read_device(
        &self,
        offset: usize,
        r_width: Width,
        _ctx: &mut VirtContext,
    ) -> Result<usize, &'static str> {
        self.read_clint(offset, r_width)
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        self.write_clint_from_firmwarwe(offset, w_width, value, ctx)
    }
}

impl VirtClint {
    /// Creates a new virtual CLINT device backed by a physical CLINT.
    pub const fn new(driver: &'static Mutex<ClintDriver>) -> Self {
        Self {
            driver,
            vmsi: [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS],
            policy_msi: [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS],
            next_timestamps: [const { TimestampEntry::max_value() }; PLATFORM_NB_HARTS],
        }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), &'static str> {
        if offset >= CLINT_SIZE {
            log::warn!("Invalid CLINT offset: 0x{:x}", offset);
            Err("Invalid CLINT offset")
        } else {
            Ok(())
        }
    }

    pub fn handle_machine_timer_interrupt(&self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        let mut clint = Plat::get_clint().lock();

        let current_timestamp: usize = clint.read_mtimecmp(mctx.hw.hart).unwrap();

        if current_timestamp
            >= self.next_timestamps[mctx.hw.hart]
                .counter_firmware
                .load(Ordering::SeqCst)
        {
            self.next_timestamps[mctx.hw.hart]
                .counter_firmware
                .store(usize::MAX, Ordering::SeqCst);
            // Inject a virtual interrupt to the firmware
            ctx.csr.mip |= mie::MTIE_FILTER;
        }

        if current_timestamp
            >= self.next_timestamps[mctx.hw.hart]
                .counter_payload
                .load(Ordering::SeqCst)
        {
            self.next_timestamps[mctx.hw.hart]
                .counter_payload
                .store(usize::MAX, Ordering::SeqCst);
            // Inject a virtual interrupt to the payload
            ctx.csr.mip |= mie::STIE_FILTER;
            self.propagate_payload_interupt_physically(ctx);
        }

        let new_timestamp_firmware = self.next_timestamps[mctx.hw.hart]
            .counter_firmware
            .load(Ordering::SeqCst);
        let new_timestamp_payload = self.next_timestamps[mctx.hw.hart]
            .counter_payload
            .load(Ordering::SeqCst);

        // Write the minimum of the two back
        clint
            .write_mtimecmp(
                mctx.hw.hart,
                min(new_timestamp_firmware, new_timestamp_payload),
            )
            .expect("Failed to write mtimecmp");
    }

    pub fn read_clint(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        logger::trace!("Read from CLINT at offset 0x{:x}", offset);
        self.validate_offset(offset)?;
        let driver = self.driver.lock();

        match (offset, r_width) {
            (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                driver.read_msip(hart)
            }
            (o, Width::Byte8) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                driver.read_mtimecmp(hart)
            }
            (o, Width::Byte8) if o == MTIME_OFFSET => Ok(driver.read_mtime()),
            // We also handle the case of 4 bytes reads to mtime
            (o, Width::Byte4) if o == MTIME_OFFSET => Ok(driver.read_mtime() & 0xffffffff),
            (o, Width::Byte4) if o == MTIME_OFFSET + 4 => Ok(driver.read_mtime() >> 32),
            _ => {
                log::warn!(
                    "Invalid clint read: offset is 0x{:x}, width is {}",
                    offset,
                    r_width.to_bytes()
                );
                Err("Invalid CLINT offset")
            }
        }
    }

    pub fn write_clint_from_payload(
        &self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
        value: usize,
    ) {
        self.write_clint(
            MTIMECMP_OFFSET + 8 * mctx.hw.hart,
            Width::Byte8,
            value,
            ctx,
            ClintTimer::Payload,
        )
        .unwrap();
    }

    pub fn write_clint_from_firmwarwe(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        self.write_clint(offset, w_width, value, ctx, ClintTimer::Firmware)
    }

    fn write_clint(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
        origin: ClintTimer,
    ) -> Result<(), &'static str> {
        logger::trace!(
            "Write to CLINT at offset 0x{:x} with a value 0x{:x}",
            offset,
            value
        );
        self.validate_offset(offset)?;
        let mut driver = self.driver.lock();

        match (offset, w_width) {
            (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                if hart >= PLATFORM_NB_HARTS {
                    return Err("Invalid hart when writting MSIP");
                }
                match value & 0b1 {
                    0 => {
                        // Clear pending MSI
                        self.vmsi[hart].store(false, Ordering::SeqCst);
                        if hart == ctx.hart_id {
                            // On the current hart clear mip.MSIE
                            ctx.csr.mip &= !mie::MSIE_FILTER;
                            Ok(())
                        } else {
                            // On remote hart send a physical MSI
                            driver.write_msip(hart, 1)
                        }
                    }
                    1 => {
                        // Set pending MSI
                        self.vmsi[hart].store(true, Ordering::SeqCst);
                        if hart == ctx.hart_id {
                            // On the current hart set mip.MSIE
                            ctx.csr.mip |= mie::MSIE_FILTER;
                            Ok(())
                        } else {
                            // On remote hart send a physical MSI
                            driver.write_msip(hart, 1)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (o, _) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let mtime = driver.read_mtime();
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                if hart >= PLATFORM_NB_HARTS {
                    return Err("Invalid hart when writting MSIP");
                }
                if hart != ctx.hart_id {
                    todo!("Setting mtime for another hart is not yet supported");
                }

                // When we receive a timer, we clear the corresponding interrupt bit
                if origin == ClintTimer::Payload {
                    self.next_timestamps[ctx.hart_id]
                        .counter_payload
                        .store(value, Ordering::SeqCst);
                    ctx.csr.mip &= !mie::STIE_FILTER;
                } else {
                    self.next_timestamps[ctx.hart_id]
                        .counter_firmware
                        .store(value, Ordering::SeqCst);
                    ctx.csr.mip &= !mie::MTIE_FILTER;
                }

                let is_interrupt_ready: bool = mtime >= value;

                if is_interrupt_ready {
                    ctx.csr.mip |= if origin == ClintTimer::Payload {
                        mie::STIE_FILTER
                    } else {
                        mie::MTIE_FILTER
                    };
                } else {
                    let mtimecmp_firmware = self.next_timestamps[ctx.hart_id]
                        .counter_firmware
                        .load(Ordering::SeqCst);
                    let mtimecmp_payload = self.next_timestamps[ctx.hart_id]
                        .counter_payload
                        .load(Ordering::SeqCst);

                    driver.write_mtimecmp(hart, min(mtimecmp_firmware, mtimecmp_payload))?;
                }

                if origin == ClintTimer::Payload {
                    self.propagate_payload_interupt_physically(ctx);
                }

                Ok(())
            }
            (o, _) if o == MTIME_OFFSET => {
                // We don't do it for now so we might loose interrupts.
                debug::warn_once!(
                    "Write to mtime not yet fully supported (might cause interrupt loss)"
                );
                driver.write_mtime(value);
                Ok(())
            }
            _ => {
                log::warn!(
                    "Invalid CLINT offset: 0x{:x} or width: {}",
                    offset,
                    w_width.to_bytes()
                );
                Err("Invalid CLINT offset")
            }
        }
    }

    /// Return true if a vMSI is pending for the given hart
    pub fn get_vmsi(&self, hart: usize) -> bool {
        assert!(
            hart < PLATFORM_NB_HARTS,
            "Invalid hart ID when clearing vMSI"
        );
        self.vmsi[hart].load(Ordering::SeqCst)
    }

    /// Mark the policy MSI as pending for each hart.
    pub fn set_all_policy_msi(&self) {
        for hart_idx in 0..PLATFORM_NB_HARTS {
            self.policy_msi[hart_idx].store(true, Ordering::SeqCst);
        }
    }

    /// Get the policy MSI pending status for the given hart.
    pub fn get_policy_msi(&self, hart: usize) -> bool {
        assert!(
            hart < PLATFORM_NB_HARTS,
            "Invalid hart ID when clearing vMSI"
        );
        self.policy_msi[hart].load(Ordering::SeqCst)
    }

    /// Clear the policy MSI pending status for the given hart.
    pub fn clear_policy_msi(&self, hart: usize) {
        assert!(
            hart < PLATFORM_NB_HARTS,
            "Invalid hart ID when clearing vMSI"
        );
        self.policy_msi[hart].store(false, Ordering::SeqCst)
    }

    fn propagate_payload_interupt_physically(&self, ctx: &mut VirtContext) {
        // In this case, we explicitly need to update the register physically
        // We are coming from the payload, and we jump back to the payload, therefore there is no mode transition
        // Currently, Miralis updates the registers physically only during mode transitions.
        unsafe {
            Arch::write_csr(Csr::Mip, ctx.csr.mip);
        }
    }
}
