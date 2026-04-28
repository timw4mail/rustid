#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(all(target_os = "none", target_arch = "x86"))]
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

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::TCpu;
    use rustid::cpuid::dos::{exit, init_heap};
    use rustid::cpuid::quirks::debug_quirks;
    use rustid::{Cpu, cyrix_cpuid_check, println, version};

    unsafe { init_heap() };

    version();
    cyrix_cpuid_check();
    debug_quirks();
    println!("---");

    Cpu::detect().debug();

    exit();
}

#[cfg(not(all(target_os = "none", target_arch = "x86")))]
pub fn main() {}
