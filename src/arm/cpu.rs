//! Contains the Cpu struct for ARM.
use super::micro_arch::*;
use super::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub system: Option<String>,
    pub soc_model: Option<String>,
    pub cores: Vec<CpuCore>,
    pub raw: BTreeMap<String, String>,
    pub features: BTreeMap<&'static str, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

impl Cpu {
    /// Returns true if this CPU has multiple core types (hybrid architecture).
    pub fn is_hybrid(&self) -> bool {
        self.cores.len() > 1
    }

    /// Total physical cores across all clusters
    pub fn total_cores(&self) -> u32 {
        self.cores.iter().map(|c| c.count).sum()
    }

    /// Total logical threads across all clusters
    pub fn total_threads(&self) -> u32 {
        self.cores.iter().map(|c| c.threads).sum()
    }
}

impl TDetect for Cpu {
    fn detect() -> Self {
        let info = crate::arm::os::detect();
        let features = super::get_all_features();

        Self {
            midrs: info.midrs,
            vendor: info.vendor,
            cpu_arch: info.cpu_arch,
            model: info.model,
            system: OS::get_system_name(),
            soc_model: OS::get_soc(),
            cores: info.cores,
            raw: info.raw,
            features,
            midr_source: info.midr_source,
            features_source: info.features_source,
        }
    }
}

impl TArmCpu for Cpu {
    fn model(&self) -> Option<&str> {
        if self.model.is_empty() {
            None
        } else {
            Some(&self.model)
        }
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}
