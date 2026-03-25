#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[allow(unused)]
fn main() {
    rustid::cli_main();
}
