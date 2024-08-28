//! Base device classes

use crate::device;
use crate::device::clint::CLINT_SIZE;
use crate::device::tester::TEST_DEVICE_SIZE;
use crate::platform::{Plat, Platform};

pub mod clint;
pub mod tester;

// ———————————————————————————— Virtual Devices ————————————————————————————— //

/// Represents different data widths:
///  - `Byte`: 8 bits (1 byte)
///  - `Byte2`: 16 bits (2 bytes)
///  - `Byte4`: 32 bits (4 bytes)
///  - `Byte8`: 64 bits (8 bytes)
pub enum Width {
    Byte = 8,
    Byte2 = 16,
    Byte4 = 32,
    Byte8 = 64,
}

impl Width {
    pub fn to_bits(&self) -> usize {
        match self {
            Width::Byte => 8,
            Width::Byte2 => 16,
            Width::Byte4 => 32,
            Width::Byte8 => 64,
        }
    }

    pub fn to_bytes(&self) -> usize {
        self.to_bits() / 8
    }
}

impl From<usize> for Width {
    fn from(value: usize) -> Self {
        match value {
            8 => Width::Byte,
            16 => Width::Byte2,
            32 => Width::Byte4,
            64 => Width::Byte8,
            _ => panic!("Invalid width value"),
        }
    }
}

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
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str>;
    fn write_device(&self, offset: usize, w_width: Width, value: usize)
        -> Result<(), &'static str>;
}

// ———————————————————————————— Pass Through module ————————————————————————————— //

pub struct PassThroughModule {
    devices: [VirtDevice; 2],
    base_address: usize,
    size: usize,
}

impl DeviceAccess for PassThroughModule {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        /*for device in &self.devices {
            if self.is_included(device, offset) {
                return device.device_interface.read_device(offset, r_width)
            }
        }*/

        Err("Mapping to non existent device")
    }

    fn write_device(&self, offset: usize, w_width: Width, value: usize) -> Result<(), &'static str> {
        /*for device in &self.devices {
            if self.is_included(device, offset) {
                return device.device_interface.write_device(offset, w_width, value)
            }
        }*/

        Err("Mapping to non existent device")
    }
}

struct emptyModule {

}

impl DeviceAccess for emptyModule {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        todo!()
    }

    fn write_device(&self, offset: usize, w_width: Width, value: usize) -> Result<(), &'static str> {
        todo!()
    }
}

impl PassThroughModule {
    pub const fn new(base_address: usize, size: usize) -> PassThroughModule {
        let empty_device = VirtDevice {
            start_addr:0,
            size: 0,
            name: "",
            device_interface: &emptyModule{},
        };

        let empty_device2 = VirtDevice {
            start_addr:0,
            size: 0,
            name: "",
            device_interface: &emptyModule{},
        };


        PassThroughModule {
            devices: [empty_device, empty_device2],
            base_address: base_address,
            size: size,
        }
    }

    pub fn attach_devices(&mut self) {
        self.devices = Plat::create_virtual_devices()
    }

    const fn is_included(&self, device: &VirtDevice, offset: usize) -> bool{
        let current_addr = self.base_address + offset;
        device.start_addr <= current_addr && current_addr < device.start_addr + device.size
    }
}
