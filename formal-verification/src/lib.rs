pub mod sail;

use miralis::arch::{mie, mstatus, Architecture, ExtensionsCapability, Mode};
use miralis::platform::Platform;
use miralis::virt::traits::RegisterContextGetter;
use miralis::virt::VirtContext;
use sail::Privilege;
use sail_prelude::{BitField, BitVector};
use miralis::virt::traits::RegisterContextSetter;


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

        // Transfer hart id
        sail_ctx.mhartid = BitVector::new(ctx.hart_id as u64);

        // Transfer all csr
        sail_ctx.misa = BitField::new(ctx.csr.misa as u64);
        sail_ctx.mie = BitField::new(ctx.csr.mie as u64);
        sail_ctx.mip = BitField::new(ctx.csr.mip as u64);
        sail_ctx.mtvec = BitField::new(ctx.csr.mtvec as u64);
        sail_ctx.mscratch = BitVector::new(ctx.csr.mscratch as u64);
        sail_ctx.mvendorid = BitVector::new(ctx.csr.mvendorid as u64);
        sail_ctx.marchid = BitVector::new(ctx.csr.marchid as u64);
        sail_ctx.mimpid = BitVector::new(ctx.csr.mimpid as u64);
        sail_ctx.mcycle = BitVector::new(ctx.csr.mcycle as u64);
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

        // Transfer hart id
        ctx.hart_id = self.mhartid.bits() as usize;

        // Transfer all csr
        ctx.csr.misa = self.misa.bits.bits() as usize;
        ctx.csr.mie = self.mie.bits.bits() as usize;
        ctx.csr.mip = self.mip.bits.bits() as usize;
        ctx.csr.mtvec = self.mtvec.bits.bits() as usize;
        ctx.csr.mscratch = self.mscratch.bits() as usize;
        ctx.csr.mvendorid = self.mvendorid.bits() as u32;
        ctx.csr.marchid = self.marchid.bits() as usize;
        ctx.csr.mimpid = self.mimpid.bits() as usize;
        ctx.csr.mcycle = self.mcycle.bits() as usize;
        ctx.csr.minstret = self.minstret.bits() as usize;
        ctx.csr.mcountinhibit = self.mcountinhibit.bits.bits() as u32;
        ctx.csr.mcounteren = self.mcounteren.bits.bits() as u32;
        ctx.csr.menvcfg = self.menvcfg.bits.bits() as usize;
        // ctx.csr.mseccfg= self.mseccfg.bits.bits() as usize;
        ctx.csr.mcause = self.mcause.bits.bits() as usize;
        ctx.csr.mepc = self.mepc.bits() as usize;
        ctx.csr.mtval = self.mtval.bits() as usize;
        ctx.csr.mconfigptr = self.mconfigptr.bits() as usize;
        ctx.csr.stvec = self.stvec.bits.bits() as usize;
        ctx.csr.scounteren = self.scounteren.bits.bits() as u32;
        ctx.csr.senvcfg = self.senvcfg.bits.bits() as usize;
        ctx.csr.sscratch = self.sscratch.bits() as usize;
        ctx.csr.sepc = self.sepc.bits() as usize;
        ctx.csr.scause = self.scause.bits.bits() as usize;
        ctx.csr.stval = self.stval.bits() as usize;
        ctx.csr.satp = self.satp.bits() as usize;
        // ctx.csr.scontext= self.scontext.bits.bits() as usize;
        ctx.csr.medeleg = self.medeleg.bits.bits() as usize;
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

    // Add hart id
    ctx.hart_id = kani::any();

    // Add other csr
    ctx.csr.misa = kani::any();
    ctx.csr.mie = kani::any();
    ctx.csr.mip = kani::any();
    ctx.csr.mtvec = kani::any();
    ctx.csr.mscratch = kani::any();
    ctx.csr.mvendorid = kani::any();
    ctx.csr.marchid = kani::any();
    // ctx.csr.mimpid= kani::any();
    ctx.csr.mcycle = kani::any();
    ctx.csr.minstret = kani::any();
    ctx.csr.mcountinhibit = kani::any();
    ctx.csr.mcounteren = kani::any();
    ctx.csr.menvcfg = kani::any();
    // ctx.csr.mseccfg= kani::any();
    ctx.csr.mcause = kani::any();
    ctx.csr.mepc = kani::any();
    ctx.csr.mtval = kani::any();
    // ctx.csr.mtval2= kani::any(); - TODO: What should we do with this?
    // ctx.csr.mstatus= kani::any();
    // ctx.csr.mtinst= kani::any();
    ctx.csr.mconfigptr = kani::any();
    // ctx.csr.stvec= kani::any();
    ctx.csr.scounteren = kani::any();
    ctx.csr.senvcfg= kani::any();
    ctx.csr.sscratch= kani::any();
    ctx.csr.sepc= kani::any();
    ctx.csr.scause= kani::any();
    ctx.csr.stval= kani::any();
    ctx.csr.satp= kani::any();
    //ctx.csr.scontext= kani::any(); // TODO: What should we do with this?
    ctx.csr.medeleg= kani::any();
    ctx.csr.mideleg = mie::MIDELEG_READ_ONLY_ONE;
    // ctx.csr.pmpcfg = [kani::any(); 8];
    // ctx.csr.pmpaddr = [kani::any(); 64];
    // ctx.csr.mhpmcounter=  [kani::any(); 29];
    // ctx.csr.mhpmevent=  [kani::any(); 29];

    ctx
}

use miralis::arch::Csr;

#[cfg(kani)]
mod verification {
    use miralis::arch::Arch;
    use miralis::host::MiralisContext;
    use miralis::platform::Plat;
    use sail_prelude::{bitvector_access, bitvector_concat, subrange_bits, zero_extend_16, zero_extend_64};

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


    // #[kani::proof]
    pub fn read_csr() {
        let mut csr_register = kani::any::<u64>() & ((1<<13) - 1);
        
        // tmp filtering of the registers
        let csr_register = match csr_register {
            0b111100010001 => 0b111100010001,
            0b111100010011 => 0b111100010011,
            0b111100010100 => 0b111100010100,
            0b111100010101 => 0b111100010101, 
            0b001100000000 => 0b001100000000,
            0b001100000001 => 0b001100000001,
            0b001100000010 => 0b001100000010,
            0b001100000011 => 0b001100000011,
            0b001100000100 => 0b001100000100,
            0b001100000101 => 0b001100000101,
            0b001100000110 => 0b001100000110,
            0b001100001010 => 0b001100001010,
            0b001100100000 => 0b001100100000,
            0b001101000000 => 0b001101000000,
            // 0b001101000001 => 0b001101000001,  - TODO: Bug found in MEPC - fix it
            0b001101000010 => 0b001101000010,
            0b001101000011 => 0b001101000011,
            0b001101000100 => 0b001101000100,
            // Tested until here
            0b101100000000 => 0b101100000000,
            0b101100000010 => 0b101100000010,
            // 0b011110100000 => 0b011110100000, - TODO: Not equivalent - fix it - add tselect
            0b000100000000 => 0b000100000000,
            // 0b000100000010 => 0b000100000010, - TODO: Not equivalent - fix sedeleg
            // 0b000100000011 => 0b000100000011, - TODO: Not equivalent - fix sideleg
            // 0b000100000100 => 0b000100000100, - TODO: Not equivalent - fix sie
            0b000100000101 => 0b000100000101,
            0b000100000110 => 0b000100000110,
            0b000100001010 => 0b000100001010,
            0b000101000000 => 0b000101000000,
            // 0b000101000001 => 0b000101000001, - TODO: Not equivalent - fix sepc
            0b000101000010 => 0b000101000010,
            0b000101000011 => 0b000101000011,
            // 0b000101000100 => 0b000101000100, - TODO: Not equivalent - fix sip
            0b000110000000 => 0b000110000000,
            // 0b000000010101 => 0b000000010101, - TODO: Not equivalent - fix seed register
            // 0b000000001000 => 0b000000001000, - TODO: Not equivalent - fix vstart
            // 0b000000001001 => 0b000000001001, - TODO: Not equivalent - fix vxstat
            // 0b000000001010 => 0b000000001010, fix vxrm
            // 0b000000001111 => 0b000000001111, fix vcsr
            // 0b110000100000 => 0b110000100000, fix vl
            // 0b110000100001 => 0b110000100001, fix vtype
            // 0b110000100010 => 0b110000100010, fix vlenb

            _ => 0b111100010001, // Default take mvendor id
        };

        // TODO : Handle decoding of v__22 and friends

        let mut ctx = KaniVirtCtx();
        let mut sail_ctx = SailVirtCtx::from(&mut ctx);

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        let decoded_csr = mctx.decode_csr(csr_register as usize);

        // Decoded miralis value
        let decoded_miralis_value = ctx.get(decoded_csr); 
        // Decoded sail value
        let decoded_sail_value = sail::readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register)).bits as usize;

        // Verify value is the same
        assert_eq!(decoded_miralis_value,decoded_sail_value , "Read equivalence");
    }

    // TODO: Implement write csr as next step
    #[kani::proof]
    pub fn write_csr() {
        let mut csr_register = kani::any::<u64>() & ((1<<13) - 1);

        // tmp filtering of the registers
        let csr_register = match csr_register {
            0b111100010001 => 0b111100010001,
            0b111100010011 => 0b111100010011,
            0b111100010100 => 0b111100010100,
            0b111100010101 => 0b111100010101,
            0b001100000000 => 0b001100000000,
            0b001100000001 => 0b001100000001,
            0b001100000010 => 0b001100000010,
            0b001100000011 => 0b001100000011,
            0b001100000100 => 0b001100000100,
            0b001100000101 => 0b001100000101,
            0b001100000110 => 0b001100000110,
            0b001100001010 => 0b001100001010,
            0b001100100000 => 0b001100100000,
            0b001101000000 => 0b001101000000,
            // 0b001101000001 => 0b001101000001,  - TODO: Bug found in MEPC - fix it
            0b001101000010 => 0b001101000010,
            0b001101000011 => 0b001101000011,
            0b001101000100 => 0b001101000100,
            // Tested until here
            0b101100000000 => 0b101100000000,
            0b101100000010 => 0b101100000010,
            // 0b011110100000 => 0b011110100000, - TODO: Not equivalent - fix it - add tselect
            0b000100000000 => 0b000100000000,
            // 0b000100000010 => 0b000100000010, - TODO: Not equivalent - fix sedeleg
            // 0b000100000011 => 0b000100000011, - TODO: Not equivalent - fix sideleg
            // 0b000100000100 => 0b000100000100, - TODO: Not equivalent - fix sie
            0b000100000101 => 0b000100000101,
            0b000100000110 => 0b000100000110,
            0b000100001010 => 0b000100001010,
            0b000101000000 => 0b000101000000,
            // 0b000101000001 => 0b000101000001, - TODO: Not equivalent - fix sepc
            0b000101000010 => 0b000101000010,
            0b000101000011 => 0b000101000011,
            // 0b000101000100 => 0b000101000100, - TODO: Not equivalent - fix sip
            0b000110000000 => 0b000110000000,
            // 0b000000010101 => 0b000000010101, - TODO: Not equivalent - fix seed register
            // 0b000000001000 => 0b000000001000, - TODO: Not equivalent - fix vstart
            // 0b000000001001 => 0b000000001001, - TODO: Not equivalent - fix vxstat
            // 0b000000001010 => 0b000000001010, fix vxrm
            // 0b000000001111 => 0b000000001111, fix vcsr
            // 0b110000100000 => 0b110000100000, fix vl
            // 0b110000100001 => 0b110000100001, fix vtype
            // 0b110000100010 => 0b110000100010, fix vlenb

            _ => 0b111100010001, // Default take mvendor id
        };

        // TODO : Handle decoding of v__22 and friends

        let mut ctx = KaniVirtCtx();
        let mut sail_ctx = SailVirtCtx::from(&mut ctx);

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        let decoded_csr = mctx.decode_csr(csr_register as usize);

        let value_to_write: usize = kani::any();

        // Write register in Miralis context
        // ctx.set(decoded_csr, value_to_write);

        sail::writeCSR(&mut sail_ctx, BitVector::<12>::new(csr_register), BitVector::<64>::new(value_to_write as u64));

        // Verify value is the same
        // assert_eq!(decoded_miralis_value,decoded_sail_value , "Write equivalence");
    }
}
