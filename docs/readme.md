# Miralis Developer Documentation

This folder contains documentation that is primarily intended for people working on (rather than with) Miralis.
This documentation is intended to be living thing!
It will and should evolve along with the project.

## Architecture

The purpose of Miralis is to virtualize firmware.
In a standard RISC-V deployment, the firmware runs in M-mode, below the OS:

```
        ┌──────────────┐
U-mode  │   User App   │
        ├──────────────┤
S-mode  │      OS      │
        ├──────────────┤
M-mode  │   Firmware   │
        └──────────────┘
```

Because M-mode is all-powerful, it has absolute control over the OS, including reading and modifying its data.
The traditional purpose of the firmware is the manage the SoC, that is initializing and configuring all devices, manage power, monitor temperature and device health, etc...
In addition, the firmware is increasingly used for security-critical features, such as enforcing isolation.
For instance, on Arm the firmware (EL3 on that architecture) is responsible for enforcing the security guarantees of confidential VMs (see Arm CCA extension).

We end-up in a situation where the firmware has two roles: to manage the physical board, and to enforce security policies.
Unfortunately those two roles are in tension: hardware manufacturers tend to ship opaque firmware blob to manage proprietary hardware, while security require measured and open-source software to allow scrutiny.

The purpose of Miralis is decouple those two functions: on one hand it can support opaque firmware for managing the board, and on the other it can enforce security by isolating the OS.
The way Miralis achieve this is through firmware virtualization.
At a high level, a deployment on top of Miralis looks like this:

```
        ┌──────────────┐ ┌────────────┐
U-mode  │   User App   │ │  Firmware  │
        ├──────────────┤ └────────────┘
S-mode  │      OS      │
        ├──────────────┴──────────────┐
M-mode  │           Miralis           │
        └─────────────────────────────┘
```

Miralis itself runs in M-mode in the place where one would usually find the firmware.
But because the hardware still requires a firmware to function properly, Miralis actually runs the firmware in U-mode and virtualizes all privileged operations, such as interacting with M-mode registers.
At the same time, Miralis allows running a standard OS like it usually would, in S-mode.
The OS can call into the firmware, and miralis will take care of forwarding those calls appropriately.
That way, Miralis manages to keep all the firmware functionalities, but can enforce strong security guarantees, such as ensuring that the firmware can never access the  OS memory.

## PMP Virtualization

One of the main aspect of OS virtualization is MMU (Memory Management Unit) virtualization.
The MMU can be virtualized using either pure software shadow page tables, or using hardware assisted 2-level page tables.

In the case of Miralis we have no such concerns, because M-mode doesn't have access to an MMU (S-mode does, but Miralis doesn't need to virtualize it).
Instead, M-mode has access to PMP (Physical Memory Protection) registers, which falls under the category of MPU (Memory Protection Unit) often found in embedded micro-controllers.
Miralis needs to protect its own memory using PMP while still exposing PMP to the firmware to protect itself from the OS.
For that purpose Miralis needs to virtualize and multiplex the physical PMP registers.

PMP registers form an ordered list of physical memory ranges with attached access rights.
The first entry that matches a given address determines the access rights for that particular load or store.
For more details regarding PMP, please refer to the RISC-V privileged specification.

Miralis split PMP registers in four groups, as depicted bellow with the example of 8 physical PMP registers:

```
┌─────────┐ ─┐
│  PMP 0  │  │
├─────────┤  │ For Miralis use
│  PMP 1  │  │
├─────────┤ ─┤
│    0    │  │ Null entry
├─────────┤ ─┤
│ vPMP 0  │  │
├─────────┤  │
│ vPMP 1  │  │ Virtual PMP registers,
├─────────┤  │ dedicated for firmware
│ vPMP 2  │  │ use
├─────────┤  │
│ vPMP 3  │  │
├─────────┤ ─┤
│   All   │  │ Default allow/deny all
└─────────┘ ─┘
```

The first few registers are reserved for Miralis's own use.
They are placed first to take priority over firmware-controlled PMP registers.
Then Miralis inserts a null entry with address 0, this is required to ensure that the first virtual PMP behaves like the first physical PMP when using TOR (Top Of Range) addressing (refer to the spec for details).
Then the next PMP registers are exposed to the firmware as virtual PMP registers.
From the firmware point of view, it looks like if there were only PMP 0 to 3 in the example above.
Finally, the last entry is used by Miralis to either allow access to all memory when running the firmware (to emulate full memory access in virtual M-mode), or disallow all access when running in S or U-mode.

## Terminology

Throughout the project we strive to use clear and precise terminology.
This section serves as the reference for the definitions of the technical terms we use.

- **Miralis**:
  The software we are building.
  Miralis executes in M-mode and exposes a virtual M-mode.
- **Firmware**:
  We call 'firmware' an M-mode software.
  Examples of firmware are OpenSBI and FreeRTOS.
  In the context of Miralis, the firmware is the software we virtualize.
- **Payload**:
  Similar to the OpenSBI terminology, the payload is any S or U-mode software managed by the firmware.
- **Host**:
  Following the traditional virtualization terminology Miralis is called the host.
- **Guest**:
  The guest is the sum of the virtualized firmware and its payload.
  In other words, anything that executes in non M-mode on top of Miralis.

## Contributing

The development of Miralis is done through pull requests against the `main` branch.
We strive to maintain a clean linear history for the `main` branch, we rebase all PRs before merging and expect PRs to be rebased against the latest `main` branch.

An explicit goal is to ensure that all commits in the `main` branch pass the test suite of that commit, in other words `just test` must always succeed.
To enforce this the CI run the tests against each new commit when submitting a PR, if you see a failure in the CI check for the details to find out which commit caused the issue.
Of course writing code requires iteration, a good rule of thumb is to write a first version while committing along the way, and to rework those commits in a second time using tools such as `git rebase --interactive` or [jj](https://steveklabnik.github.io/jujutsu-tutorial/).

## Code Style

This section describes the style we strive to enforce across the code base, and serves as a reference when arbitrary choices need to be made.

**Comments**:

Comment starts with a leading white space and a capital letter, like this:

```rs
// Our comment style
```

But **not** like this:

```rs
//We avoid this
// or this
//and this
```

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
