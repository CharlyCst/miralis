//! Miralis test runner

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::artifacts::{build_target, prepare_firmware_artifact, Target};
use crate::config::{read_config, Config, Platforms};
use crate::path::{get_project_config_path, make_path_relative_to_root};
use crate::project::{ProjectConfig, Test};
use crate::run::{get_qemu_cmd, get_spike_cmd, qemu_is_available, spike_is_available, QEMU, SPIKE};
use crate::TestArgs;

#[derive(Debug, PartialEq, Eq)]
struct TestGroup {
    config_path: PathBuf,
    config_name: String,
    tests: Vec<(String, Test)>,
}

#[derive(Default)]
struct TestStats {
    total: usize,
    success: usize,
    skipped: SkippedTests,
}

/// The count of skipped tests
#[derive(Default)]
struct SkippedTests {
    /// Skipped because QEMU is not available
    qemu: usize,
    /// Skipped because Spike is not available
    spike: usize,
}

/// The test command, run all the tests.
pub fn run_tests(args: &TestArgs) -> ExitCode {
    let mut stats = TestStats::default();
    let path = get_project_config_path();
    let config = match fs::read_to_string(&path) {
        Ok(config) => config,
        Err(_) => {
            log::error!("Could not read '{}'", &path.display());
            return ExitCode::FAILURE;
        }
    };

    // Parse the config
    let config = match toml::from_str::<ProjectConfig>(&config) {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to parse configuration:\n{}", err.message());
            return ExitCode::FAILURE;
        }
    };

    // Group tests by config files
    let mut test_groups = HashMap::new();
    for (cfg_name, cfg) in &config.config {
        test_groups.insert(
            cfg_name.clone(),
            TestGroup {
                config_path: make_path_relative_to_root(&cfg.path),
                config_name: cfg_name.clone(),
                tests: Vec::new(),
            },
        );
    }
    for (name, test) in &config.test {
        match test_groups.get_mut(&test.config) {
            Some(test_group) => {
                test_group.tests.push((name.clone(), test.clone()));
                stats.total += 1;
            }
            None => {
                log::error!("Invalid config name '{}' for test '{}'", test.config, name);
                return ExitCode::FAILURE;
            }
        }
    }

    // Check which emulators are available
    let qemu_available = qemu_is_available();
    let spike_available = spike_is_available();

    // Run tests, grouped by config (to minimize the need to re-compile)
    for (cfg_name, _) in &config.config {
        let test_group = &test_groups[cfg_name];
        let cfg = read_config(&Some(&test_group.config_path));
        for (test_name, test) in &test_group.tests {
            // Filter tests if a pattern is provided
            if let Some(pattern) = &args.pattern {
                if !test_name.starts_with(pattern) {
                    continue;
                }
            }

            // Skip tests if emulator not available
            match cfg.platform.name {
                None | Some(Platforms::QemuVirt) if !qemu_available => {
                    stats.skipped.qemu += 1;
                    continue;
                }
                Some(Platforms::Spike) if !spike_available => {
                    stats.skipped.spike += 1;
                    continue;
                }
                _ => (),
            }

            if let Err(cmd) = run_one_test(test, test_name, &cfg) {
                log::error!("Failed to run test '{}'", test_name);
                if let Some(cmd) = cmd {
                    log::info!("To reproduce, run:\n{}", cmd);
                }
                return ExitCode::FAILURE;
            } else {
                stats.success += 1;
            }
        }
    }

    // Display stats
    log::info!("\nTest done: {}/{}", stats.success, stats.total);
    if !qemu_available && stats.skipped.qemu > 0 {
        log::warn!(
            "{} is not available, skipped {} test{}",
            QEMU,
            stats.skipped.qemu,
            if stats.skipped.qemu > 1 { "s" } else { "" }
        );
    }
    if !spike_available && stats.skipped.spike > 0 {
        log::warn!(
            "{} is not available, skipped {} test{}",
            SPIKE,
            stats.skipped.spike,
            if stats.skipped.spike > 1 { "s" } else { "" }
        );
    }

    ExitCode::SUCCESS
}

/// Run one test, building the required artifacts as needed.
pub fn run_one_test(test: &Test, test_name: &str, cfg: &Config) -> Result<(), Option<String>> {
    log::info!("Running {}", test_name);

    // Build or retrieve the artifacts to run
    let miralis = build_target(Target::Miralis, cfg);
    let Some(firmware) = test.firmware.as_ref().or(cfg.target.firmware.name.as_ref()) else {
        log::error!("No firmware specified for test '{}'", test_name);
        return Err(None);
    };
    let Some(firmware) = prepare_firmware_artifact(firmware, cfg) else {
        log::error!("Failed to prepare firmware artifact '{}'", test_name);
        return Err(None);
    };

    let cmd = match cfg.platform.name.unwrap_or(Platforms::QemuVirt) {
        Platforms::QemuVirt => {
            get_qemu_cmd(cfg, miralis, firmware, test.payload.as_ref(), false, false)
        }
        Platforms::Spike => get_spike_cmd(cfg, miralis, firmware),
        invalid_platform => {
            log::error!("Invalid test platform: '{}'", invalid_platform);
            return Err(None);
        }
    };
    let Ok(mut cmd) = cmd else {
        log::error!("Failed to build command");
        return Err(None);
    };

    log::debug!(
        "{} {}",
        cmd.get_program().to_str().unwrap(),
        cmd.get_args()
            .map(|arg| arg.to_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let exit_status = cmd.status().expect("Failed to run");

    if !exit_status.success() {
        let cmd_str = format!(
            "{} {}",
            cmd.get_program().to_str().unwrap(),
            cmd.get_args()
                .map(|arg| arg.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(" ")
        );
        Err(Some(cmd_str))
    } else {
        Ok(())
    }
}
