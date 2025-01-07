//! # Artifacts Management
//!
//! This module contains helper functions to manage the various artifacts built from sources or
//! downloaded.

use core::panic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::{env, fs};

use serde::Deserialize;

use crate::config::{Config, Profiles};
use crate::path::{
    extract_file_extension, extract_file_name, get_artifact_manifest_path, get_artifacts_path,
    get_target_config_path, get_target_dir_path, get_workspace_path, is_file_present, is_older,
    remove_file_extention, EXT2_EXTENSION, GZ_COMPRESSION, IMG_EXTENSION, XZ_COMPRESSION,
    ZST_COMPRESSION,
};
use crate::ArtifactArgs;

// —————————————————————————— Target & Build Info ——————————————————————————— //

/// Target triple used to build the monitor.
pub const MIRALIS_TARGET: &str = "riscv-unknown-miralis";

/// Target triple used to build the firmware or the payload.
pub const FIRMWARE_TARGET: &str = "riscv-unknown-firmware";

/// Target triple used to build the payload.
pub const PAYLOAD_TARGET: &str = "riscv-unknown-payload";

/// Extra cargo arguments.
const CARGO_ARGS: &[&str] = &[
    "-Zbuild-std=core,alloc",
    "-Zbuild-std-features=compiler-builtins-mem",
];

#[derive(PartialEq, Eq)]
pub enum Target {
    Miralis,
    Firmware(String),
    Payload(String),
}

// —————————————————————————— Artifact Definitions —————————————————————————— //

trait Artifact {
    /// Creates an artifact that can be downloaded.
    fn from_url(name: &str, url: &str) -> Self;
}

/// A binary artifact, typically a firmware or payload.
#[derive(Clone, Debug)]
pub enum BinArtifact {
    /// Artifacts that are built from sources.
    Source { name: String },
    /// Artifacts that are downloaded.
    Downloaded { name: String, url: String },
    /// Artifact available as binaries on the local file system.
    Binary { path: PathBuf },
}

/// A disk artifact
#[derive(Clone, Debug)]
pub enum DiskArtifact {
    /// A disk image that can be downloaded.
    Downloaded { name: String, url: String },
}

/// A collection of artifacts
pub struct AllArtifacts {
    /// Binary artifacts
    pub bin: HashMap<String, BinArtifact>,
    /// Disk artifacts
    pub disk: HashMap<String, DiskArtifact>,
}

impl Artifact for BinArtifact {
    fn from_url(name: &str, url: &str) -> Self {
        Self::Downloaded {
            name: name.to_owned(),
            url: url.to_owned(),
        }
    }
}

impl Artifact for DiskArtifact {
    fn from_url(name: &str, url: &str) -> Self {
        Self::Downloaded {
            name: name.to_owned(),
            url: url.to_owned(),
        }
    }
}

// ——————————————————————————— Artifact Manifest ———————————————————————————— //

/// A toml manifest that list external artifacts.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct ArtifactManifest {
    #[serde(default)]
    bin: HashMap<String, Bin>,
    disk: HashMap<String, Disk>,
}

/// Binaries artifacts.
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct Bin {
    description: Option<String>,
    url: Option<String>,
    repo: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct Disk {
    description: Option<String>,
    url: Option<String>,
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

fn append_artifact_url<A: Artifact>(
    name: &str,
    url: &Option<String>,
    map: &mut HashMap<String, A>,
) {
    let Some(url) = url else { return };

    if url.starts_with("https://") || url.starts_with("http://") {
        map.insert(name.to_string(), A::from_url(name, url));
    } else {
        log::warn!("Invalid artifact url '{}'", url);
    }
}

pub fn get_external_artifacts() -> AllArtifacts {
    let manifest = read_artifact_manifest();
    let mut bins = HashMap::new();
    let mut disks = HashMap::new();

    for (key, bin) in manifest.bin {
        append_artifact_url(key.as_str(), &bin.url, &mut bins)
    }
    for (key, disk) in manifest.disk {
        append_artifact_url(key.as_str(), &disk.url, &mut disks)
    }

    AllArtifacts {
        bin: bins,
        disk: disks,
    }
}

// ———————————————————————————— Locate Artifacts ———————————————————————————— //

/// Prepare a firmware artifact to be used.
///
/// Artifacts can be provided as sources, binaries, or URL to download. This functions takes care
/// of preparing the final binaries that can be imported in an emulator or run on hardware.
pub fn prepare_firmware_artifact(name: &str, cfg: &Config) -> Option<PathBuf> {
    match locate_bin_artifact(name) {
        Some(BinArtifact::Source { name }) => Some(build_target(Target::Firmware(name), cfg)),
        Some(BinArtifact::Downloaded { name, url }) => Some(download_artifact(&name, &url)),
        Some(BinArtifact::Binary { path }) => Some(path),
        None => None,
    }
}

/// Prepare a payload artifact to be used.
///
/// Artifacts can be provided as sources, binaries, or URL to download. This functions takes care
/// of preparing the final binaries that can be imported in an emulator or run on hardware.
pub fn prepare_payload_artifact(name: &str, cfg: &Config) -> Option<PathBuf> {
    match locate_bin_artifact(name) {
        Some(BinArtifact::Source { name }) => Some(build_target(Target::Payload(name), cfg)),
        Some(BinArtifact::Downloaded { name, url }) => Some(download_artifact(&name, &url)),
        Some(BinArtifact::Binary { path }) => Some(path),
        None => None,
    }
}

/// Try to locate the desired binary artifact.
///
/// Artifacts can be either available as sources, as external binaries that can be downloaded, or
/// as path on the local filesystem.
fn locate_bin_artifact(name: &str) -> Option<BinArtifact> {
    // Miralis as firmware?
    if name == "miralis" {
        return Some(BinArtifact::Source {
            name: String::from(name),
        });
    }

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

    // Get the path to the payload directory
    let mut payload_path = get_workspace_path();
    payload_path.push("payload");
    assert!(payload_path.is_dir(), "Could not find 'payload' directory");

    // Check if one entry match the name
    for entry in fs::read_dir(&payload_path).unwrap() {
        let Ok(file_path) = entry.map(|e| e.path()) else {
            continue;
        };
        let Some(file_name) = file_path.file_name() else {
            continue;
        };
        if file_name == name {
            return Some(BinArtifact::Source {
                name: name.to_string(),
            });
        }
    }

    // Else check if the artifact is defined in the manifest
    let external_binary = &get_external_artifacts().bin;
    if let Some(artifact) = external_binary.get(name) {
        return Some(artifact.clone());
    }

    // Finally look for a local file as a last resort
    let path = PathBuf::from(name);
    if path.is_file() {
        return Some(BinArtifact::Binary { path });
    }

    // Could not find artifact - exit process
    log::error!("Artifact {} not found exiting runner build.rs", name);
    None
}

/// Check if one entry match the name
fn find_artifact(firmware_path: &PathBuf, name: &str) -> Option<BinArtifact> {
    for entry in fs::read_dir(firmware_path).unwrap() {
        let Ok(file_path) = entry.map(|e| e.path()) else {
            continue;
        };
        let Some(file_name) = file_path.file_name() else {
            continue;
        };
        if file_name == name {
            return Some(BinArtifact::Source {
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
    let mode = match target {
        Target::Miralis => cfg.target.miralis.profile.unwrap_or(Profiles::Debug),
        Target::Firmware(_) => cfg.target.firmware.profile.unwrap_or(Profiles::Debug),
        Target::Payload(_) => {
            if let Some(payload) = &cfg.target.payload {
                payload.profile.unwrap_or(Profiles::Debug)
            } else {
                Profiles::Debug
            }
        }
    };

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd
        .arg("build")
        .args(CARGO_ARGS)
        .arg("--target")
        .arg(get_target_config_path(&target));

    build_cmd.arg("--profile");
    match mode {
        Profiles::Debug => {
            build_cmd.arg("dev");
        }
        Profiles::Release => {
            build_cmd.arg("release");
        }
    }

    match target {
        Target::Miralis => {
            // Linker arguments
            let start_address = cfg.target.miralis.start_address.unwrap_or(0x80000000);
            let linker_args = format!("-C link-arg=-Tmisc/linker-script.x -C link-arg=--defsym=_start_address={start_address}");
            build_cmd.arg("--package").arg("miralis");
            build_cmd.env("RUSTFLAGS", linker_args);

            // Environment variables
            build_cmd.envs(cfg.build_envs());
        }

        Target::Firmware(ref firmware) => {
            let firmware_address = cfg.target.firmware.start_address.unwrap_or(0x80200000);
            let linker_args = format!("-C link-arg=-Tmisc/linker-script.x -C link-arg=--defsym=_start_address={firmware_address}");
            build_cmd.env("RUSTFLAGS", linker_args);
            build_cmd.env("IS_TARGET_FIRMWARE", "true");
            build_cmd.envs(cfg.build_envs());
            build_cmd.arg("--package").arg(firmware);

            if firmware == "miralis" {
                build_cmd.env("MIRALIS_PLATFORM_NAME", "miralis");
            }
        }

        Target::Payload(ref payload_name) => {
            let payload_address = cfg
                .target
                .payload
                .as_ref()
                .and_then(|payload| payload.start_address)
                .unwrap_or(0x80400000);
            let linker_args = format!("-C link-arg=-Tmisc/linker-script.x -C link-arg=--defsym=_start_address={payload_address}");
            build_cmd.env("RUSTFLAGS", linker_args);
            build_cmd.env("IS_TARGET_FIRMWARE", "false");
            build_cmd.envs(cfg.build_envs());
            build_cmd.arg("--package").arg(payload_name);
        }
    }

    if !build_cmd.status().unwrap().success() {
        panic!("build failed with command : {:?}", build_cmd);
    }
    objcopy(&target, mode)
}

/// Extract raw binary from elf file.
///
/// Returns the path of the resulting binary.
fn objcopy(target: &Target, mode: Profiles) -> PathBuf {
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

// ————————————————————————————— List artifacts ————————————————————————————— //

pub fn list_artifacts(args: &ArtifactArgs) -> ExitCode {
    // Collect and sort the artifacts
    let manifest = read_artifact_manifest();
    let mut binary_artifacts: Vec<(&String, &Bin)> = manifest.bin.iter().collect();
    binary_artifacts.sort_by_key(|(name, _)| *name);

    // Display the list
    if args.markdown {
        println!("## Binary artifacts\n\n");
    }
    for (name, metadata) in binary_artifacts {
        if args.markdown {
            // Print as markdown
            println!("### {}\n", name);
            if let Some(ref desc) = metadata.description {
                println!("{}", desc);
            }
            if let Some(ref url) = metadata.url {
                println!("- [Download link]({})", url);
            }
            if let Some(ref repo) = metadata.repo {
                println!("- [Source repository]({})", repo)
            }
            println!();
        } else {
            // Otherwise simply print the name
            log::info!("{}", name)
        }
    }

    let mut disk_artifacts: Vec<(&String, &Disk)> = manifest.disk.iter().collect();
    disk_artifacts.sort_by_key(|(name, _)| *name);

    if args.markdown {
        println!("## Disk artifacts\n\n");
    }
    for (name, metadata) in disk_artifacts {
        if args.markdown {
            // Print as markdown
            println!("### {}\n", name);
            if let Some(ref desc) = metadata.description {
                println!("{}", desc);
            }
            if let Some(ref url) = metadata.url {
                println!("- [Download link]({})", url);
            }
            if let Some(ref repo) = metadata.repo {
                println!("- [Source repository]({})", repo)
            }
            println!();
        } else {
            // Otherwise simply print the name
            log::info!("{}", name)
        }
    }

    ExitCode::SUCCESS
}

// ————————————————————————————— Process disk image ————————————————————————————— //

/// Download a disk image if not already downloaded.
pub fn download_disk_image(name: &str, url_path: &str) {
    let file_name: &str = extract_file_name(url_path);

    if !is_file_present(&format!("artifacts/{}-miralis.img", name)) {
        // Download image
        log::info!("Disk image not found. Fetching the image...");
        let mut image_downloader = Command::new("wget");
        image_downloader.arg(url_path);
        image_downloader
            .status()
            .expect("Failed to download disk image");

        // Extract image
        log::info!("Extracting the disk image");

        // Handle case where it is .tar.smth
        let mut filename_without_extension = remove_file_extention(file_name);
        if filename_without_extension.contains(".tar") {
            todo!("Implement support for extracting files of type .tar.something");
        }

        let mut image_extractor;
        let compression_type = extract_file_extension(file_name);

        match compression_type {
            XZ_COMPRESSION => {
                image_extractor = Command::new("xz");
                image_extractor.arg("-dk").arg(file_name);
            }
            ZST_COMPRESSION => {
                image_extractor = Command::new("zstd");
                image_extractor.arg("-d").arg(file_name);
            }
            GZ_COMPRESSION => {
                image_extractor = Command::new("gunzip");
                image_extractor.arg("-d").arg(file_name);
            }
            IMG_EXTENSION | EXT2_EXTENSION => {
                image_extractor = Command::new("echo");
                image_extractor.arg("provided image is uncompressed, no extraction is required");
                filename_without_extension = file_name.parse().unwrap();
            }
            _ => todo!("Implement extraction for the compression type"),
        }

        image_extractor
            .status()
            .expect("Failed to downlaod Disk image");

        // Move and clean
        log::info!("Moving and cleaning the download");

        fs::rename(
            filename_without_extension,
            format!("artifacts/{}-miralis.img", name),
        )
        .unwrap();
        fs::remove_file(file_name).unwrap();

        log::info!("Disk image ready to use");
    } else {
        log::debug!("Disk image already exists.");
    }
}
