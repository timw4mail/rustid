#![cfg_attr(all(not(test), dos_os), no_std)]
#![cfg_attr(all(not(test), dos_os), no_main)]

#[cfg(dos_ext)]
extern crate alloc;

#[cfg(dos_real)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        ".code16",
        // Basic segment setup
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",
        // Ensure SP is clean
        ".byte 0x66, 0x0F, 0xB7, 0xE4", // movzx esp, sp
        // Jump to rust_main (E9 XX XX)
        // Manual 16-bit near jump to avoid 32-bit mis-encoding
        ".byte 0xE9",
        ".word rust_main - 1f",
        "1:",
        ".align 4"
    );
}

#[cfg(dos_real)]
fn help() {
    use rustid::println;
    println!("Usage: RUST86 [/FLAGS]");
    println!();
    println!("Flags (use / or - prefix):");
    println!("  /D, /DEBUG    Display detailed debug and quirk diagnostics");
    println!("  /V, /VERBOSE  Output more detailed information");
    println!("  /?, /H, /HELP Show this help message");
}

#[cfg(dos_real)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::x86::dos::{exit, get_args, init_heap};
    use rustid::x86::quirks::debug_quirks;
    use rustid::{Cpu, cyrix_cpuid_check, println, version};

    unsafe { init_heap() };

    let args = get_args();
    let mut flags = CliFlags::default();
    let mut is_debug = false;

    for token in args.as_slice() {
        let stripped = if let Some(rest) = token.strip_prefix('/') {
            rest
        } else if let Some(rest) = token.strip_prefix('-') {
            rest
        } else {
            *token
        };

        if stripped.eq_ignore_ascii_case("D") || stripped.eq_ignore_ascii_case("DEBUG") {
            is_debug = true;
        } else if stripped.eq_ignore_ascii_case("V") || stripped.eq_ignore_ascii_case("VERBOSE") {
            flags.verbose = true;
        } else if stripped == "?"
            || stripped.eq_ignore_ascii_case("H")
            || stripped.eq_ignore_ascii_case("HELP")
        {
            help();
            exit(0);
        } else {
            for b in stripped.bytes() {
                match b.to_ascii_uppercase() {
                    b'D' => is_debug = true,
                    b'V' => flags.verbose = true,
                    b'?' | b'H' => {
                        help();
                        exit(0);
                    }
                    _ => {}
                }
            }
        }
    }

    cyrix_cpuid_check();
    version();

    if is_debug {
        debug_quirks();
        println!("---");
        Cpu::detect().debug();
    } else {
        let cpu = Cpu::detect();
        cpu.display_table(flags);
    }

    exit(0);
}

#[cfg(dos_ext)]
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

#[cfg(dos_ext)]
fn help() {
    use rustid::println;
    println!("Usage: RUSTID [/FLAGS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (no args)      Display CPU information");
    println!("  D, DEBUG       Display detailed debug information");
    println!("  E, EVERYTHING  Show CPU information and debug information");
    println!("  R, DUMP        Dump raw CPUID values");
    println!("  V, VERSION     Display version info");
    println!("  ?, H, HELP     Show this help message");
    println!();
    println!("Flags (use / or - prefix):");
    println!("  /C  /COMPACT   Display information in compact mode");
    println!("  /M, /MONO      Don't output color");
    println!("  /V, /VERBOSE   Output more detailed information");
    println!();
    println!("Examples:  RUSTID /E   RUSTID /VERBOSE");
}

#[cfg(dos_ext)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::x86::dos::{exec_dos_binary, exit, get_args, init_heap, set_color_mode};
    use rustid::{Cpu, cyrix_cpuid_check, version};

    unsafe { init_heap() };

    cyrix_cpuid_check();

    let args = get_args();

    let mut flags = CliFlags {
        color: true,
        ..Default::default()
    };
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
                "?" | "H" | "HELP" => {
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
            "COMPACT" => {
                flags.compact = false;
                continue 'args;
            }
            "VERBOSE" => {
                flags.verbose = true;
                continue 'args;
            }
            "MONO" => {
                flags.color = false;
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
                'C' => flags.compact = true,
                'V' => flags.verbose = true,
                'M' => flags.color = false,
                'D' => action = "debug",
                'E' => action = "everything",
                'R' => action = "dump",
                'H' | '?' => action = "help",
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

    set_color_mode(flags.color);

    if action == "help" || had_error {
        help();
        exit(if had_error { 1 } else { 0 });
    }

    // Real-mode fallback for non-Cyrix pre-CPUID CPUs in default and debug modes
    if (action == "default" || action == "debug")
        && !rustid::x86::has_cpuid()
        && !rustid::x86::vendor::Cyrix::has_device_ids()
    {
        let cmd_tail = if action == "debug" { "/D" } else { "" };
        if let Err(err) = exec_dos_binary("rust86.exe", cmd_tail) {
            use rustid::println;
            println!(
                "Failed to execute real mode binary rust86.exe (error {})",
                err
            );
            exit(1);
        }
        exit(0);
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

#[cfg(not(dos_os))]
pub fn main() {}
