//! Base device classes

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
