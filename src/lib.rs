#![no_std]

#[cfg(not(target_os = "none"))]
extern crate std;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod cpuid;

#[cfg(target_os = "none")]
#[macro_use]
pub mod dos;
