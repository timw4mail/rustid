use super::micro_arch::CpuCore;
use super::micro_arch::*;
use crate::common::*;
use std::collections::BTreeMap;

/// Platform-specific CPU detection result, used by `cpu::Cpu::detect()`.
pub struct OsCpuInfo {
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub isa_string: String,
    pub cores: Vec<CpuCore>,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

// ----------------------------------------------------------------------------
// Linux
// ----------------------------------------------------------------------------

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod linux;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use linux::*;
