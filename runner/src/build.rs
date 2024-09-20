//! Build

use std::path::PathBuf;
use std::process::ExitCode;

use crate::artifacts::{build_target, download_artifact, locate_artifact, Artifact, Target};
use crate::config::read_config;
use crate::BuildArgs;

pub fn build(args: &BuildArgs) -> ExitCode {
    let cfg = read_config(&args.config);
    if let Some(firmware) = &args.firmware {
        let firmware = match locate_artifact(firmware) {
            Some(Artifact::Source { name }) => build_target(Target::Firmware(name), &cfg),
            Some(Artifact::Downloaded { name, url }) => download_artifact(&name, &url),
            None => {
                let path = PathBuf::from(firmware);
                if path.exists() {
                    path
                } else {
                    return ExitCode::FAILURE;
                }
            }
        };
        log::info!("Built firmware, binary available at:");
        log::info!("{}", firmware.display());
    } else {
        let miralis = build_target(Target::Miralis, &cfg);

        if let Some(config) = &args.config {
            log::info!(
                "Built Miralis with config '{}', binary available at:",
                config.display()
            );
        } else {
            log::info!("Built Miralis, binary available at:");
        }
        log::info!("{}", miralis.display());
    }

    ExitCode::SUCCESS
}
