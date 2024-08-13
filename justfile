default          := "default"
benchmark        := "ecall_benchmark"
qemu_virt        := "./config/test/qemu-virt.toml"
qemu_virt_2harts := "./config/test/qemu-virt.toml"
qemu_virt_benchmark := "./config/test/qemu-virt-benchmark.toml"
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
	cargo run -q --package runner -- check-config ./config/example.config.toml
	cargo run -q --package runner -- check-config ./config/qemu-virt.toml
	cargo run -q --package runner -- check-config ./config/visionfive2.toml
	cargo run -q --package runner -- check-config ./config/qemu-virt-benchmark.toml

	# Running integration tests...
	cargo run --package runner -- run --config {{qemu_virt}} --firmware ecall
	cargo run --package runner -- run --config {{qemu_virt}} --firmware csr_ops
	cargo run --package runner -- run --config {{qemu_virt}} --firmware pmp
	cargo run --package runner -- run --config {{qemu_virt}} --firmware breakpoint
	cargo run --package runner -- run --config {{qemu_virt}} --firmware mepc
	cargo run --package runner -- run --config {{qemu_virt}} --firmware mcause
	cargo run --package runner -- run --config {{qemu_virt}} --firmware mret
	cargo run --package runner -- run --config {{qemu_virt}} --firmware os_ctx_switch
	cargo run --package runner -- run --config {{qemu_virt}} --firmware sandbox
	cargo run --package runner -- run --config {{qemu_virt}} --firmware interrupt
	cargo run --package runner -- run --config {{qemu_virt}} --firmware os_ecall

	# Testing with external projects
	cargo run --package runner -- run --config {{qemu_virt}} --firmware opensbi
	cargo run --package runner -- run --config {{qemu_virt}} --firmware zephyr --max-exits 1000000

	# Test benchmark code
	cargo run --package runner -- run --config {{qemu_virt_benchmark}} --firmware csr_write --benchmark
	cargo run --package runner -- run --config {{qemu_virt_benchmark}} --firmware ecall_benchmark --benchmark

	# Test firmware build
	just build-firmware {{qemu_virt}} default

# Run unit tests
unit-test:
	cargo test --features userspace -p miralis

# Run Miralis
run firmware=default:
	cargo run --package runner -- run -v --firmware {{firmware}}

# Build Miralis with the provided config
build config:
	cargo run --package runner -- build --config {{config}}

# Build a given firmware with the provided config
build-firmware config firmware:
	cargo run --package runner -- build -v --config {{config}} --firmware {{firmware}}

# Run Miralis but wait for a debugger to connect
debug firmware=default:
	cargo run --package runner -- run -v --firmware {{firmware}} --debug --stop

# Connect a debugger to a running Miralis instance
gdb:
	cargo run --package runner -- gdb

# Install the rust toolchain and required components
install-toolchain:
	rustup toolchain install $(cat rust-toolchain)
	rustup component add rustfmt --toolchain "$(cat rust-toolchain)"
	rustup component add rust-src --toolchain "$(cat rust-toolchain)"
	rustup component add llvm-tools-preview --toolchain "$(cat rust-toolchain)"
	cargo install cargo-binutils

benchmark firmware=benchmark iterations=default_iterations:
	cargo run --package runner -- run -v --firmware {{firmware}} --benchmark --benchmark-iterations {{iterations}}

analyze-benchmark input_path:
	cargo run --package benchmark_analyzer -- {{input_path}}

# The following line gives highlighting on vim
# vim: set ft=make :
