//! Contains the Cpu struct for ARM.
use super::brand::Vendor;
use super::micro_arch::*;
use super::CpuDisplay;
use super::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: HashSet<usize>,
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: BTreeMap<CoreType, CpuCore>,
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let mut raw_midr: HashSet<usize> = HashSet::new();
        let mut midrs: HashSet<Midr> = HashSet::new();

        #[cfg(not(target_os = "macos"))]
        {
            if let Some(core_ids) = core_affinity::get_core_ids() {
                for core_id in core_ids {
                    core_affinity::set_for_current(core_id);
                    let midr_val = super::get_midr();
                    raw_midr.insert(midr_val);
                    midrs.insert(Midr::new(midr_val));
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let midr_val = super::get_midr();
            raw_midr.insert(midr_val);
            midrs.insert(Midr::new(midr_val));
        }

        let primary_midr = midrs.iter().next().unwrap_or(&Midr::default());
        let vendor = Vendor::from(primary_midr.implementer);
        let cpu_arch = CpuArch::find(
            primary_midr.implementer,
            primary_midr.part,
            primary_midr.variant,
        );

        let cores = Self::detect_cores(&midrs);

        Self {
            raw_midr,
            midrs,
            vendor: vendor.into(),
            cpu_arch,
            cores,
        }
    }

    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        println!(
            "Main ID Register (MIDR): 0x{:X}",
            self.raw_midr().iter().next().unwrap_or(&0)
        );
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
    fn raw_midr(&self) -> HashSet<usize> {
        self.raw_midr.clone()
    }

    fn midr(&self) -> &Midr {
        self.midrs.iter().next().unwrap_or(&Midr::default())
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}

impl Cpu {
    fn detect_cores(midrs: &HashSet<Midr>) -> BTreeMap<CoreType, CpuCore> {
        let mut cores: BTreeMap<CoreType, CpuCore> = BTreeMap::new();

        for midr in midrs {
            let part = midr.part;
            let core_type = match part {
                0xD05 | 0xD13 | 0xD14 | 0xD15 | 0xD16 | 0xD17 => CoreType::Performance,
                0xD03 | 0xD04 => CoreType::Performance,
                0xD01 | 0xD02 | 0xD06 | 0xD07 | 0xD08 | 0xD09 | 0xD0A | 0xD0B => {
                    CoreType::Performance
                }
                0xD4A | 0xD4B | 0xD4C | 0xD4D | 0xD4E => CoreType::Efficiency,
                0xD4F => CoreType::Efficiency,
                0xD0C | 0xD0D | 0xD0E | 0xD0F | 0xD11 | 0xD12 => CoreType::Efficiency,
                0xA10 | 0xA11 | 0xA12 | 0xA13 | 0xA14 | 0xA15 | 0xA16 | 0xA17 => {
                    CoreType::Performance
                }
                0xA20 | 0xA21 | 0xA22 | 0xA23 | 0xA24 | 0xA25 | 0xA26 | 0xA27 => {
                    CoreType::Performance
                }
                0xA30 | 0xA31 | 0xA32 | 0xA33 | 0xA34 | 0xA35 | 0xA36 | 0xA37 => {
                    CoreType::Efficiency
                }
                0xA40 | 0xA41 | 0xA42 | 0xA43 | 0xA44 | 0xA45 | 0xA46 | 0xA47 => {
                    CoreType::Efficiency
                }
                0xA50 | 0xA51 | 0xA52 | 0xA53 | 0xA54 | 0xA55 | 0xA56 | 0xA57 => {
                    CoreType::Efficiency
                }
                0xA60 | 0xA61 | 0xA62 | 0xA63 | 0xA64 | 0xA65 | 0xA66 | 0xA67 => {
                    CoreType::Efficiency
                }
                0xA70 | 0xA71 | 0xA72 | 0xA73 | 0xA74 | 0xA75 | 0xA76 | 0xA77 => {
                    CoreType::Performance
                }
                0xA80 | 0xA81 | 0xA82 | 0xA83 | 0xA84 | 0xA85 | 0xA86 | 0xA87 => {
                    CoreType::Performance
                }
                0xAA0 | 0xAA1 | 0xAA2 | 0xAA3 | 0xAA4 | 0xAA5 | 0xAA6 | 0xAA7 => {
                    CoreType::Performance
                }
                0xAB0 | 0xAB1 | 0xAB2 | 0xAB3 | 0xAB4 | 0xAB5 | 0xAB6 | 0xAB7 => {
                    CoreType::Performance
                }
                0xAC0 | 0xAC1 | 0xAC2 | 0xAC3 | 0xAC4 | 0xAC5 | 0xAC6 | 0xAC7 => {
                    CoreType::Performance
                }
                _ => CoreType::Performance,
            };

            cores
                .entry(core_type)
                .and_modify(|c| c.count += 1)
                .or_insert(CpuCore {
                    kind: core_type,
                    name: None,
                    cache: None,
                    count: 1,
                });
        }

        cores
    }
}
