rustflags           := "RUSTFLAGS='-C link-arg=-Tlinker-script.x'"
build_std           := "-Zbuild-std=core,alloc"
build_features      := "-Zbuild-std-features=compiler-builtins-mem"
cargo_args          := build_std + " " + build_features
cargo_target        := "--target ./riscv-unknown-kernel.json"
mirage_elf          := "target/riscv-unknown-kernel/debug/mirage"
mirage_img          := "target/riscv-unknown-kernel/debug/mirage.img"

build:
	{{rustflags}} cargo build {{cargo_target}} {{cargo_args}}
	rust-objcopy -O binary {{mirage_elf}} {{mirage_img}}

run:
	@just build
	qemu-system-riscv64 -machine virt -bios {{mirage_img}} -nographic

run-dbg:
	@just build
	qemu-system-riscv64 -machine virt -bios {{mirage_img}} -nographic -s -S

gdb:
	rust-gdb {{mirage_elf}} -q -ex "target remote :1234"

# The following line gives highlighting on vim
# vim: set ft=make :
