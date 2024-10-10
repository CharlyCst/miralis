//! Run subcommand
//!
//! The run subcommand launches a Miralis instance in QEMU with the provided Miralis and firmware
//! images.

use core::str;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::str::FromStr;

use crate::artifacts::{
    build_target, download_disk_image, get_external_artifacts, prepare_firmware_artifact,
    prepare_payload_artifact, DiskArtifact, Target,
};
use crate::config::{read_config, Config, Platforms};
use crate::RunArgs;

// ————————————————————————————— QEMU Arguments ————————————————————————————— //


/// The QEMU executable
pub const QEMU: &str = "/home/francois/Documents/ACE-RISCV/ace-build/qemu/bin/qemu-system-riscv64";
// pub const QEMU: &str = "qemu-system-riscv64";

/// The Spike executable
pub const SPIKE: &str = "spike";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];
/// Address at which the firmware is loaded in memory.
const _FIRMWARE_ADDR: u64 = 0x80200000;

/// Address at which the payload is loaded in memory.
const PAYLOAD_ADDR: u64 = 0x80400000;

// —————————————————————————————————— Run ——————————————————————————————————— //

/// The run command, runs Miralis with the provided arguments.
pub fn run(args: &RunArgs) -> ExitCode {
    let cfg = get_config(args);

    // Build or retrieve the artifacts to run
    let miralis = build_target(Target::Miralis, &cfg);
    let firmware = if let Some(fw) = &args.firmware {
        fw
    } else if let Some(fw) = &cfg.target.firmware.name {
        fw
    } else {
        "default"
    };
    log::info!("Running Miralis with '{}' firmware", firmware);
    let Some(firmware) = prepare_firmware_artifact(firmware, &cfg) else {
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
    if let Some(disk) = &args.disk {
        cfg.qemu.disk = Some(disk.to_owned());
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

    qemu_cmd.arg("-m");
    if let Some(memory) = &cfg.qemu.memory {
        qemu_cmd.arg(memory);
    } else {
        qemu_cmd.arg("2048");
    }

    qemu_cmd
        .arg("-bios")
        .arg(miralis)
        .arg("-m")
        .arg("8G")
        .arg("-machine")
        .arg("virt")
        .arg("-cpu")
        .arg("rv64")
        //.arg("-kernel") // Loaded at 0x80200000
        //.arg(firmware.to_str().unwrap());
        .arg("-kernel")
        .arg("./artifacts/opensbi_cove")
        .arg("-append")
        .arg("console=ttyS0 ro root=/dev/vda")
        .arg("-drive").arg("if=none,format=raw,file=/home/francois/Documents/ACE-RISCV/ace-build/hypervisor/buildroot/images/rootfs.ext4,id=hd0")
        .arg("-device")
        .arg("virtio-blk-device,scsi=off,drive=hd0")
        .arg("-netdev")
        .arg("user,id=net0,net=192.168.100.1/24,dhcpstart=192.168.100.128,hostfwd=tcp::3024-:22")
        .arg("-device")
        .arg("virtio-net-device,netdev=net0")
        .arg("-device")
        .arg("virtio-rng-pci");

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

    // If a disk is present add the appropriate device
    if let Some(disk) = &cfg.qemu.disk {
        if let Some(DiskArtifact::Downloaded { name, url }) =
            get_external_artifacts().disk.get(disk)
        {
            download_disk_image(name, url);

            qemu_cmd
                .arg("-device")
                .arg("virtio-net-device,netdev=eth0")
                .arg("-netdev")
                .arg("user,id=eth0")
                .arg("-device")
                .arg("virtio-rng-pci")
                .arg("-drive")
                .arg(format!(
                    "file=artifacts/{}-miralis.img,format=raw,if=virtio",
                    name
                ));
        }
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

/// Returns true if QEMU is available.
pub fn qemu_is_available() -> bool {
    let mut qemu_cmd = Command::new(QEMU);
    qemu_cmd.arg("--version");
    qemu_cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    qemu_cmd.status().is_ok()
}

/// Returns true if Spike is available.
pub fn spike_is_available() -> bool {
    let mut spike_cmd = Command::new(SPIKE);
    spike_cmd.arg("--help");
    spike_cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    spike_cmd.status().is_ok()
}
