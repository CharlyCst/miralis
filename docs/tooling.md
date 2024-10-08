# Tooling

To make development easier and faster the Miralis project relies on a set of custom tools.
The main tool is the `runner`, which indeed started as a simple runner and build tool but whose scope has been extended over time.

## Configuration

Miralis uses [toml](https://toml.io/) for configuration.
Pre-defined configurations can be found in the `./config` directory.
In particular, the `./config/example.config.toml` list the valid configuration fields with explanatory comments.

The Miralis build tool will by default look for a configuration named `config.toml` at the root of this repository (this file is ignored by the `.gitignore`, so that each developer can keep its own), but there are default value for every field in case none is found.
Using a custom `config.toml` file is recommended for developing and debugging, it exposes utilities such as configuring the log level (globally or on a per-module basis) and limiting the maximum number of firmware traps to prevent infinite loops.

Configurations are especially important when building for a particular platform.
The `just build` command takes a configuration as argument for instance, from which the appropriate platform will be selected (for instance `visionfive2` or `qemu_virt`).

## Test Artifacts

In addition to writing our own test firmware, we test Miarge against third party projects.
But because it would be inconvenient for developers (and expensive on the CI) to build all third-party projects from source, our runner can download pre-compiled binaries that can be used for testing.
Available artifacts are described in an artifact manifest located at `mist/artifacts.toml`.
The manifest is a [toml](https://toml.io/) file that lists artifacts in the `[bin]` section with their corresponding download URL.

We externalize the build of artifacts to other repositories, so that we don't need to build them for each commit on the CI.
For instance, our test version of OpenSBI is built [here](https://github.com/CharlyCst/miralis-artifact-opensbi).
Those repos then expose artifacts as binary through releases, whose download link can be added to the artifact manifest.

When using `just run opensbi` the runner will check if we have a custom firmware in a crate named `opensbi`, and because that is not the case it will look-up the artifact manifest.
We do have an artifact named `opensbi`, so the runner will check if it is already downloaded, and download it if that is not the case (or if the artifact manifest has been updated since then).

## Benchmark

One can choose to activate benchmarking and what to benchmark in the `benchmark` section of the `config.toml` file.

Statistics are computed during the runtime in a streaming maner, then printed at the end of the run, either in a fancy way or in csv format. The firmware should ecall Miralis with FID 3 in order to ends the benchmark before exiting.

If you collect the csv output of a run into file (should be in csv format using `csv_format` in the config), you can feed the file to the just `analyze-benchmark` command to get the statistics of the run. You can also put multiple files of multiple runs into a folder and give the path of the folder. This will compute the average of all runs.

