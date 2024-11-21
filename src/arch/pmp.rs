//! Physical Memory Protection
//!
//! This module handles exposes structure to store and manipulate PMPs, including checking for
//! addresses matching PMP ranges.

use core::fmt;
use core::fmt::Formatter;

use super::Architecture;
use crate::arch::pmp::pmpcfg::{INACTIVE, NAPOT, TOR};
use crate::arch::pmp::pmplayout::{
    ALL_CATCH_OFFSET, DEVICES_OFFSET, INACTIVE_ENTRY_OFFSET, MIRALIS_OFFSET, MIRALIS_TOTAL_PMP,
    POLICY_OFFSET, POLICY_SIZE, VIRTUAL_PMP_OFFSET,
};
use crate::arch::Arch;
use crate::config;
use crate::platform::{Plat, Platform};

// ——————————————————————————— PMP Configuration ———————————————————————————— //

pub mod pmplayout {
    use crate::policy::{Policy, PolicyModule};

    /// First entry used to catch all pmp entries
    pub const ALL_CATCH_SIZE: usize = 1;
    pub const ALL_CATCH_OFFSET: usize = 0;

    // PMP entry used to protect Miralis
    pub const MIRALIS_SIZE: usize = 1;
    pub const MIRALIS_OFFSET: usize = ALL_CATCH_SIZE;

    /// PMP entries used to protect the devices
    pub const DEVICES_SIZE: usize = 2;
    pub const DEVICES_OFFSET: usize = MIRALIS_OFFSET + MIRALIS_SIZE;

    /// PMP entries used by the policy
    pub const POLICY_SIZE: usize = Policy::NUMBER_PMPS;
    pub const POLICY_OFFSET: usize = DEVICES_OFFSET + DEVICES_SIZE;

    /// Last PMP entry used in to emulate TOR correctly in the firmware
    pub const INACTIVE_ENTRY_SIZE: usize = 1;
    pub const INACTIVE_ENTRY_OFFSET: usize = POLICY_OFFSET + POLICY_SIZE;

    /// Offset at which the virtual PMPs can start
    pub const VIRTUAL_PMP_OFFSET: usize = INACTIVE_ENTRY_OFFSET + INACTIVE_ENTRY_SIZE;
    /// At the very end, there is a last PMP entry
    pub const MIRALIS_TOTAL_PMP: usize = VIRTUAL_PMP_OFFSET + 1;
}

/// PMP Configuration
///
/// Hold constants for the pmpcfg CSRs.
pub mod pmpcfg {
    /// Read access
    pub const R: u8 = 0b00000001;
    /// Write access
    pub const W: u8 = 0b00000010;
    /// Execute access
    pub const X: u8 = 0b00000100;
    /// Read, Write, and Execute access
    pub const RWX: u8 = R | W | X;
    /// No permissions
    pub const NO_PERMISSIONS: u8 = 0x0;

    /// Address is Top Of Range (TOP)
    pub const TOR: u8 = 0b00001000;
    /// Naturally aligned four-byte region
    pub const NA4: u8 = 0b00010000;
    /// Naturally aligned power of two
    pub const NAPOT: u8 = 0b00011000;
    /// Bit mask for the A attributes of pmpcfg
    pub const A_MASK: u8 = 0b00011000;

    /// Locked
    pub const L: u8 = 0b10000000;

    /// An inactive entry, ignored by the matching rules
    pub const INACTIVE: u8 = 0b00000000;

    /// Valid bits for pmpcfg
    pub const VALID_BITS: u8 = RWX | NAPOT | L;
}

// —————————————————————————————— PMP Address ——————————————————————————————— //

/// Build a valid NAPOT pmpaddr value from a provided start and size.
///
/// This function checks for a minimum size of 8 and for proper alignment. If the requirements are
/// not satisfied None is returned instead.
pub const fn build_napot(start: usize, size: usize) -> Option<usize> {
    if start == 0 && size == usize::MAX {
        return Some(usize::MAX);
    }

    if size < 8 {
        // Minimum NAPOT size is 8
        return None;
    }
    if size & (size - 1) != 0 {
        // Size is not a power of 2
        return None;
    }
    if start & (size - 1) != 0 {
        // Start does not have an alignment of at least 'size'.
        return None;
    }

    Some((start >> 2) | ((size - 1) >> 3))
}

/// Build a valid TOR pmpaddr value from a provided until memory location.
pub const fn build_tor(until: usize) -> usize {
    until >> 2
}

// ——————————————————————————————— PMP Group ———————————————————————————————— //

pub struct PmpGroup {
    pmpaddr: [usize; 64],
    pmpcfg: [usize; 8],
    /// Number of supported PMP registers
    pub nb_pmp: u8,
    /// Number of virtual PMP available
    pub nb_virt_pmp: usize,
    /// The offset of the virtual PMP registers, compared to physical PMP.
    pub virt_pmp_offset: usize,
}

/// A struct that can be consumed to flush the caches, making the latest PMP configuration
/// effective immediately.
///
/// This struct is marked as `#[must_use]`, which will rise a warning if the struct is not
/// consumed. This is handy to prevent forgetting to flush the caches and introduce suble bugs.
#[must_use = "caches must be flushed before PMP change can take effect"]
pub struct PmpFlush();

impl fmt::Display for PmpGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n===============================")?;
        writeln!(f, "\n           PMP Entries       \n")?;

        for i in 0..self.nb_pmp {
            writeln!(f, "===============================")?;
            writeln!(
                f,
                "{:16x} | {}",
                self.pmpaddr[i as usize],
                self.get_cfg(i as usize)
            )?;
        }

        Ok(())
    }
}

impl PmpGroup {
    const fn new(nb_pmp: usize) -> Self {
        PmpGroup {
            pmpaddr: [0; 64],
            pmpcfg: [0; 8],
            nb_pmp: nb_pmp as u8,
            nb_virt_pmp: 0,
            virt_pmp_offset: 0,
        }
    }

    pub fn init_pmp_group(nb_pmp: usize) -> PmpGroup {
        let mut pmp = Self::new(nb_pmp);
        let virtual_devices = Plat::create_virtual_devices();

        // Configure PMP registers, if available
        if pmp.nb_pmp >= 8 {
            // By activating this entry it's possible to catch all memory accesses
            pmp.set_inactive(ALL_CATCH_OFFSET, 0);

            // Protect Miralis
            let (start, size) = Plat::get_miralis_memory_start_and_size();
            pmp.set_napot(MIRALIS_OFFSET, start, size, pmpcfg::NO_PERMISSIONS);

            // Protect virtual devices
            pmp.set_napot(
                DEVICES_OFFSET,
                virtual_devices[0].start_addr,
                virtual_devices[0].size,
                pmpcfg::NO_PERMISSIONS,
            );

            pmp.set_napot(
                DEVICES_OFFSET + 1,
                virtual_devices[1].start_addr,
                virtual_devices[1].size,
                pmpcfg::NO_PERMISSIONS,
            );

            // This PMP entry is used by the policy module for its own purpose
            #[allow(clippy::reversed_empty_ranges)]
            for idx in 0..POLICY_SIZE {
                pmp.set_inactive(POLICY_OFFSET + idx, 0);
            }

            // Add an inactive 0 entry so that the next PMP sees 0 with TOR configuration
            pmp.set_inactive(INACTIVE_ENTRY_OFFSET, 0);

            // Finally, set the last PMP to grant access to the whole memory
            pmp.set_napot((pmp.nb_pmp - 1) as usize, 0, usize::MAX, pmpcfg::RWX);

            // Compute the number of virtual PMPs available
            // It's whatever is left after setting pmp's for devices, pmp for address translation,
            // inactive entry and the last pmp to allow all the access
            let remaining_pmp_entries = pmp.nb_pmp as usize - MIRALIS_TOTAL_PMP;
            if let Some(max_virt_pmp) = config::VCPU_MAX_PMP {
                pmp.nb_virt_pmp = core::cmp::min(remaining_pmp_entries, max_virt_pmp);
            } else {
                pmp.nb_virt_pmp = remaining_pmp_entries;
            }
        } else {
            pmp.nb_virt_pmp = 0;
        }

        // Finally we can set the PMP offset
        pmp.virt_pmp_offset = VIRTUAL_PMP_OFFSET;

        pmp
    }

    /// This function builds a PMP Napot entry, note that the caller must only set the permissions bits and don't have to care about the low level formatting details to build the napot entry.
    pub fn set_napot(&mut self, idx: usize, from: usize, to: usize, permissions: u8) {
        assert!(
            permissions < 8,
            "Permissions should not set NAPOT or TOP bits"
        );
        self.set(idx, build_napot(from, to).unwrap(), permissions | NAPOT);
    }

    /// This function builds a PMP Tor entry, note that the caller must only set the permissions bits and don't have to care about the low level formatting details such as dividing the address by 4.
    pub fn set_tor(&mut self, idx: usize, until: usize, permissions: u8) {
        assert!(
            permissions < 8,
            "Permissions should not set NAPOT or TOP bits"
        );
        self.set(idx, build_tor(until), permissions | TOR);
    }

    /// This function builds a PMP inactive entry, note that the caller must not set the permission bits and can set a base address for the next pmp entry and it can simply give the address without dividing by 4.
    pub fn set_inactive(&mut self, idx: usize, addr: usize) {
        self.set(idx, build_tor(addr), INACTIVE);
    }

    /// Set a pmpaddr and its corresponding pmpcfg.
    fn set(&mut self, idx: usize, addr: usize, cfg: u8) {
        // Sanitize CFG
        let cfg = cfg & pmpcfg::VALID_BITS;
        assert!(cfg & pmpcfg::L == 0, "Lock bit not yet supported on PMPs");

        self.pmpaddr[idx] = addr;
        self.set_pmpcfg(idx, cfg);
    }

    pub fn set_from_policy(&mut self, idx: usize, addr: usize, cfg: u8) {
        #[allow(clippy::absurd_extreme_comparisons)]
        if idx >= POLICY_SIZE {
            panic!(
                "Policy isn't writing to its pmp entries index: {} number of registers: {} ",
                idx, POLICY_SIZE
            );
        }

        self.set(POLICY_OFFSET + idx, addr, cfg);
    }

    /// Returns the array of pmpaddr registers.
    pub fn pmpaddr(&self) -> &[usize; 64] {
        &self.pmpaddr
    }

    /// Returns the array of pmpcfg registers.
    pub fn pmpcfg(&self) -> &[usize; 8] {
        &self.pmpcfg
    }

    pub fn set_pmpaddr(&mut self, idx: usize, value: usize) {
        self.pmpaddr[idx] = value;
    }

    pub fn set_pmpcfg_raw(&mut self, idx: usize, value: usize) {
        self.pmpcfg[idx]= value;
    }

    pub fn set_pmpcfg(&mut self, index: usize, cfg: u8) {
        let reg_idx = index / 8;
        let inner_idx = index % 8;
        let shift = inner_idx * 8;
        // Clear old config
        self.pmpcfg[reg_idx] &= !(0xff << shift);
        // Set new config
        self.pmpcfg[reg_idx] |= (cfg as usize) << shift
    }

    pub fn get_cfg(&self, index: usize) -> u8 {
        let reg_idx = index / 8;
        let inner_idx = index % 8;
        let reg = self.pmpcfg[reg_idx];
        let cfg = (reg >> (inner_idx * 8)) & 0xff;
        cfg as u8
    }

    /// Loads PMP registers into the PMP group at the provided offset.
    ///
    /// This functions is used to import PMP registers, which is useful to load the virtual PMP
    /// registers into the set of physical PMP.
    pub fn load_with_offset(
        &mut self,
        pmpaddr: &[usize; 64],
        pmpcfg: &[usize; 8],
        offset: usize,
        nb_pmp: usize,
    ) {
        // Load pmpaddr
        self.pmpaddr[offset..(nb_pmp + offset)].copy_from_slice(&pmpaddr[..nb_pmp]);

        // Load pmpcfg
        for idx in 0..nb_pmp {
            let reg_idx = idx / 8;
            let inner_idx = idx % 8;
            let shift = inner_idx * 8; // 8 bits per config
            let cfg = (pmpcfg[reg_idx] >> shift) & 0xff;
            self.set_pmpcfg(idx + offset, cfg as u8);
        }
    }

    /// Clears `nb_pmp` PMP registers starting from `start`.
    pub fn clear_range(&mut self, start: usize, nb_pmp: usize) {
        for idx in 0..nb_pmp {
            self.pmpaddr[start + idx] = 0;
            self.set_pmpcfg(start + idx, pmpcfg::INACTIVE);
        }
    }
}

// ————————————————————————————— Memory Segment ————————————————————————————— //

/// A segment of memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Segment {
    start: usize,
    size: usize,
}

impl Segment {
    pub const fn new(start: usize, size: usize) -> Segment {
        // Sanitize size so that start + size does not overflow
        let end = start.saturating_add(size);
        let size = end - start;
        Segment { start, size }
    }

    /// Returns the segment start address
    pub fn start(self) -> usize {
        self.start
    }

    /// Returns the segment end address
    pub fn end(self) -> usize {
        self.start
            .checked_add(self.size)
            .expect("Invalid segment size")
    }

    /// Returns the segment size
    pub fn size(self) -> usize {
        self.size
    }

    /// Returns true if the two segment overlap.
    pub fn overlap(&self, other: Self) -> bool {
        other.end() > self.start && other.start < self.end()
    }

    /// Returns true if the other segment is contained within self.
    pub fn contain(&self, other: Self) -> bool {
        other.start >= self.start && other.end() <= self.end()
    }
}

// —————————————————————————————— PMP Iterator —————————————————————————————— //

pub struct PmpIterator<'a> {
    pmps: &'a PmpGroup,
    idx: usize,
    prev_addr: usize,
}

impl Iterator for PmpIterator<'_> {
    type Item = (Segment, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let pmps = &self.pmps;
        let nb_pmps = pmps.nb_pmp as usize;
        while self.idx < nb_pmps {
            let cfg = pmps.get_cfg(self.idx);
            let addr = pmps.pmpaddr[self.idx];
            let prev_addr = self.prev_addr;
            self.idx += 1;
            self.prev_addr = addr;

            match cfg & pmpcfg::A_MASK {
                pmpcfg::NA4 => {
                    let addr = addr << 2;
                    return Some((Segment::new(addr, 4), cfg & pmpcfg::RWX));
                }
                pmpcfg::NAPOT => {
                    let trailing_ones = addr.trailing_ones();
                    let addr_mask = !((1 << trailing_ones) - 1);
                    let addr = (addr & addr_mask) << 2;
                    let shift = trailing_ones + 3;
                    return Some((Segment::new(addr, 1 << shift), cfg & pmpcfg::RWX));
                }
                pmpcfg::TOR => {
                    // if prev_addr is bigger then that entry does not match anything
                    if prev_addr >= addr {
                        continue;
                    }
                    let size = addr - prev_addr;
                    return Some((Segment::new(prev_addr, size), cfg & pmpcfg::RWX));
                }
                _ => {
                    // Inactive PMP entry
                    continue;
                }
            }
        }

        None
    }
}

impl<'a> IntoIterator for &'a PmpGroup {
    type Item = (Segment, u8);

    type IntoIter = PmpIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PmpIterator {
            pmps: self,
            idx: 0,
            prev_addr: 0,
        }
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn napot() {
        // Size is too small
        assert_eq!(None, build_napot(0x1000, 0));
        assert_eq!(None, build_napot(0x1000, 1));
        assert_eq!(None, build_napot(0x1000, 2));
        assert_eq!(None, build_napot(0x1000, 4));
        assert_eq!(None, build_napot(0x1000, 7));

        // Address is not aligned
        assert_eq!(None, build_napot(0x1001, 8));
        assert_eq!(None, build_napot(0x1002, 8));
        assert_eq!(None, build_napot(0x1004, 8));
        assert_eq!(None, build_napot(0x1008, 16));

        // Valid address and size
        assert_eq!(Some(0x400), build_napot(0x1000, 8));
        assert_eq!(Some(0x401), build_napot(0x1000, 16));
        assert_eq!(Some(0x403), build_napot(0x1000, 32));
    }

    #[test]
    fn segments() {
        // Segment [20, 30].
        let segment = Segment::new(20, 10);

        // Check basic overlap cases
        assert!(!segment.overlap(Segment::new(10, 5)));
        assert!(!segment.overlap(Segment::new(10, 10)));
        assert!(segment.overlap(Segment::new(10, 15)));
        assert!(segment.overlap(Segment::new(10, 15)));
        assert!(segment.overlap(Segment::new(10, 20)));
        assert!(segment.overlap(Segment::new(10, 30)));
        assert!(segment.overlap(Segment::new(20, 10)));
        assert!(segment.overlap(Segment::new(20, 20)));
        assert!(segment.overlap(Segment::new(25, 2)));
        assert!(segment.overlap(Segment::new(25, 5)));
        assert!(segment.overlap(Segment::new(25, 10)));
        assert!(!segment.overlap(Segment::new(30, 10)));
        assert!(!segment.overlap(Segment::new(35, 10)));

        // A segment where start + size overflow
        let overflow_segment = Segment::new(usize::MAX - 10, 100);
        assert_eq!(overflow_segment.size(), 10);
        assert_eq!(overflow_segment.end(), usize::MAX);
    }

    #[test]
    fn pmp_groups() {
        use pmpcfg::*;

        // Initialize an empty group of PMPS
        let mut pmps: PmpGroup = PmpGroup::new(8);
        for _ in &pmps {
            panic!("A PMP group should be initialized with no valid entry");
        }

        // Configure some PMP entries
        pmps.set(0, 1000, RWX | TOR);
        pmps.set(1, 1500, R | W | TOR);
        pmps.set(2, 2000 >> 2, RWX | NA4); // NA4 addresses are shifted by 2
        pmps.set(3, 0x8000 >> 2 | 0b0111, RWX | NAPOT); // NAPOT addresses are shifted by 2

        // The expected regions
        let expected = [
            (Segment::new(0, 1000), RWX),
            (Segment::new(1000, 500), R | W),
            (Segment::new(2000, 4), RWX),
            (Segment::new(0x8000, 64), RWX),
        ];

        // Check that the config is indeed properly configured
        for (actual, expected) in pmps.into_iter().zip(expected.into_iter()) {
            assert_eq!(actual, expected, "Unexpected PMP region")
        }
    }
}

impl PmpFlush {
    /// Flush the caches, which is required for PMP changes to take effect.
    pub fn flush(self) {
        unsafe { Arch::sfencevma(None, None) }
    }

    /// Do not flush the caches, PMP changes will not take effect predictably which can lead to
    /// suble bugs.
    #[allow(dead_code)] // TODO: remove once used or part of the public API
    pub fn no_flush(self) {
        // Do nothing
    }
}
