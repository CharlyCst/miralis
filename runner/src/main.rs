use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

// ————————————————————————————— QEMU Arguments ————————————————————————————— //

const QEMU: &str = "qemu-system-riscv64";

#[rustfmt::skip]
const QEMU_ARGS: &[&str] = &[
    "--no-reboot",
    "-nographic",
    "-machine", "virt",
];

// ————————————————————————————— Configurationn ————————————————————————————— //

/// Address at which the paylod is loaded in memory.
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
fn build_target(target: Target) -> PathBuf {
    let path = get_target_dir_path(&target);
    println!("{:?}", path);

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd
        .arg("build")
        .args(CARGO_ARGS)
        .arg("--target")
        .arg(get_target_config_path(&target));
    build_cmd.env("RUSTFLAGS", "-C link-arg=-Tconfig/linker-script.x");

    match target {
        Target::Mirage => (),
        Target::Payload(ref payload) => {
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

    let mut objopy_cmd = Command::new("rust-objcopy");
    objopy_cmd
        .arg("-O")
        .arg("binary")
        .arg(elf_path)
        .arg(&bin_path);

    if !objopy_cmd.status().unwrap().success() {
        panic!("objcopy failed - is `rust-objcopy` installed? Try installing with `rustup component add llvm-tools`");
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

    let exit_status = qemu_cmd.status().unwrap();
    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

// —————————————————————————————— Path Helpers —————————————————————————————— //

fn get_workspace_path() -> PathBuf {
    let Ok(runner_manifest) = std::env::var("CARGO_MANIFEST_DIR") else {
        panic!("Could not locate workspace root");
    };
    let path = PathBuf::from_str(&runner_manifest).unwrap();
    path.parent().unwrap().to_owned()
}

fn get_target_dir_path(target: &Target) -> PathBuf {
    let mut path = get_workspace_path();
    path.push("target");
    match target {
        Target::Mirage => path.push(MIRAGE_TARGET),
        Target::Payload(_) => path.push(PAYLOAD_TARGET),
    }
    path.push("debug"); // TODO: add support for release mode
    path
}

fn get_config_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("config");
    path
}

fn get_target_config_path(target: &Target) -> PathBuf {
    let mut path = get_config_path();
    match target {
        Target::Mirage => {
            path.push(format!("{}.json", MIRAGE_TARGET));
        }
        Target::Payload(_) => path.push(format!("{}.json", PAYLOAD_TARGET)),
    }
    path
}

// —————————————————————————————— Entry Point ——————————————————————————————— //

fn main() {
    let args = parse_args();

    println!("Running Mirage with '{}' payload", &args.payload);

    let mirage = build_target(Target::Mirage);
    let payload = build_target(Target::Payload(args.payload.clone()));
    run(mirage, payload, &args);
}
