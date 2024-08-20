//! Miralis configuration
//!
//! The configuration is read from the `config.toml` file by the runner which will configure the
//! appropriate environment variables during Miralis's build.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::path::get_workspace_path;
use crate::CheckConfigArgs;

// ——————————————————————————— Config Definition ———————————————————————————— //

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub debug: Debug,
    #[serde(default)]
    pub vcpu: VCpu,
    #[serde(default)]
    pub platform: Platform,
    #[serde(default)]
    pub benchmark: Benchmark,
    #[serde(default)]
    pub target: Targets,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub level: Option<String>,
    pub color: Option<bool>,
    pub error: Option<Vec<String>>,
    pub warn: Option<Vec<String>>,
    pub info: Option<Vec<String>>,
    pub debug: Option<Vec<String>>,
    pub trace: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Debug {
    pub max_firmware_exits: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct VCpu {
    pub s_mode: Option<bool>,
    pub max_pmp: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub name: Option<Platforms>,
    pub nb_harts: Option<usize>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Platforms {
    #[serde(rename = "qemu_virt")]
    QemuVirt,
    #[serde(rename = "visionfive2")]
    VisionFive2,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    pub enable: Option<bool>,
    pub time: Option<bool>,
    pub instruction: Option<bool>,
    pub nb_exits: Option<bool>,
    pub nb_firmware_exits: Option<bool>,
    pub world_switches: Option<bool>,
    pub nb_iter: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Targets {
    pub miralis: Target,
    pub firmware: Target,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub profile: Option<Profiles>,
    pub start_address: Option<usize>,
    pub stack_size: Option<usize>,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
pub enum Profiles {
    #[serde(rename = "dev")]
    #[default]
    Debug,
    #[serde(rename = "release")]
    Release,
}

// ————————————————————————— Environment Variables —————————————————————————— //

impl Config {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.extend(self.log.build_envs());
        envs.extend(self.debug.build_envs());
        envs.extend(self.vcpu.build_envs());
        envs.extend(self.platform.build_envs());
        envs.extend(self.benchmark.build_envs());
        envs.extend(self.target.build_envs());
        envs
    }
}

fn format_env_array(text: &Vec<String>) -> String {
    format!("{}", text.join(","))
}

impl Log {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();

        // Global log level
        if let Some(level) = &self.level {
            envs.insert(String::from("MIRALIS_LOG_LEVEL"), level.clone());
        }

        // Decides between colored and gray output
        if let Some(color) = self.color {
            envs.insert(String::from("MIRALIS_LOG_COLOR"), format!("{}", color));
        }

        // Modules logged at error level
        if let Some(error) = &self.error {
            envs.insert(String::from("MIRALIS_LOG_ERROR"), format_env_array(error));
        }

        // Modules logged at warn level
        if let Some(warn) = &self.warn {
            envs.insert(String::from("MIRALIS_LOG_WARN"), format_env_array(warn));
        }

        // Modules logged at info level
        if let Some(info) = &self.info {
            envs.insert(String::from("MIRALIS_LOG_INFO"), format_env_array(info));
        }

        // Modules logged at debug level
        if let Some(debug) = &self.debug {
            envs.insert(String::from("MIRALIS_LOG_DEBUG"), format_env_array(debug));
        }

        // Modules logged at trace level
        if let Some(trace) = &self.trace {
            envs.insert(String::from("MIRALIS_LOG_TRACE"), format_env_array(trace));
        }

        envs
    }
}

impl Debug {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(max_firmware_exits) = self.max_firmware_exits {
            envs.insert(
                String::from("MIRALIS_DEBUG_MAX_FIRMWARE_EXITS"),
                format!("{}", max_firmware_exits),
            );
        }
        envs
    }
}

impl VCpu {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(s_mode) = self.s_mode {
            envs.insert(String::from("MIRALIS_VCPU_S_MODE"), format!("{}", s_mode));
        }
        if let Some(max_pmp) = self.max_pmp {
            envs.insert(String::from("MIRALIS_VCPU_MAX_PMP"), format!("{}", max_pmp));
        }
        envs
    }
}

impl Platform {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(nb_harts) = self.nb_harts {
            envs.insert(
                String::from("MIRALIS_PLATFORM_NB_HARTS"),
                format!("{}", nb_harts),
            );
        }
        envs
    }
}

impl Benchmark {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(enable) = self.enable {
            envs.insert(String::from("MIRALIS_BENCHMARK"), format!("{}", enable));
        }
        if let Some(time) = self.time {
            envs.insert(String::from("MIRALIS_BENCHMARK_TIME"), format!("{}", time));
        }
        if let Some(instr) = self.instruction {
            envs.insert(
                String::from("MIRALIS_BENCHMARK_INSTRUCTION"),
                format!("{}", instr),
            );
        }
        if let Some(nb_exits) = self.nb_exits {
            envs.insert(
                String::from("MIRALIS_BENCHMARK_NB_EXISTS"),
                format!("{}", nb_exits),
            );
        }
        if let Some(nb_firmware_exits) = self.nb_firmware_exits {
            envs.insert(
                String::from("MIRALIS_BENCHMARK_NB_FIRMWARE_EXITS"),
                format!("{}", nb_firmware_exits),
            );
        }
        if let Some(world_switches) = self.world_switches {
            envs.insert(
                String::from("MIRALIS_BENCHMARK_WORLD_SWITCHES"),
                format!("{}", world_switches),
            );
        }
        if let Some(nb_iter) = self.nb_iter {
            envs.insert(
                String::from("MIRALIS_BENCHMARK_NB_ITER"),
                format!("{}", nb_iter),
            );
        }
        envs
    }
}

impl Targets {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        let firmware_address = self.firmware.start_address.unwrap_or(0x80200000);
        envs.insert(
            String::from("MIRALIS_TARGET_FIRMWARE_ADDRESS"),
            format!("{}", firmware_address),
        );
        let start_address = self.miralis.start_address.unwrap_or(0x80000000);
        envs.insert(
            String::from("MIRALIS_TARGET_START_ADDRESS"),
            format!("{}", start_address),
        );
        let firmware_stack_size = self.firmware.stack_size.unwrap_or(0x8000);
        envs.insert(
            String::from("MIRALIS_TARGET_STACK_SIZE"),
            format!("{}", firmware_stack_size),
        );
        let stack_size = self.miralis.stack_size.unwrap_or(0x8000);
        envs.insert(
            String::from("MIRALIS_TARGET_FIRMWARE_STACK_SIZE"),
            format!("{}", stack_size),
        );
        envs
    }
}

// ————————————————————————————— Config Loader —————————————————————————————— //

pub fn read_config(path: &Option<PathBuf>) -> Config {
    // Try to read config
    let config = if let Some(path) = path {
        fs::read_to_string(path)
    } else {
        let mut config_path = get_workspace_path();
        config_path.push("config.toml");
        fs::read_to_string(config_path)
    };
    let config = match config {
        Ok(config) => config,
        Err(_) => {
            println!("No config file found");
            // Creating a default config
            String::from("")
        }
    };

    // Parse the config and returns it
    match toml::from_str::<Config>(&config) {
        Ok(config) => config,
        Err(err) => panic!("Failed to parse configuration:\n{:#?}", err),
    }
}

// —————————————————————————————— Check Config —————————————————————————————— //

/// Print an error if the config is not valid.
pub fn check_config(args: &CheckConfigArgs) {
    let config = match fs::read_to_string(&args.config) {
        Ok(config) => config,
        Err(error) => {
            println!("Could not read config: {}", error);
            std::process::exit(1);
        }
    };

    match toml::from_str::<Config>(&config) {
        Ok(_) => println!("Config is valid"),
        Err(err) => println!("Config is not valid: {:?}", err),
    }
}
