use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use log::LevelFilter;

use crate::logger::RunnerLogger;

mod artifacts;
mod build;
mod config;
mod gdb;
mod logger;
mod path;
mod run;
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
    #[arg(short, long, default_value = "default")]
    firmware: String,
    #[arg(long)]
    /// Maximum number of firmware exits
    max_exits: Option<usize>,
    #[arg(long)]
    /// Path to the configuration file to use
    config: Option<PathBuf>,
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

fn main() {
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
        Subcommands::Gdb(args) => gdb::gdb(&args),
        Subcommands::CheckConfig(args) => config::check_config(&args),
        Subcommands::Artifact(args) => artifacts::list_artifacts(&args),
    };
}
