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

use crate::config::Config;
use crate::path::{
    get_artifact_manifest_path, get_artifacts_path, get_target_config_path, get_target_dir_path,
    get_workspace_path, is_older,
};

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
    bin: Bin,
}

/// Binaries artifacts.
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct Bin {
    opensbi: Option<String>,
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

    append_artifact("opensbi", &manifest.bin.opensbi, &mut map);
    map
}

// ———————————————————————————— Locate Artifacts ———————————————————————————— //

/// Try to locate the desired artifact.
///
/// Artifacts can be either available as sources, or as external binaries that can be downloaded.
pub fn locate_artifact(name: &str) -> Option<Artifact> {
    // Get the path to the payloads directory
    let mut payloads_path = get_workspace_path();
    payloads_path.push("payloads");
    assert!(
        payloads_path.is_dir(),
        "Could not find 'payloads' directory"
    );

    // Check if one entry match the name
    for entry in fs::read_dir(&payloads_path).unwrap() {
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
        }
    }

    // Else check if the artifact is defined in the manifest
    let external = get_external_artifacts();
    if let Some(artifact) = external.get(name) {
        return Some(artifact.clone());
    }

    // Could not find artifact
    None
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