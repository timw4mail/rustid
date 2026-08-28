//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.
/// Compile-time check to ensure this crate is only used on x86/x86_64.
#[cfg(not(x86_cpu))]
compile_error!("This crate only supports x86 and x86_64 architectures.");

// ----------------------------------------------------------------------------

pub mod brand;

#[cfg(not(dos_real))]
pub mod cache;

pub mod constants;
pub mod count;
pub mod cpu;
pub mod display;

#[cfg(dos_os)]
pub mod dos;

#[cfg(uefi)]
pub mod efi;

pub mod dump;
pub mod features;
pub mod fns;
pub mod micro_arch;

#[cfg(std_os)]
pub mod os;

#[cfg(std_os)]
pub mod provider;

pub mod topology;
pub mod vendor;

pub mod quirks;

// ----------------------------------------------------------------------------

pub use brand::*;
pub use constants::*;
pub use count::*;
pub use cpu::*;
pub use features::*;
pub use fns::*;
pub use micro_arch::*;

pub use quirks::*;
