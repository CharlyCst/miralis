//! # Artifacts Management
//!
//! This module contains helper functions to manage the various artifacts built from sources or
//! downloaded.

use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::path::{get_target_config_path, get_target_dir_path, is_older};

// —————————————————————————— Target & Build Info ——————————————————————————— //

/// Target triple used to build the monitor.
pub const MIRAGE_TARGET: &str = "riscv-unknown-mirage";

/// Target triple used to build the payload.
pub const PAYLOAD_TARGET: &str = "riscv-unknown-payload";

/// Extra cargo arguments.
const CARGO_ARGS: &[&str] = &[
    "-Zbuild-std=core,alloc",
    "-Zbuild-std-features=compiler-builtins-mem",
];

pub enum Target {
    Mirage,
    Payload(String),
}

// ————————————————————————————————— Build —————————————————————————————————— //

/// Perform the actual build by invoking cargo.
///
/// Returns the path of the resulting binary.
pub fn build_target(target: Target, cfg: &Config) -> PathBuf {
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
