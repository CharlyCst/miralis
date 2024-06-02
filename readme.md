# Mirage

Mirage is an experimental system that virtualises firmware to enforce strong isolation between opaque and SoC-dependant firmware and user-controlled hypervisor or operating system.

## Getting Started

The Mirage project uses [`just`](https://github.com/casey/just) to easily build, run, and test code.
You can easily install `just` with your favorite package manager or `cargo` by following [the instructions](https://just.systems/man/en/chapter_4.html).

Mirage is primary developed and tested on QEMU, therefore you will need to install `qemu-system-riscv64` on your system.
Then you will need to install the rust toolchain, if rust is installed through rustup on the machine this can be done by running `just install-toolchain`

Then running Mirage is as simple as invoking `just run`.

## Project Organisation

The Mirage sources live in `src`, that is where the main body of code live for now.
Mirage does nothing by itself, however, and it needs a _firmware_ to virtualise.
The `firmware` folder contains simple firmware used for testing, including the `default` firmware selected by `just run`.

To make development and testing easier, the `runner` folder contains a simple command line tool to build, prepare, and load executables on QEMU.
The runner is invoked automatically by `just run`.

The `justfile` holds a collection of useful commands, you can think of it as similar to a Makefile without the C build system bits.
Running `just` or `just help` give the list of available commands.

## Testing and Debugging

Integration tests can be run with `just test`.

We provide a GDB script (in `misc/setup.gdb`) and `just` commands to facilitate debugging.
To start a GDB session, first run Mirage with `just debug` and then run `just gdb` in another terminal.

The firmware can be selected as an additional argument to `just run`.
Valid firmware are either names of firmware under the `./firmware/` directory, some pre-build binaries (such as `opensbi`), or paths to external firmware images.

The log level can be adjusted using a `config.toml` file. See `example.config.toml` for reference.

## Contributing

See [docs/readme.md](./docs/readme.md).
