//! Contains the Cpu struct for RISC-V.
use super::micro_arch::*;
use crate::common::*;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub system: Option<String>,
    pub isa_string: String,
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
        let info = crate::riscv::os::detect();
        let features = super::get_all_features(&info.isa_string);

        Self {
            vendor: info.vendor,
            cpu_arch: info.cpu_arch,
            model: info.model,
            system: OS::get_system_name(),
            isa_string: info.isa_string,
            cores: info.cores,
            raw: info.raw,
            features,
            midr_source: info.midr_source,
            features_source: info.features_source,
        }
    }
}

impl TCpuDisplay for Cpu {
    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        CpuDisplay::display_riscv(self, flags);
    }
}
