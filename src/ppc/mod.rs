//! PowerPC CPU detection.

#[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64")))]
compile_error!("This crate only supports PowerPC architectures.");

pub mod cpu;
pub mod fns;
