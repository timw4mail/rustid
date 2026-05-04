#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(target_arch = "x86")]
use rustid::cyrix_cpuid_check;

use rustid::common::TCpu;
use rustid::{Cpu, version};

// --- DOS entry point ---
#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        ".code16",
        // DS on entry = PSP segment; save it in CX
        "mov cx, ds",
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",
        // Ensure ESP is clean (only 16-bit SP is set by loader)
        ".byte 0x66, 0x31, 0xC0", // xor eax, eax
        "mov ax, sp",
        ".byte 0x66, 0x89, 0xC4", // mov esp, eax
        // Jump to rust_main, PSP seg in CX
        "jmp rust_main",
        ".align 4"
    );
}

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::cpuid::dos::{
        DosWriter, exit, exit_unreal_mode, init_dos_environment, init_heap, is_unreal_mode_enabled,
        peek_u8,
    };
    use rustid::cpuid::{dump::dump_cpu, has_cpuid, topology::Topology};
    use rustid::println;

    // Initialize DOS environment (includes unreal mode for flat 32-bit addressing)
    init_dos_environment();

    unsafe { init_heap() };

    // PSP segment is in CX (passed from _start)
    let psp_seg: u16;
    unsafe {
        core::arch::asm!("mov {0:x}, cx", out(reg) psp_seg);
    }

    // Parse command line from PSP
    // PSP+0x80 = command line length (byte), PSP+0x81+ = command line (starts with space)
    let cmd_len = peek_u8(psp_seg, 0x80) as usize;

    let mut action = "default";

    if cmd_len > 0 {
        // Read first token (skip leading space at 0x81)
        let mut idx = 0x82u16;
        let end = 0x81u16.wrapping_add(cmd_len as u16);

        // Skip whitespace
        while idx < end {
            let b = peek_u8(psp_seg, idx);
            if b != b' ' && b != b'\t' {
                break;
            }
            idx = idx.wrapping_add(1);
        }

        // Read token bytes into a stack buffer
        let mut token_buf = [0u8; 32];
        let mut token_len = 0usize;
        while idx < end && token_len < token_buf.len() {
            let b = peek_u8(psp_seg, idx);
            if b == b' ' || b == b'\t' {
                break;
            }
            token_buf[token_len] = b;
            token_len += 1;
            idx = idx.wrapping_add(1);
        }

        // Parse token as argument
        if token_len > 0 && token_buf[0] == b'/' {
            let stripped = &token_buf[1..token_len];
            if stripped.len() > 0 {
                match stripped {
                    b"r" | b"dump" => action = "dump",
                    b"v" | b"version" => action = "version",
                    b"h" | b"?" | b"help" => action = "help",
                    _ => {}
                }
            }
        }
    }

    match action {
        "dump" => {
            if has_cpuid() {
                let mut output = DosWriter {};

                let topo = Topology::detect();

                let logical_cores = topo.threads as usize;
                for i in 0..logical_cores {
                    dump_cpu(&mut output, i);
                }
            } else {
                version();
                cyrix_cpuid_check();
                println!("This cpu does not support cpuid. Cpuid info cannot be dumped.");
            }
        }
        "version" => {
            version();
        }
        "help" => {
            version();
            println!("Usage: rustid [command]");
            println!();
            println!("Commands:");
            println!("  /r, /dump          Dump raw CPUID values");
            println!("  /v, /version       Display version info");
            println!("  /?, /h, /help      Show this help message");
            println!();
        }
        "default" => {
            version();
            cyrix_cpuid_check();
            Cpu::detect().display_table(false);
        }
        _ => unreachable!(),
    }

    // Exit unreal mode - now back to real mode with flat 31-bit memory space
    if is_unreal_mode_enabled() {
        unsafe { exit_unreal_mode() };
    }

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
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let mut file_path = None;

    let mut args = std::env::args().skip(1);

    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args.next() {
        let stripped = arg
            .strip_prefix("--")
            .unwrap_or_else(|| arg.strip_prefix('-').unwrap_or(&arg));

        match stripped {
            "m" | "mono" => color = false,
            "e" | "everything" => action = "everything",
            "r" | "dump" => action = "dump",
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
                        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
    }

    // Display the version header
    if action != "dump" {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if file_path.is_none() {
            version();
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        version();
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
            use rustid::cpuid::{dump::dump_cpu, topology::Topology};

            let mut output = String::new();
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
