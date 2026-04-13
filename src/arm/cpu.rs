//! Contains the Cpu struct for ARM.
use super::CpuDisplay;
use super::brand::Vendor;
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
    pub cores: BTreeMap<(CoreType, Option<String>), CpuCore>,
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let mut raw_midr: HashSet<usize> = HashSet::new();
        let mut midrs: HashSet<Midr> = HashSet::new();
        let mut all_midrs: Vec<Midr> = Vec::new();

        #[cfg(not(target_os = "macos"))]
        {
            if let Some(core_ids) = core_affinity::get_core_ids() {
                for core_id in core_ids {
                    core_affinity::set_for_current(core_id);
                    let midr_val = super::get_midr();
                    raw_midr.insert(midr_val);
                    let midr = Midr::new(midr_val);
                    midrs.insert(midr);
                    all_midrs.push(midr);
                }
            } else {
                let midr_val = super::get_midr();
                raw_midr.insert(midr_val);
                let midr = Midr::new(midr_val);
                midrs.insert(midr);
                all_midrs.push(midr);
            }

            // On Linux, if we only found one type of core, or only one core total,
            // the kernel might be emulating a uniform MIDR for MRS, or core_affinity might be limited.
            // Try to get more accurate info from /proc/cpuinfo or /sys
            #[cfg(target_os = "linux")]
            if midrs.len() == 1 || all_midrs.len() <= 1 {
                let linux_midrs = Self::detect_linux_midrs();
                if !linux_midrs.is_empty() {
                    all_midrs.clear();
                    midrs.clear();
                    raw_midr.clear();
                    for m_val in linux_midrs {
                        raw_midr.insert(m_val);
                        let midr = Midr::new(m_val);
                        midrs.insert(midr);
                        all_midrs.push(midr);
                    }
                }
            }

            // On Windows, MRS is also emulated and might return uniform MIDR.
            // Try to get more accurate info from the registry.
            #[cfg(target_os = "windows")]
            {
                let windows_midrs = super::get_windows_midrs();
                if !windows_midrs.is_empty() {
                    all_midrs.clear();
                    midrs.clear();
                    raw_midr.clear();
                    for m_val in windows_midrs {
                        raw_midr.insert(m_val);
                        let midr = Midr::new(m_val);
                        midrs.insert(midr);
                        all_midrs.push(midr);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let midr_val = super::get_midr();
            raw_midr.insert(midr_val);
            midrs.insert(Midr::new(midr_val));
            // macOS core count is handled in apple.rs, but we'll fill all_midrs for consistency
            all_midrs.push(Midr::new(midr_val));
        }

        let primary_midr = midrs.iter().next().copied().unwrap_or(Midr::default());
        let vendor = Vendor::from(primary_midr.implementer);
        let cpu_arch = CpuArch::find(
            primary_midr.implementer,
            primary_midr.part,
            primary_midr.variant,
        );

        let cores = Self::detect_cores(&all_midrs);

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
        if let Some(midr) = self.midr() {
            println!("Implementer: 0x{:X} ({})", midr.implementer, self.vendor());
            println!("Variant: 0x{:X}", midr.variant);
            println!("Part Number: 0x{:X}", midr.part);
            println!("Revision: 0x{:X}", midr.revision);
        }
        println!("{:#?}", self);
    }

    fn display_table(&self) {
        CpuDisplay::display(&self.cpu_arch, &self.cores);
    }
}

impl TArmCpu for Cpu {
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

impl Cpu {
    #[cfg(target_os = "linux")]
    fn detect_linux_midrs() -> Vec<usize> {
        let mut midrs = Vec::new();

        // 1. Try /sys/devices/system/cpu/cpu*/regs/identification/midr_el1
        let mut i = 0;
        loop {
            let path = format!(
                "/sys/devices/system/cpu/cpu{}/regs/identification/midr_el1",
                i
            );
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(midr) = usize::from_str_radix(content.trim().trim_start_matches("0x"), 16)
                {
                    midrs.push(midr);
                }
            } else {
                break;
            }
            i += 1;
        }

        if !midrs.is_empty() {
            return midrs;
        }

        // 2. Fallback to /proc/cpuinfo
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut impl_ = None;
            let mut var = None;
            let mut arch = None;
            let mut part = None;
            let mut rev = None;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("processor") {
                    if let (Some(i), Some(p)) = (impl_, part) {
                        let m = (i << 24)
                            | (var.unwrap_or(0) << 20)
                            | (arch.unwrap_or(0) << 16)
                            | (p << 4)
                            | rev.unwrap_or(0);
                        midrs.push(m);

                        impl_ = None;
                        var = None;
                        arch = None;
                        part = None;
                        rev = None;
                    }
                    continue;
                }

                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 2 {
                    continue;
                }
                let key = parts[0].trim();
                let val = parts[1].trim();
                let first_word = val.split_whitespace().next().unwrap_or("");

                match key {
                    "CPU implementer" => {
                        impl_ = usize::from_str_radix(first_word.trim_start_matches("0x"), 16).ok()
                    }
                    "CPU variant" => {
                        var = usize::from_str_radix(first_word.trim_start_matches("0x"), 16).ok()
                    }
                    "CPU architecture" => arch = first_word.parse().ok(),
                    "CPU part" => {
                        part = usize::from_str_radix(first_word.trim_start_matches("0x"), 16).ok()
                    }
                    "CPU revision" => rev = first_word.parse().ok(),
                    _ => {}
                }
            }
            // Handle last entry
            if let (Some(i), Some(p)) = (impl_, part) {
                let m = (i << 24)
                    | (var.unwrap_or(0) << 20)
                    | (arch.unwrap_or(0) << 16)
                    | (p << 4)
                    | rev.unwrap_or(0);
                midrs.push(m);
            }
        }

        midrs
    }

    fn detect_cores(midrs: &[Midr]) -> BTreeMap<(CoreType, Option<String>), CpuCore> {
        let mut cores: BTreeMap<(CoreType, Option<String>), CpuCore> = BTreeMap::new();

        for midr in midrs {
            let arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
            let core_type = arch.micro_arch.core_type();
            let core_name: String = arch.micro_arch.into();

            let name = if core_name != micro_arch::UNK {
                Some(core_name)
            } else {
                None
            };

            cores
                .entry((core_type, name.clone()))
                .and_modify(|c| c.count += 1)
                .or_insert(CpuCore {
                    kind: core_type,
                    name,
                    cache: None,
                    count: 1,
                });
        }

        cores
    }
}
