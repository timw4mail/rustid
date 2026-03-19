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
//! use rustid::TCpu;
//!
//! let cpu = Cpu::detect();
//! cpu.display_table();
//! # assert_ne!(cpu, Cpu::default());
//! ```
//!
//! # CLI Usage
//!
//! When compiled as a standalone binary (non-dos build):
//! - `rustid` - Display basic CPU information
//! - `rustid debug` - Display detailed debug information
//! - `rustid version` - Display version info
//! - `rustid help` - Show help message
//! - `rustid everything` - Show all information
#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(target_arch = "powerpc", feature(asm_experimental_arch))]

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_os = "none"))]
extern crate std;

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
pub use arm::cpu::Cpu;

#[cfg(target_os = "none")]
pub mod dos;

#[cfg(target_os = "none")]
pub use dos::*;

#[cfg(not(target_os = "none"))]
pub use std::println;

pub trait TCpu {
    /// Display the Rust debug output of the CPU object
    fn debug(&self);

    /// Display the CPU information in a table format
    fn display_table(&self);
}

fn version() {
    println!("---------------------");
    println!("Rustid version {}", VERSION);
    println!("---------------------");
}

pub fn cli_main() {
    #[cfg(target_arch = "x86")]
    {
        use crate::println;

        if cpuid::vendor::Cyrix::can_enable_cpuid() {
            println!("This CPU has CPUID support, but it is disabled.");
            println!("Some BIOSes have an option to enable CPUID for Cyrix chips.");
            println!("For DOS, you can download a utility from ");
            println!("  https://www.deinmeister.de/e_cy6x86cr.htm");
            println!("If run before rustid, CPUID should be enabled");
        }
    }

    let cpu = Cpu::detect();

    #[cfg(target_os = "none")]
    {
        version();
        cpu.display_table();
    }

    #[cfg(not(target_os = "none"))]
    {
        match std::env::args().nth(1) {
            Some(arg) => match arg.as_str() {
                "debug" => {
                    version();
                    cpu.debug();
                }
                "e" | "everything" => {
                    version();
                    cpu.display_table();
                    println!("---");
                    cpu.debug();
                }
                _ => {
                    version();
                    cpu.display_table()
                }
            },
            None => {
                version();
                cpu.display_table();
            }
        }
    }
}
