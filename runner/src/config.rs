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

// ————————————————————————— Environment Variables —————————————————————————— //

impl Config {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.extend(self.log.build_envs());
        envs.extend(self.debug.build_envs());
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
    let config = toml::from_str::<Config>(&config).expect("Failed to parse configuration");
    config
}
