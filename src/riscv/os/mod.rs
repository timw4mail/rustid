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

#[cfg(linux_os)]
pub mod linux;
#[cfg(linux_os)]
pub use linux::*;

#[cfg(not(linux_os))]
pub mod fallback {
    use super::*;
    pub fn detect() -> OsCpuInfo {
        OsCpuInfo {
            vendor: String::new(),
            cpu_arch: CpuArch::default(),
            model: String::new(),
            isa_string: String::new(),
            cores: Vec::new(),
            raw: BTreeMap::new(),
            midr_source: DataSource::default(),
            features_source: DataSource::default(),
        }
    }
    pub fn get_all_features(_isa: &str) -> BTreeMap<&'static str, String> {
        BTreeMap::new()
    }
}
#[cfg(not(linux_os))]
pub use fallback::*;
