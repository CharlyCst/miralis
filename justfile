mirage_elf          := "target/riscv-unknown-mirage/debug/mirage"

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

	# Checking formatting...
	cargo fmt --all -- --check

# Run Mirage with the default payload
run:
	cargo run --package runner -- -v --payload default

# Run Mirage but wait for a debugger to connect
run-dbg:
	cargo run --package runner -- -v --payload default --dbg --stop

# Connect a debugger to a running Mirage instance
gdb:
	rust-gdb {{mirage_elf}} -q -x "./config/setup.gdb"

# The following line gives highlighting on vim
# vim: set ft=make :
