mod sail;

use miralis::arch::{mstatus, Architecture, ExtensionsCapability, Mode};
use miralis::platform::Platform;
use miralis::virt::VirtContext;
use sail::Privilege;
use sail_prelude::BitVector;

#[derive(Debug)]
pub struct SailVirtContext {
    mepc: BitVector<64>,
    sepc: BitVector<64>,
    mstatus: BitVector<64>,
    next_pc: BitVector<64>,
    pc: BitVector<64>,
    cur_privilege: sail::Privilege,
}

impl SailVirtContext {
    pub fn from(ctx: &mut VirtContext) -> Self {
        SailVirtContext {
            mepc: BitVector::new(ctx.csr.mepc as u64),
            sepc: BitVector::new(ctx.csr.sepc as u64),
            mstatus: BitVector::new(ctx.csr.mstatus as u64),
            next_pc: BitVector::new(ctx.pc as u64),
            pc: BitVector::new(ctx.pc as u64),
            cur_privilege: match ctx.mode {
                Mode::U => Privilege::User,
                Mode::S => Privilege::Supervisor,
                Mode::M => Privilege::Machine,
            },
        }
    }

    pub fn into_virt_context(self) -> VirtContext {
        let mut ctx = VirtContext::new(
            0,
            0,
            ExtensionsCapability {
                has_h_extension: false,
                has_s_extension: false,
            },
        );

        ctx.csr.mepc = self.mepc.bits() as usize;
        ctx.csr.sepc = self.sepc.bits() as usize;
        ctx.csr.mstatus = self.mstatus.bits() as usize;

        ctx.mode = match self.cur_privilege {
            Privilege::User => Mode::U,
            Privilege::Supervisor => Mode::S,
            Privilege::Machine => Mode::M,
        };

        ctx.pc = self.next_pc.bits() as usize;

        ctx
    }
}

#[cfg(test)]
mod tests {
    use sail::Privilege;

    use super::*;

    #[test]
    fn simple_mret() {
        let mepc = 0x1000;
        let mut ctx = SailVirtContext {
            mepc: BitVector::new(mepc),
            sepc: BitVector::new(0x2000),
            mstatus: BitVector::new(0x0),
            next_pc: BitVector::new(0x4004),
            pc: BitVector::new(0x4000),
            cur_privilege: Privilege::Machine,
        };

        sail::execute_MRET(&mut ctx);
        assert_eq!(ctx.next_pc.bits(), mepc);
    }
}

#[cfg(kani)]
mod verification {
    use miralis::arch::Arch;
    use miralis::host::MiralisContext;
    use miralis::platform::Plat;

    use super::*;

    #[kani::proof]
    pub fn mret() {
        let mpp = match kani::any::<u8>() % 3 {
            0 => 0b00,
            1 => 0b01,
            2 => 0b11,
            _ => unreachable!(),
        };
        let mstatus = kani::any::<usize>() & !(0b11 << mstatus::MPP_OFFSET);

        let mut ctx = VirtContext::new(
            0,
            0,
            ExtensionsCapability {
                has_h_extension: false,
                has_s_extension: false,
            },
        );

        ctx.csr.mepc = kani::any::<usize>() & (!0b11);
        ctx.csr.sepc = kani::any();
        ctx.csr.mstatus = mstatus | (mpp << mstatus::MPP_OFFSET);
        ctx.mode = Mode::M;
        ctx.pc = kani::any();

        let mut sail_ctx = SailVirtContext::from(&mut ctx);
        sail::execute_MRET(&mut sail_ctx);

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        ctx.emulate_mret(&mut mctx);

        assert_eq!(ctx, sail_ctx.into_virt_context(), "mret equivalence");
    }
}
