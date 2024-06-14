use std::io;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

mod artifacts;
mod config;
mod path;

use artifacts::{build_target, download_artifact, locate_artifact, Artifact, Target};
use config::Config;
use path::get_target_dir_path;

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
struct CliArgs {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Run Mirage on QEMU
    Run(RunArgs),
    /// Start GDB and connect to a running instance
    Gdb(GdbArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(long, default_value = "1")]
    smp: usize,
    #[arg(long, action)]
    debug: bool,
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

#[derive(Args)]
struct GdbArgs {}

// —————————————————————————————————— Run ——————————————————————————————————— //

/// Run Mirage on QEMU
fn run(args: &RunArgs) {
    println!("Running Mirage with '{}' firmware", &args.firmware);
    assert!(args.smp > 0, "Must use at least one core");
    let cfg = get_config(args);

    // Build or retrieve the artifacts to run
    let mirage = build_target(Target::Mirage, &cfg);
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
        .arg(mirage)
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            firmware.to_str().unwrap(),
            FIRMWARE_ADDR
        ))
        .arg("-smp")
        .arg(format!("{}", args.smp));

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
    let mut cfg = config::read_config();

    // Override some aspect of the config, if required by the arguments
    if let Some(max_exits) = args.max_exits {
        cfg.debug.max_firmware_exits = Some(max_exits);
    }

    cfg
}

// —————————————————————————————————— GDB ——————————————————————————————————— //

/// A list of GDB executables that support RISC-V 64
static GDB_EXECUTABLES: &[&'static str] = &["gdb-multiarch", "riscv64-elf-gdb"];

/// Build a command to invoke GDB using the provided executable.
///
/// GDB can be distributed under different names, depending on the available targets, hence the
/// need for such a function.
fn build_gdb_command(gdb_executable: &str) -> Command {
    // Retrieve the path of Mirage's binary
    let mut mirage_path = get_target_dir_path(&Target::Mirage);
    mirage_path.push("mirage");

    let mut gdb_cmd = Command::new(gdb_executable);
    gdb_cmd
        .arg(&mirage_path)
        .arg("-q")
        .args(["-x", "./misc/setup.gdb"]);
    gdb_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    gdb_cmd
}

/// Start a GDB session
fn gdb(_args: &GdbArgs) -> ! {
    for gdb in GDB_EXECUTABLES {
        let mut gdb_cmd = build_gdb_command(gdb);
        match gdb_cmd.output() {
            Ok(_) => exit(0), // Successfully launched GDB
            Err(err) => {
                if let io::ErrorKind::NotFound = err.kind() {
                    // This GDB executable is not installed, try another one
                    continue;
                } else {
                    panic!("Failed to run GDB: {:?}", err);
                }
            }
        }
    }

    // No GDB executable available.
    eprintln!("Could not find a GDB binary with RSIC-V support, try installing one of:");
    for gdb in GDB_EXECUTABLES {
        eprintln!("  - {}", gdb);
    }

    // Exit with non-zero exit code
    exit(1);
}

// —————————————————————————————— Entry Point ——————————————————————————————— //

fn main() {
    let args = CliArgs::parse();
    match args.command {
        Subcommands::Run(args) => run(&args),
        Subcommands::Gdb(args) => gdb(&args),
    };
}
