#[derive(PartialEq, Eq, Debug)]
pub struct VirtContext {
    pub csr: VirtCsr,
    pub mode: Mode,
    pub pc: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct VirtCsr {
    pub mepc: usize,
    pub sepc: usize,
    pub mstatus: usize,
}

impl VirtContext {
    #[allow(unused)]
    pub fn mret(&mut self) {
        match parse_mpp_return_mode(self.csr.mstatus) {
            Mode::M => {
                // Mret is jumping back to machine mode, do nothing
            }
            Mode::S => {
                // Mret is jumping to supervisor mode, the runner is the guest OS
                self.mode = Mode::S;

                VirtCsr::set_mstatus_field(
                    &mut self.csr.mstatus,
                    mstatus::MPRV_OFFSET,
                    mstatus::MPRV_FILTER,
                    0,
                );
            }
            Mode::U => {
                // Mret is jumping to user mode, the runner is the guest OS
                self.mode = Mode::U;

                VirtCsr::set_mstatus_field(
                    &mut self.csr.mstatus,
                    mstatus::MPRV_OFFSET,
                    mstatus::MPRV_FILTER,
                    0,
                );
            }
            _ => {
                panic!(
                    "MRET is not going to M/S/U mode: {} with MPP {:x}",
                    self.csr.mstatus,
                    ((self.csr.mstatus >> mstatus::MPP_OFFSET) & mstatus::MPP_FILTER)
                );
            }
        }
        // Modify mstatus
        // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
        if false {
            VirtCsr::set_mstatus_field(
                &mut self.csr.mstatus,
                mstatus::MPV_OFFSET,
                mstatus::MPV_FILTER,
                0,
            );
        }

        // MIE = MPIE, MPIE = 1, MPRV = 0
        let mpie = mstatus::MPIE_FILTER & (self.csr.mstatus >> mstatus::MPIE_OFFSET);

        VirtCsr::set_mstatus_field(
            &mut self.csr.mstatus,
            mstatus::MPIE_OFFSET,
            mstatus::MPIE_FILTER,
            1,
        );
        VirtCsr::set_mstatus_field(
            &mut self.csr.mstatus,
            mstatus::MIE_OFFSET,
            mstatus::MIE_FILTER,
            mpie,
        );
        VirtCsr::set_mstatus_field(
            &mut self.csr.mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            0,
        );

        // Jump back to firmware
        self.pc = self.csr.mepc;
    }
}

impl VirtCsr {
    pub fn set_mstatus_field(csr: &mut usize, offset: usize, filter: usize, value: usize) {
        // Clear field
        *csr &= !(filter << offset);
        // Set field
        *csr |= value << offset;
    }
}

// ————————————————————————————————— Utils —————————————————————————————————— //

/// Privilege modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// User
    U,
    /// Supervisor
    S,
    /// Machine
    M,
}

/// Returns the mode corresponding to the bit pattern
pub fn parse_mpp_return_mode(mstatus_reg: usize) -> Mode {
    match (mstatus_reg >> mstatus::MPP_OFFSET) & mstatus::MPP_FILTER {
        0 => Mode::U,
        1 => Mode::S,
        3 => Mode::M,
        _ => panic!("Unknown mode!"),
    }
}

/// Constants for the Machine Status (mstatus) CSR.
#[allow(unused)]
pub mod mstatus {
    /// MIE
    pub const MIE_OFFSET: usize = 3;
    pub const MIE_FILTER: usize = 0b1;
    /// MPIE
    pub const MPIE_OFFSET: usize = 7;
    pub const MPIE_FILTER: usize = 0b1;
    /// MPP
    pub const MPP_OFFSET: usize = 11;
    pub const MPP_FILTER: usize = 0b11;
    /// MPRV
    pub const MPRV_OFFSET: usize = 17;
    pub const MPRV_FILTER: usize = 0b1;
    /// MPV
    pub const MPV_OFFSET: usize = 39;
    pub const MPV_FILTER: usize = 0b1;
}