#![cfg_attr(all(not(test), dos_real), no_std)]
#![cfg_attr(all(not(test), dos_real), no_main)]

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

#[cfg(not(dos_real))]
pub fn main() {}
