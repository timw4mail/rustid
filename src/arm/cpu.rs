//! Contains the Cpu struct for ARM.
use super::micro_arch::*;
use super::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: HashSet<usize>,
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub system: Option<String>,
    pub soc_model: Option<String>,
    pub cores: BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
    pub raw: BTreeMap<String, String>,
    pub features: BTreeMap<&'static str, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

impl TDetect for Cpu {
    fn detect() -> Self {
        let info = crate::arm::os::detect();
        let features = super::get_all_features();

        Self {
            raw_midr: info.raw_midr,
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

impl TCpuDisplay for Cpu {
    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        println!(
            "Main ID Register (MIDR): 0x{:X}",
            self.raw_midr().iter().next().unwrap_or(&0)
        );
        if let Some(midr) = self.midr() {
            println!("Implementer: 0x{:X} ({})", midr.implementer, self.vendor());
            println!("Variant: 0x{:X}", midr.variant);
            println!("Part Number: 0x{:X}", midr.part);
            println!("Revision: 0x{:X}", midr.revision);
        }
        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        CpuDisplay::display(self, flags);
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

    fn raw_midr(&self) -> HashSet<usize> {
        self.raw_midr.clone()
    }

    fn midr(&self) -> Option<&Midr> {
        self.midrs.iter().next()
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}
