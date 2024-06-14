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
	# Running unit tests...
	cargo test --features userspace -p mirage

	# Running integration tests...
	cargo run --package runner -- run --max-exits 200 --firmware ecall
	cargo run --package runner -- run --max-exits 200 --firmware csr_ops
	cargo run --package runner -- run --max-exits 200 --firmware pmp
	cargo run --package runner -- run --max-exits 200 --firmware breakpoint
	cargo run --package runner -- run --max-exits 200 --firmware mepc
	cargo run --package runner -- run --max-exits 200 --firmware mcause
	cargo run --package runner -- run --max-exits 200 --firmware mret
	cargo run --package runner -- run --max-exits 200 --firmware os_ctx_switch
	cargo run --package runner -- run --max-exits 200 --firmware sandbox

	# Testing with external projects
	cargo run --package runner -- run --max-exits 2000 --firmware opensbi

	# Checking formatting...
	cargo fmt --all -- --check

# Run Mirage
run firmware=default:
	cargo run --package runner -- run -v --firmware {{firmware}}

# Run Mirage but wait for a debugger to connect
debug firmware=default:
	cargo run --package runner -- run -v --firmware {{firmware}} --debug --stop

# Connect a debugger to a running Mirage instance
gdb:
	rust-gdb {{mirage_elf}} -q -x "./misc/setup.gdb"

# Install the rust toolchain and required components
install-toolchain:
	rustup toolchain install $(cat rust-toolchain)
	rustup component add rustfmt --toolchain "$(cat rust-toolchain)"
	rustup component add rust-src --toolchain "$(cat rust-toolchain)"
	rustup component add llvm-tools-preview --toolchain "$(cat rust-toolchain)"
	cargo install cargo-binutils

# The following line gives highlighting on vim
# vim: set ft=make :
