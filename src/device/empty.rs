use crate::device::{DeviceAccess, VirtDevice, Width};

// ———————————————————————————— Empty device ————————————————————————————— //

pub struct EmptyDevice {}

impl DeviceAccess for EmptyDevice {
    fn read_device(&self, _: usize, _: Width) -> Result<usize, &'static str> {
        panic!("Trying to read the empty device!");
    }

    fn write_device(&self, _: usize, _: Width, _: usize) -> Result<(), &'static str> {
        panic!("Trying to write the empty device!");
    }
}

impl EmptyDevice {
    pub const fn new() -> VirtDevice {
        VirtDevice {
            start_addr: 0,
            size: 0,
            name: "",
            device_interface: &EmptyDevice {},
        }
    }
}
