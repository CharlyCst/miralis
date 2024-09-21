default          := "default"
config           := "config.toml"
benchmark        := "ecall_benchmark"
spike            := "./config/spike.toml"
qemu_virt        := "./config/test/qemu-virt.toml"
qemu_virt_2harts := "./config/test/qemu-virt.toml"
qemu_virt_benchmark := "./config/test/qemu-virt-benchmark.toml"
qemu_virt_release := "./config/test/qemu-virt-release.toml"
qemu_virt_hello_world_payload := "./config/test/qemu-virt-hello-world-payload.toml"
qemu_virt_hello_world_payload_spike := "./config/test/qemu-virt-hello-world-payload-spike.toml"
qemu_virt_sifive_u54 := "./config/test/qemu-virt-sifive-u54.toml"
qemu_virt_sifive_u54_spike := "./config/test/qemu-virt-sifive-u54-spike.toml"
qemu_virt_rustsbi_test_kernel := "./config/test/qemu-rustsbi-test-kernel.toml"
qemu_virt_rustsbi_test_kernel_spike := "./config/test/qemu-rustsbi-test-kernel-spike.toml"
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
	cargo test --features userspace -p miralis

	# Checking formatting...
	cargo fmt --all -- --check

	# Checking configs...
	cargo run -q -- check-config ./config

	# Run linter....
	cargo clippy --features userspace -p miralis
	cargo clippy -p runner
	cargo clippy -p benchmark_analyzer

	# Running integration tests...
	cargo run -- run --config {{qemu_virt}} --firmware ecall
	cargo run -- run --config {{qemu_virt}} --firmware csr_ops
	cargo run -- run --config {{qemu_virt}} --firmware pmp
	cargo run -- run --config {{qemu_virt}} --firmware breakpoint
	cargo run -- run --config {{qemu_virt}} --firmware mepc
	cargo run -- run --config {{qemu_virt}} --firmware mcause
	cargo run -- run --config {{qemu_virt}} --firmware mret
	cargo run -- run --config {{qemu_virt}} --firmware os_ctx_switch
	cargo run -- run --config {{qemu_virt}} --firmware sandbox
	cargo run -- run --config {{qemu_virt}} --firmware interrupt
	cargo run -- run --config {{qemu_virt}} --firmware os_ecall
	cargo run -- run --config {{qemu_virt}} --firmware vectored_mtvec
	cargo run -- run --config {{qemu_virt}} --firmware device
	cargo run -- run --config {{qemu_virt}} --firmware hypervisor
	cargo run -- run --config {{qemu_virt_release}} --firmware default

	# Testing with Miralis as firmware
	cargo run -- run --config {{qemu_virt}} --firmware miralis

	# Testing with external projects
	cargo run -- run --config {{qemu_virt}} --firmware opensbi
	cargo run -- run --config {{qemu_virt}} --firmware zephyr --max-exits 1000000
	cargo run -- run --config {{qemu_virt_hello_world_payload}} --firmware opensbi-jump
	cargo run -- run --config {{qemu_virt_sifive_u54}} --firmware linux
	cargo run -- run --config {{qemu_virt_rustsbi_test_kernel}} --firmware rustsbi-qemu

	# Test benchmark code
	cargo run -- run --config {{qemu_virt_benchmark}} --firmware csr_write
	cargo run -- run --config {{qemu_virt_benchmark}} --firmware ecall_benchmark

	# Test firmware build
	just build-firmware default {{qemu_virt}}

spike-test:
	# Running integration tests...
	cargo run -- run --config {{spike}} --firmware ecall
	cargo run -- run --config {{spike}} --firmware csr_ops
	cargo run -- run --config {{spike}} --firmware pmp
	cargo run -- run --config {{spike}} --firmware breakpoint
	cargo run -- run --config {{spike}} --firmware mepc
	cargo run -- run --config {{spike}} --firmware mcause
	cargo run -- run --config {{spike}} --firmware mret
	cargo run -- run --config {{spike}} --firmware os_ctx_switch
	cargo run -- run --config {{spike}} --firmware sandbox
	cargo run -- run --config {{spike}} --firmware interrupt
	cargo run -- run --config {{spike}} --firmware os_ecall
	cargo run -- run --config {{spike}} --firmware vectored_mtvec
	cargo run -- run --config {{spike}} --firmware device
	cargo run -- run --config {{spike}} --firmware default

	# Testing with Miralis as firmware
	cargo run -- run --config {{spike}} --firmware miralis

	# Testing with external projects
	# cargo run -- run --config {{spike}} --firmware opensbi
	cargo run -- run --config {{spike}} --firmware zephyr --max-exits 1000000
	# cargo run -- run --config {{qemu_virt_hello_world_payload_spike}} --firmware opensbi-jump
	# cargo run -- run --config {{qemu_virt_sifive_u54_spike}} --firmware linux
	# cargo run -- run --config {{qemu_virt_rustsbi_test_kernel_spike}} --firmware rustsbi-qemu

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
