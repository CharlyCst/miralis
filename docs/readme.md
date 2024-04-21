# Mirage Developer Documentation

This folder contains documentation that is primarily intended for people working on (rather than with) Mirage.
This documentation is intended to be living thing!
It will and should evolve along with the project.

## Architecture

The purpose of Mirage is to virtualize firmware.
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

The purpose of Mirage is decouple those two functions: on one hand it can support opaque firmware for managing the board, and on the other it can enforce security by isolating the OS.
The way Mirage achieve this is through firmware virtualization.
At a high level, a deployment on top of Mirage looks like this:

```
        ┌──────────────┐ ┌────────────┐
U-mode  │   User App   │ │  Firmware  │
        ├──────────────┤ └────────────┘
S-mode  │      OS      │               
        ├──────────────┴──────────────┐
M-mode  │            Mirage           │
        └─────────────────────────────┘
```

Mirage itself runs in M-mode in the place where one would usually find the firmware.
But because the hardware still requires a firmware to function properly, Mirage actually runs the firmware in U-mode and virtualizes all privileged operations, such as interacting with M-mode registers.
At the same time, Mirage allows running a standard OS like it usually would, in S-mode.
The OS can call into the firmware, and mirage will take care of forwarding those calls appropriately.
That way, Mirage manages to keep all the firmware functionalities, but can enforce strong security guarantees, such as ensuring that the firmware can never access the  OS memory.

## Contributing

The development of Mirage is done through pull requests against the `main` branch.
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
