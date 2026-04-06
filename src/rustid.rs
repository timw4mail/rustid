#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    rustid::cli_main();

    rustid::cpuid::dos::exit();
}

#[allow(unused)]
fn main() {
    #[cfg(not(feature = "debug"))]
    rustid::cli_main();

    #[cfg(feature = "debug")]
    rustid::debug_main();
}
