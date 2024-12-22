//! Verification through model checking

use std::env;
use std::process::{Command, ExitCode};

use crate::{VerifyArgs, RUNNER_STRICT_MODE};

pub fn verify(args: &mut VerifyArgs) -> ExitCode {
    if env::var(RUNNER_STRICT_MODE).is_ok() && !args.strict {
        log::info!("Runing checks in strict mode");
        args.strict = true;
    }

    // When Kani is not installed we return an error in strict mode but simply mark it as a warning
    // if not. This makes the tests pass in local for people who did not install Kani yet.
    if !kani_is_available() {
        let message = "Kani is not installed, could not run model checking";
        if args.strict {
            log::error!("{}", message);
            return ExitCode::FAILURE;
        } else {
            log::warn!("{}", message);
            return ExitCode::SUCCESS;
        }
    }

    let mut kani_cmd = Command::new("cargo");
    kani_cmd
        .arg("kani")
        .args(["--output-format", "terse"])
        .args(["-p", "model_checking"]);

    // Filter by pattern, if any
    if let Some(pattern) = &args.pattern {
        kani_cmd.arg("--harness").arg(pattern);
    }

    let exit_status = kani_cmd.status().expect("Failed to run Kani");
    if exit_status.success() {
        log::info!("Successfully passed verification");
        ExitCode::SUCCESS
    } else {
        log::error!("Failed verification");
        ExitCode::FAILURE
    }
}

/// Returns true if Kani is available.
pub fn kani_is_available() -> bool {
    let mut kani_cmd = Command::new("cargo");
    kani_cmd.arg("kani").arg("--version");
    kani_cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    kani_cmd
        .status()
        .ok()
        .map_or(false, |status| status.success())
}
