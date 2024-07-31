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
    pub benchmark: Option<bool>,
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
    pub stack_size: Option<usize>,
    pub start_address: Option<usize>,
    pub firmware_address: Option<usize>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Platforms {
    #[serde(rename = "qemu_virt")]
    QemuVirt,
    #[serde(rename = "visionfive2")]
    VisionFive2,
}

// ————————————————————————— Environment Variables —————————————————————————— //

impl Config {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.extend(self.log.build_envs());
        envs.extend(self.debug.build_envs());
        envs.extend(self.vcpu.build_envs());
        envs.extend(self.platform.build_envs());
        envs.insert(
            String::from("BENCHMARK"),
            format!("{}", self.benchmark.unwrap_or(false)),
        );
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
        if let Some(stack_size) = self.stack_size {
            envs.insert(
                String::from("MIRALIS_PLATFORM_STACK_SIZE"),
                format!("{}", stack_size),
            );
        }
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
