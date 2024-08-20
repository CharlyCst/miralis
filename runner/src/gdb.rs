//! GDB subcommand
//!
//! The gdb subcommand launches a GDB session and attach it to a running Miralis instance.

use core::panic;
use std::io;
use std::process::{exit, Command, Stdio};

use crate::artifacts::Target;
use crate::config::{read_config, Profiles};
use crate::path::get_target_dir_path;
use crate::GdbArgs;

// ——————————————————————————————— Constants ———————————————————————————————— //

/// A list of GDB executables that support RISC-V 64
static GDB_EXECUTABLES: &[&'static str] = &[
    "gdb-multiarch",
    "riscv64-elf-gdb",
    "riscv64-unknown-linux-gnu-gdb",
];

// —————————————————————————————————— GDB ——————————————————————————————————— //

/// Build a command to invoke GDB using the provided executable.
///
/// GDB can be distributed under different names, depending on the available targets, hence the
/// need for such a function.
fn build_gdb_command(gdb_executable: &str, mode: Profiles) -> Command {
    // Retrieve the path of Miralis's binary
    let mut miralis_path = get_target_dir_path(&Target::Miralis, mode);
    miralis_path.push("miralis");

    let mut gdb_cmd = Command::new(gdb_executable);
    gdb_cmd
        .arg(&miralis_path)
        .arg("-q")
        .args(["-x", "./misc/setup.gdb"]);
    gdb_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    gdb_cmd
}

/// Start a GDB session
pub fn gdb(args: &GdbArgs) -> ! {
    let cfg = read_config(&args.config);
    let mode = cfg.target.miralis.profile.unwrap_or_default();

    for gdb in GDB_EXECUTABLES {
        let mut gdb_cmd = build_gdb_command(gdb, mode);

        // On Unix systems we can exec into the GDB command, this is a better solution as all
        // signals will be redirected to GDB rather than being handled by the parent process (i.e.
        // the runner).
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::process::CommandExt;
            let err = gdb_cmd.exec();
            if let io::ErrorKind::NotFound = err.kind() {
                // This GDB executable is not installed, try another one
                continue;
            } else {
                panic!("Failed to run GDB: {:?}", err);
            }
        }

        // On non-unix system we simply spawn a new process. This is not ideal, as the runner won't
        // relay signals, but it is the best we can do without adding too much complexity.
        #[allow(unreachable_code)]
        match gdb_cmd.output() {
            Ok(_) => exit(0), // Successfully launched GDB
            Err(err) => {
                if let io::ErrorKind::NotFound = err.kind() {
                    // This GDB executable is not installed, try another one
                    continue;
                } else {
                    panic!("Failed to run GDB: {:?}", err);
                }
            }
        }
    }

    // No GDB executable available.
    eprintln!("Could not find a GDB binary with RSIC-V support, try installing one of:");
    for gdb in GDB_EXECUTABLES {
        eprintln!("  - {}", gdb);
    }

    // Exit with non-zero exit code
    exit(1);
}
