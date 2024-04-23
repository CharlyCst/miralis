use core::arch::asm;

pub fn test_perf_counters() {
    // For now we expose 0 performance counters to the payload, but we might expose more in the
    // future.
    test_simple_regs();
    test_some_counters_events();
}

fn test_simple_regs() {
    let mut res: usize;

    // Test mcycle
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcycle, {0}",
            "csrr {1}, mcycle",
            out(reg) _,
            out(reg) res,
        );
    }
    assert_eq!(res, 0);

    // Test minstret
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw minstret, {0}",
            "csrr {1}, minstret",
            out(reg) _,
            out(reg) res,
        );
    }
    assert_eq!(res, 0);

    // Test mcountinhibit
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcountinhibit, {0}",
            "csrr {1}, mcountinhibit",
            out(reg) _,
            out(reg) res,
        );
    }
    assert_eq!(res, 0);

    // Test mcounteren
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcounteren, {0}",
            "csrr {1}, mcounteren",
            out(reg) _,
            out(reg) res,
        );
    }
    assert_eq!(res, 0);
}

fn test_some_counters_events() {
    let mut res: usize;

    // Test mhpmcounter3
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter3, {0}",
            "csrr {1}, mhpmcounter3",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);

    // Test mhpmcounter5
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter5, {0}",
            "csrr {1}, mhpmcounter5",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);

    // Test mhpmcounter7
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter7, {0}",
            "csrr {1}, mhpmcounter7",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);

    // Test mhpmevent3
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent3, {0}",
            "csrr {1}, mhpmevent3",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);

    // Test mhpmevent5
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent5, {0}",
            "csrr {1}, mhpmevent5",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);

    // Test mhpmevent7
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent7, {0}",
            "csrr {1}, mhpmevent7",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0);
}
