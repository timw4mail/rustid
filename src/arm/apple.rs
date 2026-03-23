use super::brand::Vendor;
use super::micro_arch::*;
use crate::TCpu;
use crate::common::cache::*;
use std::collections::HashMap;
use std::process::Command;

fn get_sysctl_map() -> HashMap<String, String> {
    let mut values: HashMap<String, String> = HashMap::new();
    TryInto::<String>::try_into(
        Command::new("sysctl")
            .arg("-a")
            .output()
            .expect("Failed to load cpu details from sysctl")
            .stdout,
    )
    .unwrap()
    .split('\n')
    .filter(|l| !l.is_empty())
    .for_each(|x| {
        let line: Vec<_> = x.split(": ").collect();
        if let Some(key) = line.first()
            && let Some(val) = line.get(1)
            && (key.starts_with("machdep.cpu")
                || (key.starts_with("hw") && (key.contains("cpu") || key.contains("cache"))))
        {
            values.insert(String::from(*key), String::from(*val));
        }
    });

    values
}

pub fn get_synth_midr() -> usize {
    let values = get_sysctl_map();

    let cpufamily = if let Some(family) = values.get("hw.cpufamily") {
        family.parse::<u64>().ok()
    } else {
        None
    };

    let brand_string = values.get("machdep.cpu.brand_string");

    if let (Some(family), Some(brand)) = (cpufamily, brand_string) {
        cpufamily_to_midr(family, brand)
    } else {
        0
    }
}

fn cpufamily_to_midr(cpufamily: u64, brand_string: &str) -> usize {
    const APPLE_IMPLEMENTER: usize = 0x61;
    let midr_base = APPLE_IMPLEMENTER << 24;

    match cpufamily {
        // Apple A7 - Cyclone
        0x000C_0C0C_0C0E => midr_base | (0x001 << 4),
        // Apple A8 - Typhoon
        0x0000_1F2F_0E08 => midr_base | (0x002 << 4),
        // Apple A9 - Twister
        0x0000_0021_0C0A => midr_base | (0x003 << 4),
        // Apple A10 - Hurricane
        0x0000_0022_0C0A => midr_base | (0x004 << 4),
        // Apple A11 - Monsoon
        0x0000_0023_0C0A => midr_base | (0x005 << 4),
        // Apple A12 - Vortex
        0x0000_0024_0C0A => midr_base | (0x006 << 4),
        // Apple A13 - Lightning
        0x0000_0025_0C0A => midr_base | (0x007 << 4),

        // Apple M1 family (0x1b588bb3) - need brand string to distinguish variants
        0x0000_1B58_8BB3 => {
            if brand_string.contains("M1 Pro") || brand_string.contains("M1 Max") {
                midr_base | (0x009 << 4)
            } else if brand_string.contains("M1 Ultra") {
                midr_base | (0x00A << 4)
            } else {
                midr_base | (0x008 << 4) // M1 base
            }
        }

        // Apple A15 / M2 family (0xda33d83d) - Avalanche/Blizzard
        0x0000_DA33_D83D => {
            if brand_string.contains("M2 Pro") || brand_string.contains("M2 Max") {
                midr_base | (0x00B << 4)
            } else if brand_string.contains("M2 Ultra") {
                midr_base | (0x00C << 4)
            } else {
                midr_base | (0x00D << 4) // A15, M2 base
            }
        }

        // Apple A16 / M3 family (0x8765edea) - Everest/Sawtooth
        0x0000_8765_EDEA => {
            if brand_string.contains("M3 Pro") || brand_string.contains("M3 Max") {
                midr_base | (0x00E << 4)
            } else {
                midr_base | (0x00F << 4) // A16, M3 base
            }
        }

        // Apple A18 / A18 Pro (0x75D4ACB9)
        0x0000_75D4_ACB9 => {
            if brand_string.contains("A18 Pro") {
                midr_base | (0x101 << 4)
            } else {
                midr_base | (0x100 << 4) // A18
            }
        }

        // Apple M4 family
        0x0000_4B4F_AE0A => {
            if brand_string.contains("M4 Pro") || brand_string.contains("M4 Max") {
                midr_base | (0x011 << 4)
            } else if brand_string.contains("M4 Ultra") {
                midr_base | (0x012 << 4)
            } else {
                midr_base | (0x010 << 4) // M4 base
            }
        }

        _ => 0,
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum CoreType {
    Super,
    #[default]
    Performance,
    Efficiency,
}

#[derive(Debug, Default, PartialEq)]
pub struct CpuCore {
    pub kind: CoreType,
    pub name: Option<String>,
    pub cache: Option<Cache>,
    pub count: usize,
}

impl CpuCore {
    // fn detect() -> Self {
    //     CpuCore {
    //
    //     }
    // }
}

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub raw: HashMap<String, String>,
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let raw_midr = get_synth_midr();
        let midr = Midr::new(raw_midr);
        let vendor = Vendor::from(midr.implementer);
        let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
        let values = get_sysctl_map();

        Self {
            model: values.get("machdep.cpu.brand_string").unwrap().to_string(),
            raw_midr,
            midr,
            vendor: vendor.into(),
            cpu_arch,
            raw: values,
        }
    }
    fn debug(&self) {
        println!("{:#?}", self);
    }
    fn display_table(&self) {
        let label: fn(&str) -> String = |label| format!("{:>17}:{:1}", label, "");
        // let sublabel: fn(&str) -> String = |label| format!("{:>19}{}:{:1}", "", label, "");

        let simple_line = |l, v: &str| {
            let l = label(l);
            println!("{}{}", l, v);
            println!();
        };

        println!();
        simple_line("Brand/Implementor", self.cpu_arch.implementer.into());
        simple_line("Model", &self.model);
        simple_line("Microarchitecture", &String::from(self.cpu_arch.micro_arch));
        simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            simple_line("Process", tech);
        }
        println!();
    }
}
