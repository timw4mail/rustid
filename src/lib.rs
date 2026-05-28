//! Rustid - A cross-platform CPU identification library.
//!
//! This crate provides a unified interface for detecting CPU information
//! across different architectures including x86/x86_64, ARM/AArch64, and PowerPC.
//!
//! # Supported Architectures
//!
//! - **x86/x86_64**: Uses the CPUID instruction to detect CPU vendor, model,
//!   microarchitecture, features, and other hardware details.
//! - **ARM/AArch64**: Reads the Main ID Register (MIDR) to identify the CPU.
//! - **PowerPC**: Reads the Processor Version Register (PVR) for identification.
//!
//! # Usage
//!
//! ```
//! use rustid::Cpu;
//! use rustid::common::{CliFlags, TCpu};
//!
//! let cpu = Cpu::detect();
//! let flags = CliFlags::default();
//! cpu.display_table(flags);
//! # assert_ne!(cpu, Cpu::default());
//! ```
#![cfg_attr(all(not(test), dos), no_std)]

extern crate alloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(dos))]
const ARCH: &str = std::env::consts::ARCH;
#[cfg(dos)]
const ARCH: &str = "x86";

#[cfg(not(dos))]
const OS: &str = std::env::consts::OS;
#[cfg(dos)]
const OS: &str = "DOS";

#[cfg(not(dos))]
extern crate std;

pub mod common;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cpuid;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use cpuid::Cpu;

#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
pub mod ppc;
#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
pub use ppc::cpu::Cpu;

#[cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
pub mod arm;
#[cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
pub use arm::Cpu;

#[cfg(dos)]
pub use cpuid::dos::*;

#[cfg(not(dos))]
pub use std::{print, println};

pub fn version() {
    println!(
        "--------------- Rustid {} ({}-{}) ---------------",
        VERSION, ARCH, OS
    );
}

#[cfg(not(dos))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn file_version() {
    println!("--------------- Rustid {VERSION} ({ARCH}-{OS}:from-cpuid-dump) ---------------");
}

#[cfg(target_arch = "x86")]
pub fn cyrix_cpuid_check() {
    use crate::println;

    if cpuid::vendor::Cyrix::can_enable_cpuid() {
        println!("This CPU has CPUID support, but it is disabled by default.");
        println!("Some BIOSes have an option to enable CPUID for Cyrix chips.");
        println!("For DOS, you can download a utility from ");
        println!("  https://www.deinmeister.de/e_cy6x86cr.htm");
        println!("If run before rustid, CPUID should be enabled");
    }
}
