# Writing Miralis Modules

Miralis is designed with a modular architecture to promote code reusability, maintainability, and clear separation of concerns. Modules are the fundamental building blocks of Miralis, encapsulating specific functionalities or features. Each module operates independently yet can interact with other modules through well-defined interfaces.

The core of a Miralis module is defined by the `Module` trait. This trait establishes a contract that all modules must adhere to, ensuring a consistent structure and interaction pattern across the system. By implementing the `Module` trait, developers can create new modules that seamlessly integrate into the Miralis ecosystem. This approach allows for extending Miralis's capabilities in a structured and maintainable manner.

## The `Module` Trait

The `Module` trait is the cornerstone of Miralis's modular system. It defines a set of hooks that Miralis calls at various points during its execution lifecycle. Implementing these hooks allows modules to intercept events, modify behavior, and extend Miralis's functionality.

```rust
pub trait Module {
    const NAME: &'static str;
    const NUMBER_PMPS: usize = 0;

    fn init() -> Self;
    fn name(&self) -> &'static str;

    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction;

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction;

    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction;

    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction;

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    );

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    );

    fn decided_next_exec_mode(
        &mut self,
        ctx: &mut VirtContext,
        previous_mode: ExecutionMode,
        next_mode: ExecutionMode,
    );

    fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext);
    fn on_shutdown(&mut self);
}
```

### Hooks

Hooks are functions defined in the `Module` trait that Miralis calls at specific moments. Modules can implement these hooks to customize Miralis's behavior.

#### `NAME: &'static str`
- **Purpose**: A constant that defines the unique name of the module. This name is used for logging and identification purposes.
- **Called**: Not called directly by Miralis, but used for identification.

#### `NUMBER_PMPS: usize = 0`
- **Purpose**: A constant that specifies the number of Physical Memory Protection (PMP) entries that the module requires. Miralis uses this information to allocate PMP resources.
- **Called**: Not called directly, but its value is used during Miralis's PMP setup.

#### `init() -> Self`
- **Purpose**: This function is responsible for initializing the module's state. It's the entry point for the module.
- **Called**: Once by Miralis at boot time for each hart.
- **Returns**: An instance of the module (`Self`).

#### `name(&self) -> &'static str`
- **Purpose**: Returns the name of the module. By default, it returns the value of the `NAME` constant.
- **Called**: Can be called by Miralis or other parts of the system for logging or identification.
- **Returns**: The static string slice representing the module's name.

#### `ecall_from_firmware(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction`
- **Purpose**: Handles an environment call (ecall) originating from the virtualized firmware. Modules can use this hook to implement custom ecall handlers specific to the firmware context.
- **Called**: When the virtualized firmware executes an `ecall` instruction.
- **Arguments**:
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context, providing access to global Miralis state.
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context of the hart that trapped, containing information about the firmware's state.
- **Returns**: A `ModuleAction` indicating whether Miralis should proceed with its default ecall handling (`Ignore`) or if the module has fully handled the ecall (`Overwrite`).

#### `ecall_from_payload(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction`
- **Purpose**: Handles an ecall originating from the payload (S-mode or U-mode software). Modules can use this hook to implement custom ecall handlers for the payload.
- **Called**: When the payload executes an `ecall` instruction.
- **Arguments**:
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context of the hart that trapped.
- **Returns**: A `ModuleAction` indicating whether the default ecall handling should be skipped.

#### `trap_from_firmware(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction`
- **Purpose**: Handles any trap (not just ecalls) originating from the virtualized firmware. This allows modules to react to various exceptions and interrupts occurring in the firmware.
- **Called**: When any trap (exception or interrupt) occurs while the virtualized firmware is executing.
- **Arguments**:
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context of the hart that trapped.
- **Returns**: A `ModuleAction` indicating whether Miralis should proceed with its default trap handling.

#### `trap_from_payload(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction`
- **Purpose**: Handles any trap originating from the payload. This allows modules to react to exceptions and interrupts occurring in the payload.
- **Called**: When any trap occurs while the payload is executing.
- **Arguments**:
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context of the hart that trapped.
- **Returns**: A `ModuleAction` indicating whether the default trap handling should be skipped.

#### `switch_from_payload_to_firmware(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext)`
- **Purpose**: Allows modules to perform actions when Miralis is about to switch execution from the payload to the virtualized firmware. This can be used for tasks like context saving or modifying CPU state before the firmware resumes.
- **Called**: Just before Miralis switches the execution context from payload mode to firmware mode.
- **Arguments**:
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context.
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
- **Returns**: Nothing. Miralis will always proceed with the switch.

#### `switch_from_firmware_to_payload(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext)`
- **Purpose**: Allows modules to perform actions when Miralis is about to switch execution from the virtualized firmware to the payload. This can be used for tasks like context restoration or enforcing security policies.
- **Called**: Just before Miralis switches the execution context from firmware mode to payload mode.
- **Arguments**:
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context.
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
- **Returns**: Nothing. Miralis will always proceed with the switch.

#### `decided_next_exec_mode(&mut self, ctx: &mut VirtContext, previous_mode: ExecutionMode, next_mode: ExecutionMode)`
- **Purpose**: This hook is called after Miralis has determined the next execution mode (e.g., firmware or payload) but before any actual context switch occurs. Modules can use this to gather statistics or perform logging based on execution mode transitions.
- **Called**: After a trap has been processed and Miralis has decided which mode (firmware or payload) will execute next.
- **Arguments**:
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context.
    - `previous_mode: ExecutionMode`: The execution mode before the trap.
    - `next_mode: ExecutionMode`: The execution mode that Miralis has decided will run next.
- **Returns**: Nothing.

#### `on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext)`
- **Purpose**: A callback for policy-defined Machine Software Interrupts (MSIs). Modules can use this to handle custom inter-hart synchronization or communication.
- **Called**: When a hart receives a policy MSI, typically triggered by another hart.
- **Arguments**:
    - `ctx: &mut VirtContext`: A mutable reference to the virtual context of the hart receiving the interrupt.
    - `mctx: &mut MiralisContext`: A mutable reference to the Miralis context.
- **Returns**: Nothing.

#### `on_shutdown(&mut self)`
- **Purpose**: Allows modules to perform cleanup or finalization tasks before Miralis shuts down.
- **Called**: When Miralis is initiated to shut down, before halting the system.
- **Returns**: Nothing.

## `ModuleAction` Enum

Several hooks in the `Module` trait return a `ModuleAction`. This enum tells Miralis whether it should proceed with its default behavior for the event or if the module has already handled it.

```rust
pub enum ModuleAction {
    Overwrite,
    Ignore,
}
```

- **`Overwrite`**: Signals to Miralis that the module has fully handled the event (e.g., an ecall or a trap). Miralis should not perform its default actions for this event and should consider it resolved.
- **`Ignore`**: Signals to Miralis that the module has observed the event but has not fully handled it, or that the module wishes Miralis to proceed with its default behavior. Miralis will continue its normal processing for the event.

## Creating a New Module

Creating a new module in Miralis involves defining a Rust struct and implementing the `Module` trait for it. This is typically done in a new Rust file within the Miralis source tree, often under a relevant subdirectory like `src/policy` or `src/benchmark`.

### Basic Structure

A module file generally follows this structure:

1.  **Module Struct Definition**: Define a struct that will hold any state your module needs. If your module is stateless, an empty struct is sufficient.
2.  **`Module` Trait Implementation**: Implement the `Module` trait for your struct. This involves:
    *   Defining the `NAME` constant.
    *   Optionally, defining the `NUMBER_PMPS` constant if your module requires PMP entries.
    *   Implementing the `init()` function to return an instance of your module struct.
    *   Implementing any other hooks your module needs to interact with Miralis.

### Example: A Simple Logging Module

Let's create a basic module that logs a message when the firmware makes an ecall.

```rust
// src/modules/simple_logger.rs (or any other appropriate path)

use crate::host::MiralisContext;
use crate::modules::{Module, ModuleAction};
use crate::virt::VirtContext;
use log::info; // Assuming the 'log' crate is used for logging

// 1. Define the module struct
pub struct SimpleLoggerModule;

// 2. Implement the Module trait
impl Module for SimpleLoggerModule {
    const NAME: &'static str = "SimpleLogger";
    // This module doesn't require PMP entries, so NUMBER_PMPS defaults to 0.

    fn init() -> Self {
        info!("[{}] Initializing SimpleLoggerModule", Self::NAME);
        SimpleLoggerModule
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn ecall_from_firmware(
        &mut self,
        _mctx: &mut MiralisContext, // Mark as unused if not needed
        _ctx: &mut VirtContext,    // Mark as unused if not needed
    ) -> ModuleAction {
        info!("[{}] Firmware ecall detected!", self.name());
        // We just want to log, not interfere with default handling.
        ModuleAction::Ignore
    }

    // Other hooks can be implemented here if needed.
    // If a hook is not implemented, its default empty behavior is used.
}
```

In this example:
- We define an empty struct `SimpleLoggerModule` as our module is stateless.
- We implement `Module` for `SimpleLoggerModule`.
    - `NAME` is set to "SimpleLogger".
    - `init()` prints an initialization message and returns an instance of `SimpleLoggerModule`.
    - `ecall_from_firmware()` prints a message indicating a firmware ecall has occurred. It returns `ModuleAction::Ignore` because we don't want to change Miralis's default ecall handling; we only want to log the event.

To integrate this module into Miralis, you would then need to add it to the `build_modules!` macro invocation in `src/modules.rs`.

## Enabling a Module

Miralis modules are enabled at compile-time. This means that to include a module in a Miralis build, you need to explicitly list it in the Miralis source code. This is done using the `build_modules!` macro found in `src/modules.rs`.

The `build_modules!` macro takes a list of module identifiers and their corresponding struct paths. Miralis uses this macro to generate the necessary code to incorporate the specified modules into the main Miralis application.

### Modifying `build_modules!`

To enable your newly created module, you need to edit the `src/modules.rs` file and add an entry for your module to the `build_modules!` macro invocation.

For example, if you created the `SimpleLoggerModule` in `src/modules/simple_logger.rs` (assuming `simple_logger` is a submodule of `crate::modules`), you would add it as follows:

```rust
// src/modules.rs

// ... other use statements and code ...

build_modules! {
    "keystone" => crate::policy::keystone::KeystonePolicy,
    "protect_payload" => crate::policy::protect_payload::ProtectPayloadPolicy,
    "offload" => crate::policy::offload::OffloadPolicy,
    "exit_counter" => crate::benchmark::counter::CounterBenchmark,
    "exit_counter_per_cause" => crate::benchmark::counter_per_cause::CounterPerMcauseBenchmark,
    "boot_counter" => crate::benchmark::boot::BootBenchmark,
    // Add your new module here:
    "simple_logger" => crate::modules::simple_logger::SimpleLoggerModule,
}

// ... rest of the file ...
```

**Explanation:**

-   `"simple_logger"`: This is a string literal that acts as a unique key or identifier for your module. Conventionally, this matches the module's file name or a descriptive short name.
-   `crate::modules::simple_logger::SimpleLoggerModule`: This is the full path to your module's struct.
    -   `crate` refers to the current crate (Miralis).
    -   `modules` would be the parent module where `simple_logger.rs` resides (if you've structured your project this way, e.g. by adding `mod simple_logger;` to `src/modules/mod.rs`).
    -   `simple_logger` is the name of your module file (and the submodule it defines).
    -   `SimpleLoggerModule` is the name of the struct implementing the `Module` trait.

After adding your module to `build_modules!` and ensuring the path is correct, recompiling Miralis will include your module's functionality. If the module is not listed here, its code will not be part of the compiled Miralis binary, and its hooks will not be called.

## Examples of Existing Modules

Miralis comes with several pre-built modules that serve as good examples of how to implement various functionalities. These are broadly categorized into Policy modules and Benchmark modules.

### Policy Modules

Policy modules are primarily concerned with security, isolation, and modifying the behavior of the virtualized firmware or payload. They often interact with PMP registers, handle specific ecalls, or manage transitions between execution modes.

-   **`KeystonePolicy`**:
    -   **Purpose**: Implements core components of the Keystone Enclave API. It manages secure enclaves, handles ecalls related to enclave creation, destruction, and attestation.
    -   **Source**: [`src/policy/keystone.rs`](../../src/policy/keystone.rs)
-   **`ProtectPayloadPolicy`**:
    -   **Purpose**: Aims to isolate the payload (e.g., an OS kernel) from the firmware. It achieves this by configuring PMP entries to restrict firmware access to payload memory during payload execution.
    -   **Source**: [`src/policy/protect_payload.rs`](../../src/policy/protect_payload.rs)
-   **`OffloadPolicy`**:
    -   **Purpose**: This module facilitates offloading certain operations or function calls from the firmware to Miralis itself or to a trusted execution environment. This can be used to provide services to the firmware in a more controlled manner.
    -   **Source**: [`src/policy/offload.rs`](../../src/policy/offload.rs)

### Benchmark Modules

Benchmark modules are designed to measure performance, count events, or gather statistics about the execution of Miralis, the virtualized firmware, or the payload.

-   **`CounterBenchmark` (`exit_counter`)**:
    -   **Purpose**: Counts the total number of exits (traps) from the virtualized firmware back to Miralis. This provides a simple measure of the frequency of firmware interventions.
    -   **Source**: [`src/benchmark/counter.rs`](../../src/benchmark/counter.rs)
-   **`CounterPerMcauseBenchmark` (`exit_counter_per_cause`)**:
    -   **Purpose**: Similar to `CounterBenchmark`, but it counts exits from the firmware grouped by the `mcause` (Machine Cause Register) value. This helps in understanding the types of traps that are most frequent.
    -   **Source**: [`src/benchmark/counter_per_cause.rs`](../../src/benchmark/counter_per_cause.rs)
-   **`BootBenchmark` (`boot_counter`)**:
    -   **Purpose**: Measures the "time" or number of instructions taken for the guest (firmware and payload) to boot. This is useful for performance analysis of the boot process.
    -   **Source**: [`src/benchmark/boot.rs`](../../src/benchmark/boot.rs)

These examples illustrate the diverse functionalities that can be implemented using the Miralis module system. Reviewing their source code can provide valuable insights into how to leverage different module hooks and interact with Miralis contexts.
