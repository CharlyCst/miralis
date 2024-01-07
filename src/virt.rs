//! Firmware Virtualisation

/// The context of a virtual firmware.
#[derive(Debug, Default)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    pub regs: [usize; 31],
    pub csr: VirtCsr,
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug, Default)]
pub struct VirtCsr {
    mscratch: usize,
}
