use crate::device::empty::EmptyDevice;
use crate::device::{find_matching_device, DeviceAccess, VirtDevice, Width};
use crate::platform::{Plat, Platform};

// ———————————————————————————— Pass Through module ————————————————————————————— //

pub struct PassThroughModule {
    devices: [VirtDevice; 2],
    base_address: usize,
}

impl DeviceAccess for PassThroughModule {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        match find_matching_device(self.base_address + offset, &self.devices) {
            Some(device) => device
                .device_interface
                .read_device(self.recalc_offset(device.start_addr, offset), r_width),
            _ => Err("Mapping to non existent device"),
        }
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
    ) -> Result<(), &'static str> {
        match find_matching_device(self.base_address + offset, &self.devices) {
            Some(device) => device.device_interface.write_device(
                self.recalc_offset(device.start_addr, offset),
                w_width,
                value,
            ),
            _ => Err("Mapping to non existent device"),
        }
    }
}

impl PassThroughModule {
    pub const fn new(base_address: usize) -> PassThroughModule {
        let empty_clint_device = EmptyDevice::new();
        let empty_test_device = EmptyDevice::new();

        PassThroughModule {
            devices: [empty_clint_device, empty_test_device],
            base_address: base_address,
        }
    }

    pub fn attach_devices(&mut self) {
        self.devices = Plat::create_virtual_devices()
    }

    pub fn recalc_offset(&self, device_start: usize, offset: usize) -> usize {
        return self.base_address + offset - device_start;
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use crate::device::passthrough::PassThroughModule;

    #[test]
    fn recalc_offset() {
        let mock_pass_through_module = PassThroughModule::new(1000);

        // A few test cases
        assert_eq!(mock_pass_through_module.recalc_offset(1000, 0), 0);
        assert_eq!(mock_pass_through_module.recalc_offset(1000, 50), 50);
        assert_eq!(mock_pass_through_module.recalc_offset(1050, 50), 0);
        assert_eq!(mock_pass_through_module.recalc_offset(1000, 1000), 1000);
        assert_eq!(mock_pass_through_module.recalc_offset(2000, 2000), 1000);
    }
}
