#![cfg(not(dos))]

use rustid::common::TCpu;
use rustid::{Cpu, version};

#[cfg(target_arch = "x86")]
use rustid::cyrix_cpuid_check;

#[cfg(not(dos))]
fn help() {
    println!("Usage: rustid [FLAGS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (no args)        Display CPU information");
    println!("  d, debug         Display detailed debug information");
    println!("  e, everything    Show CPU information and debug information");
    #[cfg(x86_cpu)]
    println!("  r, dump          Dump raw CPUID values");
    #[cfg(x86_cpu)]
    println!("  f, file <file>   Load CPUID dump from file and display CPU information");
    println!("  V, version       Display version info");
    println!("  h, help          Show this help message");
    println!();
    println!("Flags:");
    println!("  m, mono          Don't output color");
    println!("  v, verbose       Output more detailed information");
    println!();
    println!("All commands accept optional leading dashes. Flags can be combined, e.g. -me");
}

#[cfg(not(dos))]
fn main() {
    use rustid::common::CliFlags;
    #[cfg(x86_cpu)]
    use rustid::cpuid;

    #[cfg(target_arch = "x86")]
    cyrix_cpuid_check();

    let mut flags = CliFlags {
        color: true,
        ..Default::default()
    };

    let mut action = "default";
    #[cfg(x86_cpu)]
    let mut file_path = None;

    let mut args = std::env::args().skip(1);

    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args.next() {
        let stripped = arg
            .strip_prefix("--")
            .unwrap_or_else(|| arg.strip_prefix('-').unwrap_or(&arg));

        match stripped {
            "m" | "mono" => flags.color = false,
            "e" | "everything" => action = "everything",
            "r" | "dump" => action = "dump",
            #[cfg(x86_cpu)]
            "f" | "file" => {
                file_path = args.next();
                if file_path.is_none() {
                    eprintln!("Error: 'file' option requires a path");
                    help();
                    return;
                }
            }
            "v" | "verbose" => flags.verbose = true,
            "V" | "version" => action = "version",
            "h" | "help" => action = "help",
            "d" | "debug" => action = "debug",
            _ if arg.starts_with('-') && !arg.starts_with("--") => {
                for c in arg.chars().skip(1) {
                    match c {
                        'm' => flags.color = false,
                        'e' => action = "everything",
                        'r' => action = "dump",
                        #[cfg(x86_cpu)]
                        'f' => {
                            file_path = args.next();
                            if file_path.is_none() {
                                eprintln!("Error: '-f' flag requires a path");
                                help();
                                return;
                            }
                        }
                        'v' => flags.verbose = true,
                        'V' => action = "version",
                        'h' => action = "help",
                        _ => {
                            eprintln!("Unknown flag: -{c}");
                            help();
                            return;
                        }
                    }
                }
            }
            _ => {
                eprintln!("Unknown command: {arg}");
                help();
                return;
            }
        }
    }

    #[cfg(x86_cpu)]
    if let Some(path) = &file_path {
        use cpuid::provider::{self, CpuDump};

        rustid::file_version();
        let dump = CpuDump::parse_file(path);
        provider::set_cpuid_provider(dump);
    }

    // Display the version header
    if action != "dump" {
        #[cfg(x86_cpu)]
        if file_path.is_none() {
            version();
        }

        #[cfg(not(x86_cpu))]
        version();
    }

    match action {
        "debug" => {
            Cpu::detect().debug();
        }
        "everything" => {
            flags.verbose = true;
            let cpu = Cpu::detect();
            cpu.display_table(flags);
            println!("---");
            cpu.debug();
        }
        #[cfg(x86_cpu)]
        "dump" => {
            use rustid::cpuid::{dump::dump_cpu, topology::Topology};

            let mut output = String::new();
            let topo = Topology::detect();

            let logical_cores = topo.threads as usize;
            for i in 0..logical_cores {
                dump_cpu(&mut output, i);
            }

            print!("{output}");
        }
        "help" => help(),
        "version" => {}
        "default" => {
            Cpu::detect().display_table(flags);
        }
        _ => unreachable!(),
    }

    #[cfg(x86_cpu)]
    if file_path.is_some() {
        use rustid::cpuid::provider;
        provider::reset_cpuid_provider();
    }
}
