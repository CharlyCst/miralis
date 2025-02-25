use core::cmp::min;
use core::mem::size_of;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::arch::{mie, Arch, Architecture, Csr, Mode};
use crate::config::PLATFORM_NB_HARTS;
use crate::device::{DeviceAccess, Width};
use crate::driver::clint::{
    clint_driver, MSIP_OFFSET, MSIP_WIDTH, MTIMECMP_OFFSET, MTIMECMP_WIDTH, MTIME_OFFSET,
};
use crate::host::MiralisContext;
use crate::virt::VirtContext;
use crate::{debug, logger};

// ————————————————————————————— Virtual CLINT —————————————————————————————— //

/// Total size of the CLINT, in bytes.
pub const CLINT_SIZE: usize = 0x10000;

/// Padding size in the [TimestampEntry] struct, in bytes.
///
/// This constant assumes the typical cache line size of 64 bytes.
///
/// NOTE: remember to update the factor in front of [size_of] to the number of timestamp fields in
/// [TimestampEntry].
const TIMESTAMP_PADDING_SIZE: usize = 64 - 2 * size_of::<AtomicUsize>();

/// A collection of timestamps entries for a given hart.
///
/// This struct is used to multiplex a single physical counters among multiple contexts, such as
/// the virtual firmware, the payload (when offloading supervisor timers, e.g. emulating Sstc), or
/// Miralis itself.
#[repr(C, align(64))]
#[derive(Debug)]
struct TimestampEntry {
    deadline_firmware: AtomicUsize,
    deadline_payload: AtomicUsize,
    _padding: [u8; TIMESTAMP_PADDING_SIZE],
}

impl TimestampEntry {
    const fn max_value() -> Self {
        TimestampEntry {
            deadline_firmware: AtomicUsize::new(usize::MAX),
            deadline_payload: AtomicUsize::new(usize::MAX),
            _padding: [0; TIMESTAMP_PADDING_SIZE],
        }
    }
}

/// Represents a virtual CLINT (Core Local Interruptor) device
#[derive(Debug)]
pub struct VirtClint {
    /// Virtual Machine Software Interrupt (MSI) map
    vmsi: [AtomicBool; PLATFORM_NB_HARTS],
    /// Policy Machine Software Interrupt (MSI) map
    policy_msi: [AtomicBool; PLATFORM_NB_HARTS],
    /// A per-hart struct to multiplex timer interrupts. Stores the next timestamp for each
    /// contexts.
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
        // For now devices are only emulated for the virtual firmware, hence a firmware access.
        self.write_clint(offset, w_width, value, ctx)
    }
}

impl VirtClint {
    /// Creates a new virtual CLINT device backed by a physical CLINT.
    pub const fn default() -> Self {
        Self {
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

    /// React to a physical timer interrupt.
    ///
    /// The virtual CLINT can multiplexe the physical timer for multiple sources. This function
    /// handles the multiplexing by finding the timer origin and configuring the next deadline.
    pub fn handle_machine_timer_interrupt(&self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        let timestamps = &self.next_timestamps[mctx.hw.hart];

        // Read current timestamp and drop the lock right away
        let current_timestamp: usize = clint_driver::read_mtimecmp(mctx.hw.hart).unwrap();

        // If the timer is for the firmware
        if current_timestamp >= timestamps.deadline_firmware.load(Ordering::SeqCst) {
            timestamps
                .deadline_firmware
                .store(usize::MAX, Ordering::SeqCst);
            // Inject a virtual interrupt to the firmware
            ctx.csr.mip |= mie::MTIE_FILTER;
        }

        // If the timer is for the payload
        if current_timestamp >= timestamps.deadline_payload.load(Ordering::SeqCst) {
            timestamps
                .deadline_payload
                .store(usize::MAX, Ordering::SeqCst);
            // Inject a virtual interrupt to the payload
            ctx.csr.mip |= mie::STIE_FILTER;

            // If the payload is running, then we need to inject the timer in the physical `mip`
            // register. The hardware will triger an interrupt right after Miralis jumps to the
            // payload.
            if ctx.mode != Mode::M {
                unsafe { self.set_physical_stip() };
            }
        }

        self.update_deadline(mctx.hw.hart);
    }

    /// Update the physical deadline for the given hart.
    ///
    /// The next deadline will be set as the minimum of the virtual deadlines of each harts.
    fn update_deadline(&self, hart_id: usize) {
        // Collect the deadlines
        let timestamps = &self.next_timestamps[hart_id];
        let firmware_deadline = timestamps.deadline_firmware.load(Ordering::SeqCst);
        let payload_deadline = timestamps.deadline_payload.load(Ordering::SeqCst);
        let next_deadline = min(firmware_deadline, payload_deadline);

        // Write the next deadline back
        clint_driver::write_mtimecmp(hart_id, next_deadline).expect("Failed to write mtimecmp");
    }

    pub fn read_clint(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        logger::trace!("Read from CLINT at offset 0x{:x}", offset);
        self.validate_offset(offset)?;

        match (offset, r_width) {
            (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                clint_driver::read_msip(hart)
            }
            (o, Width::Byte8) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                clint_driver::read_mtimecmp(hart)
            }
            (o, Width::Byte8) if o == MTIME_OFFSET => Ok(clint_driver::read_mtime()),
            // We also handle the case of 4 bytes reads to mtime
            (o, Width::Byte4) if o == MTIME_OFFSET => Ok(clint_driver::read_mtime() & 0xffffffff),
            (o, Width::Byte4) if o == MTIME_OFFSET + 4 => Ok(clint_driver::read_mtime() >> 32),
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

    /// Write the next payload deadline
    pub fn set_payload_deadline(
        &self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
        value: usize,
    ) {
        let mtime = clint_driver::read_mtime();
        if mtime >= value {
            // If the deadline already passed we install the timer interrupt
            ctx.csr.mip |= mie::STIE_FILTER;
            // Here we assume that we will jump back into the payload, that is we upload the
            // payload deadline without a world switch.
            unsafe { self.set_physical_stip() };
            // Then we reset the virtual deadline
            self.next_timestamps[mctx.hw.hart]
                .deadline_payload
                .store(usize::MAX, Ordering::SeqCst);
        } else {
            // The deadline is not yet passed, so we remove the pending interrupt
            ctx.csr.mip &= !mie::STIE_FILTER;
            // Here we assume that we will jump back into the payload, that is we upload the
            // payload deadline without a world switch.
            unsafe { self.clear_physical_stip() };
            // Then we set the virtual deadline
            self.next_timestamps[mctx.hw.hart]
                .deadline_payload
                .store(value, Ordering::SeqCst);
        }
        self.update_deadline(mctx.hw.hart);
    }

    /// Write to the virtual CLINT
    fn write_clint(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        logger::trace!(
            "Write to CLINT at offset 0x{:x} with a value 0x{:x}",
            offset,
            value
        );
        self.validate_offset(offset)?;

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
                            clint_driver::write_msip(hart, 1)
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
                            clint_driver::write_msip(hart, 1)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (o, _) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let mtime = clint_driver::read_mtime();
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                if hart >= PLATFORM_NB_HARTS {
                    return Err("Invalid hart when writting MSIP");
                }
                if hart != ctx.hart_id {
                    todo!("Setting mtime for another hart is not yet supported");
                }

                // Update the virtual `mip` according to the relative ordering of mtime and
                // mtimecmp.
                if mtime >= value {
                    ctx.csr.mip |= mie::MTIE_FILTER;
                } else {
                    // Register a timer to trigger the virtual interrupt once appropriate
                    self.next_timestamps[ctx.hart_id]
                        .deadline_firmware
                        .store(value, Ordering::SeqCst);
                    self.update_deadline(hart);
                    ctx.csr.mip &= !mie::MTIE_FILTER;
                }

                Ok(())
            }
            (o, _) if o == MTIME_OFFSET => {
                // TODO: when updating mtime we should check on which core the timer should fire.
                // We don't do it for now so we might loose interrupts.
                debug::warn_once!(
                    "Write to mtime not yet fully supported (might cause interrupt loss)"
                );
                clint_driver::write_mtime(value);
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

    /// Mark the policy MSI as pending for the harts given by the mask (bit 0 represents hart 0, bit 1 hart 1,....)
    pub fn set_all_policy_msi(&self, mask: usize) {
        for hart_idx in 0..PLATFORM_NB_HARTS {
            if mask & (1 << hart_idx) != 0 {
                self.policy_msi[hart_idx].store(true, Ordering::SeqCst);
            }
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

    /// Set the `mip.STIP` bit within the physical registers.
    ///
    /// This function will write to the physical `mip` to set the STIP bit. Calling this function
    /// is required when the physical registers are not updated automatically, for instance when
    /// handling timer interrupt from the payload and returning to the payload without a world
    /// switch.
    unsafe fn set_physical_stip(&self) {
        Arch::set_csr_bits(Csr::Mip, mie::STIE_FILTER);
    }

    /// Clear the `mip.STIP` bit within the physical registers.
    ///
    /// This function will write to the physical `mip` to clear the STIP bit. Calling this function
    /// is required when the physical registers are not updated automatically, for instance when
    /// handling timer interrupt from the payload and returning to the payload without a world
    /// switch.
    unsafe fn clear_physical_stip(&self) {
        Arch::clear_csr_bits(Csr::Mip, mie::STIE_FILTER);
    }
}
