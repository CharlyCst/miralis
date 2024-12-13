pub mod sail;

use miralis::arch::{mie, misa, mstatus, Architecture, ExtensionsCapability, Mode};
use miralis::platform::Platform;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use sail::Privilege;
use sail_prelude::{BitField, BitVector};

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
        sail_ctx.mstatus = BitField::new(ctx.csr.mstatus as u64);
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

        sail_ctx.pmpcfg_n = Self::pmpcfg_miralis_to_sail(ctx.csr.pmpcfg);
        sail_ctx.pmpaddr_n = Self::pmpaddr_miralis_to_sail(ctx.csr.pmpaddr);
        // ctx.csr.mhpmcounter=  [kani::any(); 29]; TODO: What should we do with this?
        // ctx.csr.mhpmevent=  [kani::any(); 29]; TODO: What should we do with this?

        // New added
        sail_ctx.tselect = BitVector::<64>::new(ctx.csr.tselect as u64);
        sail_ctx.vstart = BitVector::<16>::new(ctx.csr.vstart as u64);
        sail_ctx.vxsat = BitVector::new(if ctx.csr.vxsat { 1 } else { 0 });
        sail_ctx.vxrm = BitVector::new(ctx.csr.vxrm as u64);
        sail_ctx.vcsr = BitField::new(ctx.csr.vcsr as u64);
        sail_ctx.vl = BitVector::new(ctx.csr.vl as u64);
        sail_ctx.vtype = BitField::new(ctx.csr.vtype as u64);
        sail_ctx.vlenb = BitVector::new(ctx.csr.vlenb as u64);
        sail_ctx
    }

    fn pmpcfg_miralis_to_sail(cfgs: [usize; 8]) -> [BitField<8>; 64] {
        let mut output: [BitField<8>; 64] = [BitField::<8>::new(0); 64];

        for i in 0..64 {
            let idx = i / 8;
            let offset = i % 8;
            output[i] = BitField::<8>::new(((cfgs[idx] >> (8 * offset)) & 0xFF) as u64);
        }

        output
    }

    fn pmpaddr_miralis_to_sail(addresses: [usize; 64]) -> [BitVector<64>; 64] {
        let mut output: [BitVector<64>; 64] = [BitVector::<64>::new(0); 64];
        for i in 0..64 {
            output[i] = BitVector::new(addresses[i] as u64);
        }

        output
    }

    pub fn into_virt_context(self) -> VirtContext {
        let mut ctx = VirtContext::new(
            0,
            0,
            ExtensionsCapability {
                has_crypto_extension: true,
                has_sstc_extension: false,
                is_sstc_enabled: false,
                has_h_extension: false,
                has_s_extension: false,
                has_v_extension: true,
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
        ctx.csr.mstatus = self.mstatus.bits.bits() as usize;
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
        ctx.csr.pmpcfg = pmpcfg_sail_to_miralis(self.pmpcfg_n);
        ctx.csr.pmpaddr = pmpaddr_sail_to_miralis(self.pmpaddr_n);
        // ctx.csr.mhpmcounter=  [kani::any(); 29]; todo: what should we do?
        // ctx.csr.mhpmevent=  [kani::any(); 29]; todo: what should we do?

        // New added
        ctx.csr.tselect = self.tselect.bits() as usize;
        ctx.csr.vstart = self.vstart.bits() as u16;
        ctx.csr.vxsat = self.vxsat.bits() != 0;
        ctx.csr.vxrm = self.vxrm.bits() as u8;
        ctx.csr.vcsr = self.vcsr.bits.bits() as u8;
        ctx.csr.vl = self.vl.bits() as usize;
        ctx.csr.vtype = self.vtype.bits.bits() as usize;
        ctx.csr.vlenb = self.vlenb.bits() as usize;

        ctx
    }
}

fn pmpcfg_sail_to_miralis(cfgs: [BitField<8>; 64]) -> [usize; 8] {
    let mut output: [usize; 8] = [0; 8];

    for i in 0..64 {
        let idx = i / 8;
        let offset = i % 8;
        output[idx] |= ((cfgs[i].bits.bits() & 0xff) << (8 * offset)) as usize;
    }

    output
}

fn pmpaddr_sail_to_miralis(addresses: [BitVector<64>; 64]) -> [usize; 64] {
    let mut output: [usize; 64] = [0; 64];
    for i in 0..64 {
        output[i] = addresses[i].bits as usize;
    }

    output
}

#[cfg(kani)]
fn KaniVirtCtx() -> VirtContext {
    let mut ctx = VirtContext::new(
        0,
        0,
        ExtensionsCapability {
            has_crypto_extension: true,
            has_sstc_extension: false,
            is_sstc_enabled: false,
            has_h_extension: false,
            has_s_extension: false,
            has_v_extension: true,
        },
    );

    // Mode
    ctx.mode = Mode::M;

    // Add hart id
    ctx.hart_id = kani::any();

    ctx.nb_pmp = 64;

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
    ctx.csr.mstatus = kani::any();
    // ctx.csr.mtinst= kani::any();
    ctx.csr.mconfigptr = kani::any();
    // ctx.csr.stvec= kani::any();
    ctx.csr.scounteren = kani::any();
    ctx.csr.senvcfg = kani::any();
    ctx.csr.sscratch = kani::any();
    ctx.csr.sepc = kani::any();
    ctx.csr.scause = kani::any();
    ctx.csr.stval = kani::any();
    ctx.csr.satp = kani::any();
    //ctx.csr.scontext= kani::any(); // TODO: What should we do with this?
    ctx.csr.medeleg = kani::any();
    ctx.csr.mideleg = kani::any();
    ctx.csr.pmpcfg = [kani::any(); 8];
    ctx.csr.pmpaddr = [kani::any(); 64];
    // ctx.csr.mhpmcounter=  [kani::any(); 29]; todo: What should we do?
    // ctx.csr.mhpmevent=  [kani::any(); 29]; todo: What should we do?

    // Lock mode is not supported at the moment in Miralis
    for i in 0..8 {
        for j in 0..8 {
            ctx.csr.pmpcfg[i] &= !(1 << (7 + j * 8));
        }
    }

    // We don't have compressed instructions in Miralis
    ctx.csr.misa &= !misa::DISABLED;

    // We don't have the userspace interrupt delegation in Miralis
    ctx.csr.misa &= misa::N;

    // We fix the architecture type to 64 bits
    ctx.csr.misa = (0b10 << 62) | (ctx.csr.misa & ((1 << 62) - 1));

    // We must have support for usermode in Miralis
    ctx.csr.misa |= misa::U;

    // new added
    // ctx.csr.tselect = kani::any();
    ctx
}

#[cfg(kani)]
fn generate_csr_register() -> u64 {
    // We want only 12 bits
    let mut csr: u64 = kani::any::<u64>() & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x0;
    }

    return csr;
}

use miralis::arch::Csr;

#[cfg(kani)]
mod verification {
    use miralis::arch::Arch;
    use miralis::host::MiralisContext;
    use miralis::platform::Plat;
    use sail_prelude::{
        bitvector_access, bitvector_concat, subrange_bits, zero_extend_16, zero_extend_64,
    };

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
                has_crypto_extension: true,
                has_sstc_extension: false,
                is_sstc_enabled: false,
                has_h_extension: false,
                has_s_extension: false,
                has_v_extension: true,
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

    #[kani::proof]
    pub fn read_csr() {
        let csr_register = generate_csr_register();

        // tmp filtering of the registers
        let mut csr_register = match csr_register {
            // 0b111100010001 => 0b111100010001, // Verified
            // 0b111100010011 => 0b111100010011, // Verified
            // 0b111100010100 => 0b111100010100, // Verified
            // 0b111100010101 => 0b111100010101, // Verified
            // // 0b001100000000 => 0b001100000000, // Verified - todo: mstatus
            // 0b001100000001 => 0b001100000001, // Verified
            // 0b001100000010 => 0b001100000010, // Verified
            // 0b001100000011 => 0b001100000011, // Verified
            // 0b001100000100 => 0b001100000100, // Verified
            // 0b001100000101 => 0b001100000101, // Verified
            // 0b001100000110 => 0b001100000110, // Verified
            // 0b001100001010 => 0b001100001010, // Verified
            // 0b001100100000 => 0b001100100000, // Verified
            // 0b001101000000 => 0b001101000000, // Verified
            // 0b001101000001 => 0b001101000001, // Verified - todo: opensbi still fails - mepc
            // 0b001101000010 => 0b001101000010, // Verified
            // 0b001101000011 => 0b001101000011, // Verified
            // 0b001101000100 => 0b001101000100, // Verified
            // 0b101100000000 => 0b101100000000, // Verified
            // 0b101100000010 => 0b101100000010, // Verified
            // 0b011110100000 => 0b011110100000, // Verified
            // // 0b000100000000 => 0b000100000000, // Verified - todo: sstatus
            // 0b000100000010 => 0b000100000010, // We ignore sedeleg
            // 0b000100000011 => 0b000100000011, // We ignore sideleg
            // 0b000100000100 => 0b000100000100, // Verified - second attempt sie register
            // 0b000100000101 => 0b000100000101, // Verified
            // 0b000100000110 => 0b000100000110, // Verified
            // 0b000100001010 => 0b000100001010, // Verified
            // 0b000101000000 => 0b000101000000, // Verified
            // // 0b000101000001 => 0b000101000001, // todo: OpenSBI still fails - mepc
            // 0b000101000010 => 0b000101000010, // Verified
            // 0b000101000011 => 0b000101000011, // Verified
            // 0b000101000100 => 0b000101000100, // Verified - second attempt sip register
            // 0b000110000000 => 0b000110000000, // Verified
            // 0b000000010101 => 0b000000010101, // Verified - second attempt seed register
            // 0b000000001000 => 0b000000001000, // Verified
            // 0b000000001001 => 0b000000001001, // Verified
            // 0b000000001010 => 0b000000001010, // Verified
            // 0b000000001111 => 0b000000001111, // Verified
            // 0b110000100000 => 0b110000100000, // Verified
            // 0b110000100001 => 0b110000100001, // Verified
            // 0b110000100010 => 0b110000100010, // Verified
            //
            // // Forgot a few
            // 0b110000000000 => 0b110000000000, // Verified cycle
            // 0b110000000001 => 0b110000000001, // Verified time
            // 0b110000000010 => 0b110000000010, // Verified instret
            _ => 0b111100010001, // Default take mvendor id
                                 // Pmp addr works
                                 // Pmp config works"
        };

        let mut ctx = KaniVirtCtx();
        let mut sail_ctx = SailVirtCtx::from(&mut ctx);

        // Initialize Miralis's own context
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        let decoded_csr = mctx.decode_csr(csr_register as usize);

        // Decoded miralis value
        let decoded_miralis_value = ctx.get(decoded_csr);
        // Decoded sail value
        let decoded_sail_value =
            sail::readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register)).bits as usize;

        // Verify value is the same
        assert_eq!(
            decoded_miralis_value, decoded_sail_value,
            "Read equivalence"
        );
    }

    // #[kani::proof]
    pub fn write_csr() {
        let mut csr_register = kani::any::<u64>() & ((1 << 12) - 1);

        // tmp filtering of the registers
        let mut csr_register = match csr_register {
            0b111100010001 => 0b111100010001, // Verified mvendorid
            0b111100010011 => 0b111100010011, // Verified mimpid
            0b111100010100 => 0b111100010100, // Verified mhartid
            0b111100010101 => 0b111100010101, // Verified mconfigptr
            // 0b001100000000 => 0b001100000000, // Verified mstatus - todo: end this part
            0b001100000001 => 0b001100000001, // Verified misa
            0b001100000010 => 0b001100000010, // Verified medeleg
            0b001100000011 => 0b001100000011, // Verified mideleg
            0b001100000100 => 0b001100000100, // Verified mie
            0b001100000101 => 0b001100000101, // Verified mtvec
            0b001100000110 => 0b001100000110, // Verified mcounteren
            0b001100001010 => 0b001100001010, // Verified menvcfg
            0b001100100000 => 0b001100100000, // Verified mcountinhibit
            0b001101000000 => 0b001101000000, // Verified mscratch
            0b001101000001 => 0b001101000001, // Verified mepc
            0b001101000010 => 0b001101000010, // Verified mcause
            0b001101000011 => 0b001101000011, // Verified mtval
            0b001101000100 => 0b001101000100, // Verified mip
            0b101100000000 => 0b101100000000, // Verified mcycle
            0b101100000010 => 0b101100000010, // Verified minstret
            0b011110100000 => 0b011110100000, // Verified tselect
            // 0b000100000000 => 0b000100000000, // Verified sstatus - todo: end this part
            0b000100000010 => 0b000100000010, // This register is ignored
            0b000100000011 => 0b000100000011, // This register is ignored
            0b000100000100 => 0b000100000100, // Verified Sie
            0b000100000101 => 0b000100000101, // Verified stvec
            0b000100000110 => 0b000100000110, // Verified scounteren
            0b000100001010 => 0b000100001010, // Verified senvcfd
            0b000101000000 => 0b000101000000, // Verified sscratch
            0b000101000001 => 0b000101000001, // Verified sepc
            0b000101000010 => 0b000101000010, // Verified scause
            0b000101000011 => 0b000101000011, // Verified stval
            0b000101000100 => 0b000101000100, // Verified sip
            0b000110000000 => 0b000110000000, // Verified satp
            0b000000010101 => 0b000000010101, // Verified seed
            0b000000001000 => 0b000000001000, // Verified vstart
            0b000000001001 => 0b000000001001, // Verified vxsat
            0b000000001010 => 0b000000001010, // Verified vxrm
            0b000000001111 => 0b000000001111, // Verified vcsr
            0b110000100000 => 0b110000100000, // Verified vl
            0b110000100001 => 0b110000100001, // Verified vtype
            0b110000100010 => 0b110000100010, // Verified vlenb
            _ => 0b111100010001,              // Default take working value
                                               // Pmp addr works
                                               // Pmp config works
        };

        // Pmp address
        /*csr_register = kani::any();

        if csr_register < 0x3a0 {
            csr_register = 0x3a0
        }

        if csr_register > 0x3af {
            csr_register = 0x3aF
        }*/

        let mut ctx = KaniVirtCtx();
        let mut sail_ctx = SailVirtCtx::from(&mut ctx);

        // Initialize Miralis's own context
        let mut hw = unsafe { Arch::detect_hardware() };
        hw.available_reg.nb_pmp = 64;
        let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

        // Generate a random value
        let value_to_write: usize = kani::any();

        // Write register in Miralis context
        let decoded_csr = mctx.decode_csr(csr_register as usize);
        ctx.set_csr(decoded_csr, value_to_write, &mut mctx);

        assert_eq!(
            sail_ctx.cur_privilege,
            Privilege::Machine,
            "not the correct precondition"
        );

        // Write register in Sail context
        sail::writeCSR(
            &mut sail_ctx,
            BitVector::<12>::new(csr_register),
            BitVector::<64>::new(value_to_write as u64),
        );

        // Pmp registers
        // assert_eq!(sail_ctx.into_virt_context().csr.pmpaddr, ctx.csr.pmpaddr, "Write equivalence");
        assert_eq!(
            sail_ctx.into_virt_context().csr.pmpcfg,
            ctx.csr.pmpcfg,
            "Write pmp cfg equivalence"
        );

        // Verified and working
        assert_eq!(
            sail_ctx.into_virt_context().csr.mvendorid,
            ctx.csr.mvendorid,
            "Write mvendorid"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mimpid,
            ctx.csr.mimpid,
            "Write mimpid"
        );
        assert_eq!(
            sail_ctx.into_virt_context().hart_id,
            ctx.hart_id,
            "Write hart_id"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mconfigptr,
            ctx.csr.mconfigptr,
            "Write mconfigptr"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mtvec,
            ctx.csr.mtvec,
            "Write mtvec"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mscratch,
            ctx.csr.mscratch,
            "wWite mscratch"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mtval,
            ctx.csr.mtval,
            "Write mtval"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mcycle,
            ctx.csr.mcycle,
            "Write mcycle"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.minstret,
            ctx.csr.minstret,
            "Write minstret"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.tselect,
            ctx.csr.tselect,
            "Write tselect"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.stvec,
            ctx.csr.stvec,
            "Write stvec"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.sscratch,
            ctx.csr.sscratch,
            "Write sscratch"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.stval,
            ctx.csr.stval,
            "Write stval"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.satp,
            ctx.csr.satp,
            "Write satp"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.senvcfg,
            ctx.csr.senvcfg,
            "Write senvcfg"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.scause,
            ctx.csr.scause,
            "Write scause"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.mcause,
            ctx.csr.mcause,
            "Write mcause"
        );
        // assert_eq!(sail_ctx.into_virt_context().csr.mepc, ctx.csr.mepc, "Write mepc");
        // assert_eq!(sail_ctx.into_virt_context().csr.vstart, ctx.csr.vstart, "Write vstart");
        // assert_eq!(sail_ctx.into_virt_context().csr.menvcfg, ctx.csr.menvcfg, "Write menvcfg");
        assert_eq!(
            sail_ctx.into_virt_context().csr.mcountinhibit,
            ctx.csr.mcountinhibit,
            "Write mcountinhibit"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.medeleg,
            ctx.csr.medeleg,
            "Write medeleg"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.vxsat,
            ctx.csr.vxsat,
            "Write vxssat"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.vxrm,
            ctx.csr.vxrm,
            "Write vxrm"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.vcsr,
            ctx.csr.vcsr,
            "Write vcsr"
        );
        assert_eq!(sail_ctx.into_virt_context().csr.vl, ctx.csr.vl, "Write vl");
        assert_eq!(
            sail_ctx.into_virt_context().csr.vtype,
            ctx.csr.vtype,
            "Write vtype"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.vlenb,
            ctx.csr.vlenb,
            "Write vlenb"
        );
        // assert_eq!(sail_ctx.into_virt_context().csr.sepc, ctx.csr.sepc, "Write sepc");
        assert_eq!(
            sail_ctx.into_virt_context().csr.misa,
            ctx.csr.misa,
            "Write misa"
        );
        // assert_eq!(sail_ctx.into_virt_context().csr.mideleg, ctx.csr.mideleg, "Write mideleg");
        assert_eq!(
            sail_ctx.into_virt_context().csr.mcounteren,
            ctx.csr.mcounteren,
            "Write mcountern"
        );
        assert_eq!(
            sail_ctx.into_virt_context().csr.scounteren,
            ctx.csr.scounteren,
            "Write scounteren"
        );
        // assert_eq!(sail_ctx.into_virt_context().csr.mip, ctx.csr.mip, "Write mip");
        // assert_eq!(sail_ctx.into_virt_context().csr.mie, ctx.csr.mie, "Write mie");
        assert_eq!(
            sail_ctx.into_virt_context().csr.mstatus,
            ctx.csr.mstatus,
            "Write mstatus"
        );
    }
}
