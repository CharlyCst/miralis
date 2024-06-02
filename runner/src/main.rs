use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use clap::Parser;

mod artifacts;
mod config;
mod path;

use artifacts::{build_target, download_artifact, locate_artifact, Artifact, Target};

// ————————————————————————————— QEMU Arguments ————————————————————————————— //

const QEMU: &str = "qemu-system-riscv64";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];

/// Address at which the firmware is loaded in memory.
const FIRMWARE_ADDR: u64 = 0x80100000;

// —————————————————————————————— CLI Parsing ——————————————————————————————— //

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "1")]
    smp: usize,
    #[arg(long, action)]
    dbg: bool,
    #[arg(long, action)]
    stop: bool,
    #[arg(short, long, action)]
    verbose: bool,
    #[arg(short, long, default_value = "default")]
    firmware: String,
    #[arg(long)]
    /// Maximum number of firmware exits
    max_exits: Option<usize>,
}

fn parse_args() -> Args {
    let args = Args::parse();
    assert!(args.smp > 0, "Must use at least one core");
    return args;
}

// —————————————————————————————————— Run ——————————————————————————————————— //

/// Run the binaries on QEMU
fn run(mirage: PathBuf, firmware: PathBuf, args: &Args) {
    let mut qemu_cmd = Command::new(QEMU);
    qemu_cmd.args(QEMU_ARGS);
    qemu_cmd
        .arg("-bios")
        .arg(mirage)
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            firmware.to_str().unwrap(),
            FIRMWARE_ADDR
        ))
        .arg("-smp")
        .arg(format!("{}", args.smp));

    if args.dbg {
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

// —————————————————————————————— Entry Point ——————————————————————————————— //

fn main() {
    let args = parse_args();

    println!("Running Mirage with '{}' firmware", &args.firmware);

    let cfg = config::read_config(&args);

    let mirage = build_target(Target::Mirage, &cfg);
    let firmware = match locate_artifact(&args.firmware) {
        Some(Artifact::Source { name }) => build_target(Target::Firmware(name), &cfg),
        Some(Artifact::Downloaded { name, url }) => download_artifact(&name, &url),
        None => PathBuf::from_str(&args.firmware).expect("Invalid firmware path"),
    };

    run(mirage, firmware, &args);
}
