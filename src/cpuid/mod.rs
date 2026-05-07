//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.
/// Compile-time check to ensure this crate is only used on x86/x86_64.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");

// ----------------------------------------------------------------------------

pub mod brand;
pub mod cache;
pub mod constants;
pub mod cpu;

#[cfg(target_os = "none")]
pub mod dos;

pub mod dump;
pub mod fns;
pub mod micro_arch;
pub mod mp;

#[cfg(not(target_os = "none"))]
pub mod provider;

pub mod topology;
pub mod vendor;

pub mod quirks;

// ----------------------------------------------------------------------------

pub use brand::*;
pub use constants::*;
pub use cpu::*;
pub use fns::*;

pub use quirks::*;
