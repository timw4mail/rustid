#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
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
