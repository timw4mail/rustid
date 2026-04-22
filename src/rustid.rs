#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

use rustid::common::TCpu;
use rustid::{Cpu, version};

#[cfg(target_arch = "x86")]
use rustid::cyrix_cpuid_check;

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    use rustid::cpuid::dos::exit;

    cyrix_cpuid_check();

    let cpu = Cpu::detect();
    version();
    cpu.display_table();

    exit();
}

#[cfg(not(target_os = "none"))]
fn help() {
    println!("Usage: rustid [FLAGS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (no args)        Display CPU information");
    println!("  d, debug         Display detailed debug information");
    println!("  e, everything    Show CPU information and debug information");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    println!("  r, dump          Dump raw CPUID values");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    println!("  f, file <file>   Load CPUID dump from file and display CPU information");
    println!("  v, version       Display version info");
    println!("  h, help          Show this help message");
    println!();
    println!("Flags:");
    println!("  m, mono          Don't output color");
    println!();
    println!("All commands accept optional leading dashes. Flags can be combined, e.g. -me");
}

#[cfg(not(target_os = "none"))]
fn main() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    use rustid::cpuid;

    #[cfg(target_arch = "x86")]
    cyrix_cpuid_check();

    let mut color = true;
    let mut action = "default";
    let mut file_path = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let stripped = arg
            .strip_prefix("--")
            .unwrap_or_else(|| arg.strip_prefix('-').unwrap_or(&arg));

        match stripped {
            "m" | "mono" => color = false,
            "e" | "everything" => action = "everything",
            "r" | "dump" => action = "dump",
            "f" | "file" => {
                file_path = args.next();
                if file_path.is_none() {
                    eprintln!("Error: 'file' option requires a path");
                    help();
                    return;
                }
            }
            "v" | "version" => action = "version",
            "h" | "help" => action = "help",
            "d" | "debug" => action = "debug",
            _ if arg.starts_with('-') && !arg.starts_with("--") => {
                for c in arg.chars().skip(1) {
                    match c {
                        'm' => color = false,
                        'e' => action = "everything",
                        'r' => action = "dump",
                        'f' => {
                            file_path = args.next();
                            if file_path.is_none() {
                                eprintln!("Error: '-f' flag requires a path");
                                help();
                                return;
                            }
                        }
                        'v' => action = "version",
                        'h' => action = "help",
                        _ => {
                            eprintln!("Unknown flag: -{}", c);
                            help();
                            return;
                        }
                    }
                }
            }
            _ => {
                eprintln!("Unknown command: {}", arg);
                help();
                return;
            }
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if let Some(path) = &file_path {
        use cpuid::provider::{self, CpuDump};

        rustid::file_version();
        let dump = CpuDump::parse_file(path);
        provider::set_cpuid_provider(dump);
    } else {
        match action {
            "debug" | "everything" | "default" | "version" => version(),
            _ => {}
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    match action {
        "debug" | "everything" | "default" | "version" => version(),
        _ => {}
    }

    match action {
        "debug" => {
            Cpu::detect().debug();
        }
        "everything" => {
            let cpu = Cpu::detect();
            cpu.display_table(color);
            println!("---");
            cpu.debug();
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        "dump" => {
            use rustid::cpuid::{Str, dump::dump_cpu, topology::Topology};

            let mut output: Str<16384> = Str::new();
            let topo = Topology::detect();

            let logical_cores = topo.threads as usize;
            for i in 0..logical_cores {
                dump_cpu(&mut output, i);
            }

            print!("{}", output);
        }
        "help" => help(),
        "version" => {}
        "default" => {
            Cpu::detect().display_table(color);
        }
        _ => unreachable!(),
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if file_path.is_some() {
        use rustid::cpuid::provider;
        provider::reset_cpuid_provider();
    }
}
