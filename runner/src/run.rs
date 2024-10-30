//! Run subcommand
//!
//! The run subcommand launches a Miralis instance in QEMU with the provided Miralis and firmware
//! images.

use core::str;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::str::FromStr;

use crate::artifacts::{build_target, prepare_firmware_artifact, prepare_payload_artifact, Target};
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

/// The run command, runs Miralis with the provided arguments.
pub fn run(args: &RunArgs) -> ExitCode {
    log::info!("Running Miralis with '{}' firmware", &args.firmware);
    let cfg = get_config(args);

    // Build or retrieve the artifacts to run
    let miralis = build_target(Target::Miralis, &cfg);
    let Some(firmware) = prepare_firmware_artifact(&args.firmware, &cfg) else {
        return ExitCode::FAILURE;
    };

    let cmd = match cfg.platform.name.unwrap_or(Platforms::QemuVirt) {
        Platforms::QemuVirt => get_qemu_cmd(&cfg, miralis, firmware, None, args.debug, args.stop),
        Platforms::Spike => get_spike_cmd(&cfg, miralis, firmware),
        Platforms::VisionFive2 => {
            log::error!("We can't run VisionFive2 on simulator.");
            return ExitCode::FAILURE;
        }
    };
    let Ok(mut cmd) = cmd else {
        log::error!("Failed to build command");
        return ExitCode::FAILURE;
    };

    log::debug!(
        "{} {}",
        cmd.get_program().to_str().unwrap(),
        cmd.get_args()
            .map(|arg| arg.to_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let exit_status = cmd.status().expect("Failed to run");

    if !exit_status.success() {
        ExitCode::from(exit_status.code().unwrap_or(1) as u8)
    } else {
        ExitCode::SUCCESS
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

/// Return the command to run Miralis on QEMU.
pub fn get_qemu_cmd(
    cfg: &Config,
    miralis: PathBuf,
    firmware: PathBuf,
    payload: Option<&String>,
    debug: bool,
    stop: bool,
) -> Result<Command, ()> {
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
        .arg("-m")
        .arg("2048")
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            firmware.to_str().unwrap(),
            FIRMWARE_ADDR
        ));

    // If a payload is defined in the config, try to load it at the specified address.
    let payload = payload.or_else(|| {
        cfg.target
            .payload
            .as_ref()
            .and_then(|payload| payload.name.as_ref())
    });
    if let Some(payload_name) = payload {
        let payload = match prepare_payload_artifact(payload_name, cfg) {
            Some(payload_path) => payload_path,
            None => {
                let payload_path = PathBuf::from_str(payload_name).expect("Invalid payload name");
                if payload_path.is_file() {
                    payload_path
                } else {
                    log::error!("Invalid payload '{}'", payload_name);
                    return Err(());
                }
            }
        };

        qemu_cmd.arg("-device").arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            payload.to_str().unwrap(),
            PAYLOAD_ADDR
        ));
    }

    if let Some(nb_harts) = cfg.platform.nb_harts {
        assert!(nb_harts > 0, "Must use at least one core");
        qemu_cmd.arg("-smp").arg(format!("{}", nb_harts));
    }
    if debug {
        qemu_cmd.arg("-s");
    }
    if stop {
        qemu_cmd.arg("-S");
    }

    Ok(qemu_cmd)
}

/// Return the command to run Miralis on Spike.
pub fn get_spike_cmd(cfg: &Config, miralis: PathBuf, firmware: PathBuf) -> Result<Command, ()> {
    let mut spike_cmd = Command::new(SPIKE);

    spike_cmd.arg("--kernel");

    spike_cmd.arg(firmware.to_str().unwrap());
    spike_cmd.arg(raw_to_elf(miralis.to_str().unwrap()));

    if let Some(nb_harts) = cfg.platform.nb_harts {
        assert!(nb_harts > 0, "Must use at least one core");
        spike_cmd.arg("-p").arg(format!("{}", nb_harts));
    }

    Ok(spike_cmd)
}

fn raw_to_elf(raw_path: &str) -> &str {
    &raw_path[..raw_path.len() - 4]
}
