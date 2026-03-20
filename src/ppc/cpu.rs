//! Contains the Cpu struct for PowerPC.

use crate::TCpu;
use crate::ppc::micro_arch::{CpuArch, MicroArch};

#[derive(Debug, PartialEq)]
pub struct Cpu {
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
    pub cpu_arch: CpuArch,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::detect()
    }
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let pvr = super::get_pvr();
        let version = (pvr >> 16) as u16;
        let revision = (pvr & 0xFFFF) as u16;
        let cpu_arch = CpuArch::find(pvr);

        Self {
            pvr,
            version,
            revision,
            cpu_arch,
        }
    }

    fn debug(&self) {
        crate::println!("{:#?}", self);
    }

    fn display_table(&self) {
        let label: fn(&str) -> String = |label| format!("{:>17}:{:1}", label, "");
        let simple_line = |l, v: &str| {
            let l = label(l);
            crate::println!("{}{}", l, v);
            crate::println!();
        };

        crate::println!();
        simple_line("Marketing Name", self.cpu_arch.marketing_name.as_str());
        simple_line("Microarchitecture", &String::from(self.cpu_arch.micro_arch));
        simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            simple_line("Process", tech);
        }
        crate::println!();
    }
}
