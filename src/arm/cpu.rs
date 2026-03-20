//! Contains the Cpu struct for ARM.

use super::brand::Vendor;
use crate::TCpu;
use crate::arm::micro_arch::CpuArch;
use crate::arm::micro_arch::Midr;

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
    pub cpu_arch: CpuArch,
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let raw_midr = super::get_midr();
        let midr = Midr::new(raw_midr);
        let vendor = Vendor::from(midr.implementer);
        let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);

        Self {
            raw_midr,
            midr,
            vendor: vendor.into(),
            cpu_arch,
        }
    }

    fn debug(&self) {
        crate::println!("{:#?}", self);
    }

    fn display_table(&self) {
        crate::println!();
        crate::println!("Main ID Register (MIDR): 0x{:X}", self.raw_midr);
        crate::println!(
            "Implementer: 0x{:X} ({})",
            self.midr.implementer,
            self.vendor
        );
        crate::println!("Variant: 0x{:X}", self.midr.variant);
        crate::println!("Part Number: 0x{:X}", self.midr.part);
        crate::println!("Revision: 0x{:X}", self.midr.revision);
        crate::println!("---");
        crate::println!("Marketing Name: {}", self.cpu_arch.marketing_name.as_str());
        crate::println!(
            "Microarchitecture: {}",
            String::from(self.cpu_arch.micro_arch)
        );
        crate::println!("Code Name: {}", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            crate::println!("Process: {}", tech);
        }
        crate::println!();
    }
}
