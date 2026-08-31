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
//! use rustid::common::{CliFlags, TCpuDisplay, TDetect};
//!
//! let cpu = Cpu::detect();
//! let flags = CliFlags::default();
//! cpu.display_table(flags);
//! # assert_ne!(cpu, Cpu::default());
//! ```
#![cfg_attr(all(not(test), nostd_os), no_std)]

extern crate alloc;

pub use alloc::format;

#[cfg(not(dos_real))]
const APP: &str = "Rustid";

#[cfg(dos_real)]
const APP: &str = "Rust86";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(std_os)]
const ARCH: &str = std::env::consts::ARCH;
#[cfg(dos_os)]
const ARCH: &str = "x86";
#[cfg(uefi)]
const ARCH: &str = if cfg!(target_arch = "x86_64") {
    "x86_64"
} else {
    "x86"
};

#[cfg(std_os)]
const OS: &str = std::env::consts::OS;
#[cfg(dos_os)]
const OS: &str = "DOS";
#[cfg(uefi)]
const OS: &str = "UEFI";

#[cfg(std_os)]
extern crate std;

pub mod common;

#[cfg(x86_cpu)]
pub mod x86;
#[cfg(x86_cpu)]
pub use x86::Cpu;

#[cfg(any(ppc_cpu, test))]
pub mod ppc;
#[cfg(ppc_cpu)]
pub use ppc::cpu::Cpu;

#[cfg(any(arm_cpu, test))]
pub mod arm;
#[cfg(arm_cpu)]
pub use arm::Cpu;

#[cfg(any(riscv_cpu, test))]
pub mod riscv;
#[cfg(riscv_cpu)]
pub use riscv::Cpu;

#[cfg(dos_os)]
pub use x86::dos::*;

#[cfg(uefi)]
pub use x86::efi::*;

#[cfg(std_os)]
pub use std::{print, println};

pub fn format_version() -> alloc::string::String {
    alloc::format!(
        "--------------- {} {} ({}-{}) ---------------",
        APP,
        VERSION,
        ARCH,
        OS
    )
}

pub fn version() {
    println!("{}", format_version());
}

pub fn format_file_version() -> alloc::string::String {
    alloc::format!(
        "--------------- {} {} ({}-{}:from-cpuid-dump) ---------------",
        APP,
        VERSION,
        ARCH,
        OS
    )
}

#[cfg(std_os)]
#[cfg(x86_cpu)]
pub fn file_version() {
    println!("{}", format_file_version());
}

#[cfg(any(x86_cpu, dos_os))]
pub fn cyrix_cpuid_check() {
    use crate::println;

    #[cfg(x86_cpu)]
    if x86::vendor::Cyrix::can_enable_cpuid() {
        println!("This CPU has CPUID support, but it is disabled by default.");
        println!("Some BIOSes have an option to enable CPUID for Cyrix chips.");
        println!("For DOS, you can download a utility from ");
        println!("  https://www.deinmeister.de/e_cy6x86cr.htm");
        println!("If run before rustid, CPUID should be enabled");
    }
}
