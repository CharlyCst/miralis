# Miralis Module Development Guide

## Table of Contents

1. [Overview](#overview)
2. [Module System Architecture](#module-system-architecture)
3. [Creating a Module](#creating-a-module)
4. [Module Hooks Reference](#module-hooks-reference)
5. [Registering and Enabling Modules](#registering-and-enabling-modules)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

## Overview

Miralis provides a modular extension system that allows developers to add custom functionality to the hypervisor without modifying core virtualization logic. By default, Miralis only virtualizes a RISC-V firmware (M-mode program). All additional functionality—such as intercepting firmware traps, enforcing isolation policies, or monitoring behavior—must be implemented through modules.

### When to Use Modules

Modules are appropriate for:

- **Security Policies**: Enforcing isolation between firmware and payload (e.g., `protect_payload`, `keystone`)
- **Performance Optimization**: Reducing world switches by handling operations in Miralis (e.g., `offload`)
- **Monitoring & Benchmarking**: Collecting statistics about firmware/payload behavior (e.g., `exit_counter`, `boot`)
- **Custom Extensions**: Implementing new SBI extensions or custom ecalls

## Module System Architecture

### The Module Trait

The `Module` trait (defined in `src/modules.rs`) is the core interface for extending Miralis. It exposes several hooks that are called during firmware virtualization:

```rust
pub trait Module {
    /// The name of the module
    const NAME: &'static str;

    /// Number of PMP entries required by this module
    const NUMBER_PMPS: usize = 0;

    /// Initialize the module at boot time
    fn init() -> Self;

    // Optional hooks (see Module Hooks Reference section)
    fn ecall_from_firmware(...) -> ModuleAction { ... }
    fn ecall_from_payload(...) -> ModuleAction { ... }
    fn trap_from_firmware(...) -> ModuleAction { ... }
    fn trap_from_payload(...) -> ModuleAction { ... }
    fn switch_from_payload_to_firmware(...) { ... }
    fn switch_from_firmware_to_payload(...) { ... }
    fn decided_next_exec_mode(...) { ... }
    fn on_interrupt(...) { ... }
    fn on_shutdown(&mut self) { ... }
}
```

### Module Actions

Hooks that return `ModuleAction` can control whether Miralis continues normal processing:

- **`ModuleAction::Overwrite`**: The module has handled the event completely. Miralis will not perform its default action.
- **`ModuleAction::Ignore`**: The module did not handle the event. Miralis will proceed normally.

### Compilation and Aggregation

Modules are selected at **compile time** using the `build_modules!` macro in `src/modules.rs`. This macro generates a `MainModule` struct that aggregates all enabled modules and dispatches hook calls to them in order.

## Creating a Module

### Step 1: Choose a Location

Place your module in one of these directories:

- `src/policy/` - For security policy modules
- `src/benchmark/` - For benchmarking/monitoring modules
- Create a new directory for other module types

### Step 2: Implement the Module Trait

Here's a minimal module template:

```rust
use crate::modules::{Module, ModuleAction};
use crate::host::MiralisContext;
use crate::virt::VirtContext;

pub struct MyModule {
    // Module state
    counter: usize,
}

impl Module for MyModule {
    const NAME: &'static str = "My Module";
    const NUMBER_PMPS: usize = 0; // Set to number of PMPs needed

    fn init() -> Self {
        // Initialize module state
        MyModule { counter: 0 }
    }

    // Implement only the hooks you need
    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        // Your logic here
        ModuleAction::Ignore
    }
}
```

### Step 3: Key Parameters

#### Context Parameters

Most hooks receive these parameters:

- **`&mut self`**: Mutable reference to module state (per-hart)
- **`mctx: &mut MiralisContext`**: Miralis hardware context including:
  - `mctx.pmp`: PMP configuration
  - `mctx.hw`: Hardware state (hart ID, available registers)
- **`ctx: &mut VirtContext`**: Virtualized context including:
  - `ctx.regs`: General-purpose registers (x0-x31)
  - `ctx.pc`: Program counter
  - `ctx.csr`: Virtualized CSR registers
  - `ctx.trap_info`: Information about the current trap
  - `ctx.hart_id`: Current hart ID
  - `ctx.mode`: Current execution mode

#### PMP Requirements

If your module needs to use PMPs (Physical Memory Protection entries):

1. Set `NUMBER_PMPS` to the required count
2. Use PMP IDs starting from `MODULE_OFFSET + your_offset`
3. Example from `keystone.rs`:

```rust
const NUMBER_PMPS: usize = ENCL_MAX * 2; // 2 PMPs per enclave

// Usage:
let pmp_id = MODULE_OFFSET + enclave.eid * 2;
mctx.pmp.set_inactive(pmp_id, enclave.epm.start());
mctx.pmp.set_tor(pmp_id + 1, end_addr, pmpcfg::RWX);
```

## Module Hooks Reference

### Initialization

#### `fn init() -> Self`

**When called**: Once per hart during Miralis boot

**Purpose**: Initialize module state

**Example**:
```rust
fn init() -> Self {
    MyModule {
        state: Default::default(),
        initialized: true,
    }
}
```

### Trap Handling

#### `fn ecall_from_firmware(&mut self, mctx, ctx) -> ModuleAction`

**When called**: When virtualized firmware makes an ecall

**Purpose**: Intercept or monitor firmware ecalls

**Common use cases**:
- Implementing custom firmware interfaces
- Monitoring firmware behavior

**Example** (from `counter.rs`):
```rust
fn ecall_from_firmware(&mut self, _mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction {
    if ctx.get(Register::X17) == abi::MIRALIS_EID {
        // Handle custom Miralis ecall
        self.read_counters(ctx);
        ctx.pc += 4; // Skip the ecall instruction
        return ModuleAction::Overwrite;
    }
    ModuleAction::Ignore
}
```

#### `fn ecall_from_payload(&mut self, mctx, ctx) -> ModuleAction`

**When called**: When payload (OS/application) makes an ecall

**Purpose**: Intercept SBI calls, implement custom extensions

**Common use cases**:
- Implementing new SBI extensions (e.g., Keystone enclave management)
- Filtering/modifying SBI calls for security

**Example** (from `keystone.rs`):
```rust
fn ecall_from_payload(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction {
    let eid = ctx.get(Register::X17); // Extension ID
    let fid = ctx.get(Register::X16); // Function ID

    if eid != sbi::KEYSTONE_EID {
        return ModuleAction::Ignore; // Not our extension
    }

    let return_code = match fid {
        sbi::CREATE_ENCLAVE_FID => self.create_enclave(mctx, ctx),
        sbi::RUN_ENCLAVE_FID => self.run_enclave(mctx, ctx),
        _ => ReturnCode::NotImplemented,
    };

    ctx.set(Register::X10, return_code as usize); // Return value in a0
    ctx.pc += 4; // Skip the ecall instruction
    ModuleAction::Overwrite
}
```

#### `fn trap_from_firmware(&mut self, mctx, ctx) -> ModuleAction`

**When called**: When firmware takes any trap (including but not limited to ecalls)

**Purpose**: Handle all types of firmware traps

**Example** (from `protect_payload.rs`):
```rust
fn trap_from_firmware(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction {
    match ctx.trap_info.get_cause() {
        MCause::LoadAddrMisaligned => {
            if emulate_misaligned_read(ctx, mctx).is_err() {
                ctx.emulate_payload_trap();
            }
            ModuleAction::Overwrite
        }
        _ => ModuleAction::Ignore,
    }
}
```

#### `fn trap_from_payload(&mut self, mctx, ctx) -> ModuleAction`

**When called**: When payload takes any trap

**Purpose**: Handle payload traps (page faults, illegal instructions, etc.)

**Common use cases**:
- Emulating privileged instructions
- Handling page faults
- Emulating misaligned accesses

**Example** (from `offload.rs`):
```rust
fn trap_from_payload(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction {
    match ctx.trap_info.get_cause() {
        MCause::IllegalInstr => {
            let instr = unsafe { get_raw_faulting_instr(ctx) };

            // Check if it's a CSRR time instruction
            if is_time_read(instr) {
                let rd = extract_rd(instr);
                ctx.set(Register::try_from(rd).unwrap(), Arch::read_csr(Csr::Time));
                ctx.pc += 4;
                return ModuleAction::Overwrite;
            }
            ModuleAction::Ignore
        }
        _ => ModuleAction::Ignore,
    }
}
```

### World Switch Hooks

#### `fn switch_from_payload_to_firmware(&mut self, ctx, mctx)`

**When called**: Just before switching from payload to firmware mode

**Purpose**: Save/modify state, configure PMPs for firmware execution

**Important**: This hook **cannot abort** the switch. It's for interposition only.

**Example** (from `protect_payload.rs`):
```rust
fn switch_from_payload_to_firmware(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    // Save payload registers
    for i in 0..32 {
        self.saved_regs[i] = ctx.regs[i];
    }

    // Hide sensitive supervisor CSRs from firmware
    self.clear_supervisor_csr(ctx);

    // Lock payload memory using PMPs
    mctx.pmp.set_inactive(MODULE_OFFSET, TARGET_PAYLOAD_ADDRESS);
    mctx.pmp.set_tor(MODULE_OFFSET + 1, usize::MAX, pmpcfg::NO_PERMISSIONS);
}
```

#### `fn switch_from_firmware_to_payload(&mut self, ctx, mctx)`

**When called**: Just before switching from firmware to payload mode

**Purpose**: Restore state, unlock memory, verify payload integrity

**Example** (from `protect_payload.rs`):
```rust
fn switch_from_firmware_to_payload(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    // Restore payload registers
    for i in 0..32 {
        if !self.forward_to_payload[i] {
            ctx.regs[i] = self.saved_regs[i];
        }
    }

    // Unlock payload memory
    mctx.pmp.set_inactive(MODULE_OFFSET, TARGET_PAYLOAD_ADDRESS);
    mctx.pmp.set_tor(MODULE_OFFSET + 1, usize::MAX, pmpcfg::RWX);

    // Restore supervisor CSRs
    self.restore_supervisor_csr(ctx);
}
```

### Monitoring Hooks

#### `fn decided_next_exec_mode(&mut self, ctx, previous_mode, next_mode)`

**When called**: After Miralis decides the next execution mode, but before world switch

**Purpose**: Collect statistics, log transitions

**Common use cases**:
- Performance monitoring
- Trap cause analysis
- Execution flow tracing

**Example** (from `counter.rs`):
```rust
fn decided_next_exec_mode(
    &mut self,
    ctx: &mut VirtContext,
    previous_mode: ExecutionMode,
    next_mode: ExecutionMode,
) {
    match get_exception_category(ctx, previous_mode, next_mode) {
        Some(ExceptionCategory::FirmwareTrap) => {
            COUNTERS[ctx.hart_id]
                .firmware_traps
                .fetch_add(1, Ordering::Relaxed);
        }
        Some(ExceptionCategory::ReadTime) => {
            COUNTERS[ctx.hart_id]
                .timer_read
                .fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}
```

### Interrupt Handling

#### `fn on_interrupt(&mut self, ctx, mctx)`

**When called**: When a policy MSI (Machine Software Interrupt) is received

**Purpose**: Multi-hart synchronization, cross-hart communication

**Common use cases**:
- Broadcasting memory fence operations
- Synchronizing PMP changes across harts
- Coordinated policy enforcement

**Example** (from `offload.rs`):
```rust
fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    // Check if this is a supervisor software interrupt broadcast
    if POLICY_SSI_ARRAY[mctx.hw.hart]
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        // Inject interrupt into payload
        unsafe { self.set_physical_ssip() };
    }

    // Check for fence.i broadcast
    if FENCE_I_ARRAY[mctx.hw.hart]
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        unsafe { Arch::ifence() };
    }
}
```

**Broadcasting an interrupt**:
```rust
use crate::platform::{Plat, Platform};

// Broadcast to specific harts
let hart_mask = 0b11110; // Harts 1, 2, 3, 4
Plat::broadcast_policy_interrupt(hart_mask);
```

### Shutdown

#### `fn on_shutdown(&mut self)`

**When called**: Before Miralis shuts down

**Purpose**: Cleanup, final statistics reporting

**Example** (from `boot.rs`):
```rust
fn on_shutdown(&mut self) {
    self.display_benchmark(Arch::read_csr(Csr::Mhartid));
}
```

## Registering and Enabling Modules

### Step 1: Register the Module

Edit `src/modules.rs` and add your module to the `build_modules!` macro:

```rust
build_modules! {
    "keystone" => crate::policy::keystone::KeystonePolicy
    "protect_payload" => crate::policy::protect_payload::ProtectPayloadPolicy
    "offload" => crate::policy::offload::OffloadPolicy
    "exit_counter" => crate::benchmark::counter::CounterBenchmark
    "exit_counter_per_cause" => crate::benchmark::counter_per_cause::CounterPerMcauseBenchmark
    "boot_counter" => crate::benchmark::boot::BootBenchmark
    "my_module" => crate::policy::my_module::MyModule  // Add your module here
}
```

The string identifier (e.g., `"my_module"`) will be used in configuration files.

### Step 2: Add Module Declaration

If you created a new file, declare it in the appropriate `mod.rs`:

**For policy modules** (`src/policy/mod.rs`):
```rust
pub mod my_module;
```

**For benchmark modules** (`src/benchmark/mod.rs`):
```rust
pub mod my_module;
```

### Step 3: Enable in Configuration

Create or edit a configuration file in the `config/` directory:

```toml
[modules]
# List of modules to enable (must match build_modules! identifiers)
modules = ["my_module", "exit_counter"]
```

**Example configurations**:

- `config/qemu-keystone.toml`: Enables Keystone enclave support
  ```toml
  [modules]
  modules = ["keystone", "exit_counter"]
  ```

- `config/visionfive2-release-protect-payload.toml`: Payload protection policy
  ```toml
  [modules]
  modules = ["exit_counter", "offload", "protect_payload"]
  ```

### Step 4: Build and Run

Build Miralis with your configuration:

```bash
cargo miralis build --config config/my_config.toml
cargo miralis run --config config/my_config.toml
```

## Best Practices

### 1. State Management

**Per-Hart State**: Module instances are per-hart. Each hart gets its own instance via `init()`.

```rust
pub struct MyModule {
    // This is per-hart state
    local_counter: usize,
}
```

**Shared State**: Use atomics or static arrays indexed by hart ID:

```rust
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::config::PLATFORM_NB_HARTS;

static SHARED_COUNTERS: [AtomicUsize; PLATFORM_NB_HARTS] =
    [const { AtomicUsize::new(0) }; PLATFORM_NB_HARTS];

impl Module for MyModule {
    fn trap_from_payload(&mut self, _mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ModuleAction {
        SHARED_COUNTERS[ctx.hart_id].fetch_add(1, Ordering::Relaxed);
        ModuleAction::Ignore
    }
}
```

### 2. Register Access

**Reading registers**:
```rust
use crate::arch::Register;
use crate::virt::traits::*;

let a7 = ctx.get(Register::X17); // Extension ID
let a6 = ctx.get(Register::X16); // Function ID
let arg0 = ctx.get(Register::X10); // First argument
```

**Writing registers**:
```rust
ctx.set(Register::X10, return_value); // a0 = return value
```

**Advancing PC**: Always increment PC when you handle an instruction:
```rust
ctx.pc += 4; // Skip ecall/handled instruction
```

### 3. PMP Management

**Allocate PMPs in sequence**:
```rust
use crate::arch::pmp::pmplayout::MODULE_OFFSET;
use crate::arch::pmp::pmpcfg;

const NUMBER_PMPS: usize = 2;

// Use PMPs starting from MODULE_OFFSET
let base_pmp = MODULE_OFFSET;

// Configure TOR (Top of Range) addressing
mctx.pmp.set_inactive(base_pmp, region_start);
mctx.pmp.set_tor(base_pmp + 1, region_end, pmpcfg::RWX);

// Flush changes to hardware
unsafe {
    write_pmp(&mctx.pmp).flush();
}
```

**Common PMP permissions**:
- `pmpcfg::RWX` - Read, Write, Execute
- `pmpcfg::RW` - Read, Write
- `pmpcfg::RX` - Read, Execute
- `pmpcfg::NO_PERMISSIONS` - No access

### 4. Trap Cause Inspection

```rust
use crate::arch::MCause;

match ctx.trap_info.get_cause() {
    MCause::EcallFromSMode => { /* Handle ecall */ }
    MCause::IllegalInstr => { /* Emulate instruction */ }
    MCause::LoadPageFault => { /* Handle page fault */ }
    MCause::StorePageFault => { /* Handle page fault */ }
    MCause::LoadAddrMisaligned => { /* Emulate misaligned access */ }
    MCause::StoreAddrMisaligned => { /* Emulate misaligned access */ }
    _ => { /* Other traps */ }
}
```

### 5. Hook Return Values

**When to return `Overwrite`**:
- You completely handled the event
- You modified `ctx.pc` to skip the instruction
- You set return values in registers
- You DON'T want Miralis to process the event

**When to return `Ignore`**:
- You only observed/monitored the event
- You want Miralis to continue normal processing
- The event doesn't match your module's criteria

### 6. Logging

Use the `log` crate macros:

```rust
log::info!("Module initialized: {}", Self::NAME);
log::debug!("Handling ecall: eid={:#x}, fid={:#x}", eid, fid);
log::warn!("Unexpected trap cause: {:?}", cause);
log::error!("Failed to allocate enclave");
```

Configure log levels in config files:
```toml
[log]
level = "info"
debug = ["miralis::policy::my_module"]
```

### 7. Error Handling

Modules typically don't panic. Instead:

```rust
// Return error codes via SBI
ctx.set(Register::X10, sbi_codes::SBI_ERR_DENIED);

// Or emulate a trap to the payload
ctx.emulate_payload_trap();
```

### 8. Multi-Hart Synchronization

**Broadcasting interrupts**:
```rust
use crate::platform::{Plat, Platform, ALL_HARTS_MASK};

// Broadcast to all harts
Plat::broadcast_policy_interrupt(ALL_HARTS_MASK);

// Broadcast to specific harts
let hart_mask = (1 << hart_id_1) | (1 << hart_id_2);
Plat::broadcast_policy_interrupt(hart_mask);
```

**Using atomics for coordination**:
```rust
use core::sync::atomic::{AtomicBool, Ordering};

static FLAGS: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];

// Coordinator hart
FLAGS[target_hart].store(true, Ordering::SeqCst);
Plat::broadcast_policy_interrupt(1 << target_hart);

// Target hart in on_interrupt
fn on_interrupt(&mut self, _ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    if FLAGS[mctx.hw.hart]
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        // Handle the interrupt
    }
}
```

## Examples

### Example 1: Simple Monitoring Module

A module that counts the number of ecalls from the payload:

```rust
// src/benchmark/ecall_monitor.rs
use core::sync::atomic::{AtomicU64, Ordering};
use crate::modules::{Module, ModuleAction};
use crate::host::MiralisContext;
use crate::virt::VirtContext;
use crate::config::PLATFORM_NB_HARTS;

static ECALL_COUNTERS: [AtomicU64; PLATFORM_NB_HARTS] =
    [const { AtomicU64::new(0) }; PLATFORM_NB_HARTS];

pub struct EcallMonitor {}

impl Module for EcallMonitor {
    const NAME: &'static str = "Ecall Monitor";

    fn init() -> Self {
        log::info!("Initializing ecall monitor");
        EcallMonitor {}
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        ECALL_COUNTERS[ctx.hart_id].fetch_add(1, Ordering::Relaxed);
        ModuleAction::Ignore // Let Miralis handle the ecall normally
    }

    fn on_shutdown(&mut self) {
        log::info!("Ecall statistics:");
        for hart in 0..PLATFORM_NB_HARTS {
            let count = ECALL_COUNTERS[hart].load(Ordering::SeqCst);
            log::info!("  Hart {}: {} ecalls", hart, count);
        }
    }
}
```

### Example 2: Custom SBI Extension

A module that implements a custom SBI extension:

```rust
// src/policy/custom_ext.rs
use crate::modules::{Module, ModuleAction};
use crate::host::MiralisContext;
use crate::virt::VirtContext;
use crate::arch::Register;
use crate::virt::traits::*;

const CUSTOM_EID: usize = 0x09000000;
const CUSTOM_FID_HELLO: usize = 0;
const CUSTOM_FID_GOODBYE: usize = 1;

pub struct CustomExtension {}

impl Module for CustomExtension {
    const NAME: &'static str = "Custom SBI Extension";

    fn init() -> Self {
        CustomExtension {}
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        let eid = ctx.get(Register::X17);
        let fid = ctx.get(Register::X16);

        if eid != CUSTOM_EID {
            return ModuleAction::Ignore;
        }

        match fid {
            CUSTOM_FID_HELLO => {
                log::info!("Custom extension: Hello from hart {}", ctx.hart_id);
                ctx.set(Register::X10, 0); // Success
            }
            CUSTOM_FID_GOODBYE => {
                log::info!("Custom extension: Goodbye from hart {}", ctx.hart_id);
                ctx.set(Register::X10, 0); // Success
            }
            _ => {
                log::warn!("Unknown FID: {:#x}", fid);
                ctx.set(Register::X10, usize::MAX); // Error
            }
        }

        ctx.pc += 4; // Skip the ecall instruction
        ModuleAction::Overwrite // We handled it
    }
}
```

### Example 3: Memory Protection Policy

A module that uses PMPs to protect a memory region:

```rust
// src/policy/region_protect.rs
use crate::modules::{Module, ModuleAction};
use crate::host::MiralisContext;
use crate::virt::VirtContext;
use crate::arch::pmp::pmplayout::MODULE_OFFSET;
use crate::arch::pmp::{pmpcfg, write_pmp};

const PROTECTED_REGION_START: usize = 0x90000000;
const PROTECTED_REGION_SIZE: usize = 0x1000000; // 16 MB

pub struct RegionProtect {}

impl Module for RegionProtect {
    const NAME: &'static str = "Region Protection";
    const NUMBER_PMPS: usize = 2; // Need 2 PMPs for TOR addressing

    fn init() -> Self {
        log::info!("Protecting region {:#x}-{:#x}",
            PROTECTED_REGION_START,
            PROTECTED_REGION_START + PROTECTED_REGION_SIZE);
        RegionProtect {}
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        _ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Lock the protected region when switching to firmware
        mctx.pmp.set_inactive(MODULE_OFFSET, PROTECTED_REGION_START);
        mctx.pmp.set_tor(
            MODULE_OFFSET + 1,
            PROTECTED_REGION_START + PROTECTED_REGION_SIZE,
            pmpcfg::NO_PERMISSIONS,
        );
        unsafe {
            write_pmp(&mctx.pmp).flush();
        }
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        _ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Unlock the protected region for payload access
        mctx.pmp.set_inactive(MODULE_OFFSET, PROTECTED_REGION_START);
        mctx.pmp.set_tor(
            MODULE_OFFSET + 1,
            PROTECTED_REGION_START + PROTECTED_REGION_SIZE,
            pmpcfg::RWX,
        );
        unsafe {
            write_pmp(&mctx.pmp).flush();
        }
    }
}
```

### Example 4: Instruction Emulation

A module that emulates a custom instruction:

```rust
// src/policy/custom_instr.rs
use crate::modules::{Module, ModuleAction};
use crate::host::MiralisContext;
use crate::virt::VirtContext;
use crate::arch::{get_raw_faulting_instr, Register, MCause};
use crate::virt::traits::*;

pub struct CustomInstrEmulator {}

impl Module for CustomInstrEmulator {
    const NAME: &'static str = "Custom Instruction Emulator";

    fn init() -> Self {
        CustomInstrEmulator {}
    }

    fn trap_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        if ctx.trap_info.get_cause() != MCause::IllegalInstr {
            return ModuleAction::Ignore;
        }

        let instr = unsafe { get_raw_faulting_instr(ctx) };

        // Check if this is our custom instruction (example opcode)
        if (instr & 0x7f) == 0b0001011 {
            // Extract register fields
            let rd = ((instr >> 7) & 0b11111) as u8;
            let rs1 = ((instr >> 15) & 0b11111) as u8;
            let rs2 = ((instr >> 20) & 0b11111) as u8;

            // Emulate: rd = rs1 XOR rs2 (example)
            let val1 = ctx.get(Register::try_from(rs1).unwrap());
            let val2 = ctx.get(Register::try_from(rs2).unwrap());
            ctx.set(Register::try_from(rd).unwrap(), val1 ^ val2);

            log::debug!("Emulated custom instruction at {:#x}", ctx.pc);

            ctx.pc += 4; // Move to next instruction
            return ModuleAction::Overwrite;
        }

        ModuleAction::Ignore // Not our instruction
    }
}
```

## Testing Your Module

### Unit Testing

Add unit tests to your module file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_init() {
        let module = MyModule::init();
        assert_eq!(module.counter, 0);
    }
}
```

### Integration Testing

1. Create a test configuration in `config/test_my_module.toml`
2. Create a test payload that exercises your module
3. Run with your configuration:

```bash
cargo miralis run --config config/test_my_module.toml
```

### Debugging

Enable debug logging for your module:

```toml
[log]
level = "info"
debug = ["miralis::policy::my_module"]
```

Add strategic log statements:

```rust
log::debug!("Hook called: pc={:#x}, cause={:?}", ctx.pc, ctx.trap_info.get_cause());
```

## Common Pitfalls

1. **Forgetting to increment PC**: Always advance `ctx.pc` when handling an instruction that should be skipped.

2. **PMP conflicts**: Ensure your PMP ranges don't conflict with other modules. Check `NUMBER_PMPS` of all enabled modules.

3. **Not flushing PMPs**: Always call `write_pmp(&mctx.pmp).flush()` after PMP modifications.

4. **Returning wrong ModuleAction**: Return `Overwrite` only when you've fully handled the event.

5. **Race conditions**: Use proper atomic operations when sharing state between harts.

6. **Forgetting to register**: Add your module to `build_modules!` and enable it in config.

7. **Module ordering**: Modules are called in the order listed in `build_modules!`. First module to return `Overwrite` wins.

## Further Reading

- **Architecture documentation**: `docs/architecture.md`
- **Miralis overview**: `docs/overview.md`
- **Existing modules**: Study `src/policy/` and `src/benchmark/` for real-world examples
- **VirtContext API**: `src/virt/mod.rs`
- **PMP management**: `src/arch/pmp/`
- **SBI codes**: `miralis-core/src/sbi_codes.rs`

## Contributing

When contributing a new module:

1. Follow the code style of existing modules
2. Add comprehensive documentation comments
3. Include usage examples in comments
4. Add log statements for key operations
5. Test on both QEMU and hardware if possible
6. Update this documentation if needed

For questions or issues, refer to the [contributing guide](contributing.md).
