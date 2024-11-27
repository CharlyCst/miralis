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
        sail_ctx.misa = BitField::new(ctx.csr.misa as u64);
        sail_ctx.mie = BitField::new(ctx.csr.mie as u64);
        sail_ctx.mip = BitField::new(ctx.csr.mip as u64);
        sail_ctx.mtvec = BitField::new(ctx.csr.mtvec as u64);
        sail_ctx.mscratch = BitVector::new(ctx.csr.mscratch as u64);
        sail_ctx.mvendorid = BitVector::new(ctx.csr.mvendorid as u64);
        sail_ctx.marchid = BitVector::new(ctx.csr.marchid as u64);
        sail_ctx.mimpid = BitVector::new(ctx.csr.mimpid as u64);
        /*sail_ctx.mcycle = BitVector::new(ctx.csr.mcycle as u64);
        sail_ctx.minstret = BitVector::new(ctx.csr.minstret as u64);
        sail_ctx.mcountinhibit = BitField::new(ctx.csr.mcountinhibit as u64);
        sail_ctx.mcounteren = BitField::new(ctx.csr.mcounteren as u64);
        sail_ctx.menvcfg = BitField::new(ctx.csr.menvcfg as u64);
        // sail_ctx.mseccfg = BitField::new(ctx.csr.mseccfg as u64);
        sail_ctx.mcause = BitField::new(ctx.csr.mcause as u64);

        sail_ctx.mepc = BitVector::new(ctx.csr.mepc as u64);
        sail_ctx.mtval = BitVector::new(ctx.csr.mtval as u64);
        // sail_ctx.mtval2 = BitField::new(ctx.csr.mtval2 as u64);
        sail_ctx.mstatus = BitField::new(ctx.csr.mstatus as u64);
        // sail_ctx.mtinst = BitField::new(ctx.csr.mtinst as u64);

        sail_ctx.mconfigptr = BitVector::new(ctx.csr.mconfigptr as u64);
        sail_ctx.stvec = BitField::new(ctx.csr.stvec as u64);
        sail_ctx.scounteren = BitField::new(ctx.csr.scounteren as u64);
        sail_ctx.senvcfg = BitField::new(ctx.csr.senvcfg as u64);

        sail_ctx.sscratch = BitVector::new(ctx.csr.sscratch as u64);
        sail_ctx.sepc = BitVector::new(ctx.csr.sepc as u64);
        sail_ctx.scause = BitField::new(ctx.csr.scause as u64);
        sail_ctx.stval = BitVector::new(ctx.csr.stval as u64);
        sail_ctx.satp = BitVector::new(ctx.csr.satp as u64);
        // sail_ctx.scontext = BitField::new(ctx.csr.scontext as u64);
        sail_ctx.medeleg = BitField::new(ctx.csr.medeleg as u64);
        sail_ctx.mideleg = BitField::new(ctx.csr.mideleg as u64);

        // ctx.csr.pmpcfg = [kani::any(); 8]; 
        // ctx.csr.pmpaddr = [kani::any(); 64];
        // ctx.csr.mhpmcounter=  [kani::any(); 29];
        // ctx.csr.mhpmevent=  [kani::any(); 29];*/

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
        ctx.csr.misa= self.misa.bits.bits() as usize;
        ctx.csr.mie= self.mie.bits.bits() as usize;
        ctx.csr.mip= self.mip.bits.bits() as usize;
        ctx.csr.mtvec=self.mtvec.bits.bits() as usize;
        ctx.csr.mscratch=self.mscratch.bits() as usize;
        ctx.csr.mvendorid= self.mvendorid.bits() as usize;
        ctx.csr.marchid= self.marchid.bits() as usize;
        ctx.csr.mimpid= self.mimpid.bits() as usize;
        /*ctx.csr.mcycle= self.mcycle.bits() as usize;
        ctx.csr.minstret= self.minstret.bits() as usize;
        ctx.csr.mcountinhibit= self.mcountinhibit.bits.bits() as usize;
        ctx.csr.mcounteren= self.mcounteren.bits.bits() as usize;
        ctx.csr.menvcfg= self.menvcfg.bits.bits() as usize;
        // ctx.csr.mseccfg= self.mseccfg.bits.bits() as usize;
        ctx.csr.mcause= self.mcause.bits.bits() as usize;
        ctx.csr.mepc= self.mepc.bits() as usize;
        ctx.csr.mtval= self.mtval.bits() as usize;ext_write_CSR
        ctx.csr.mconfigptr= self.mconfigptr.bits() as usize;
        ctx.csr.stvec= self.stvec.bits.bits() as usize;
        ctx.csr.scounteren= self.scounteren.bits.bits() as usize;
        ctx.csr.senvcfg= self.senvcfg.bits.bits() as usize;
        ctx.csr.sscratch= self.sscratch.bits() as usize;
        ctx.csr.sepc= self.sepc.bits() as usize;
        ctx.csr.scause= self.scause.bits.bits() as usize;
        ctx.csr.stval= self.stval.bits() as usize;
        ctx.csr.satp= self.satp.bits() as usize;
        // ctx.csr.scontext= self.scontext.bits.bits() as usize;
        ctx.csr.medeleg= self.medeleg.bits.bits() as usize;
        ctx.csr.mideleg = self.mideleg.bits.bits() as usize;
        // ctx.csr.pmpcfg = [kani::any(); 8]; 
        // ctx.csr.pmpaddr = [kani::any(); 64];
        // ctx.csr.mhpmcounter=  [kani::any(); 29];
        // ctx.csr.mhpmevent=  [kani::any(); 29];*/

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
    ctx.csr.mscratch=kani::any();
    ctx.csr.mvendorid= kani::any();
    ctx.csr.marchid = 0;
    // ctx.csr.mimpid= kani::any();
    /*ctx.csr.mcycle= kani::any();
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
    ctx.csr.pmpcfg = [kani::any(); 8]; 
    ctx.csr.pmpaddr = [kani::any(); 64];
    ctx.csr.mhpmcounter=  [kani::any(); 29];
    ctx.csr.mhpmevent=  [kani::any(); 29];*/

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

        assert_eq!(ctx, sail_ctx.into_virt_context(), "mret equivalence");
    }

    fn filter_possible_instructions(value: usize) {




    }

    #[kani::proof]
    pub fn read_csr() {
        
        let csr_register = 0b111100010011; // kani::any::<u64>() & ((1<<13) - 1);

        let mut ctx = KaniVirtCtx();

        let mut sail_ctx = SailVirtCtx::from(&mut ctx);
        sail::readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register));

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        let decoded_csr  = mctx.decode_csr(csr_register as usize);
        ctx.csr.marchid = ctx.get(decoded_csr);

        assert_eq!(ctx.csr.misa, sail_ctx.into_virt_context().csr.misa, "read csr equivalence");
        assert_eq!(ctx.csr.mie, sail_ctx.into_virt_context().csr.mie, "read csr equivalence");
        assert_eq!(ctx.csr.mip, sail_ctx.into_virt_context().csr.mip, "read csr equivalence");
        assert_eq!(ctx.csr.mtvec, sail_ctx.into_virt_context().csr.mtvec, "read csr equivalence");
        assert_eq!(ctx.csr.marchid, sail_ctx.into_virt_context().csr.marchid, "read csr equivalence");
        assert_eq!(ctx.csr.mimpid, sail_ctx.into_virt_context().csr.mimpid, "read csr equivalence");
        assert_eq!(ctx.csr.mcycle, sail_ctx.into_virt_context().csr.mcycle, "read csr equivalence");
        assert_eq!(ctx.csr.minstret, sail_ctx.into_virt_context().csr.minstret, "read csr equivalence");
        assert_eq!(ctx.csr.mcountinhibit, sail_ctx.into_virt_context().csr.mcountinhibit, "read csr equivalence");
        assert_eq!(ctx.csr.mcounteren, sail_ctx.into_virt_context().csr.mcounteren, "read csr equivalence");
        assert_eq!(ctx.csr.menvcfg, sail_ctx.into_virt_context().csr.menvcfg, "read csr equivalence");
        assert_eq!(ctx.csr.mseccfg, sail_ctx.into_virt_context().csr.mseccfg, "read csr equivalence");
        assert_eq!(ctx.csr.mcause, sail_ctx.into_virt_context().csr.mcause, "read csr equivalence");
        assert_eq!(ctx.csr.mepc, sail_ctx.into_virt_context().csr.mepc, "read csr equivalence");
        assert_eq!(ctx.csr.mtval, sail_ctx.into_virt_context().csr.mtval, "read csr equivalence");
        assert_eq!(ctx.csr.mtval2, sail_ctx.into_virt_context().csr.mtval2, "read csr equivalence");
        assert_eq!(ctx.csr.mstatus, sail_ctx.into_virt_context().csr.mstatus, "read csr equivalence");
        assert_eq!(ctx.csr.mtinst, sail_ctx.into_virt_context().csr.mtinst, "read csr equivalence");
        assert_eq!(ctx.csr.mconfigptr, sail_ctx.into_virt_context().csr.mconfigptr, "read csr equivalence");
        assert_eq!(ctx.csr.stvec, sail_ctx.into_virt_context().csr.stvec, "read csr equivalence");
        assert_eq!(ctx.csr.scounteren, sail_ctx.into_virt_context().csr.scounteren, "read csr equivalence");
        assert_eq!(ctx.csr.senvcfg, sail_ctx.into_virt_context().csr.senvcfg, "read csr equivalence");
        assert_eq!(ctx.csr.sscratch, sail_ctx.into_virt_context().csr.sscratch, "read csr equivalence");
        assert_eq!(ctx.csr.sepc, sail_ctx.into_virt_context().csr.sepc, "read csr equivalence");
        assert_eq!(ctx.csr.scause, sail_ctx.into_virt_context().csr.scause, "read csr equivalence");
        assert_eq!(ctx.csr.stval, sail_ctx.into_virt_context().csr.stval, "read csr equivalence");
        assert_eq!(ctx.csr.satp, sail_ctx.into_virt_context().csr.satp, "read csr equivalence");
        assert_eq!(ctx.csr.scontext, sail_ctx.into_virt_context().csr.scontext, "read csr equivalence");
        assert_eq!(ctx.csr.medeleg, sail_ctx.into_virt_context().csr.medeleg, "read csr equivalence");
        assert_eq!(ctx.csr.mideleg, sail_ctx.into_virt_context().csr.mideleg, "read csr equivalence");
        assert_eq!(ctx.csr.pmpcfg, sail_ctx.into_virt_context().csr.pmpcfg, "read csr equivalence");
        assert_eq!(ctx.csr.pmpaddr, sail_ctx.into_virt_context().csr.pmpaddr, "read csr equivalence");
        assert_eq!(ctx.csr.mhpmcounter, sail_ctx.into_virt_context().csr.mhpmcounter, "read csr equivalence");
        assert_eq!(ctx.csr.mhpmevent, sail_ctx.into_virt_context().csr.mhpmevent, "read csr equivalence");
    }
}
