//! World Switch
//!
//! A world switch is a transition from the virtual firmware to the native payload, or vice-versa.

use super::{VirtContext, VirtCsr};
use crate::arch::pmp::pmpcfg;
use crate::arch::pmp::pmpcfg::NO_PERMISSIONS;
use crate::arch::{mie, mstatus, set_mpp, Arch, Architecture, Csr, Mode};
use crate::config::DELEGATE_PERF_COUNTER;
use crate::host::MiralisContext;

impl VirtContext {
    /// Loads the S-mode CSR registers into the physical registers configures M-mode registers for
    /// payload execution.
    ///
    /// # Safety
    ///
    /// This function changes the configuration of the hardware CSR registers. It assumes the
    /// hardware is under the full control of Miralis.
    pub unsafe fn switch_from_firmware_to_payload(&mut self, mctx: &mut MiralisContext) {
        let mut mstatus = self.csr.mstatus; // We need to set the next mode bits before mret
        VirtCsr::set_csr_field(
            &mut mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            self.mode.to_bits(),
        );

        if mctx.hw.available_reg.senvcfg {
            Arch::write_csr(Csr::Senvcfg, self.csr.senvcfg);
        }
        if mctx.hw.available_reg.henvcfg {
            Arch::write_csr(Csr::Henvcfg, self.csr.henvcfg);
        }
        if mctx.hw.available_reg.menvcfg {
            Arch::write_csr(Csr::Menvcfg, self.csr.menvcfg);
        }

        Arch::write_csr(Csr::Mstatus, mstatus & !mstatus::MIE_FILTER);
        Arch::write_csr(Csr::Mideleg, self.csr.mideleg);
        Arch::write_csr(Csr::Medeleg, self.csr.medeleg);
        Arch::write_csr(Csr::Mcounteren, self.csr.mcounteren as usize);

        // NOTE: `mip` mut be set _after_ `menvcfg`, because `menvcfg` might change which bits in
        // `mip` are writeable. For more information see the Sstc extension specification.
        Arch::write_csr(Csr::Mip, self.csr.mip);
        // NOTE: we do not enable Machine External Interrupts (MIE) as we can't emulate them
        // efficiently with the PLIC. This mean that firmware can't receive external interrupts for
        // now. In the future we should add the MIE bit if the virtual PLIC is enabled.
        Arch::write_csr(
            Csr::Mie,
            (self.csr.mie | mie::MIDELEG_READ_ONLY_ZERO) & !mie::MEIE_FILTER,
        );

        // If S extension is present - save the registers
        if mctx.hw.extensions.has_s_extension {
            Arch::write_csr(Csr::Stvec, self.csr.stvec);
            Arch::write_csr(Csr::Scounteren, self.csr.scounteren as usize);
            Arch::write_csr(Csr::Satp, self.csr.satp);
            Arch::write_csr(Csr::Sscratch, self.csr.sscratch);
            Arch::write_csr(Csr::Sepc, self.csr.sepc);
            Arch::write_csr(Csr::Scause, self.csr.scause);
            Arch::write_csr(Csr::Stval, self.csr.stval);
            if mctx.hw.extensions.is_sstc_enabled {
                Arch::write_csr(Csr::Stimecmp, self.csr.stimecmp);
            }
        }

        // If H extension is present - save the registers
        if mctx.hw.extensions.has_h_extension {
            Arch::write_csr(Csr::Hstatus, self.csr.hstatus);
            Arch::write_csr(Csr::Hedeleg, self.csr.hedeleg);
            Arch::write_csr(Csr::Hideleg, self.csr.hideleg);
            Arch::write_csr(Csr::Hvip, self.csr.hvip);
            Arch::write_csr(Csr::Hip, self.csr.hip);
            Arch::write_csr(Csr::Hie, self.csr.hie);
            Arch::write_csr(Csr::Hgeip, self.csr.hgeip);
            Arch::write_csr(Csr::Hgeie, self.csr.hgeie);
            Arch::write_csr(Csr::Hcounteren, self.csr.hcounteren);
            Arch::write_csr(Csr::Htval, self.csr.htval);
            Arch::write_csr(Csr::Htinst, self.csr.htinst);
            Arch::write_csr(Csr::Hgatp, self.csr.hgatp);

            Arch::write_csr(Csr::Vsstatus, self.csr.vsstatus);
            Arch::write_csr(Csr::Vsie, self.csr.vsie);
            Arch::write_csr(Csr::Vstvec, self.csr.vstvec);
            Arch::write_csr(Csr::Vsscratch, self.csr.vsscratch);
            Arch::write_csr(Csr::Vsepc, self.csr.vsepc);
            Arch::write_csr(Csr::Vscause, self.csr.vscause);
            Arch::write_csr(Csr::Vstval, self.csr.vstval);
            Arch::write_csr(Csr::Vsip, self.csr.vsip);
            Arch::write_csr(Csr::Vsatp, self.csr.vsatp);
        }

        // Load virtual PMP registers into Miralis's own registers
        mctx.pmp.load_with_offset(
            &self.csr.pmpaddr,
            &self.csr.pmpcfg,
            mctx.pmp.virt_pmp_offset,
            self.nb_pmp,
        );
        // Deny all addresses by default if at least one PMP is implemented
        if self.nb_pmp > 0 {
            let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
            mctx.pmp
                .set_napot(last_pmp_idx, 0, usize::MAX, NO_PERMISSIONS);
        }
    }

    /// Loads the S-mode CSR registers into the virtual context and install sensible values (mostly
    /// 0) for running the virtual firmware in U-mode.
    ///
    /// # Safety
    ///
    /// This function changes the configuration of the hardware CSR registers. It assumes the
    /// hardware is under the full control of Miralis.
    pub unsafe fn switch_from_payload_to_firmware(&mut self, mctx: &mut MiralisContext) {
        // Now save M-mode registers which are (partially) exposed as S-mode registers.
        // For mstatus we read the current value and clear the two MPP bits to jump into U-mode
        // (virtual firmware) during the next mret.

        self.csr.mstatus = self.csr.mstatus & !mstatus::SSTATUS_FILTER
            | Arch::read_csr(Csr::Mstatus) & mstatus::SSTATUS_FILTER;
        set_mpp(Mode::U);
        Arch::write_csr(Csr::Mideleg, 0); // Do not delegate any interrupts
        Arch::write_csr(Csr::Medeleg, 0); // Do not delegate any exceptions

        let mie_hw_bits = Arch::read_csr(Csr::Mie) & !(mie::MIDELEG_READ_ONLY_ZERO);
        let mie_sw_bits = self.csr.mie & mie::MIDELEG_READ_ONLY_ZERO;
        self.csr.mie = mie_hw_bits | mie_sw_bits;
        // NOTE: we do not enable Machine External Interrupts (MIE) as we can't emulate them
        // efficiently with the PLIC. This mean that firmware can't receive external interrupts for
        // now. In the future we should add the MIE bit if the virtual PLIC is enabled.
        Arch::write_csr(Csr::Mie, mie::MSIE_FILTER | mie::MTIE_FILTER);

        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        // MSIE, MTIE, and MEIE are virtualized by Miralis.
        let mip_hw_bits =
            Arch::read_csr(Csr::Mip) & !(mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        let mip_sw_bits = self.csr.mip & (mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        self.csr.mip = mip_hw_bits | mip_sw_bits;

        let delegate_perf_counter_mask: usize = if DELEGATE_PERF_COUNTER { 1 } else { 0 };

        self.csr.mcounteren = Arch::write_csr(Csr::Mcounteren, delegate_perf_counter_mask) as u32;

        if mctx.hw.available_reg.senvcfg {
            self.csr.senvcfg = Arch::write_csr(Csr::Senvcfg, 0);
        }
        if mctx.hw.available_reg.henvcfg {
            self.csr.henvcfg = Arch::write_csr(Csr::Henvcfg, 0);
        }
        if mctx.hw.available_reg.menvcfg {
            self.csr.menvcfg = Arch::write_csr(Csr::Menvcfg, 0);
        }

        // If S extension is present - save the registers
        if mctx.hw.extensions.has_s_extension {
            self.csr.stvec = Arch::write_csr(Csr::Stvec, 0);
            self.csr.scounteren =
                Arch::write_csr(Csr::Scounteren, delegate_perf_counter_mask) as u32;
            self.csr.satp = Arch::write_csr(Csr::Satp, 0);
            self.csr.sscratch = Arch::write_csr(Csr::Sscratch, 0);
            self.csr.sepc = Arch::write_csr(Csr::Sepc, 0);
            self.csr.scause = Arch::write_csr(Csr::Scause, 0);
            self.csr.stval = Arch::write_csr(Csr::Stval, 0);
            if mctx.hw.extensions.is_sstc_enabled {
                self.csr.stimecmp = Arch::write_csr(Csr::Stimecmp, 0);
            }
        }

        // If H extension is present - save the registers
        if mctx.hw.extensions.has_h_extension {
            self.csr.hstatus = Arch::write_csr(Csr::Hstatus, 0);
            self.csr.hedeleg = Arch::write_csr(Csr::Hedeleg, 0);
            self.csr.hideleg = Arch::write_csr(Csr::Hideleg, 0);
            self.csr.hvip = Arch::write_csr(Csr::Hvip, 0);
            self.csr.hip = Arch::write_csr(Csr::Hip, 0);
            self.csr.hie = Arch::write_csr(Csr::Hie, 0);
            self.csr.hgeip = Arch::write_csr(Csr::Hgeip, 0); // Read only register, this write will have no effect
            self.csr.hgeie = Arch::write_csr(Csr::Hgeie, 0);
            self.csr.hcounteren = Arch::write_csr(Csr::Hcounteren, 0);
            self.csr.htval = Arch::write_csr(Csr::Htval, 0);
            self.csr.htinst = Arch::write_csr(Csr::Htinst, 0);
            self.csr.hgatp = Arch::write_csr(Csr::Hgatp, 0);

            self.csr.vsstatus = Arch::write_csr(Csr::Vsstatus, 0);
            self.csr.vsie = Arch::write_csr(Csr::Vsie, 0);
            self.csr.vstvec = Arch::write_csr(Csr::Vstvec, 0);
            self.csr.vsscratch = Arch::write_csr(Csr::Vsscratch, 0);
            self.csr.vsepc = Arch::write_csr(Csr::Vsepc, 0);
            self.csr.vscause = Arch::write_csr(Csr::Vscause, 0);
            self.csr.vstval = Arch::write_csr(Csr::Vstval, 0);
            self.csr.vsip = Arch::write_csr(Csr::Vsip, 0);
            self.csr.vsatp = Arch::write_csr(Csr::Vsatp, 0);
        }

        // Remove Firmware PMP from the hardware
        mctx.pmp.clear_range(mctx.pmp.virt_pmp_offset, self.nb_pmp);
        // Allow all addresses by default
        let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
        mctx.pmp.set_napot(last_pmp_idx, 0, usize::MAX, pmpcfg::RWX);
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use core::usize;

    use crate::arch::{mstatus, Arch, Architecture, Csr, Mode};
    use crate::host::MiralisContext;
    use crate::virt::VirtContext;

    /// We test value of mstatus.MPP.
    /// When switching from firmware to payload,
    /// virtual mstatus.MPP must to be S (because we are jumping to payload)
    /// and mstatus.MPP must be M (coming from Miralis).
    ///
    /// When switching from payload to firmware,
    /// virtual mstatus.MPP must to be S (coming from payload)
    /// and mstatus.MPP must be U (going to firmware).
    #[test]
    fn switch_context_mpp() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        ctx.csr.mstatus |= Mode::S.to_bits() << mstatus::MPP_OFFSET;

        unsafe { ctx.switch_from_firmware_to_payload(&mut mctx) }

        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP must be set to S mode (going to payload)"
        );

        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::M.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP must be set to M mode (coming from Miralis)"
        );

        // Simulate a trap
        unsafe { Arch::write_csr(Csr::Mstatus, Mode::S.to_bits() << mstatus::MPP_OFFSET) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        // VirtContext Mstatus.MPP has been set to M mode
        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP has been set to S mode (coming from payload)"
        );

        // Mstatus.MPP has been set to U mode
        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::U.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP has been set to U mode (going to firmware)"
        );
    }

    /// We test value of mideleg when switching from payload to firmware.
    /// Mideleg must always be 0 when executing the firware.
    #[test]
    fn switch_to_firmware_mideleg() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        unsafe { Arch::write_csr(Csr::Mideleg, usize::MAX) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        assert_eq!(Arch::read_csr(Csr::Mideleg), 0, "Mideleg must be 0");
    }
}
