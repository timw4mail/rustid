#![no_std]

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_os = "none"))]
extern crate std;

use crate::cpuid::{Cpu, init};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cpuid;

#[cfg(target_os = "none")]
pub mod dos;

#[cfg(target_os = "none")]
pub use dos::*;

#[cfg(not(target_os = "none"))]
use std::println;

fn version() {
    println!("---------------------");
    println!("Rustid version {}", VERSION);
    println!("---------------------");
}

pub fn cli_main() {
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
                "debug" => cpu.debug(),
                "help" => cli_help(),
                "everything" => {
                    version();
                    cli_help();
                    cpu.display_table();
                    println!("---");
                    cpu.debug();
                }
                _ => cpu.display_table(),
            },
            None => {
                cpu.display_table();
            }
        }
    }
}
