//! Run subcommand
//!
//! The run subcommand launches a Miralis instance in QEMU with the provided Miralis and firmware
//! images.

use core::str;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use crate::artifacts::{build_target, download_artifact, locate_artifact, Artifact, Target};
use crate::config::{read_config, Config};
use crate::RunArgs;

// ————————————————————————————— QEMU Arguments ————————————————————————————— //

const QEMU: &str = "qemu-system-riscv64";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];

/// Address at which the firmware is loaded in memory.
const FIRMWARE_ADDR: u64 = 0x80200000;

// —————————————————————————————————— Run ——————————————————————————————————— //

/// Run Miralis on QEMU
pub fn run(args: &RunArgs) {
    println!("Running Miralis with '{}' firmware", &args.firmware);
    let cfg = get_config(args);

    // Build or retrieve the artifacts to run
    let miralis = build_target(Target::Miralis, &cfg);
    let firmware = match locate_artifact(&args.firmware) {
        Some(Artifact::Source { name }) => build_target(Target::Firmware(name), &cfg),
        Some(Artifact::Downloaded { name, url }) => download_artifact(&name, &url),
        None => PathBuf::from_str(&args.firmware).expect("Invalid firmware path"),
    };

    // Prepare the actual command
    let mut qemu_cmd = Command::new(QEMU);
    qemu_cmd.args(QEMU_ARGS);
    qemu_cmd
        .arg("-bios")
        .arg(miralis)
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            firmware.to_str().unwrap(),
            FIRMWARE_ADDR
        ));

    if let Some(nb_harts) = cfg.platform.nb_harts {
        assert!(nb_harts > 0, "Must use at least one core");
        qemu_cmd.arg("-smp").arg(format!("{}", nb_harts));
    }
    if args.debug {
        qemu_cmd.arg("-s");
    }
    if args.stop {
        qemu_cmd.arg("-S");
    }

    if args.verbose {
        println!();
        print!("{}", QEMU);
        for arg in qemu_cmd.get_args() {
            print!(" {}", arg.to_str().unwrap());
        }
        println!();
        println!();
    }

    let exit_status = qemu_cmd.status().expect("Failed to run QEMU");

    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

fn get_config(args: &RunArgs) -> Config {
    // Read config and build (or download) artifacts
    let mut cfg = read_config(&args.config);

    // Override some aspect of the config, if required by the arguments
    if let Some(max_exits) = args.max_exits {
        cfg.debug.max_firmware_exits = Some(max_exits);
    }
    if let Some(nb_harts) = args.smp {
        cfg.platform.nb_harts = Some(nb_harts);
    }

    cfg
}
