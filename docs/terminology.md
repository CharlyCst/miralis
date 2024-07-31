# Terminology

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

