
/// Contains all the information automatically written by the hardware during a trap
#[repr(C)]
pub struct TrapInfo {
    pc : usize,
    mepc : usize,
    mstatus : usize, 
    mcause : usize,
    mip : usize,
    mtval : usize,
    mtval2 : usize,
    mtinst : usize,
}

impl Default for TrapInfo{
    fn default() -> TrapInfo{
        TrapInfo{
            pc : 0,
            mepc : 0,
            mstatus : 0,
            mcause : 0,
            mip : 0,
            mtval : 0,
            mtval2 : 0,
            mtinst : 0,
        }
    }

    /// Whether the trap comes from M mode
    fn from_mmode() -> bool{ 
        let mpp : usize = (self.mstatus >> 11) & 0b11;
        return mpp == 3; // Mpp : 3 = M mode 
    }

    /// Return the trap cause 
    fn get_cause() -> MCause {
        return MCause::new(self.mcause);
    }

}