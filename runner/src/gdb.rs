//! GDB subcommand
//!
//! The gdb subcommand launches a GDB session and attach it to a running Mirage instance.

use std::io;
use std::process::{exit, Command, Stdio};

use crate::artifacts::Target;
use crate::path::get_target_dir_path;
use crate::GdbArgs;

// ——————————————————————————————— Constants ———————————————————————————————— //

/// A list of GDB executables that support RISC-V 64
static GDB_EXECUTABLES: &[&'static str] = &["gdb-multiarch", "riscv64-elf-gdb"];

// —————————————————————————————————— GDB ——————————————————————————————————— //

/// Build a command to invoke GDB using the provided executable.
///
/// GDB can be distributed under different names, depending on the available targets, hence the
/// need for such a function.
fn build_gdb_command(gdb_executable: &str) -> Command {
    // Retrieve the path of Mirage's binary
    let mut mirage_path = get_target_dir_path(&Target::Mirage);
    mirage_path.push("mirage");

    let mut gdb_cmd = Command::new(gdb_executable);
    gdb_cmd
        .arg(&mirage_path)
        .arg("-q")
        .args(["-x", "./misc/setup.gdb"]);
    gdb_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    gdb_cmd
}

/// Start a GDB session
pub fn gdb(_args: &GdbArgs) -> ! {
    for gdb in GDB_EXECUTABLES {
        let mut gdb_cmd = build_gdb_command(gdb);
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
