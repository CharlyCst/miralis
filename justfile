default      := "default"
config       := "config.toml"
benchmark    := "ecall_benchmark"
spike        := "./config/spike.toml"
qemu_virt    := "./config/test/qemu-virt.toml"
visionfive_2 := "./config/visionfive2.toml"
premier_p550 := "./config/premierp550.toml"

# Print the list of commands
help:
	@just --list --unsorted

# Format all code
fmt:
	cargo fmt

# Run all the tests
test:
	# Running unit tests...
	@just unit-test

	# Checking formatting...
	cargo fmt --all -- --check

	# Checking configs...
	cargo run -q -- check-config ./config

	# Run linter...
	cargo clippy --features userspace -p miralis
	cargo clippy -p runner

	# Run integration tests...
	cargo run -- test --strict

	# Test firmware build
	just build-firmware default {{qemu_virt}}

	# Build Miralis for our boards
	just build {{visionfive_2}}
	just build {{premier_p550}}

# Run unit tests
unit-test:
	cargo test --features userspace --lib \
		-p miralis \
		-p model_checking

# Run Miralis
run firmware=default config=config:
	cargo run -- --verbose run  --config {{config}} --firmware {{firmware}}

# Build Miralis with the provided config
build config:
	cargo run -- build --config {{config}}

# Build a given firmware with the provided config
build-firmware firmware config=config:
	cargo run -- --verbose build --config {{config}} --firmware {{firmware}}

# Run Miralis but wait for a debugger to connect
debug firmware=default:
	cargo run -- --verbose run --firmware {{firmware}} --debug --stop

# Connect a debugger to a running Miralis instance
gdb:
	cargo run -- gdb

# Install the rust toolchain and required components
install-toolchain:
	rustup toolchain install $(cat rust-toolchain)
	rustup component add rustfmt --toolchain "$(cat rust-toolchain)"
	rustup component add rust-src --toolchain "$(cat rust-toolchain)"
	rustup component add llvm-tools-preview --toolchain "$(cat rust-toolchain)"
	rustup component add clippy --toolchain "$(cat rust-toolchain)"
	cargo install cargo-binutils
	cargo install --locked kani-verifier
	cargo kani setup

# The following line gives highlighting on vim
# vim: set ft=make :
