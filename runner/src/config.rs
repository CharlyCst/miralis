//! Miralis configuration
//!
//! The configuration is read from the `config.toml` file by the runner which will configure the
//! appropriate environment variables during Miralis's build.

use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;
use std::{fmt, fs};

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
    pub benchmark: Benchmark,
    #[serde(default)]
    pub target: Targets,
    #[serde(default)]
    pub policy: Policy,
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
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Platforms {
    #[serde(rename = "qemu_virt")]
    QemuVirt,
    #[serde(rename = "visionfive2")]
    VisionFive2,
    #[serde(rename = "spike")]
    Spike,
}

impl fmt::Display for Platforms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platforms::QemuVirt => write!(f, "qemu_virt"),
            Platforms::VisionFive2 => write!(f, "visionfive2"),
            Platforms::Spike => write!(f, "spike"),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    pub enable: Option<bool>,
    pub csv_format: Option<bool>,
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
pub struct Policy {
    pub name: Option<PolicyModule>,
    pub payload_size: Option<usize>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum PolicyModule {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "keystone")]
    Keystone,
    #[serde(rename = "protect_payload")]
    ProtectPayload,
    #[serde(rename = "ace")]
    Ace,
}

impl fmt::Display for PolicyModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyModule::Default => write!(f, "default"),
            PolicyModule::Keystone => write!(f, "keystone"),
            PolicyModule::ProtectPayload => write!(f, "protect_payload"),
            PolicyModule::Ace => write!(f, "ace"),
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
        envs.extend(self.benchmark.build_envs());
        envs.extend(self.target.build_envs());
        envs.extend(self.policy.buid_envs());
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
        envs.insert("MIRALIS_LOG_LEVEL", &self.level);

        // Decides between colored and gray output
        envs.insert("MIRALIS_LOG_COLOR", &self.color);

        // Modules logged at error level
        envs.insert_array("MIRALIS_LOG_ERROR", &self.error);

        // Modules logged at warn level
        envs.insert_array("MIRALIS_LOG_WARN", &self.warn);

        // Modules logged at info level
        envs.insert_array("MIRALIS_LOG_INFO", &self.info);

        // Modules logged at debug level
        envs.insert_array("MIRALIS_LOG_DEBUG", &self.debug);

        // Modules logged at trace level
        envs.insert_array("MIRALIS_LOG_TRACE", &self.trace);

        envs.envs
    }
}

impl Debug {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert("MIRALIS_DEBUG_MAX_FIRMWARE_EXITS", &self.max_firmware_exits);
        envs.envs
    }
}

impl VCpu {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert("MIRALIS_VCPU_MAX_PMP", &self.max_pmp);
        envs.insert(
            "MIRALIS_DELEGATE_PERF_COUNTER",
            &self.delegate_perf_counters,
        );
        envs.envs
    }
}

impl Platform {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert("MIRALIS_PLATFORM_NAME", &self.name);
        envs.insert("MIRALIS_PLATFORM_NB_HARTS", &self.nb_harts);
        envs.insert("MIRALIS_PLATFORM_BOOT_HART_ID", &self.boot_hart_id);
        envs.envs
    }
}

impl Benchmark {
    pub fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert("MIRALIS_BENCHMARK", &self.enable);
        envs.insert("MIRALIS_BENCHMARK_CSV_FORMAT", &self.csv_format);
        envs.insert("MIRALIS_BENCHMARK_TIME", &self.time);
        envs.insert("MIRALIS_BENCHMARK_INSTRUCTION", &self.instruction);
        envs.insert("MIRALIS_BENCHMARK_NB_EXISTS", &self.nb_exits);
        envs.insert(
            "MIRALIS_BENCHMARK_NB_FIRMWARE_EXITS",
            &self.nb_firmware_exits,
        );
        envs.insert("MIRALIS_BENCHMARK_WORLD_SWITCHES", &self.world_switches);
        envs.insert("MIRALIS_BENCHMARK_NB_ITER", &self.nb_iter);
        envs.envs
    }
}

impl Targets {
    fn build_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert(
            "MIRALIS_TARGET_START_ADDRESS",
            &self.miralis.start_address.or(Some(0x80000000)),
        );
        envs.insert(
            "MIRALIS_TARGET_FIRMWARE_ADDRESS",
            &self.firmware.start_address.or(Some(0x80200000)),
        );
        envs.insert(
            "MIRALIS_TARGET_STACK_SIZE",
            &self.firmware.stack_size.or(Some(0x8000)),
        );
        envs.insert(
            "MIRALIS_TARGET_FIRMWARE_STACK_SIZE",
            &self.miralis.stack_size.or(Some(0x8000)),
        );

        envs.envs
    }
}

impl Policy {
    fn buid_envs(&self) -> HashMap<String, String> {
        let mut envs = EnvVars::new();
        envs.insert("MIRALIS_POLICY_NAME", &self.name);
        envs.insert("PAYLOAD_HASH_SIZE", &self.payload_size);
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
