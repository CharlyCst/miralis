//! Path helper functions

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{Target, MIRAGE_TARGET, PAYLOAD_TARGET};

/// Return the root of the workspace.
pub fn get_workspace_path() -> PathBuf {
    let Ok(runner_manifest) = std::env::var("CARGO_MANIFEST_DIR") else {
        panic!("Could not locate workspace root");
    };
    let path = PathBuf::from_str(&runner_manifest).unwrap();
    path.parent().unwrap().to_owned()
}

/// Return the target directory.
pub fn get_target_dir_path(target: &Target) -> PathBuf {
    let mut path = get_workspace_path();
    path.push("target");
    match target {
        Target::Mirage => path.push(MIRAGE_TARGET),
        Target::Payload(_) => path.push(PAYLOAD_TARGET),
    }
    path.push("debug"); // TODO: add support for release mode
    path
}

/// Return the path to the misc directory.
fn get_misc_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("misc");
    path
}

/// Return the target triple definition path for the provided target.
pub fn get_target_config_path(target: &Target) -> PathBuf {
    let mut path = get_misc_path();
    match target {
        Target::Mirage => {
            path.push(format!("{}.json", MIRAGE_TARGET));
        }
        Target::Payload(_) => path.push(format!("{}.json", PAYLOAD_TARGET)),
    }
    path
}

/// Return true if `b` is older than `b`
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

/// Return true if the payload is one of the knwon test payloads.
pub fn is_known_payload(name: &str) -> bool {
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
            return true;
        }
    }

    // Could not find payload
    false
}
