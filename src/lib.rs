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

#[cfg(not(dos))]
const APP: &str = "Rustid";

#[cfg(dos)]
const APP: &str = "Rust86";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(nostd_os))]
const ARCH: &str = std::env::consts::ARCH;
#[cfg(any(dos, dos32a))]
const ARCH: &str = "x86";
#[cfg(target_os = "uefi")]
const ARCH: &str = if cfg!(target_arch = "x86_64") {
    "x86_64"
} else {
    "x86"
};

#[cfg(not(nostd_os))]
const OS: &str = std::env::consts::OS;
#[cfg(any(dos, dos32a))]
const OS: &str = "DOS";
#[cfg(target_os = "uefi")]
const OS: &str = "UEFI";

#[cfg(not(nostd_os))]
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

#[cfg(any(target_arch = "riscv64", test))]
pub mod riscv;
#[cfg(target_arch = "riscv64")]
pub use riscv::Cpu;

#[cfg(any(dos, dos32a))]
pub use x86::dos::*;

#[cfg(target_os = "uefi")]
pub use x86::efi::*;

#[cfg(not(nostd_os))]
pub use std::{print, println};

pub fn version() {
    println!(
        "--------------- {} {} ({}-{}) ---------------",
        APP, VERSION, ARCH, OS
    );
}

#[cfg(not(nostd_os))]
#[cfg(x86_cpu)]
pub fn file_version() {
    println!("--------------- Rustid {VERSION} ({ARCH}-{OS}:from-cpuid-dump) ---------------");
}

#[cfg(any(target_arch = "x86", dos, dos32a))]
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
