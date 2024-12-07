use core::panic;
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command, ExitCode};

use clap::{Args, Parser, Subcommand};
use log::LevelFilter;

use crate::logger::RunnerLogger;

mod artifacts;
mod build;
mod config;
mod gdb;
mod logger;
mod path;
mod project;
mod run;
mod test;

// —————————————————————————————— CLI Parsing ——————————————————————————————— //

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: Subcommands,

    /// Enable verbose output
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Run Miralis on QEMU
    Run(RunArgs),
    /// Build Miralis
    Build(BuildArgs),
    /// Run the tests
    Test(TestArgs),
    /// Exit with an error if the config is not valid
    CheckConfig(CheckConfigArgs),
    /// Start GDB and connect to a running instance
    Gdb(GdbArgs),
    /// List the artifacts
    Artifact(ArtifactArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(long)]
    smp: Option<usize>,
    #[arg(long, action)]
    debug: bool,
    #[arg(long, action)]
    stop: bool,
    #[arg(short, long)]
    firmware: Option<String>,
    #[arg(long)]
    /// Maximum number of firmware exits
    max_exits: Option<usize>,
    #[arg(long)]
    /// Path to the configuration file to use
    config: Option<PathBuf>,
    /// An optional disk we can bind to qemu
    #[arg(long)]
    disk: Option<String>,
    /// Redirect the output of the run to a file
    #[arg(long)]
    output: Option<String>,
}

#[derive(Args)]
struct BuildArgs {
    #[arg(long)]
    /// Path to the configuration file to use
    config: Option<PathBuf>,
    /// Build a firmware instead of Miralis
    #[arg(short, long)]
    firmware: Option<String>,
}

#[derive(Args)]
struct TestArgs {
    /// Prefix of the tests to run, all if none
    pattern: Option<String>,
    /// The command will succeed only if all tests can be run successfully
    #[arg(long, action)]
    strict: bool,
}

#[derive(Args)]
struct CheckConfigArgs {
    /// Path to the configuration file or directory
    config: PathBuf,
}

#[derive(Args)]
struct GdbArgs {
    #[arg(long)]
    /// Path to the configuration file to use
    config: Option<PathBuf>,
}

#[derive(Args)]
struct ArtifactArgs {
    #[arg(long, action)]
    /// Print the list of artifacts in markdown format
    markdown: bool,
}

// —————————————————————————————— Entry Point ——————————————————————————————— //

fn main() -> ExitCode {
    let Some(root) = path::find_project_root() else {
        eprintln!(
            "Could not find the miralis project root, missing '{}'",
            path::PROJECT_CONFIG_FILE
        );
        return ExitCode::FAILURE;
    };

    if !is_running_through_cargo() {
        exec_cargo(root);
    }

    let args = CliArgs::parse();

    // Set the log level based on the --verbose flag
    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    RunnerLogger::init(log_level).unwrap();

    match args.command {
        Subcommands::Run(args) => run::run(&args),
        Subcommands::Build(args) => build::build(&args),
        Subcommands::Test(args) => test::run_tests(&args),
        Subcommands::Gdb(args) => gdb::gdb(&args),
        Subcommands::CheckConfig(args) => config::check_config(&args),
        Subcommands::Artifact(args) => artifacts::list_artifacts(&args),
    }
}

/// Return true if invoked by Cargo.
fn is_running_through_cargo() -> bool {
    env::var("CARGO_MANIFEST_DIR").is_ok()
}

/// Transfers the control to cargo.
///
/// When running in-tree this function makes it possible to use the latest version of the runner
/// transparently.
fn exec_cargo(path: PathBuf) -> ! {
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd
        .arg("run")
        .arg("--")
        .args(env::args().skip(1))
        .current_dir(path);

    // On Unix systems we symply exit
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::process::CommandExt;
        let err = cargo_cmd.exec();
        panic!("Failed to start cargo: {}", err);
    }
    // On other systems we start a new process
    #[allow(unreachable_code)]
    {
        match cargo_cmd.status() {
            Ok(exit_status) => exit(exit_status.code().unwrap_or(1)),
            Err(err) => panic!("Failed to start cargo: {}", err),
        }
    }
}
