//! Mirage configuration
//!
//! The configuration is read from the `config.toml` file by the runner which will configure the
//! appropriate environment variables during Mirage's build.

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::path::get_workspace_path;
use crate::Args;

// ——————————————————————————— Config Definition ———————————————————————————— //

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub debug: Debug,
    #[serde(default)]
    pub platform: Platform,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub level: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Debug {
    pub max_payload_exits: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub s_mode: Option<bool>,
}

// ————————————————————————— Environment Variables —————————————————————————— //

impl Config {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.extend(self.log.build_envs());
        envs.extend(self.debug.build_envs());
        envs.extend(self.platform.build_envs());
        envs
    }
}

impl Log {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(level) = &self.level {
            envs.insert(String::from("MIRAGE_LOG_LEVEL"), level.clone());
        }
        envs
    }
}

impl Debug {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(max_payload_exits) = self.max_payload_exits {
            envs.insert(
                String::from("MIRAGE_DEBUG_MAX_PAYLOAD_EXITS"),
                format!("{}", max_payload_exits),
            );
        }
        envs
    }
}

impl Platform {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        if let Some(s_mode) = self.s_mode {
            envs.insert(
                String::from("MIRAGE_PLATFORM_S_MODE"),
                format!("{}", s_mode),
            );
        }
        envs
    }
}

// ————————————————————————————— Config Loader —————————————————————————————— //

pub fn read_config(args: &Args) -> Config {
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
    let mut config = toml::from_str::<Config>(&config).expect("Failed to parse configuration");

    // Override some aspect of the config, if required by the arguments
    if let Some(max_exits) = args.max_exits {
        config.debug.max_payload_exits = Some(max_exits);
    }

    config
}
