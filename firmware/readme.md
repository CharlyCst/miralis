# Firmware

This folder contains different firmware for Mirage.
On Mirage, a firmware is an M-mode program that is deprivileged by Mirage through trap-and-emulate.

When running Mirage using `just run` the `default` firmware is used.
Other firmware are used by integration tests when running `just test`.
