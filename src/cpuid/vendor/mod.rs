pub mod amd;
pub mod centaur;
#[cfg(target_arch = "x86")]
pub mod cyrix;
pub mod intel;
#[cfg(target_arch = "x86")]
pub mod transmeta;

pub use amd::Amd;

#[cfg(target_arch = "x86")]
pub use cyrix::Cyrix;

pub use centaur::Centaur;

pub use intel::Intel;
#[cfg(target_arch = "x86")]
pub use transmeta::Transmeta;

pub trait TMicroArch {
    fn micro_arch(model: &str, s: super::CpuSignature) -> super::micro_arch::CpuArch;
}
