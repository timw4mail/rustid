#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    use rustid::cpuid::dos::{DosWriter, exit};
    use rustid::cpuid::{dump::dump_cpu, has_cpuid, topology::Topology};
    use rustid::{println, version};

    // Initialize segment registers for EXE format
    // Read segment offsets from the metadata header at offset 0
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!(
            "mov ax, cs",
            "mov ds, ax",
            "mov es, ax",
            options(preserves_flags, nostack)
        );
    }

    if has_cpuid() {
        let mut output = DosWriter {};

        let topo = Topology::detect();

        let logical_cores = topo.threads as usize;
        for i in 0..logical_cores {
            dump_cpu(&mut output, i);
        }
    } else {
        version();
        println!("This cpu does not support cpuid. Cpuid info cannot be dumped.");
    }

    exit();
}

#[cfg(not(all(target_os = "none", target_arch = "x86")))]
pub fn main() {}
