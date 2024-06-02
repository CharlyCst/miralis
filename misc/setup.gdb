# Connect to QEMU
target remote :1234

# Set demangling on
set print demangle
set print asm-demangle

# Define an helper function to load firmware symbols.
# The symbols are not loaded by default to prevent collisions with Mirage's own symbols
define firmware
    add-symbol-file target/riscv-unknown-firmware/debug/default 0x80100000
end

# Helper function to print the next instructions
define asm
    x/10i $pc
end
