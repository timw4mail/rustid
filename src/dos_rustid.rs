#![cfg_attr(all(not(test), dos), no_std)]
#![cfg_attr(all(not(test), dos), no_main)]

#[cfg(dos)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    #[cfg(not(feature = "dos32a"))]
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

    #[cfg(feature = "dos32a")]
    core::arch::naked_asm!(
        // In protected mode, we don't need to set segments like in real mode.
        // The extender will have already done some setup.
        // We just jump to rust_main.
        "jmp rust_main"
    );
}

#[cfg(dos)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpu, TDetect};
    use rustid::cpuid::dos::{exit, init_heap};
    use rustid::{Cpu, cyrix_cpuid_check, version};

    #[cfg(not(feature = "dos32a"))]
    unsafe {
        use rustid::cpuid::dos::init_heap;
        init_heap()
    };

    cyrix_cpuid_check();

    let cpu = Cpu::detect();
    let flags = CliFlags::default();

    version();
    cpu.display_table(flags);

    #[cfg(not(feature = "dos32a"))]
    {
        use rustid::cpuid::dos::exit;
        exit(0);
    }

    #[cfg(feature = "dos32a")]
    {
        // In 32-bit mode, we might need a different way to exit.
        // For now, let's just loop if we don't have an exit function.
        // But DOS32A should support INT 21h AH=4Ch.
        unsafe {
            core::arch::asm!("int 0x21", "mov ah, 0x4C", "mov al, 0", options(noreturn));
        }
    }
}

#[cfg(not(dos))]
pub fn main() {}
