//! Path helper functions

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::artifacts::{Target, FIRMWARE_TARGET, MIRALIS_TARGET, PAYLOAD_TARGET};
use crate::config::Profiles;

pub const XZ_COMPRESSION: &str = "xz";
pub const ZST_COMPRESSION: &str = "zst";
pub const GZ_COMPRESSION: &str = "gz";
pub const IMG_EXTENSION: &str = "img";

/// Return the root of the workspace.
pub fn get_workspace_path() -> PathBuf {
    let Ok(runner_manifest) = std::env::var("CARGO_MANIFEST_DIR") else {
        panic!("Could not locate workspace root");
    };
    let path = PathBuf::from_str(&runner_manifest).unwrap();
    path.parent().unwrap().to_owned()
}

/// Return the path to the projects config file
pub fn get_project_config_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("miralis.toml");
    path
}

/// Make a path relative to the workspace root (turning it absolute).
///
/// Absolute path are keept absolute.
pub fn make_path_relative_to_root(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        let mut absolute = get_workspace_path();
        absolute.push(path);
        absolute
    }
}

/// Return the target directory.
pub fn get_target_dir_path(target: &Target, mode: Profiles) -> PathBuf {
    let mut path = get_workspace_path();
    path.push("target");
    match target {
        Target::Miralis => path.push(MIRALIS_TARGET),
        Target::Firmware(_) => path.push(FIRMWARE_TARGET),
        Target::Payload(_) => path.push(PAYLOAD_TARGET),
    }
    match mode {
        Profiles::Debug => {
            path.push("debug");
        }
        Profiles::Release => {
            path.push("release");
        }
    }

    path
}

/// Return the path to the misc directory.
fn get_misc_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("misc");
    path
}

/// Return the path to the artifact manifest file.
pub fn get_artifact_manifest_path() -> PathBuf {
    let mut path = get_misc_path();
    path.push("artifacts.toml");
    path
}

/// Return the path to the artifacts forlder.
pub fn get_artifacts_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("artifacts");
    path
}

/// Return the target triple definition path for the provided target.
pub fn get_target_config_path(target: &Target) -> PathBuf {
    let mut path = get_misc_path();
    match target {
        Target::Miralis => path.push(format!("{}.json", MIRALIS_TARGET)),
        Target::Firmware(_) => path.push(format!("{}.json", FIRMWARE_TARGET)),
        Target::Payload(_) => path.push(format!("{}.json", PAYLOAD_TARGET)),
    }
    path
}

/// Return true if `a` is older than `b`
pub fn is_older(a: &Path, b: &Path) -> bool {
    let Ok(a_meta) = a.metadata() else {
        return false;
    };
    let Ok(b_meta) = b.metadata() else {
        return false;
    };

    match (a_meta.modified(), b_meta.modified()) {
        (Ok(a), Ok(b)) => a <= b,
        _ => false,
    }
}

pub fn is_file_present(image_path: &str) -> bool {
    Path::new(image_path).exists()
}

pub fn extract_file_extension(image_path: &str) -> &str {
    Path::new(image_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap()
}

pub fn extract_file_name(image_path: &str) -> &str {
    Path::new(image_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap()
}

pub fn remove_file_extention(path: &str) -> String {
    if let Some(stem) = Path::new(path).file_stem() {
        if let Some(parent) = Path::new(path).parent() {
            return parent.join(stem).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}
