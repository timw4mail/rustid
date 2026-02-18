#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");
pub mod cpuid;
#[cfg(target_os = "none")]
#[macro_use]
pub mod dos;

#[cfg(target_os = "none")]
pub use dos::*;

use cpuid::Cpu;

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(all(not(test), target_os = "none"))]
mod intrinsics {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        let mut i = 0;
        while i < n {
            unsafe {
                *dest.add(i) = *src.add(i);
            }

            i += 1;
        }
        dest
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
        let mut i = 0;
        while i < n {
            unsafe {
                let a = *s1.add(i);
                let b = *s2.add(i);

                if a != b {
                    return a as i32 - b as i32;
                }
            }
            i += 1;
        }
        0
    }
}

#[cfg(all(not(test), target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    Cpu::new().display();

    dos::exit();
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(unused)]
fn main() {
    Cpu::new().display_table();
}
