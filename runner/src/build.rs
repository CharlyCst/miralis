//! Build

use std::process::ExitCode;

use crate::artifacts::{build_target, prepare_firmware_artifact, Target};
use crate::config::read_config;
use crate::BuildArgs;

pub fn build(args: &BuildArgs) -> ExitCode {
    let cfg = read_config(&args.config);
    if let Some(firmware) = &args.firmware {
        let Some(firmware) = prepare_firmware_artifact(firmware, &cfg) else {
            return ExitCode::FAILURE;
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
