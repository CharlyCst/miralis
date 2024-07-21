default          := "default"
qemu_virt        := "./config/test/qemu-virt.toml"
qemu_virt_2harts := "./config/test/qemu-virt.toml"

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
	cargo run -q --package runner -- check-config ./config/qemu_virt.toml
	cargo run -q --package runner -- check-config ./config/visionfive2.toml

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

	# Testing with external projects
	cargo run --package runner -- run --config {{qemu_virt}} --firmware opensbi

# Run unit tests
unit-test:
	cargo test --features userspace -p miralis

# Run Miralis
run firmware=default:
	cargo run --package runner -- run -v --firmware {{firmware}}

# Build Miralis with the provided config
build config:
	cargo run --package runner -- build --config {{config}}

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

# The following line gives highlighting on vim
# vim: set ft=make :
