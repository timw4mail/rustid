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
//! use rustid::common::TCpu;
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
const ARCH: &str = std::env::consts::ARCH;
#[cfg(target_os = "none")]
const ARCH: &str = "x86";

#[cfg(not(target_os = "none"))]
const OS: &str = std::env::consts::OS;
#[cfg(target_os = "none")]
const OS: &str = "DOS";

#[cfg(not(target_os = "none"))]
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

#[cfg(target_os = "none")]
pub use cpuid::dos::*;

#[cfg(not(target_os = "none"))]
pub use std::println;

fn version() {
    println!(
        "--------------- Rustid {} ({}-{}) ---------------",
        VERSION, ARCH, OS
    );
}

#[cfg(not(target_os = "none"))]
fn help() {
    println!("Usage: rustid [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (no args)     Display CPU information");
    println!("  debug         Display detailed debug information");
    println!("  v, version       Display version info");
    println!("  h, help          Show this help message");
    println!("  e, everything    Show CPU information and debug information");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    println!("  d, dump          Dump raw CPUID values");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    println!("  f, file          Load CPUID dump from file and display CPU information");
    println!();
    println!("All commands accept optional leading dashes.");
}

#[cfg(not(target_os = "none"))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn file_version() {
    println!(
        "--------------- Rustid {} ({}-{}:from-cpuid-dump) ---------------",
        VERSION, ARCH, OS
    );
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

#[cfg(feature = "debug")]
pub fn debug_main() {
    use crate::common::TCpu;

    version();

    #[cfg(target_arch = "x86")]
    {
        use crate::cpuid::quirks::debug_quirks;

        cyrix_cpuid_check();
        debug_quirks();
        println!("---");
    }

    Cpu::detect().debug();
}

pub fn cli_main() {
    use crate::common::TCpu;

    #[cfg(target_arch = "x86")]
    cyrix_cpuid_check();

    #[cfg(target_os = "none")]
    {
        let cpu = Cpu::detect();
        version();
        cpu.display_table();
    }

    #[cfg(not(target_os = "none"))]
    {
        let cmd = std::env::args().nth(1);
        let cmd_stripped = cmd.as_ref().map(|a| {
            // One dash
            let s = a.strip_prefix('-').unwrap_or(a);
            // Two dashes
            s.strip_prefix("-").unwrap_or(s)
        });
        match cmd_stripped {
            Some(cmd) => match cmd {
                "debug" => {
                    version();
                    Cpu::detect().debug();
                }
                "e" | "everything" => {
                    version();
                    let cpu = Cpu::detect();
                    cpu.display_table();
                    println!("---");
                    cpu.debug();
                }
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                "d" | "dump" => cpuid::dump::dump_main(),
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                "f" | "file" => {
                    use cpuid::provider::{self, CpuDump};

                    file_version();
                    let path = std::env::args().nth(2);
                    if let Some(path) = path {
                        let dump = CpuDump::parse_file(&path);
                        provider::set_cpuid_provider(dump);
                        let cpu = Cpu::detect();
                        cpu.display_table();
                        provider::reset_cpuid_provider();
                    } else {
                        println!("Usage: rustid file <dump_file>");
                    }
                }
                "h" | "help" => help(),
                "v" | "version" => version(),
                _ => {
                    eprintln!("Unknown command: {}", std::env::args().nth(1).unwrap());
                    help();
                }
            },
            None => {
                version();
                Cpu::detect().display_table();
            }
        }
    }
}
