//! Run subcommand
//!
//! The run subcommand launches a Miralis instance in QEMU with the provided Miralis and firmware
//! images.

use core::str;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use crate::artifacts::{build_target, download_artifact, locate_artifact, Artifact, Target};
use crate::config::{read_config, Config, Platforms};
use crate::RunArgs;

// ————————————————————————————— QEMU Arguments ————————————————————————————— //

const QEMU: &str = "qemu-system-riscv64";
const SPIKE: &str = "spike";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];

/// Address at which the firmware is loaded in memory.
const FIRMWARE_ADDR: u64 = 0x80200000;

/// Address at which the payload is loaded in memory.
const PAYLOAD_ADDR: u64 = 0x80400000;

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

    match cfg.platform.name.unwrap_or(Platforms::QemuVirt) {
        Platforms::QemuVirt => launch_qemu(args, miralis, firmware),
        Platforms::Spike => launch_spike(args, miralis, firmware),
        Platforms::VisionFive2 => {
            panic!("We can't run VisionFive2 on simulator.")
        }
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

    if cfg.qemu.cpu == Some(String::from("none")) {
        cfg.qemu.cpu = None;
    }

    cfg
}

fn launch_qemu(args: &RunArgs, miralis: PathBuf, firmware: PathBuf) {
    if !simulator_exists(QEMU) {
        return;
    };

    let cfg = get_config(args);
    let mut qemu_cmd = Command::new(QEMU);
    qemu_cmd.args(QEMU_ARGS);
    if let Some(machine) = &cfg.qemu.machine {
        qemu_cmd.arg("-machine").arg(machine);
    }
    if let Some(cpu) = &cfg.qemu.cpu {
        qemu_cmd.arg("-cpu").arg(cpu);
    }
    qemu_cmd
        .arg("-bios")
        .arg(miralis)
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            firmware.to_str().unwrap(),
            FIRMWARE_ADDR
        ));

    // If a payload is defined in the config, try to load it at the specified address.
    if let Some(payload) = &cfg.target.payload {
        if let Some(payload_name) = &payload.name {
            let payload = match locate_artifact(&payload_name) {
                Some(Artifact::Source { name }) => build_target(Target::Payload(name), &cfg),
                Some(Artifact::Downloaded { name, url }) => download_artifact(&name, &url),
                None => PathBuf::from_str(&payload_name).expect("Invalid payload path"),
            };

            qemu_cmd.arg("-device").arg(format!(
                "loader,file={},addr=0x{:x},force-raw=on",
                payload.to_str().unwrap(),
                PAYLOAD_ADDR
            ));
        }
    }

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

fn launch_spike(args: &RunArgs, miralis: PathBuf, firmware: PathBuf) {
    if !simulator_exists(SPIKE) {
        return;
    };

    let cfg = get_config(args);
    let mut spike_cmd = Command::new(SPIKE);

    spike_cmd.arg("--kernel");

    spike_cmd.arg(firmware.to_str().unwrap());
    spike_cmd.arg(raw_to_elf(miralis.to_str().unwrap()));

    if let Some(nb_harts) = cfg.platform.nb_harts {
        assert!(nb_harts > 0, "Must use at least one core");
        spike_cmd.arg("-smp").arg(format!("{}", nb_harts));
    }

    let exit_status = spike_cmd.status().expect("Failed to run SPIKE");

    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

fn raw_to_elf(raw_path: &str) -> &str {
    &raw_path[..raw_path.len() - 4]
}

fn simulator_exists(program: &str) -> bool {
    // Use the "command -v" shell command to check if the program exists
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", program))
        .status();

    let exists: bool = match status {
        Ok(exit_status) => exit_status.success(),
        Err(_) => false,
    };

    if !exists {
        println!("======================================");
        println!("|                                     ");
        println!("|        {} not installed    ", program);
        println!("|                                     ");
        println!("======================================");
    }

    return exists;
}
