rustflags           := "RUSTFLAGS='-C link-arg=-Tconfig/linker-script.x'"
build_std           := "-Zbuild-std=core,alloc"
build_features      := "-Zbuild-std-features=compiler-builtins-mem"
cargo_args          := build_std + " " + build_features
mirage_target       := "--target ./config/riscv-unknown-mirage.json"
payload_target      := "--target ./config/riscv-unknown-payload.json"
mirage_elf          := "target/riscv-unknown-mirage/debug/mirage"
mirage_img          := "target/riscv-unknown-mirage/debug/mirage.img"
payload_path        := "target/riscv-unknown-payload/debug"

build:
	{{rustflags}} cargo build {{mirage_target}} {{cargo_args}}
	rust-objcopy -O binary {{mirage_elf}} {{mirage_img}}

build-payload:
	cargo build {{payload_target}} {{cargo_args}} --package ecall
	rust-objcopy -O binary {{payload_path}}/ecall {{payload_path}}/ecall.img

fmt:
	cargo fmt

run:
	@just build-payload
	@just build
	qemu-system-riscv64 -machine virt -bios {{mirage_img}} -nographic -device loader,file={{payload_path}}/ecall.img,addr=0x80100000,force-raw=on

run-dbg:
	@just build
	qemu-system-riscv64 -machine virt -bios {{mirage_img}} -nographic -s -S -device loader,file={{payload_path}}/ecall.img,addr=0x80100000,force-raw=on

gdb:
	rust-gdb {{mirage_elf}} -q -ex "target remote :1234" -x "./config/config.gdb"

# The following line gives highlighting on vim
# vim: set ft=make :
