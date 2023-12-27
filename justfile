mirage_elf          := "target/riscv-unknown-mirage/debug/mirage"

fmt:
	cargo fmt

test:
	# Running tests...
	cargo run --package runner -- --payload ecall

	# Checking formatting...
	cargo fmt --all -- --check

run:
	cargo run --package runner -- -v --payload ecall

run-dbg:
	cargo run --package runner -- -v --payload ecall --dbg --stop

gdb:
	rust-gdb {{mirage_elf}} -q -ex "target remote :1234" -x "./config/setup.gdb"

# The following line gives highlighting on vim
# vim: set ft=make :
