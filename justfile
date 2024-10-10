default          := "default"
config           := "config.toml"
benchmark        := "ecall_benchmark"
spike            := "./config/spike.toml"
qemu_virt        := "./config/test/qemu-virt.toml"
spike_virt_benchmark := "./config/test/spike-virt-benchmark.toml"
spike_latency_benchmark := "./config/test/spike-latency-benchmark.toml"
benchmark_folder := "./benchmark-out"
default_iterations := "1"


# Print the list of commands
help:
	@just --list --unsorted

# Format all code
fmt:
	cargo fmt

# Run all the tests
test:
	# Running unit tests...
	# cargo test --features userspace -p miralis

	# Checking formatting...
	cargo fmt --all -- --check

	# Checking configs...
	cargo run -q -- check-config ./config

	# Run linter...
	cargo clippy --features userspace -p miralis
	cargo clippy -p runner
	cargo clippy -p benchmark_analyzer

	# Run integration tests...
	cargo run -- test --strict

	# Ace policy
	cargo run -- run --config {{qemu_virt_u_boot_ace_policy}} --firmware opensbi-jump
	cargo run -- run --config {{qemu_virt_ace_policy}} --firmware linux

	# Test firmware build
	just build-firmware default {{qemu_virt}}

spike-benchmarks:
    cargo run -- run --config {{spike_latency_benchmark}} --firmware tracing_firmware
    cargo run -- run --config {{spike_virt_benchmark}} --firmware csr_write
    cargo run -- run --config {{spike_virt_benchmark}} --firmware ecall_benchmark

# Run unit tests
unit-test:
	cargo test --features userspace -p miralis

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

analyze-benchmark input_path:
	cargo run --package benchmark_analyzer -- {{input_path}}

# The following line gives highlighting on vim
# vim: set ft=make :
