mirage_elf          := "target/riscv-unknown-mirage/debug/mirage"
default             := "default"

# Print the list of commands
help:
	@just --list --unsorted

# Format all code
fmt:
	cargo fmt

# Run all the tests
test:
	# Running tests...
	cargo run --package runner -- --payload ecall
	cargo run --package runner -- --payload mscratch
	cargo run --package runner -- --payload csr_ops

	# Checking formatting...
	cargo fmt --all -- --check

# Run Mirage
run payload=default:
	cargo run --package runner -- -v --payload {{payload}}

# Run Mirage but wait for a debugger to connect
debug payload=default:
	cargo run --package runner -- -v --payload {{payload}} --dbg --stop

# Connect a debugger to a running Mirage instance
gdb:
	rust-gdb {{mirage_elf}} -q -x "./config/setup.gdb"

# Install the rust toolchain and required components
install-toolchain:
	rustup toolchain install $(cat rust-toolchain)
	rustup component add rustfmt --toolchain "$(cat rust-toolchain)"
	rustup component add rust-src --toolchain "$(cat rust-toolchain)"
	rustup component add llvm-tools-preview --toolchain "$(cat rust-toolchain)"
	cargo install cargo-binutils

# The following line gives highlighting on vim
# vim: set ft=make :
