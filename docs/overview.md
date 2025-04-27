---
title: Project Overview
sidebar_position: 3
---

This page gives a high-level overview of the Miralis project and how to work with it.

## Project Organization

The Miralis sources live in `src`, and contains most of the code.
Miralis does nothing by itself, however, and it needs some _firmware_ to virtualize.
The `firmware` folder contains simple test firmware, including the `default` firmware selected by `just run`.
Similarly, the `payload` folder contains simple test kernels.

The `runner` folder holds the code for the Miralis build tool (the `runner`, see [Tools](#Tools)).
For some operations, the runner downloads artifacts, such as test firmware images.
Those artifacts are stored in the `artifact` folder.

Another folder of interest is `config`, which stores pre-defined configuration files for testing purposes and for supported hardware platforms.

Finally, we use formal methods to verify correctness properties of key components of Miralis.
All the code related to verification is located under `model_checking`.

## Tools

The Miralis project uses [`just`](https://github.com/casey/just) to easily build, run, and test code.
The `just` command can be installed with your favorite package manager or with `cargo install just`.

The `justfile` holds a collection of useful commands; it is similar to a Makefile without the C build system bits.
`just` is used to provide a stable interface for basic operations, for instance `just test` is guaranteed to work for all commits.
Running `just` or `just help` gives the list of available commands.

Most of the `justfile` commands internally invoke the `runner`.
The `runner` is Miralis' build tool; it automates operations such as building Miralis and the test firmware, downloading artifacts such as Linux images, checking the format of configuration files, and orchestrates the integration tests.

The `runner` can be invoked with `cargo run -- <runner_command>` from the root of the repository.
Alternatively, the recommended option is to install the runner with `cargo install --path runner` and then simply invoke it with `runner <runner_command>` from anywhere in the repository.

To list all `runner` commands, run `runner -h`, and `runner <runner_command> -h` to list the options for a subcommand.

## Testing and Debugging

Integration tests can be run with `just test`.

The firmware can be selected as an additional argument to `just run`.
Valid firmware are either names of firmware under the `./firmware/` directory, some pre-built binaries (such as `opensbi`), or paths to external firmware images.
Thus, `just run opensbi` will execute OpenSBI on top of Miralis.

The list of integration tests is defined in `miralis.toml`.
For more control over the tests to run, the `runner` allows filtering by test name.
For instance, `runner test linux` will run all tests involving the Linux kernel.

We provide support for debugging with GDB.
To start a GDB session, first run Miralis with `just debug` and then run `just gdb` in another terminal.
Similar to `just run`, `just debug` takes an optional firmware argument which can be used to debug a particular image.
Debugging with GDB requires a RISC-V capable GDB executable in your path.
If `just gdb` can't locate such a binary it will provide a list of supported GDB binaries, installing any one of them will resolve the issue.

The log level can be adjusted using a `config.toml` file. See `./config/example.config.toml` for reference.

## Build Configuration

Miralis uses [toml](https://toml.io/) for configuration.
Pre-defined build configurations can be found in the `./config` directory.
In particular, the `./config/example.config.toml` list the valid configuration fields with explanatory comments.

By default, the `runner` looks for a configuration named `config.toml` at the root of this repository (this file is ignored by the `.gitignore`, so that each developer can keep their own).
If none is found, the `runner` resorts to default values.
Using a custom `config.toml` file is recommended for developing and debugging.
It exposes utilities such as configuring the log level (globally or on a per-module basis) and limiting the maximum number of firmware traps to prevent infinite loops.

Configurations are especially important when building for a particular platform.
The `just build` command takes a configuration as argument.
This is how Miralis can be built to target specific platforms (such as the `visionfive2` or `qemu_virt`).

## Test Artifacts

In addition to writing our own test firmware, we test Miralis against third-party projects.
Because it would be inconvenient for developers (and expensive on the CI) to build all third-party projects from source, our runner can download pre-compiled binaries that can be used for testing.
Available artifacts are described in an artifact manifest located at `misc/artifacts.toml`.
The manifest is a [toml](https://toml.io/) file that lists artifacts in the `[bin]` section with their corresponding download URL.

We externalize the build of artifacts to other repositories, so that we don't need to build them for each commit on the CI.
For instance, our test version of OpenSBI is built [here](https://github.com/CharlyCst/miralis-artifact-opensbi).
These repositories then expose artifacts as binaries through releases, whose download link can be added to the artifact manifest.

When using `just run opensbi`, the runner will check if there exits a test firmware in a crate named `opensbi`, and because that is not the case, it will look-up the artifact manifest.
There is an artifact named `opensbi`, so the runner will check if it is already downloaded, and download it if that is not the case (or if the artifact manifest has been updated since then).

