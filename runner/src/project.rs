//! Global project configuration

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::Deserialize;

/// The global project configuration file
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    #[serde(default)]
    pub config: IndexMap<String, Config>,
    #[serde(default)]
    pub test: IndexMap<String, Test>,
}

/// A Miralis configuration file
#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub path: PathBuf,
}

/// An integration test
#[derive(Deserialize, Debug, Default, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Test {
    pub config: String,
    pub description: Option<String>,
    pub firmware: Option<String>,
    pub payload: Option<String>,
}
