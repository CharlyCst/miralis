//! Miralis configuration
//!
//! The configuration is read from the `config.toml` file by the runner which will configure the
//! appropriate environment variables during Miralis's build.

use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;
use std::{fmt, fs};

use miralis_config as config;
use serde::Deserialize;
use walkdir::WalkDir;

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
    pub qemu: Qemu,
    #[serde(default)]
    pub target: Targets,
    #[serde(default)]
    pub modules: Modules,
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
    pub nb_iter: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct VCpu {
    pub max_pmp: Option<usize>,
    pub delegate_perf_counters: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub name: Option<Platforms>,
    pub nb_harts: Option<usize>,
    pub boot_hart_id: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Qemu {
    pub machine: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub disk: Option<String>,
    pub path: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Platforms {
    #[serde(rename = "qemu_virt")]
    QemuVirt,
    #[serde(rename = "spike")]
    Spike,
    #[serde(rename = "visionfive2")]
    VisionFive2,
    #[serde(rename = "premierp550")]
    PremierP550,
}

impl fmt::Display for Platforms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platforms::QemuVirt => write!(f, "qemu_virt"),
            Platforms::Spike => write!(f, "spike"),
            Platforms::VisionFive2 => write!(f, "visionfive2"),
            Platforms::PremierP550 => write!(f, "premierp550"),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Targets {
    pub miralis: Target,
    pub firmware: Target,
    pub payload: Option<Target>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub name: Option<String>,
    pub profile: Option<Profiles>,
    pub start_address: Option<usize>,
    pub stack_size: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Modules {
    pub modules: Vec<ModuleName>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum ModuleName {
    #[serde(rename = "keystone")]
    Keystone,
    #[serde(rename = "protect_payload")]
    ProtectPayload,
    #[serde(rename = "offload")]
    Offload,
    #[serde(rename = "boot_counter")]
    BootCounter,
    #[serde(rename = "exit_counter_per_cause")]
    ExitCounterPerCause,
    #[serde(rename = "exit_counter")]
    ExitCounter,
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleName::Keystone => write!(f, "keystone"),
            ModuleName::ProtectPayload => write!(f, "protect_payload"),
            ModuleName::Offload => write!(f, "offload"),
            ModuleName::BootCounter => write!(f, "boot_counter"),
            ModuleName::ExitCounterPerCause => write!(f, "exit_counter_per_cause"),
            ModuleName::ExitCounter => write!(f, "exit_counter"),
        }
    }
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
        envs.extend(self.target.build_envs());
        envs.extend(self.modules.buid_envs());
        envs
    }
}

struct EnvVars {
    envs: HashMap<String, String>,
}

impl EnvVars {
    fn new() -> Self {
        EnvVars {
            envs: HashMap::new(),
        }
    }

    pub fn insert<T: std::fmt::Display>(&mut self, var_name: &str, option: &Option<T>) {
        if let Some(value) = option {
            self.envs
                .insert(String::from(var_name), format!("{}", value));
        }
    }

    pub fn insert_array(&mut self, var_name: &str, option: &Option<Vec<String>>) {
        if let Some(values) = option {
            self.envs
                .insert(String::from(var_name), values.join(",").to_string());
        }
    }
}

impl Log {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();

        // Global log level
        envs.insert(config::LOG_LEVEL_ENV, &self.level);

        // Decides between colored and gray output
        envs.insert(config::LOG_COLOR_ENV, &self.color);

        // Modules logged at error level
        envs.insert_array(config::LOG_ERROR_ENV, &self.error);

        // Modules logged at warn level
        envs.insert_array(config::LOG_WARN_ENV, &self.warn);

        // Modules logged at info level
        envs.insert_array(config::LOG_INFO_ENV, &self.info);

        // Modules logged at debug level
        envs.insert_array(config::LOG_DEBUG_ENV, &self.debug);

        // Modules logged at trace level
        envs.insert_array(config::LOG_TRACE_ENV, &self.trace);

        envs.envs
    }
}

impl Debug {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert(config::MAX_FIRMWARE_EXIT_ENV, &self.max_firmware_exits);
        envs.insert(config::BENCHMARK_NB_ITER_ENV, &self.nb_iter);
        envs.envs
    }
}

impl VCpu {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert(config::VCPU_MAX_PMP_ENV, &self.max_pmp);
        envs.insert(
            config::DELEGATE_PERF_COUNTER_ENV,
            &self.delegate_perf_counters,
        );
        envs.envs
    }
}

impl Platform {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert(config::PLATFORM_NAME_ENV, &self.name);
        envs.insert(config::PLATFORM_NB_HARTS_ENV, &self.nb_harts);
        envs.insert(config::PLATFORM_BOOT_HART_ID_ENV, &self.boot_hart_id);
        envs.envs
    }
}

impl Targets {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();

        // Miralis
        envs.insert(
            config::TARGET_START_ADDRESS_ENV,
            &self.miralis.start_address.or(Some(0x80000000)),
        );
        envs.insert(
            config::TARGET_STACK_SIZE_ENV,
            &self.miralis.stack_size.or(Some(0x8000)),
        );

        // Firmware
        envs.insert(
            config::TARGET_FIRMWARE_ADDRESS_ENV,
            &self.firmware.start_address.or(Some(0x80200000)),
        );
        envs.insert(
            config::TARGET_FIRMWARE_STACK_SIZE_ENV,
            &self.firmware.stack_size.or(Some(0x8000)),
        );

        // Payload
        if let Some(payload_target) = &self.payload {
            envs.insert(
                config::TARGET_PAYLOAD_ADDRESS_ENV,
                &payload_target.start_address.or(Some(0x80200000)),
            );
            envs.insert(
                config::TARGET_PAYLOAD_STACK_SIZE_ENV,
                &payload_target.stack_size.or(Some(0x8000)),
            );
        }

        envs.envs
    }
}

impl Modules {
    fn buid_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        let modules = self
            .modules
            .iter()
            .map(|m| format!("{}", m))
            .collect::<Vec<String>>()
            .join(",");
        if !modules.is_empty() {
            envs.insert(config::MODULES_ENV, &Some(modules));
        }
        envs.envs
    }
}

// ————————————————————————————— Config Loader —————————————————————————————— //

pub fn read_config<P: AsRef<Path>>(path: &Option<P>) -> Config {
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
            log::warn!("No config file found, using default configuration");
            // Creating a default config
            String::from("")
        }
    };

    // Parse the config and returns it
    let mut cfg = match toml::from_str::<Config>(&config) {
        Ok(config) => config,
        Err(err) => panic!("Failed to parse configuration:\n{:#?}", err),
    };

    if cfg.qemu.cpu == Some(String::from("none")) {
        cfg.qemu.cpu = None;
    }

    cfg
}

// —————————————————————————————— Check Config —————————————————————————————— //

/// Print an error if the config is not valid.
pub fn check_config(args: &CheckConfigArgs) -> ExitCode {
    if args.config.is_file() {
        check_config_file(&args.config)
    } else {
        for entry in WalkDir::new(&args.config)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().unwrap_or_default() == "toml")
        {
            check_config_file(entry.path())
        }
    }

    ExitCode::SUCCESS
}

fn check_config_file(config: &Path) {
    let content = match fs::read_to_string(config) {
        Ok(content) => content,
        Err(error) => {
            log::error!("Could not read config: {}", error);
            std::process::exit(1);
        }
    };

    match toml::from_str::<Config>(&content) {
        Ok(_) => log::info!("Config {} is valid", config.display()),
        Err(err) => {
            log::error!("Config {} is not valid:\n{:?}", config.display(), err);
            std::process::exit(1);
        }
    }
}
