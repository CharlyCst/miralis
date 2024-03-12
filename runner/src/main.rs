use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use clap::Parser;

mod config;
mod path;

use config::Config;
use path::{get_target_config_path, get_target_dir_path, is_known_payload, is_older};

// ————————————————————————————— QEMU Arguments ————————————————————————————— //

const QEMU: &str = "qemu-system-riscv64";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];

// —————————————————————————— Target & Build Info ——————————————————————————— //

/// Address at which the payload is loaded in memory.
const PAYLOAD_ADDR: u64 = 0x80100000;

/// Target triple used to build the monitor.
const MIRAGE_TARGET: &str = "riscv-unknown-mirage";

/// Target triple used to build the payload.
const PAYLOAD_TARGET: &str = "riscv-unknown-payload";

/// Extra cargo arguments.
const CARGO_ARGS: &[&str] = &[
    "-Zbuild-std=core,alloc",
    "-Zbuild-std-features=compiler-builtins-mem",
];

enum Target {
    Mirage,
    Payload(String),
}

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
    #[arg(short, long, default_value = "ecall")]
    payload: String,
    #[arg(long)]
    /// Maximum number of payload exits
    max_exits: Option<usize>,
}

fn parse_args() -> Args {
    let args = Args::parse();
    assert!(args.smp > 0, "Must use at least one core");
    return args;
}

// ————————————————————————————————— Build —————————————————————————————————— //

/// Perform the actual build by invoking cargo.
///
/// Returns the path of the resulting binary.
fn build_target(target: Target, cfg: &Config) -> PathBuf {
    let path = get_target_dir_path(&target);
    println!("{:?}", path);

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd
        .arg("build")
        .args(CARGO_ARGS)
        .arg("--target")
        .arg(get_target_config_path(&target));

    match target {
        Target::Mirage => {
            build_cmd.env("RUSTFLAGS", "-C link-arg=-Tmisc/linker-script.x");
            build_cmd.envs(cfg.build_envs());
        }
        Target::Payload(ref payload) => {
            build_cmd.env("RUSTFLAGS", "-C link-arg=-Tmisc/linker-script-payload.x");
            build_cmd.arg("--package").arg(payload);
        }
    }

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }
    objcopy(&target)
}

/// Extract raw binary from elf file.
///
/// Returns the path of the resulting binary.
fn objcopy(target: &Target) -> PathBuf {
    let path = get_target_dir_path(target);
    let mut elf_path = path.clone();
    let mut bin_path = path.clone();

    match target {
        Target::Mirage => {
            elf_path.push("mirage");
            bin_path.push("mirage.img");
        }
        Target::Payload(payload) => {
            elf_path.push(payload);
            bin_path.push(format!("{}.img", payload));
        }
    }

    if is_older(&elf_path, &bin_path) {
        // No change since last objcopy, skipping
        return bin_path;
    }

    let mut objopy_cmd = Command::new("rust-objcopy");
    objopy_cmd
        .arg("-O")
        .arg("binary")
        .arg(elf_path)
        .arg(&bin_path);

    if !objopy_cmd
        .status()
        .expect("objcopy failed. Is `rust-objcopy` installed?")
        .success()
    {
        panic!("objcopy failed");
    }

    bin_path
}

// —————————————————————————————————— Run ——————————————————————————————————— //

/// Run the binaries on QEMU
fn run(mirage: PathBuf, payload: PathBuf, args: &Args) {
    let mut qemu_cmd = Command::new(QEMU);
    qemu_cmd.args(QEMU_ARGS);
    qemu_cmd
        .arg("-bios")
        .arg(mirage)
        .arg("-device")
        .arg(format!(
            "loader,file={},addr=0x{:x},force-raw=on",
            payload.to_str().unwrap(),
            PAYLOAD_ADDR
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

    println!("Running Mirage with '{}' payload", &args.payload);

    let cfg = config::read_config(&args);

    let mirage = build_target(Target::Mirage, &cfg);
    let payload = if is_known_payload(&args.payload) {
        build_target(Target::Payload(args.payload.clone()), &cfg)
    } else {
        PathBuf::from_str(&args.payload).expect("Invalid payload path")
    };

    run(mirage, payload, &args);
}
