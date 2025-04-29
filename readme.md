<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/miralis-firmware/assets/refs/heads/main/miralis_logo_1_github_dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/miralis-firmware/assets/refs/heads/main/miralis_logo_1_github_light.svg">
    <img src="https://raw.githubusercontent.com/miralis-firmware/assets/refs/heads/main/miralis_logo_1_github_light.svg" width="50%" alt="The Miralis logo"/>
  </picture>

  [Documentation][Documentation]
</div>
<br/>

Miralis is a RISC-V firmware that virtualizes RISC-V firmware.
We call Miralis a _Virtual Firmware Monitor_.

> **Status**: Miralis is a research project, and is not fit for production yet.

## Motivation

Usually, low level software is granted high privilege.
For instance, on RISC-V, platform-specific operations such as cache configuration and power management are handled in M-mode, with full access to all the machine's code and data.
This is not a great situation: any bug or vulnerability in the machine's firmware can take down or compromise the whole system.

This can be easily solved by re-designing system firmware, leveraging ideas from the multitude of micro-kernels.
Unfortunately, it is hard to convince all hardware vendors to re-design their firmware.
Miralis provides an alternative solution by efficiently de-privileging unmodified vendor firmware.

## How Does It Work?

On RISC-V, firmware runs in M-mode.
Miralis instead runs firmware in U-mode and emulates privileged instructions and memory accesses, creating the illusion of a virtual M-mode (vM-mode).
This is a classic virtualization technique also known as _trap and emulate_.

```
        ┌──────────────┐ ┌────────────┐
U-mode  │   User App   │ │  Firmware  │ vM-mode
        ├──────────────┤ └────────────┘
S-mode  │    Kernel    │
        ├──────────────┴──────────────┐
M-mode  │           Miralis           │
        └─────────────────────────────┘
```

## Quick Start

To get started, first install the dependencies:

1. Install Rust (see the [instructions](https://rust-lang.org/tools/install)).
2. Install [Just](https://github.com/casey/just) (can be installed with `cargo install just`).
3. Install `qemu-system-riscv64`:
    - On Ubuntu: `sudo apt install qemu-system-riscv64`.
    - On macOS: `brew install qemu`.

Then, in the Miralis folder:

4. Run `just install-toolchain` to install the required Rust components.
5. Run `just run` to run Miralis with a small firmware.

You should see an "Hello, world!".

Now, to run something slightly more interesting, try `just run linux-shell`.
This will download an OpenSBI + Linux image and run it on top of Miralis.
Although it looks like a standard Linux environment, the M-mode firmware (OpenSBI) is actually running in user-space!

## Going Further

For more details, user guides, and platform support see the [documentation][Documentation].
If you think Miralis can be useful to you, get in touch!
We are always looking for new use-cases.

[Documentation]: https://miralis-firmware.github.io/docs/introduction
