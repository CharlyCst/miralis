//! Mirage configuration
//!
//! The configuration is read from the `config.toml` file by the runner which will configure the
//! appropriate environment variables during Mirage's build.

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::path::get_workspace_path;
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
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub level: Option<String>,
    pub color: Option<bool>,
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

// ————————————————————————— Environment Variables —————————————————————————— //

impl Config {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.extend(self.log.build_envs());
        envs.extend(self.debug.build_envs());
        envs.extend(self.vcpu.build_envs());
        envs
    }
}

impl Log {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(level) = &self.level {
            envs.insert(String::from("MIRAGE_LOG_LEVEL"), level.clone());
        }
        if let Some(color) = self.color {
            envs.insert(String::from("MIRAGE_LOG_COLOR"), format!("{}", color));
        }
        envs
    }
}

impl Debug {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(max_firmware_exits) = self.max_firmware_exits {
            envs.insert(
                String::from("MIRAGE_DEBUG_MAX_FIRMWARE_EXITS"),
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
            envs.insert(String::from("MIRAGE_VCPU_S_MODE"), format!("{}", s_mode));
        }
        if let Some(max_pmp) = self.max_pmp {
            envs.insert(String::from("MIRAGE_VCPU_MAX_PMP"), format!("{}", max_pmp));
        }
        envs
    }
}

// ————————————————————————————— Config Loader —————————————————————————————— //

pub fn read_config() -> Config {
    // Try to read config
    let mut config_path = get_workspace_path();
    config_path.push("config.toml");
    let config = match fs::read_to_string(config_path) {
        Ok(config) => config,
        Err(_) => {
            println!("No config file found");
            // Creating a default config
            String::from("")
        }
    };

    // Parse the config and returns it
    toml::from_str::<Config>(&config).expect("Failed to parse configuration")
}
