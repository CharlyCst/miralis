//! Base device classes

use crate::arch::Width;
use crate::virt::VirtContext;

pub mod clint;
pub mod tester;

// ———————————————————————————— Virtual Devices ————————————————————————————— //

/// Represents a virtual memory-mapped device
pub struct VirtDevice {
    pub start_addr: usize,
    pub size: usize,
    pub name: &'static str,
    pub device_interface: &'static dyn DeviceAccess,
}

pub fn find_matching_device(address: usize, devices: &[VirtDevice]) -> Option<&VirtDevice> {
    devices
        .iter()
        .find(|device| address >= device.start_addr && address < device.start_addr + device.size)
}

pub trait DeviceAccess: Sync + Send {
    fn read_device(
        &self,
        offset: usize,
        r_width: Width,
        ctx: &mut VirtContext,
    ) -> Result<usize, &'static str>;
    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str>;
}
