#![cfg_attr(all(not(test), dos), no_std)]
#![cfg_attr(all(not(test), dos), no_main)]

#[cfg(dos)]
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

#[cfg(dos)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpu, TDetect};
    use rustid::cpuid::dos::{exit, init_heap};
    use rustid::{Cpu, cyrix_cpuid_check, version};

    unsafe { init_heap() };

    cyrix_cpuid_check();

    let cpu = Cpu::detect();
    let flags = CliFlags::default();

    version();
    cpu.display_table(flags);

    exit(0);
}

#[cfg(not(dos))]
pub fn main() {}
