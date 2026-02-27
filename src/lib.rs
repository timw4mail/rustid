#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(target_arch = "powerpc", feature(asm_experimental_arch))]
#![cfg_attr(target_arch = "powerpc", feature(stdarch_powerpc_feature_detection))]
#![cfg_attr(
    any(target_arch = "arm", target_arch = "aarch64"),
    feature(stdarch_arm_feature_detection)
)]

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_os = "none"))]
extern crate std;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cpuid;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::cpuid::{Cpu, init};

#[cfg(target_arch = "powerpc")]
pub mod ppc;
#[cfg(target_arch = "powerpc")]
use crate::ppc::cpu::Cpu;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod arm;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use crate::arm::cpu::Cpu;

#[cfg(target_os = "none")]
pub mod dos;

#[cfg(target_os = "none")]
pub use dos::*;

#[cfg(not(target_os = "none"))]
pub use std::println;

fn version() {
    println!("---------------------");
    println!("Rustid version {}", VERSION);
    println!("---------------------");
}

pub fn cli_main() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    init();

    let cpu = Cpu::new();

    #[cfg(target_os = "none")]
    {
        version();
        cpu.display_table();
    }

    #[cfg(not(target_os = "none"))]
    {
        fn cli_help() {
            println!("Usage: rustid [debug] [help]");
        }

        use std::env;

        let argument = env::args().nth(1);

        match argument {
            Some(arg) => match arg.as_str() {
                "version" => version(),
                "debug" => {
                    version();
                    cpu.debug();
                }
                "help" => cli_help(),
                "everything" => {
                    version();
                    cli_help();
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
