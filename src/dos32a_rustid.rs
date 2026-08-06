#![cfg_attr(all(not(test), dos32a), no_std)]
#![cfg_attr(all(not(test), dos32a), no_main)]

#[cfg(dos32a)]
extern crate alloc;

#[cfg(dos32a)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        // Setup flat selectors
        "mov ax, ds",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        // Setup stack
        "lea esp, [_stack_top]",
        // Jump to rust_main
        "jmp rust_main"
    );
}

#[cfg(dos32a)]
fn help() {
    use rustid::println;
    println!("Usage: RUSTID32 [/FLAGS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (no args)    Display CPU information");
    println!("  D, DEBUG     Display detailed debug information");
    println!("  E, EVERYTHING  Show CPU information and debug information");
    println!("  R, DUMP      Dump raw CPUID values");
    println!("  V, VERSION   Display version info");
    println!("  H, HELP      Show this help message");
    println!();
    println!("Flags (use / or - prefix):");
    println!("  /M, /MONO    Don't output color");
    println!("  /V, /VERBOSE Output more detailed information");
    println!();
    println!("Examples:  RUSTID32 /M E   RUSTID32 /VERBOSE");
}

#[cfg(dos32a)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::x86::dos32a::{exit, get_args, init_heap};
    use rustid::{Cpu, cyrix_cpuid_check, version};

    unsafe { init_heap() };

    cyrix_cpuid_check();

    let args = get_args();

    let mut flags = CliFlags::default();
    let mut action = "default";
    let mut had_error = false;

    'args: for token in args.as_slice() {
        // DOS-style: strip a leading '/' or '-' prefix
        let stripped = if let Some(rest) = token.strip_prefix('/') {
            rest
        } else if let Some(rest) = token.strip_prefix('-') {
            rest
        } else {
            // No prefix — treat as a bare command keyword
            match &token.to_ascii_uppercase()[..] {
                "D" | "DEBUG" => {
                    action = "debug";
                    continue 'args;
                }
                "E" | "EVERYTHING" => {
                    action = "everything";
                    continue 'args;
                }
                "R" | "DUMP" => {
                    action = "dump";
                    continue 'args;
                }
                "V" | "VERSION" => {
                    action = "version";
                    continue 'args;
                }
                "H" | "HELP" => {
                    action = "help";
                    continue 'args;
                }
                _ => {
                    use rustid::println;
                    println!("Unknown command: {}", token);
                    had_error = true;
                    action = "help";
                    break 'args;
                }
            }
        };

        // The stripped token may be a single char flag, a combined run of
        // single-char flags (e.g. "MV"), or a long keyword (e.g. "MONO").
        let upper = stripped.to_ascii_uppercase();

        // Try long-form keywords first (more than one char, not all single-char flags)
        match &upper[..] {
            "MONO" => {
                flags.color = false;
                continue 'args;
            }
            "VERBOSE" => {
                flags.verbose = true;
                continue 'args;
            }
            "DEBUG" => {
                action = "debug";
                continue 'args;
            }
            "EVERYTHING" => {
                action = "everything";
                continue 'args;
            }
            "DUMP" => {
                action = "dump";
                continue 'args;
            }
            "VERSION" => {
                action = "version";
                continue 'args;
            }
            "HELP" => {
                action = "help";
                continue 'args;
            }
            _ => {}
        }

        // Fall back to per-character single-char flags (e.g. /MV = mono + verbose)
        for c in upper.chars() {
            match c {
                'M' => flags.color = false,
                'V' => flags.verbose = true,
                'D' => action = "debug",
                'E' => action = "everything",
                'R' => action = "dump",
                'H' => action = "help",
                _ => {
                    use rustid::println;
                    println!("Unknown flag: /{}", c);
                    had_error = true;
                    action = "help";
                    break 'args;
                }
            }
        }
    }

    if action == "help" || had_error {
        help();
        exit(if had_error { 1 } else { 0 });
    }

    if action != "dump" {
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
            use rustid::println;
            println!("---");
            cpu.debug();
        }
        "dump" => {
            use rustid::print;
            use rustid::x86::{dump::dump_cpu, topology::Topology};

            let mut output = alloc::string::String::new();
            let topo = Topology::detect();
            let logical_cores = topo.threads.count as usize;
            for i in 0..logical_cores {
                dump_cpu(&mut output, i);
            }
            print!("{}", output);
        }
        "version" => {}
        "default" => {
            Cpu::detect().display_table(flags);
        }
        _ => unreachable!(),
    }

    exit(0);
}

#[cfg(not(dos32a))]
pub fn main() {}
