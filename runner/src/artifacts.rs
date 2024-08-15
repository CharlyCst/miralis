//! # Artifacts Management
//!
//! This module contains helper functions to manage the various artifacts built from sources or
//! downloaded.

use core::panic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use serde::Deserialize;

use crate::config::{Config, Platforms};
use crate::path::{
    get_artifact_manifest_path, get_artifacts_path, get_target_config_path, get_target_dir_path,
    get_workspace_path, is_older,
};

// —————————————————————————— Target & Build Info ——————————————————————————— //

/// Target triple used to build the monitor.
pub const MIRALIS_TARGET: &str = "riscv-unknown-miralis";

/// Target triple used to build the firmware.
pub const FIRMWARE_TARGET: &str = "riscv-unknown-firmware";

/// Extra cargo arguments.
const CARGO_ARGS: &[&str] = &[
    "-Zbuild-std=core,alloc",
    "-Zbuild-std-features=compiler-builtins-mem",
];

pub enum Target {
    Miralis,
    Firmware(String),
}

#[derive(Clone, Copy)]
pub enum Mode {
    Debug,
    Release,
}

impl Mode {
    pub fn from_bool(debug: bool) -> Self {
        if debug {
            Mode::Debug
        } else {
            Mode::Release
        }
    }
}

#[derive(Clone, Debug)]
pub enum Artifact {
    /// Artifacts that are built from sources.
    Source { name: String },
    /// Artifacts that are downloaded.
    Downloaded { name: String, url: String },
}

// ——————————————————————————— Artifact Manifest ———————————————————————————— //

/// A toml manifest that list extartnal artifacts.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct ArtifactManifest {
    #[serde(default)]
    bin: HashMap<String, Bin>,
}

/// Binaries artifacts.
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct Bin {
    #[allow(dead_code)] // TODO: remove when eventually used
    description: Option<String>,
    url: Option<String>,
    #[allow(dead_code)] // TODO: remove when eventually used
    repo: Option<String>,
}

fn read_artifact_manifest() -> ArtifactManifest {
    // Try to read the artifact manifest
    let manifest_path = get_artifact_manifest_path();
    let manifest = match fs::read_to_string(&manifest_path) {
        Ok(manifest) => manifest,
        Err(_) => {
            println!(
                "Could not find artifact manifest at '{}'",
                &manifest_path.display()
            );
            // Creating a default config
            String::from("")
        }
    };

    // Parse the config and returns it
    toml::from_str::<ArtifactManifest>(&manifest).expect("Failed to parse configuration")
}

fn append_artifact(name: &str, url: &Option<String>, map: &mut HashMap<String, Artifact>) {
    let Some(url) = url else { return };

    if url.starts_with("https://") || url.starts_with("http://") {
        map.insert(
            name.to_string(),
            Artifact::Downloaded {
                name: name.to_string(),
                url: url.clone(),
            },
        );
    } else {
        eprintln!("Warning: invalid artifact url '{}'", url);
    }
}

pub fn get_external_artifacts() -> HashMap<String, Artifact> {
    let manifest = read_artifact_manifest();
    let mut map = HashMap::new();

    for (key, bin) in manifest.bin {
        append_artifact(key.as_str(), &bin.url, &mut map)
    }
    map
}

// ———————————————————————————— Locate Artifacts ———————————————————————————— //

/// Try to locate the desired artifact.
///
/// Artifacts can be either available as sources, or as external binaries that can be downloaded.
pub fn locate_artifact(name: &str) -> Option<Artifact> {
    // Get the path to the firmware directory
    let mut firmware_path = get_workspace_path();
    firmware_path.push("firmware");
    assert!(
        firmware_path.is_dir(),
        "Could not find 'firmware' directory"
    );

    // Look for artifact inside benchmark folder
    let artifact = find_artifact(&firmware_path, name);
    if artifact.is_some() {
        return artifact;
    }

    // Get the path to the firmware/benchmark directory
    firmware_path.push("benchmark");
    assert!(
        firmware_path.is_dir(),
        "Could not find 'firmware/benchmark' directory"
    );

    // Look for artifact inside benchmark folder
    let artifact = find_artifact(&firmware_path, name);
    if artifact.is_some() {
        return artifact;
    }

    // Else check if the artifact is defined in the manifest
    let external = get_external_artifacts();
    if let Some(artifact) = external.get(name) {
        return Some(artifact.clone());
    }

    // Could not find artifact
    None
}

/// Check if one entry match the name
fn find_artifact(firmware_path: &PathBuf, name: &str) -> Option<Artifact> {
    for entry in fs::read_dir(&firmware_path).unwrap() {
        let Ok(file_path) = entry.map(|e| e.path()) else {
            continue;
        };
        let Some(file_name) = file_path.file_name() else {
            continue;
        };
        if file_name == name {
            return Some(Artifact::Source {
                name: name.to_string(),
            });
        };
    }
    None
}

// ————————————————————————————————— Build —————————————————————————————————— //

/// Perform the actual build by invoking cargo.
///
/// Returns the path of the resulting binary.
pub fn build_target(target: Target, cfg: &Config) -> PathBuf {
    let mode = Mode::from_bool(cfg.debug.debug.unwrap_or(true));
    let path = get_target_dir_path(&target, mode);
    println!("{:?}", path);

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd
        .arg("build")
        .args(CARGO_ARGS)
        .arg("--target")
        .arg(get_target_config_path(&target));

    build_cmd.arg("--profile");
    match mode {
        Mode::Debug => {
            build_cmd.arg("dev");
        }
        Mode::Release => {
            build_cmd.arg("release");
        }
    }

    match target {
        Target::Miralis => {
            // Linker arguments
            let start_address = cfg.platform.start_address.unwrap_or(0x80000000);
            let linker_args = format!("-C link-arg=-Tmisc/linker-script.x -C link-arg=--defsym=_start_address={start_address}");
            build_cmd.env("RUSTFLAGS", linker_args);

            // Environment variables
            build_cmd.envs(cfg.build_envs());

            // Features
            if let Some(plat) = cfg.platform.name {
                match plat {
                    Platforms::QemuVirt => {
                        // Nothing to do, default platform
                    }
                    Platforms::VisionFive2 => {
                        build_cmd.arg("--features").arg("platform_visionfive2");
                    }
                }
            }
        }

        Target::Firmware(ref firmware) => {
            let firmware_address = cfg.platform.firmware_address.unwrap_or(0x80200000);
            let linker_args = format!("-C link-arg=-Tmisc/linker-script-firmware.x -C link-arg=--defsym=_firmware_address={firmware_address}");
            build_cmd.env("RUSTFLAGS", linker_args);
            build_cmd.envs(cfg.benchmark.build_envs());
            build_cmd.arg("--package").arg(firmware);
        }
    }

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }
    objcopy(&target, mode)
}

/// Extract raw binary from elf file.
///
/// Returns the path of the resulting binary.
fn objcopy(target: &Target, mode: Mode) -> PathBuf {
    let path = get_target_dir_path(target, mode);
    let mut elf_path = path.clone();
    let mut bin_path = path.clone();

    match target {
        Target::Miralis => {
            elf_path.push("miralis");
            bin_path.push("miralis.img");
        }
        Target::Firmware(firmware) => {
            elf_path.push(firmware);
            bin_path.push(format!("{}.img", firmware));
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

// ———————————————————————————————— Download ———————————————————————————————— //

/// Download an artifact from the provided URL, returning the path.
pub fn download_artifact(name: &str, url: &str) -> PathBuf {
    // Setup artifacts folder, if necessary
    let artifacts = get_artifacts_path();
    fs::create_dir_all(&artifacts).expect("Failed to create an artifacts directory");

    // Download the artifact, if necessary
    let mut artifact = artifacts;
    artifact.push(name);
    let manifest = get_artifact_manifest_path();
    if !is_older(&manifest, &artifact) {
        // We need to update the artifact
        let mut curl_cmd = Command::new("curl");
        curl_cmd.arg("-o").arg(&artifact).arg("-L").arg(url);

        if !curl_cmd.status().unwrap().success() {
            panic!("Could not download artifact");
        }
    }

    artifact
}
