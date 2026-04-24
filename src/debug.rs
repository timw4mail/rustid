#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        ".code16",
        "jmp 1f",
        ".align 4",
        "2:",       // GDT Pointer
        ".word 15", // Limit
        "3:",       // Base placeholder
        ".long 0",  // Base
        "4:",       // GDT Entries
        ".quad 0",  // Null
        ".quad 0x00CF92000000FFFF", // Data
        "1:",
        // Basic setup
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",
        "cli",
        "push ds",
        "push es",
        // Calculate linear address: EAX = (CS << 4) + offset 4:
        "mov eax, cs",
        "movzx eax, ax",
        "shl eax, 4",
        "xor ebx, ebx",
        "lea bx, [4b]",
        "add eax, ebx",
        // Patch GDT pointer base (label 3:)
        "lea bx, [3b]",
        ".byte 0x66, 0x89, 0x07",
        // Load GDT (label 2:)
        "lea bx, [2b]",
        ".byte 0x0F, 0x01, 0x17", // lgdt [bx]
        // Switch to PM
        "mov eax, cr0",
        "or al, 1",
        "mov cr0, eax",
        "jmp 6f",
        "6:",
        "mov ax, 8",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",
        "mov fs, ax",
        "mov gs, ax",
        "and al, 0xFE",
        "mov cr0, eax",
        "jmp 7f",
        "7:",
        "pop es",
        "pop ds",
        "sti",
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",
        // Ensure SP is clean
        ".byte 0x66, 0x0F, 0xB7, 0xE4", // movzx esp, sp
        // Jump to rust_main (E9 XX XX)
        // Manual 16-bit near jump to avoid 32-bit mis-encoding
        ".byte 0xE9",
        ".word rust_main - 8f",
        "8:",
        ".align 4"
    );
}

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() {
    use rustid::common::TCpu;
    use rustid::cpuid::dos::exit;
    use rustid::cpuid::quirks::debug_quirks;
    use rustid::{Cpu, cyrix_cpuid_check, println, version};

    version();
    cyrix_cpuid_check();
    debug_quirks();
    println!("---");

    Cpu::detect().debug();

    exit();
}

#[cfg(not(all(target_os = "none", target_arch = "x86")))]
pub fn main() {}
