//! Contains the Cpu struct for ARM.
use super::CpuDisplay;
use super::brand::Vendor;
use super::micro_arch::*;
use super::*;
use crate::common::*;
use std::collections::BTreeMap;

// TODO: Wrap raw_mdir and midr fields into HashSets or some other collection type
#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: BTreeMap<CoreType, CpuCore>,
}

impl TCpu for Cpu {
    // TODO: Update detection logic to probe every thread so that Big.Little cpus
    // have all their core types detected
    fn detect() -> Self {
        let cores: BTreeMap<CoreType, CpuCore> = BTreeMap::new();
        let raw_midr = super::get_midr();
        let midr = Midr::new(raw_midr);
        let vendor = Vendor::from(midr.implementer);
        let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);

        Self {
            raw_midr,
            midr,
            vendor: vendor.into(),
            cpu_arch,
            cores,
        }
    }

    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        println!("Main ID Register (MIDR): 0x{:X}", self.raw_midr());
        println!(
            "Implementer: 0x{:X} ({})",
            self.midr().implementer,
            self.vendor()
        );
        println!("Variant: 0x{:X}", self.midr().variant);
        println!("Part Number: 0x{:X}", self.midr().part);
        println!("Revision: 0x{:X}", self.midr().revision);
        println!("{:#?}", self);
    }

    fn display_table(&self) {
        CpuDisplay::display(&self.cpu_arch, &self.cores, None);
    }
}

impl TArmCpu for Cpu {
    fn raw_midr(&self) -> usize {
        self.raw_midr
    }

    fn midr(&self) -> &Midr {
        &self.midr
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}
