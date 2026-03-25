//! Contains the Cpu struct for ARM.
use super::brand::Vendor;
use super::micro_arch::*;
use crate::common::*;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: BTreeMap<CoreType, CpuCore>,
}

impl TCpu for Cpu {
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

    fn debug(&self) {
        crate::println!("Main ID Register (MIDR): 0x{:X}", self.raw_midr);
        crate::println!(
            "Implementer: 0x{:X} ({})",
            self.midr.implementer,
            self.vendor
        );
        crate::println!("Variant: 0x{:X}", self.midr.variant);
        crate::println!("Part Number: 0x{:X}", self.midr.part);
        crate::println!("Revision: 0x{:X}", self.midr.revision);
        crate::println!("{:#?}", self);
    }

    fn display_table(&self) {
        let label: fn(&str) -> String = |label| format!("{:>17}:{:1}", label, "");
        let sublabel: fn(&str) -> String = |label| format!("{:>19}{}:{:1}", "", label, "");
        let terlabel: fn(&str) -> String = |label| format!("{:>21}{}:{:1}", "", label, "");

        let simple_line = |l, v: &str| {
            let l = label(l);
            println!("{}{}", l, v);
            println!();
        };

        crate::println!();
        simple_line("Brand/Implementor", self.cpu_arch.implementer.into());
        simple_line("Model", &self.cpu_arch.model);
        simple_line("Microarchitecture", &String::from(self.cpu_arch.micro_arch));
        simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            simple_line("Process", tech);
        }

        [CoreType::Super, CoreType::Performance, CoreType::Efficiency]
            .iter()
            .for_each(|k| {
                if let Some(core) = self.cores.get(k) {
                    let name = format!("{} Cores", Into::<String>::into(*k));
                    println!("{}{}", label(&name), core.count);

                    if let Some(name) = core.name.clone() {
                        println!("{}{}", sublabel("Name"), name);
                    }

                    if let Some(cache) = core.cache {
                        let cache_count = |share_count| {
                            if share_count == 0u32 || (core.count as u32 / share_count) <= 1 {
                                String::new()
                            } else {
                                format!("{}x ", core.count as u32 / share_count)
                            }
                        };

                        println!("{}", sublabel("Cache"));

                        match cache.l1 {
                            Level1Cache::Unified(cache) => {
                                println!("L1: Unified {:>4} KB", cache.size);
                            }
                            Level1Cache::Split { data, instruction } => {
                                let data_count: String = cache_count(data.share_count);
                                let instruction_count = cache_count(instruction.share_count);

                                println!("{}{}{} KB", terlabel("L1d"), &data_count, data.size);
                                println!(
                                    "{}{}{} KB",
                                    terlabel("L1i"),
                                    &instruction_count,
                                    instruction.size,
                                );
                            }
                        }

                        if let Some(cache) = cache.l2 {
                            let count = cache_count(cache.share_count);

                            let mut num = cache.size / 1024;
                            let unit = if num >= 1024 { "MB" } else { "KB" };

                            if num >= 1024 {
                                num /= 1024;
                            }

                            println!("{} {}{} {}", terlabel("L2"), &count, num, unit);
                        }

                        if let Some(cache) = cache.l3 {
                            let mut num = cache.size;
                            let unit = if num >= 1024 { "MB" } else { "KB" };

                            if num >= 1024 {
                                num /= 1024
                            }

                            println!("{} {} {}", terlabel("L3"), num, unit);
                        }

                        println!();
                    }
                }
            });
        crate::println!();
    }
}
