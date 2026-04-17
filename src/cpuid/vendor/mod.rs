pub mod amd;
pub mod centaur;

#[cfg(target_arch = "x86")]
pub mod cyrix;

pub mod intel;

#[cfg(target_arch = "x86")]
pub mod transmeta;

// ----------------------------------------------------------------------------

pub use amd::Amd;
pub use centaur::Centaur;

#[cfg(target_arch = "x86")]
pub use cyrix::Cyrix;

pub use intel::Intel;

#[cfg(target_arch = "x86")]
pub use transmeta::Transmeta;

// ----------------------------------------------------------------------------

/// Trait for vendor-specific microarchitecture detection.
///
/// Each vendor (Intel, AMD, etc.) implements this trait to provide
/// CPU identification based on their unique CPU signatures and feature flags.
pub trait TMicroArch {
    /// Detects the microarchitecture based on the CPU model string and signature.
    fn micro_arch(model: &str, s: super::CpuSignature) -> super::micro_arch::CpuArch;
}
