pub mod sail;

use miralis::arch::{mstatus, Architecture, ExtensionsCapability, Mode};
use miralis::platform::Platform;
use miralis::virt::VirtContext;
use sail::Privilege;
use sail_prelude::{BitField, BitVector};
use crate::sail::SailVirtCtx;

impl SailVirtCtx {
    pub fn from(ctx: &mut VirtContext) -> Self {

        let mut sail_ctx = SailVirtCtx::new();

        sail_ctx.mepc = BitVector::new(ctx.csr.mepc as u64);
        sail_ctx.sepc = BitVector::new(ctx.csr.sepc as u64);

        sail_ctx.mstatus = BitField {
                bits: BitVector::new(ctx.csr.mstatus as u64),
        };

        sail_ctx.nextPC = BitVector::new(ctx.pc as u64);
        sail_ctx.PC = BitVector::new(ctx.pc as u64);

        sail_ctx.cur_privilege = match ctx.mode {
                Mode::U => Privilege::User,
                Mode::S => Privilege::Supervisor,
                Mode::M => Privilege::Machine,
        };

        sail_ctx
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
        ctx.csr.mstatus = self.mstatus.bits.bits() as usize;

        ctx.mode = match self.cur_privilege {
            Privilege::User => Mode::U,
            Privilege::Supervisor => Mode::S,
            Privilege::Machine => Mode::M,
        };

        ctx.pc = self.nextPC.bits() as usize;

        ctx
    }
}

#[cfg(test)]
mod tests {
    use sail::Privilege;
    use crate::sail::SailVirtCtx;

    use super::*;

    /*#[test]
    fn simple_mret() {
        let mepc = 0x1000;
        let mut ctx = SailVirtCtx {
            mepc: BitVector::new(mepc),
            sepc: BitVector::new(0x2000),
            mstatus: BitVector::new(0x0),
            next_pc: BitVector::new(0x4004),
            pc: BitVector::new(0x4000),
            cur_privilege: Privilege::Machine,
        };

        sail::execute_MRET(&mut ctx);
        assert_eq!(ctx.next_pc.bits(), mepc);
    }*/
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

        let mut sail_ctx = SailVirtCtx::from(&mut ctx);
        sail::execute_MRET(&mut sail_ctx);

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        ctx.emulate_mret(&mut mctx);

        // assert_eq!(ctx, sail_ctx.into_virt_context(), "mret equivalence");
        // assert_eq!(ctx.csr.mepc, sail_ctx.into_virt_context().csr.mepc, "mret equivalence");
        // assert_eq!(ctx.csr.sepc, sail_ctx.into_virt_context().csr.sepc, "mret equivalence");
        // assert_eq!(ctx.csr.mstatus, sail_ctx.into_virt_context().csr.mstatus, "mret equivalence");
        // assert_eq!(ctx.pc, sail_ctx.into_virt_context().pc, "mret equivalence");
        assert_eq!(ctx.mode, sail_ctx.into_virt_context().mode, "mret equivalence");
    }
}
