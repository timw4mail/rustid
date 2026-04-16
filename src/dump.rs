#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(all(target_os = "none", target_arch = "x86"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    use rustid::cpuid::dos::exit;
    use rustid::cpuid::dump::dump_main;
    use rustid::cpuid::has_cpuid;
    use rustid::println;
    use rustid::version;

    if has_cpuid() {
        dump_main();
    } else {
        version();
        println!("This cpu does not support cpuid. Cpuid info cannot be dumped.");
    }

    exit();
}

#[cfg(not(all(target_os = "none", target_arch = "x86")))]
pub fn main() {}
