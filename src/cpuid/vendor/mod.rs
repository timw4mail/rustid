pub mod centaur;
#[cfg(target_arch = "x86")]
pub mod cyrix;
#[cfg(target_arch = "x86")]
pub mod transmeta;

#[cfg(target_arch = "x86")]
pub use cyrix::Cyrix;
