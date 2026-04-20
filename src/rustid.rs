#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

use rustid::common::TCpu;
use rustid::{Cpu, version};

#[cfg(target_arch = "x86")]
use rustid::cyrix_cpuid_check;

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

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    use rustid::cpuid::dos::exit;

    cyrix_cpuid_check();

    #[cfg(feature = "debug")]
    {
        use rustid::cpuid::quirks::debug_quirks;
        debug_quirks();
    }

    let cpu = Cpu::detect();
    version();
    cpu.display_table();

    exit();
}

#[cfg(not(target_os = "none"))]
fn main() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    use rustid::cpuid;

    #[cfg(target_arch = "x86")]
    cyrix_cpuid_check();

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
            "d" | "dump" => {
                use rustid::cpuid::{Str, dump::dump_cpu, topology::Topology};

                let mut output: Str<16384> = Str::new();
                let topo = Topology::detect();

                let logical_cores = topo.threads as usize;
                for i in 0..logical_cores {
                    dump_cpu(&mut output, i);
                }

                print!("{}", output);
            }
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            "f" | "file" => {
                use cpuid::provider::{self, CpuDump};

                rustid::file_version();
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
