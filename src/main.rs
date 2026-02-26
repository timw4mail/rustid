#![cfg_attr(all(not(test), target_os = "none", target_arch = "x86"), no_std)]
#![cfg_attr(all(not(test), target_os = "none", target_arch = "x86"), no_main)]

#[allow(unused)]
fn main() {
    rustid::cli_main();
}
