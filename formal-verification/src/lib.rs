pub mod sail;

use miralis::arch::{mstatus, Architecture, ExtensionsCapability, Mode};
use miralis::platform::Platform;
use miralis::virt::VirtContext;
use sail::Privilege;
use sail_prelude::{BitField, BitVector};
use miralis::virt::traits::RegisterContextGetter;
use miralis::arch::mie;

use crate::sail::SailVirtCtx;

impl SailVirtCtx {
    pub fn from(ctx: &mut VirtContext) -> Self {
        let mut sail_ctx = SailVirtCtx::new();



        sail_ctx.nextPC = BitVector::new(ctx.pc as u64);
        sail_ctx.PC = BitVector::new(ctx.pc as u64);

        sail_ctx.cur_privilege = match ctx.mode {
            Mode::U => Privilege::User,
            Mode::S => Privilege::Supervisor,
            Mode::M => Privilege::Machine,
        };

        // Transfer all csr


        sail_ctx.mepc = BitVector::new(ctx.csr.mepc as u64);
        sail_ctx.sepc = BitVector::new(ctx.csr.sepc as u64);

        sail_ctx.mstatus = BitField {
            bits: BitVector::new(ctx.csr.mstatus as u64),
        };


        sail_ctx.marchid = BitVector::new(ctx.csr.marchid as u64);

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

        ctx.mode = match self.cur_privilege {
            Privilege::User => Mode::U,
            Privilege::Supervisor => Mode::S,
            Privilege::Machine => Mode::M,
        };

        ctx.pc = self.nextPC.bits() as usize;
        

        // Transfer all csr

        ctx.csr.mepc = self.mepc.bits() as usize;
        ctx.csr.sepc = self.sepc.bits() as usize;
        ctx.csr.mstatus = self.mstatus.bits.bits() as usize;


        ctx.csr.marchid = self.marchid.bits() as usize;

        ctx
    }
}

#[cfg(kani)]
fn KaniVirtCtx() -> VirtContext {
    let mut ctx = VirtContext::new(
        0,
        0,
        ExtensionsCapability {
            has_h_extension: false,
            has_s_extension: true,
        },
    );

    ctx.csr.misa= kani::any();
    ctx.csr.mie= kani::any();
    ctx.csr.mip= kani::any();
    ctx.csr.mtvec= kani::any();
    ctx.csr.mscratch= kani::any();
    ctx.csr.mvendorid= kani::any();
    ctx.csr.marchid= kani::any();
    ctx.csr.mimpid= kani::any();
    ctx.csr.mcycle= kani::any();
    ctx.csr.minstret= kani::any();
    ctx.csr.mcountinhibit= kani::any();
    ctx.csr.mcounteren= kani::any();
    ctx.csr.menvcfg= kani::any();
    ctx.csr.mseccfg= kani::any();
    ctx.csr.mcause= kani::any();
    ctx.csr.mepc= kani::any();
    ctx.csr.mtval= kani::any();
    ctx.csr.mtval2= kani::any();
    ctx.csr.mstatus= kani::any();
    ctx.csr.mtinst= kani::any();
    ctx.csr.mconfigptr= kani::any();
    ctx.csr.stvec= kani::any();
    ctx.csr.scounteren= kani::any();
    ctx.csr.senvcfg= kani::any();
    ctx.csr.sscratch= kani::any();
    ctx.csr.sepc= kani::any();
    ctx.csr.scause= kani::any();
    ctx.csr.stval= kani::any();
    ctx.csr.satp= kani::any();
    ctx.csr.scontext= kani::any();
    ctx.csr.medeleg= kani::any();
    ctx.csr.mideleg = mie::MIDELEG_READ_ONLY_ONE;
    ctx.csr.hstatus= kani::any();
    ctx.csr.hedeleg= kani::any();
    ctx.csr.hideleg= kani::any();
    ctx.csr.hvip= kani::any();
    ctx.csr.hip= kani::any();
    ctx.csr.hie= kani::any();
    ctx.csr.hgeip= kani::any();
    ctx.csr.hgeie= kani::any();
    ctx.csr.henvcfg= kani::any();
    ctx.csr.henvcfgh= kani::any();
    ctx.csr.hcounteren= kani::any();
    ctx.csr.htimedelta= kani::any();
    ctx.csr.htimedeltah= kani::any();
    ctx.csr.htval= kani::any();
    ctx.csr.htinst= kani::any();
    ctx.csr.hgatp= kani::any();
    ctx.csr.vsstatus= kani::any();
    ctx.csr.vsie= kani::any();
    ctx.csr.vstvec= kani::any();
    ctx.csr.vsscratch= kani::any();
    ctx.csr.vsepc= kani::any();
    ctx.csr.vscause= kani::any();
    ctx.csr.vstval= kani::any();
    ctx.csr.vsip= kani::any();
    ctx.csr.vsatp= kani::any();
    ctx.csr.pmpcfg = [kani::any(); 8]; 
    ctx.csr.pmpaddr = [kani::any(); 64];
    ctx.csr.mhpmcounter=  [kani::any(); 29];
    ctx.csr.mhpmevent=  [kani::any(); 29];

    ctx
}


use miralis::arch::Csr;

#[cfg(kani)]
mod verification {
    use miralis::arch::Arch;
    use miralis::host::MiralisContext;
    use miralis::platform::Plat;

    use super::*;

    // #[kani::proof]
    pub fn mret() {
        /*let mpp = match kani::any::<u8>() % 3 {
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

        assert_eq!(ctx, sail_ctx.into_virt_context(), "mret equivalence");*/
    }

    #[kani::proof]
    pub fn read_csr() {
        
        let csr_register = 0b111100010010; // kani::any::<u64>() & ((1<<13) - 1);

        let mut ctx = KaniVirtCtx();

        let mut sail_ctx = SailVirtCtx::from(&mut ctx);
        sail::readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register));

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        let decoded_csr  = mctx.decode_csr(csr_register as usize);
        ctx.csr.marchid = ctx.get(decoded_csr);

        assert_eq!(ctx.csr, sail_ctx.into_virt_context().csr, "read csr equivalence");
    }
}
