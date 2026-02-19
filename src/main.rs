#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    rustid::cli_main();
}
