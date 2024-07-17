use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod artifacts;
mod config;
mod gdb;
mod path;
mod run;

// —————————————————————————————— CLI Parsing ——————————————————————————————— //

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Run Miralis on QEMU
    Run(RunArgs),
    /// Start GDB and connect to a running instance
    Gdb(GdbArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(long)]
    smp: Option<usize>,
    #[arg(long, action)]
    debug: bool,
    #[arg(long, action)]
    stop: bool,
    #[arg(short, long, action)]
    verbose: bool,
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
struct GdbArgs {}

// —————————————————————————————— Entry Point ——————————————————————————————— //

fn main() {
    let args = CliArgs::parse();
    match args.command {
        Subcommands::Run(args) => run::run(&args),
        Subcommands::Gdb(args) => gdb::gdb(&args),
    };
}
