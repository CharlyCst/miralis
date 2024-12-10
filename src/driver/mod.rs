//! # Drivers
//!
//! This module regroups various drivers used by Miralis. While Miralis doesn't virtualize devices
//! such as disks and network card, it does virtualize some of the devices required to multiplex
//! interrupts, such as the CLINT and PLIC.

pub mod clint;
pub mod plic;
