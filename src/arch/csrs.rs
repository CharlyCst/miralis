use core::arch::asm;

pub fn pmpaddr_csr_read(index: usize) -> usize {
    let pmpaddr: usize;
    unsafe {
        match index {
            0 => asm!("csrr {}, pmpaddr0", out(reg) pmpaddr),
            1 => asm!("csrr {}, pmpaddr1", out(reg) pmpaddr),
            2 => asm!("csrr {}, pmpaddr2", out(reg) pmpaddr),
            3 => asm!("csrr {}, pmpaddr3", out(reg) pmpaddr),
            4 => asm!("csrr {}, pmpaddr4", out(reg) pmpaddr),
            5 => asm!("csrr {}, pmpaddr5", out(reg) pmpaddr),
            6 => asm!("csrr {}, pmpaddr6", out(reg) pmpaddr),
            7 => asm!("csrr {}, pmpaddr7", out(reg) pmpaddr),
            8 => asm!("csrr {}, pmpaddr8", out(reg) pmpaddr),
            9 => asm!("csrr {}, pmpaddr9", out(reg) pmpaddr),
            10 => asm!("csrr {}, pmpaddr10", out(reg) pmpaddr),
            11 => asm!("csrr {}, pmpaddr11", out(reg) pmpaddr),
            12 => asm!("csrr {}, pmpaddr12", out(reg) pmpaddr),
            13 => asm!("csrr {}, pmpaddr13", out(reg) pmpaddr),
            14 => asm!("csrr {}, pmpaddr14", out(reg) pmpaddr),
            15 => asm!("csrr {}, pmpaddr15", out(reg) pmpaddr),
            16 => asm!("csrr {}, pmpaddr16", out(reg) pmpaddr),
            17 => asm!("csrr {}, pmpaddr17", out(reg) pmpaddr),
            18 => asm!("csrr {}, pmpaddr18", out(reg) pmpaddr),
            19 => asm!("csrr {}, pmpaddr19", out(reg) pmpaddr),
            20 => asm!("csrr {}, pmpaddr20", out(reg) pmpaddr),
            21 => asm!("csrr {}, pmpaddr21", out(reg) pmpaddr),
            22 => asm!("csrr {}, pmpaddr22", out(reg) pmpaddr),
            23 => asm!("csrr {}, pmpaddr23", out(reg) pmpaddr),
            24 => asm!("csrr {}, pmpaddr24", out(reg) pmpaddr),
            25 => asm!("csrr {}, pmpaddr25", out(reg) pmpaddr),
            26 => asm!("csrr {}, pmpaddr26", out(reg) pmpaddr),
            27 => asm!("csrr {}, pmpaddr27", out(reg) pmpaddr),
            28 => asm!("csrr {}, pmpaddr28", out(reg) pmpaddr),
            29 => asm!("csrr {}, pmpaddr29", out(reg) pmpaddr),
            30 => asm!("csrr {}, pmpaddr30", out(reg) pmpaddr),
            31 => asm!("csrr {}, pmpaddr31", out(reg) pmpaddr),
            32 => asm!("csrr {}, pmpaddr32", out(reg) pmpaddr),
            33 => asm!("csrr {}, pmpaddr33", out(reg) pmpaddr),
            34 => asm!("csrr {}, pmpaddr34", out(reg) pmpaddr),
            35 => asm!("csrr {}, pmpaddr35", out(reg) pmpaddr),
            36 => asm!("csrr {}, pmpaddr36", out(reg) pmpaddr),
            37 => asm!("csrr {}, pmpaddr37", out(reg) pmpaddr),
            38 => asm!("csrr {}, pmpaddr38", out(reg) pmpaddr),
            39 => asm!("csrr {}, pmpaddr39", out(reg) pmpaddr),
            40 => asm!("csrr {}, pmpaddr40", out(reg) pmpaddr),
            41 => asm!("csrr {}, pmpaddr41", out(reg) pmpaddr),
            42 => asm!("csrr {}, pmpaddr42", out(reg) pmpaddr),
            43 => asm!("csrr {}, pmpaddr43", out(reg) pmpaddr),
            44 => asm!("csrr {}, pmpaddr44", out(reg) pmpaddr),
            45 => asm!("csrr {}, pmpaddr45", out(reg) pmpaddr),
            46 => asm!("csrr {}, pmpaddr46", out(reg) pmpaddr),
            47 => asm!("csrr {}, pmpaddr47", out(reg) pmpaddr),
            48 => asm!("csrr {}, pmpaddr48", out(reg) pmpaddr),
            49 => asm!("csrr {}, pmpaddr49", out(reg) pmpaddr),
            50 => asm!("csrr {}, pmpaddr50", out(reg) pmpaddr),
            51 => asm!("csrr {}, pmpaddr51", out(reg) pmpaddr),
            52 => asm!("csrr {}, pmpaddr52", out(reg) pmpaddr),
            53 => asm!("csrr {}, pmpaddr53", out(reg) pmpaddr),
            54 => asm!("csrr {}, pmpaddr54", out(reg) pmpaddr),
            55 => asm!("csrr {}, pmpaddr55", out(reg) pmpaddr),
            56 => asm!("csrr {}, pmpaddr56", out(reg) pmpaddr),
            57 => asm!("csrr {}, pmpaddr57", out(reg) pmpaddr),
            58 => asm!("csrr {}, pmpaddr58", out(reg) pmpaddr),
            59 => asm!("csrr {}, pmpaddr59", out(reg) pmpaddr),
            60 => asm!("csrr {}, pmpaddr60", out(reg) pmpaddr),
            61 => asm!("csrr {}, pmpaddr61", out(reg) pmpaddr),
            62 => asm!("csrr {}, pmpaddr62", out(reg) pmpaddr),
            63 => asm!("csrr {}, pmpaddr63", out(reg) pmpaddr),
            _ => pmpaddr = 0, // Default case when index is not
        }
    }
    return pmpaddr;
}

pub fn pmpaddr_csr_write(index: usize, pmpaddr: usize) {
    unsafe {
        match index {
            0 => asm!("csrw pmpaddr0, {}", in(reg) pmpaddr),
            1 => asm!("csrw pmpaddr1, {}", in(reg) pmpaddr),
            2 => asm!("csrw pmpaddr2, {}", in(reg) pmpaddr),
            3 => asm!("csrw pmpaddr3, {}", in(reg) pmpaddr),
            4 => asm!("csrw pmpaddr4, {}", in(reg) pmpaddr),
            5 => asm!("csrw pmpaddr5, {}", in(reg) pmpaddr),
            6 => asm!("csrw pmpaddr6, {}", in(reg) pmpaddr),
            7 => asm!("csrw pmpaddr7, {}", in(reg) pmpaddr),
            8 => asm!("csrw pmpaddr8, {}", in(reg) pmpaddr),
            9 => asm!("csrw pmpaddr9, {}", in(reg) pmpaddr),
            10 => asm!("csrw pmpaddr10, {}", in(reg) pmpaddr),
            11 => asm!("csrw pmpaddr11, {}", in(reg) pmpaddr),
            12 => asm!("csrw pmpaddr12, {}", in(reg) pmpaddr),
            13 => asm!("csrw pmpaddr13, {}", in(reg) pmpaddr),
            14 => asm!("csrw pmpaddr14, {}", in(reg) pmpaddr),
            15 => asm!("csrw pmpaddr15, {}", in(reg) pmpaddr),
            16 => asm!("csrw pmpaddr16, {}", in(reg) pmpaddr),
            17 => asm!("csrw pmpaddr17, {}", in(reg) pmpaddr),
            18 => asm!("csrw pmpaddr18, {}", in(reg) pmpaddr),
            19 => asm!("csrw pmpaddr19, {}", in(reg) pmpaddr),
            20 => asm!("csrw pmpaddr20, {}", in(reg) pmpaddr),
            21 => asm!("csrw pmpaddr21, {}", in(reg) pmpaddr),
            22 => asm!("csrw pmpaddr22, {}", in(reg) pmpaddr),
            23 => asm!("csrw pmpaddr23, {}", in(reg) pmpaddr),
            24 => asm!("csrw pmpaddr24, {}", in(reg) pmpaddr),
            25 => asm!("csrw pmpaddr25, {}", in(reg) pmpaddr),
            26 => asm!("csrw pmpaddr26, {}", in(reg) pmpaddr),
            27 => asm!("csrw pmpaddr27, {}", in(reg) pmpaddr),
            28 => asm!("csrw pmpaddr28, {}", in(reg) pmpaddr),
            29 => asm!("csrw pmpaddr29, {}", in(reg) pmpaddr),
            30 => asm!("csrw pmpaddr30, {}", in(reg) pmpaddr),
            31 => asm!("csrw pmpaddr31, {}", in(reg) pmpaddr),
            32 => asm!("csrw pmpaddr32, {}", in(reg) pmpaddr),
            33 => asm!("csrw pmpaddr33, {}", in(reg) pmpaddr),
            34 => asm!("csrw pmpaddr34, {}", in(reg) pmpaddr),
            35 => asm!("csrw pmpaddr35, {}", in(reg) pmpaddr),
            36 => asm!("csrw pmpaddr36, {}", in(reg) pmpaddr),
            37 => asm!("csrw pmpaddr37, {}", in(reg) pmpaddr),
            38 => asm!("csrw pmpaddr38, {}", in(reg) pmpaddr),
            39 => asm!("csrw pmpaddr39, {}", in(reg) pmpaddr),
            40 => asm!("csrw pmpaddr40, {}", in(reg) pmpaddr),
            41 => asm!("csrw pmpaddr41, {}", in(reg) pmpaddr),
            42 => asm!("csrw pmpaddr42, {}", in(reg) pmpaddr),
            43 => asm!("csrw pmpaddr43, {}", in(reg) pmpaddr),
            44 => asm!("csrw pmpaddr44, {}", in(reg) pmpaddr),
            45 => asm!("csrw pmpaddr45, {}", in(reg) pmpaddr),
            46 => asm!("csrw pmpaddr46, {}", in(reg) pmpaddr),
            47 => asm!("csrw pmpaddr47, {}", in(reg) pmpaddr),
            48 => asm!("csrw pmpaddr48, {}", in(reg) pmpaddr),
            49 => asm!("csrw pmpaddr49, {}", in(reg) pmpaddr),
            50 => asm!("csrw pmpaddr50, {}", in(reg) pmpaddr),
            51 => asm!("csrw pmpaddr51, {}", in(reg) pmpaddr),
            52 => asm!("csrw pmpaddr52, {}", in(reg) pmpaddr),
            53 => asm!("csrw pmpaddr53, {}", in(reg) pmpaddr),
            54 => asm!("csrw pmpaddr54, {}", in(reg) pmpaddr),
            55 => asm!("csrw pmpaddr55, {}", in(reg) pmpaddr),
            56 => asm!("csrw pmpaddr56, {}", in(reg) pmpaddr),
            57 => asm!("csrw pmpaddr57, {}", in(reg) pmpaddr),
            58 => asm!("csrw pmpaddr58, {}", in(reg) pmpaddr),
            59 => asm!("csrw pmpaddr59, {}", in(reg) pmpaddr),
            60 => asm!("csrw pmpaddr60, {}", in(reg) pmpaddr),
            61 => asm!("csrw pmpaddr61, {}", in(reg) pmpaddr),
            62 => asm!("csrw pmpaddr62, {}", in(reg) pmpaddr),
            63 => asm!("csrw pmpaddr63, {}", in(reg) pmpaddr),
            _ => (), // Default case when index is not matched
        }
    }
}

pub fn pmpcfg_csr_read(index: usize) -> usize {
    //The following code supports 64 PMP entries regardless of the value of PMP_ENTRIES. The
    //check in pmp_read is what will ensure that the index is valid. The following code should
    //execute only if that check passes, and thus should not access an index which shouldn't be supported.

    let pmpcfg: usize;
    //log::info!("index {}", index);
    match index {
        0..=7 => {
            /*log::info!("Matched 0");*/
            unsafe {
                asm!("csrr {}, pmpcfg0", out(reg) pmpcfg);
            }
        }
        8..=15 => unsafe {
            asm!("csrr {}, pmpcfg2", out(reg) pmpcfg);
        },
        16..=23 => unsafe {
            asm!("csrr {}, pmpcfg4", out(reg) pmpcfg);
        },
        24..=31 => unsafe {
            asm!("csrr {}, pmpcfg6", out(reg) pmpcfg);
        },
        32..=39 => unsafe {
            asm!("csrr {}, pmpcfg8", out(reg) pmpcfg);
        },
        40..=47 => unsafe {
            asm!("csrr {}, pmpcfg10", out(reg) pmpcfg);
        },
        48..=55 => unsafe {
            asm!("csrr {}, pmpcfg12", out(reg) pmpcfg);
        },
        56..=63 => unsafe {
            asm!("csrr {}, pmpcfg14", out(reg) pmpcfg);
        },
        _ => {
            log::info!("Matched None");
            pmpcfg = 0;
        }
    }
    return pmpcfg;
}

pub fn pmpcfg_csr_write(index: usize, pmpcfg: usize) {
    //log::info!("index {}", index);
    match index {
        0..=7 => {
            /*log::info!("Matched 0 writing: {:x}", pmpcfg);*/
            unsafe {
                asm!("csrw pmpcfg0, {}", in(reg) pmpcfg);
            }
        }
        8..=15 => unsafe {
            asm!("csrw pmpcfg2, {}", in(reg) pmpcfg);
        },
        16..=23 => unsafe {
            asm!("csrw pmpcfg4, {}", in(reg) pmpcfg);
        },
        24..=31 => unsafe {
            asm!("csrw pmpcfg6, {}", in(reg) pmpcfg);
        },
        32..=39 => unsafe {
            asm!("csrw pmpcfg8, {}", in(reg) pmpcfg);
        },
        40..=47 => unsafe {
            asm!("csrw pmpcfg10, {}", in(reg) pmpcfg);
        },
        48..=55 => unsafe {
            asm!("csrw pmpcfg12, {}", in(reg) pmpcfg);
        },
        56..=63 => unsafe {
            asm!("csrw pmpcfg14, {}", in(reg) pmpcfg);
        },
        _ => {
            log::info!("Matched None");
        }
    }
}
