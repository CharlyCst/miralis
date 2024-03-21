//! Trap handling

use core::fmt;

// ————————————————————————————————— mcause ————————————————————————————————— //

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum MCause {
    // Exceptions
    InstrAddrMisaligned = 0,
    InstrAccessFault = 1,
    IllegalInstr = 2,
    Breakpoint = 3,
    LoadAddrMisaligned = 4,
    LoadAccessFault = 5,
    StoreMisaligned = 6,
    StoreAccessFault = 7,
    EcallFromUMode = 8,
    EcallFromSMode = 9,
    EcallFromMMode = 11,
    InstrPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    UnknownException = 16,

    // Interrupts
    UserSoftInt,
    SupervisorSoftInt,
    MachineSoftInt,
    UserTimerInt,
    SupervisorTimerInt,
    MachineTimerInt,
    UserExternalInt,
    SupervisorExternalInt,
    MachineExternalInt,
    UnknownInt,
}

impl MCause {
    pub fn new(cause: usize) -> Self {
        if (cause as isize) < 0 {
            // Interrupt
            // TODO: I think this does not work, investigate when getting unknownn ints
            match cause {
                0 => Self::UserSoftInt,
                1 => Self::SupervisorSoftInt,
                3 => Self::MachineSoftInt,
                4 => Self::UserTimerInt,
                5 => Self::SupervisorTimerInt,
                7 => Self::MachineTimerInt,
                8 => Self::UserExternalInt,
                9 => Self::SupervisorExternalInt,
                11 => Self::MachineExternalInt,
                _ => Self::UnknownInt,
            }
        } else {
            // Trap
            match cause {
                0 => Self::InstrAddrMisaligned,
                1 => Self::InstrAccessFault,
                2 => Self::IllegalInstr,
                3 => Self::Breakpoint,
                4 => Self::LoadAddrMisaligned,
                5 => Self::LoadAccessFault,
                6 => Self::StoreMisaligned,
                7 => Self::StoreAccessFault,
                8 => Self::EcallFromUMode,
                9 => Self::EcallFromSMode,
                11 => Self::EcallFromMMode,
                12 => Self::InstrPageFault,
                13 => Self::LoadPageFault,
                15 => Self::StorePageFault,
                _ => Self::UnknownException,
            }
        }
    }

    pub fn is_interrupt(self) -> bool {
        match self {
            // Interrupts
            MCause::UserSoftInt => true,
            MCause::SupervisorSoftInt => true,
            MCause::MachineSoftInt => true,
            MCause::UserTimerInt => true,
            MCause::SupervisorTimerInt => true,
            MCause::MachineTimerInt => true,
            MCause::UserExternalInt => true,
            MCause::SupervisorExternalInt => true,
            MCause::MachineExternalInt => true,
            MCause::UnknownInt => true,
            // Traps
            MCause::InstrAddrMisaligned => false,
            MCause::InstrAccessFault => false,
            MCause::IllegalInstr => false,
            MCause::Breakpoint => false,
            MCause::LoadAddrMisaligned => false,
            MCause::LoadAccessFault => false,
            MCause::StoreMisaligned => false,
            MCause::StoreAccessFault => false,
            MCause::EcallFromUMode => false,
            MCause::EcallFromSMode => false,
            MCause::EcallFromMMode => false,
            MCause::InstrPageFault => false,
            MCause::LoadPageFault => false,
            MCause::StorePageFault => false,
            MCause::UnknownException => false,
        }
    }
}

// ——————————————————————————————— Trap Info ———————————————————————————————— //

/// Contains all the information automatically written by the hardware during a trap
#[repr(C)]
#[derive(Clone, Debug)]
pub struct TrapInfo {
    // mtval2 and mtinst only exist with the hypervisor extension
    pub mepc: usize,
    pub mstatus: usize,
    pub mcause: usize,
    pub mip: usize,
    pub mtval: usize,
}

impl Default for TrapInfo {
    fn default() -> TrapInfo {
        TrapInfo {
            mepc: 0,
            mstatus: 0,
            mcause: 0,
            mip: 0,
            mtval: 0,
        }
    }
}

impl TrapInfo {
    /// Whether the trap comes from M mode
    pub fn from_mmode(&self) -> bool {
        let mpp: usize = (self.mstatus >> 11) & 0b11;
        return mpp == 3; // Mpp : 3 = M mode
    }

    /// Return the trap cause
    pub fn get_cause(&self) -> MCause {
        return MCause::new(self.mcause);
    }
}

// ———————————————————————————————— Display ————————————————————————————————— //

impl fmt::Debug for MCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Interrupts
            MCause::UserSoftInt => write!(f, "user software interrupt"),
            MCause::SupervisorSoftInt => write!(f, "supervisor software interrupt"),
            MCause::MachineSoftInt => write!(f, "machine software interrupt"),
            MCause::UserTimerInt => write!(f, "user timer interrupt"),
            MCause::SupervisorTimerInt => write!(f, "supervisor timer interrupt"),
            MCause::MachineTimerInt => write!(f, "machine timer interrupt"),
            MCause::UserExternalInt => write!(f, "user external interrupt"),
            MCause::SupervisorExternalInt => write!(f, "supervisor external interrupt"),
            MCause::MachineExternalInt => write!(f, "machine external interrupt"),
            MCause::UnknownInt => write!(f, "unknown interrupt"),
            // Traps
            MCause::InstrAddrMisaligned => write!(f, "instruction address misaligned"),
            MCause::InstrAccessFault => write!(f, "instruction access fault"),
            MCause::IllegalInstr => write!(f, "illegal instruction"),
            MCause::Breakpoint => write!(f, "breakpoint"),
            MCause::LoadAddrMisaligned => write!(f, "load address misaligned"),
            MCause::LoadAccessFault => write!(f, "load access fault"),
            MCause::StoreMisaligned => write!(f, "store/amo misaligned"),
            MCause::StoreAccessFault => write!(f, "store/amo access fault"),
            MCause::EcallFromUMode => write!(f, "ecall from u-mode"),
            MCause::EcallFromSMode => write!(f, "ecall from s-mode"),
            MCause::EcallFromMMode => write!(f, "ecall from m-mode"),
            MCause::InstrPageFault => write!(f, "instruction page fault"),
            MCause::LoadPageFault => write!(f, "load page fault"),
            MCause::StorePageFault => write!(f, "store/amo page fault"),
            MCause::UnknownException => write!(f, "unknown exception"),
        }
    }
}
