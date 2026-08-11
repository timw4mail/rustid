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
#![cfg_attr(all(not(test), dos), no_std)]
#![cfg_attr(all(not(test), dos32a), no_std)]

extern crate alloc;

#[cfg(not(dos))]
const APP: &str = "Rustid";

#[cfg(dos)]
const APP: &str = "Rust86";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(any(dos, dos32a)))]
const ARCH: &str = std::env::consts::ARCH;
#[cfg(any(dos, dos32a))]
const ARCH: &str = "x86";

#[cfg(not(any(dos, dos32a)))]
const OS: &str = std::env::consts::OS;
#[cfg(any(dos, dos32a))]
const OS: &str = "DOS";

#[cfg(not(any(dos, dos32a)))]
extern crate std;

pub mod common;

#[cfg(x86_cpu)]
pub mod x86;

#[cfg(x86_cpu)]
pub use x86::Cpu;

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(ppc_cpu)]
pub mod ppc;
#[cfg(ppc_cpu)]
pub use ppc::cpu::Cpu;

#[cfg(arm_cpu)]
pub mod arm;
#[cfg(arm_cpu)]
pub use arm::Cpu;

#[cfg(target_arch = "riscv64")]
pub mod riscv;
#[cfg(target_arch = "riscv64")]
pub use riscv::Cpu;

#[cfg(any(dos, dos32a))]
pub use x86::dos::*;

#[cfg(not(any(dos, dos32a)))]
pub use std::{print, println};

pub fn version() {
    println!(
        "--------------- {} {} ({}-{}) ---------------",
        APP, VERSION, ARCH, OS
    );
}

#[cfg(not(any(dos, dos32a)))]
#[cfg(x86_cpu)]
pub fn file_version() {
    println!("--------------- Rustid {VERSION} ({ARCH}-{OS}:from-cpuid-dump) ---------------");
}

#[cfg(any(target_arch = "x86", dos, dos32a))]
pub fn cyrix_cpuid_check() {
    #[cfg(not(any(dos, dos32a)))]
    use crate::println;

    #[cfg(any(dos, dos32a))]
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
