//! Trap handling

use core::fmt;

// ————————————————————————————————— mcause ————————————————————————————————— //

const INTERRUPT_BIT: usize = 1 << (usize::BITS as usize - 1);

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(usize)]
#[allow(clippy::enum_clike_unportable_variant)]
pub enum MCause {
    // Exceptions
    InstrAddrMisaligned = 0,
    InstrAccessFault = 1,
    IllegalInstr = 2,
    Breakpoint = 3,
    LoadAddrMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddrMisaligned = 6,
    StoreAccessFault = 7,
    EcallFromUMode = 8,
    EcallFromSMode = 9,
    EcallFromMMode = 11,
    InstrPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    UnknownException = 16,

    // Interrupts
    UserSoftInt = INTERRUPT_BIT,
    SupervisorSoftInt = INTERRUPT_BIT + 1,
    MachineSoftInt = INTERRUPT_BIT + 3,
    UserTimerInt = INTERRUPT_BIT + 4,
    SupervisorTimerInt = INTERRUPT_BIT + 5,
    MachineTimerInt = INTERRUPT_BIT + 7,
    UserExternalInt = INTERRUPT_BIT + 8,
    SupervisorExternalInt = INTERRUPT_BIT + 9,
    MachineExternalInt = INTERRUPT_BIT + 11,
    UnknownInt,
}

impl MCause {
    pub fn new(cause: usize) -> Self {
        MCause::try_from(cause).unwrap()
    }

    pub fn is_interrupt(self) -> bool {
        self as usize & INTERRUPT_BIT != 0
    }

    pub fn cause_number(cause: usize) -> usize {
        if (cause as isize) < 0 {
            cause ^ INTERRUPT_BIT
        } else {
            cause
        }
    }
}

// —————————————————————————————— Conversions ——————————————————————————————— //

impl TryFrom<usize> for MCause {
    type Error = ();

    fn try_from(cause: usize) -> Result<Self, Self::Error> {
        if (cause as isize) < 0 {
            // Interrupt
            // set last bit to 0
            match cause ^ INTERRUPT_BIT {
                0 => Ok(MCause::UserSoftInt),
                1 => Ok(MCause::SupervisorSoftInt),
                3 => Ok(MCause::MachineSoftInt),
                4 => Ok(MCause::UserTimerInt),
                5 => Ok(MCause::SupervisorTimerInt),
                7 => Ok(MCause::MachineTimerInt),
                8 => Ok(MCause::UserExternalInt),
                9 => Ok(MCause::SupervisorExternalInt),
                11 => Ok(MCause::MachineExternalInt),
                _ => Ok(MCause::UnknownInt),
            }
        } else {
            // Trap
            match cause {
                0 => Ok(MCause::InstrAddrMisaligned),
                1 => Ok(MCause::InstrAccessFault),
                2 => Ok(MCause::IllegalInstr),
                3 => Ok(MCause::Breakpoint),
                4 => Ok(MCause::LoadAddrMisaligned),
                5 => Ok(MCause::LoadAccessFault),
                6 => Ok(MCause::StoreAddrMisaligned),
                7 => Ok(MCause::StoreAccessFault),
                8 => Ok(MCause::EcallFromUMode),
                9 => Ok(MCause::EcallFromSMode),
                11 => Ok(MCause::EcallFromMMode),
                12 => Ok(MCause::InstrPageFault),
                13 => Ok(MCause::LoadPageFault),
                15 => Ok(MCause::StorePageFault),
                _ => Ok(MCause::UnknownException),
            }
        }
    }
}

// ——————————————————————————————— Trap Info ———————————————————————————————— //

/// Contains all the information automatically written by the hardware during a trap
#[repr(C)]
#[derive(Clone, Default)]
pub struct TrapInfo {
    // mtval2 and mtinst only exist with the hypervisor extension
    pub mepc: usize,
    pub mstatus: usize,
    pub mcause: usize,
    pub mip: usize,
    pub mtval: usize,
}

impl TrapInfo {
    /// Whether the trap comes from M mode
    pub fn is_from_mmode(&self) -> bool {
        let mpp: usize = (self.mstatus >> 11) & 0b11;
        mpp == 3 // Mpp : 3 = M mode
    }

    /// Return the trap cause
    pub fn get_cause(&self) -> MCause {
        MCause::new(self.mcause)
    }
}

// ———————————————————————————————— Display ————————————————————————————————— //

impl fmt::Debug for TrapInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TrapInfo : Mepc=0x{:x} Mstatus=0x{:x} MCause=0x{:?} Mip=0x{:x} Mtval=0x{:x}",
            self.mepc, self.mstatus, self.mcause, self.mip, self.mtval
        )
    }
}

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
            MCause::StoreAddrMisaligned => write!(f, "store/amo misaligned"),
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
