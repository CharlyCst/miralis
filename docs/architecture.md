# Architecture

## Overview

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

## Interrupts

### RISCV interrupt system

When the firmware is running, we want to receive all interrupts the firmware wants to receive. Theses enabled interrupts can be known by having a look at some virtuals context registers: 

1. `mie` register informs us on the individually enabled interrupts. `mip` register holds the pending interrupts by setting the corresponding bit when an interrupt occurs. A pending interrupt can trap only if the corresponding bit in `mie` and in `mip` is set. Here is the layout of both registers. Please refer to the [specification](https://drive.google.com/file/d/17GeetSnT5wW3xNuAHI95-SI1gPGd5sJ_/view?usp=drive_link) for the detailed explanation of each field.

```
      15 14  13  12  11 10   9  8   7  6   5  4   3  2   1  0
      ┌───┬──────┬─┬────┬─┬────┬─┬────┬─┬────┬─┬────┬─┬────┬─┐          
mie : │0 0│LCOFIE│0│MEIE│0│SEIE│0│MTIE|0|STIE|0|MSIE|0|SSIE|0|       
      └───┴──────┴─┴────┴─┴────┴─┴────┴─┴────┴─┴────┴─┴────┴─┘   

      15 14  13  12  11 10   9  8   7  6   5  4   3  2   1  0
      ┌───┬──────┬─┬────┬─┬────┬─┬────┬─┬────┬─┬────┬─┬────┬─┐          
mip : │0 0│LCOFIP│0│MEIP│0│SEIP│0│MTIP|0|STIP|0|MSIP|0|SSIP|0|       
      └───┴──────┴─┴────┴─┴────┴─┴────┴─┴────┴─┴────┴─┴────┴─┘         
```

2. `mideleg` register allows delegating interrupts to less-privileged mode. The layout of `mideleg` matches the one of `mie` and `mip`. If an interrupt is pending and delegated, it will not trap, whatever the value in `mie`. 

3. The MIE bit in the `mstatus` register control if interrupts are globally enabled for machine mode. If `mstatus.MIE` is disabled and an interrupt is pending, it will not trap, whatever the values in `mie` and `mideleg`. If the running mode is less than M, global interrupts for M-mode are always enabled. 

To sum up, in riscv, in order to trap to M-Mode, we need:
> **(RISCV-SPEC)**  
> Executing mode = **M**:  
> `trap ⟺ mip[i] ∧ mie[i] ∧ mstatus.MIE ∧ ¬mideleg[i]`
> 
> Executing mode = **S**:  
> `trap ⟺ mip[i] ∧ mie[i] ∧ ¬mideleg[i]`

### Miralis interrupt virtualization

In order to properly virtualize interrupts and correctly handle them, we need to follow many rules. The goal is to ensure that all interrupts destined to the firmware are correctly virtualized, so that the firmware get exactly its destined interrupts.

Virtualizing interrupts requires virtualizing the interrupt source.
Miralis is designed to be minimal, and therefore explicitely avoid virtualizing devices such as disks and network cards, which are usually managed by the OS rather than the firmware.
The current implementation of Miralis focuses on virtualizing the M-mode interrupts only: M-mode timer, software, and external interrupts.
For that purpose Miralis forces the delegation of all other interrupts to the payload.

We virtualize registers inside the virtual context of the firmware. Registers `mie`, `mip`, `mideleg`, `mstatus` will have their virtual counterparts `vmie`, `vmie`, `vmideleg` and `vmstatus`. In this section, we will also say that the firmware is running in **vM-mode** (M-mode virtualized by Miralis inside U-mode). Let's now separate the cases into the three execution states that could occur:

#### Firmware

When the firmware is running, we want Miralis to receive **all** interrupts the firmware expects to receive: no interrupt is delegated to firmware. We then set `mideleg` to 0 when executing the firmware. We also want to receive **only** interrupts the firmware expects to receive. We then must filter `mie` register to not trap on delegated interrupts or disabled interrupts. If an interrupt occurs, we need to reflect `mip` into `vmip` to let the firmware know which one.  

> **(MIDELEG-VM-MODE)**  
> mideleg ≡ 0, if mode = vM
>
> **(MIE-VM-MODE)**  
> mie = ¬vmideleg ∧ vmstatus.MIE ∧ vmie, if mode = vM
>
> **(MIP-VM-MODE)** 
> vmip = mip, if mode = vM

#### Payload

When switching to S-mode, we want to install `vmideleg` into `mideleg` and `vmie` to `mie`, because the states of `mideleg` and `mie` may influence S-mode interrupts handling. 

> **(MIDELEG-S-MODE)**  
> mideleg ≡ vmideleg, if mode = S  
> 
> **(MIE-S-MODE)**  
> mie ≡ vmie, if mode = S

#### Miralis

When Miralis is running, we don't want to receive interrupts: we want to handle them one by one. A simple way to ensure that is to make sure that `mstatus.MIE` is always 0. As interrupts are globally enabled for M-mode when a less-privileged mode is running, Miralis will still get the interrupts of S-mode (e.g. payload) and U-mode (e.g. firmware). 

> **(MSTATUS-MIE)**  
> mstatus.MIE ≡ 0


Now we show that if Miralis get an interrupt from firmware or the payload, it's correctly forwarded to the firmware interrupt handler and the virtual context is properly set to a trap state for the firmware.

When running in **vM-mode**, we have the following properties:

```
             ┌──────────┐
         ┌──>│ Firmware │───┐       (1) An interrupt occurs when the firmware is
         │   └──────────┘   |           running. Switch to Miralis.
      (2)│                  |(1)    (2) Miralis virtualizes interrupt and
         │                  |           transmit handling to firmware's interrupt
         │   ┌──────────┐   |           handler.
         └───│ Miralis  │<──┘                  
             └──────────┘

Miralis receives interrupt i when executing firmware:
  (RISCV-SPEC)
  ⟹ mstatus.MIE = -, mie[i] = 1, mip[i] = 1, mideleg[i] = 0

  (MIE-VM-MODE)
  ⟹ vmstatus.MIE = 1, vmie[i] = 1, vmideleg[i] = 0

  (MIP-VM-MODE)
  ⟹ vmip[i] = mip [i] = 1

  Then, vmstatus.MIE = 1 ∧ vmip[i] = 1 ∧ vmie[i] = 1 ∧ vmideleg[i] = 0
  (RISCV-SPEC)
  ⟹ virtual context is set as a tap occured in the firmware,
     we can forward interrupt handling to firmware.
```

When running in **S-mode**, we have the following properties:

```
  ┌─────────┐           ┌─────────┐ (1) An interrupt occurs when the payload is
  │ Payload │<──┐   ┌──>│Firmware │     running. Switch to Miralis if not delegated.
  └─────────┘   │   │   └─────────┘ (2) Miralis virtualizes interrupt and transmit
        │    (4)│   │(2)    │           handling to firmware's interrupt handler. 
     (1)│       │   │       │(3)    (3) Firmware's handler handle interrupt then
        │    ┌─────────┐    │           mret to payload. 
        └───>│ Miralis │<───┘       (4) Miralis installs registers and emulate
             └─────────┘                mret to payload.

Miralis receives interrupt i when executing payload:
  (RISCV-SPEC)
  ⟹ mstatus.MIE = -, mie[i] = 1, mip[i] = 1, mideleg[i] = 0

  (MIE-S-MODE, MIDELEG-S-MODE)
  ⟹ vmie[i] = 1, vmideleg[i] = 0

  (MIP-VM-MODE)
  ⟹ vmip[i] = mip [i] = 1

  Then, vmstatus.MIE = - ∧ vmip[i] = 1 ∧ vmie[i] = 1 ∧ vmideleg[i] = 0
  (RISCV-SPEC)
  ⟹ virtual context is set as a tap occured to the firmware,
     we can forward interrupt handling to firmware.
```

### Software external interrupt virtualization

Supervisor-level external pending interrupts are a particular case. The specification of RISC-V says that the `csrr` instruction is modified when reading the **SEIP** (supervisor-level external pending interrupts) bit of the `mip` register.

From the RISC-V Instruction Set Manual: Volume II:

> Supervisor-level external interrupts are made pending based on the logical-OR of the software-writable SEIP bit and the signal from the external interrupt controller. When mip is read with a CSR instruction, the value of the SEIP bit returned in the rd destination register is the logical-OR of the software-writable bit and the interrupt signal from the interrupt controller, but the signal from the interrupt controller is not used to calculate the value written to SEIP.

The value of **SEIP** read by `csrr` is then not exactly what is inside the `mip` register. It is a logical-OR of *the software-writable bit and the interrupt signal from the interrupt controller*. This means that on world-switch between the payload and the firmware, we should not put the value of **SEIP** using only `csrr` as it doesm't represent the real value of the physical register.

**SEIP** is read-only by S-Mode (and U-Mode). Then, M-Mode is the only one that can change the value of SEIP. So we have to keep it as it is in the virtual `mip` register when world-switching.

Another thing to consider is the following scenario:

```
  Int. sig. mip.SEIP vmip.SEIP
  ┌──────┐ ┌──────┐ ┌──────┐
  │   0  │ │   1  │ │  *0  │ *Cleared by firmware
  └──────┘ └──────┘ └──────┘   but not installed.
      │        │  ∧     ╎
 csrr └──[OR]──┘  └╶╶X╶╶┘
           │
          {1} Wrong value!
```

If the firmware wants to read the `mip` register after cleaning `vmip.SEIP`, and we don't sync `vmip.SEIP` with `mip.SEIP`, it can't know if there is an interrupt signal from the interrupt controller as the CSR read will be a logical-OR of the signal and `mip.SEIP` (which is one), and so always 1. If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
Then, we need to synchronize vmip.SEIP with mip.SEIP.

### Mutliharts interrupt

To wake up harts, firmware might use a machine software interrupt (`MSI`) or a machine external interrupt (`MEI`). These interrupts need to be fetched from hardware (`mip`) for each virtual read of `vmip`, as they can occur asynchronously with the execution of the firmware. During a world switch, we need to take care that these interrupts are not installed in the virtual `vmip`, to avoid having an interrupt that can't be cleared by firmware in the virtual context.

## Handling Virtual Memory Accesses from Firmware

The Modify Privilege (MPRV) feature in RISC-V architecture provides fine-grained control over memory privilege levels. When firmware sets its virtual MPRV bit (`vMPRV`) to 1 (typically through an SBI call from the OS), we must navigate this transition with care to maintain correctness and security.

> The MPRV (Modify PRiVilege) bit modifies the effective privilege mode, i.e., the privilege level at which loads and stores execute. When MPRV=0, loads and stores behave as normal, using the translation and protection mechanisms of the current privilege mode. When MPRV=1, load and store memory addresses are translated and protected, and endianness is applied, as though the current privilege mode were set to MPP.

Specifically, when `vMPRV` is set to 1, we want memory accesses to follow the virtual address translation rules without directly affecting the physical privilege state. If any exception occurs while `pMPRV` bit is set to 1, the general trap handler will attempt to resolve physical addresses using page tables, leading to execution errors.

### Overview

1. **Virtual MPRV (`vMPRV`) Activation**:
   - When firmware sets `vMPRV` to 1, we don't immediately mirror this change in the physical MPRV (`pMPRV`).
   - Instead, we activate the first PMP entry, effectively denying all read and write accesses.
   - This approach allows us to catch all accesses that should occur with `MPRV=1` without actually altering the `pMPRV`.
   - If an access occurs while `vMPRV` is set to 1, it is treated as a virtual access, invoking a special handler.
   - When firmware sets `vMPRV` to 0, we deactivate the first entry. This way physical address space accesses won't trap to Miralis.

2. **Handling in the virtual access**:
   - Configure page tables to use the virtual ones (`pSATP` = `vSATP`).
   - Employ a new trap handler that is simpler, yet not dependent on access privilege mode.
   - Temporarily set `pMPRV=1`.
   - Emulate exactly one instruction with virtual addresses.
   - Restore all original settings.

3. **Exception Handling**:
   - If an exception occurs during the emulation of virtual access, we want firmware to attribute it to the original instruction.
   - Therefore, we return the original exception address (`mepc`) while updating other trap-related information.
