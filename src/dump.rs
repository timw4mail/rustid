#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

use rustid::cpuid::dump::dump_main;

#[allow(unused)]
fn main() {
    dump_main();
}

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    dump_main();

    rustid::cpuid::dos::exit();
}
