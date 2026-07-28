#![cfg_attr(all(not(test), dos32a), no_std)]
#![cfg_attr(all(not(test), dos32a), no_main)]

#[cfg(dos32a)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        // Setup flat 32-bit protected mode segments
        "xor eax, eax",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        // Setup stack
        "lea esp, [_stack_top]",
        // Jump to rust_main
        "jmp rust_main"
    );
}

#[cfg(dos32a)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    use rustid::x86::dos32a::{DosWriter, exit, init_heap};
    use rustid::x86::{dump::dump_cpu, has_cpuid, topology::Topology};
    use rustid::{println, version};

    unsafe { init_heap() };

    if has_cpuid() {
        let mut output = DosWriter {};

        let topo = Topology::detect();

        let logical_cores = topo.threads.count as usize;
        for i in 0..logical_cores {
            dump_cpu(&mut output, i);
        }
    } else {
        version();
        println!("This cpu does not support cpuid. Cpuid info cannot be dumped.");
    }

    exit(0);
}

#[cfg(not(dos32a))]
pub fn main() {}
