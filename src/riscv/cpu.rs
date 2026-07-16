//! Contains the Cpu struct for RISC-V.
use super::micro_arch::*;
use super::*;
use crate::common::*;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub system: Option<String>,
    pub isa_string: String,
    pub cores: BTreeMap<CoreType, CpuCore>,
    pub raw: BTreeMap<String, String>,
    pub features: BTreeMap<&'static str, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
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
        CpuDisplay::display(self, flags);
    }
}

impl TRiscvCpu for Cpu {
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
