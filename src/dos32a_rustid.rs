#![cfg_attr(all(not(test), dos32a), no_std)]
#![cfg_attr(all(not(test), dos32a), no_main)]

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
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::x86::dos32a::{exit, init_heap};
    use rustid::{Cpu, cyrix_cpuid_check, version};

    unsafe { init_heap() };

    cyrix_cpuid_check();

    let cpu = Cpu::detect();
    let flags = CliFlags::default();

    version();
    cpu.display_table(flags);

    exit(0);
}

#[cfg(not(dos32a))]
pub fn main() {}
