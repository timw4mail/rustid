use super::brand::Vendor;
use super::micro_arch::*;
use crate::TCpu;
use crate::common::cache::*;
use std::collections::BTreeMap;
use std::process::Command;

fn get_sysctl_map() -> BTreeMap<String, String> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
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
                || (key.starts_with("hw") && !key.contains("optional")))
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

#[derive(Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum CoreType {
    Super,
    #[default]
    Performance,
    Efficiency,
}

impl From<String> for CoreType {
    fn from(val: String) -> Self {
        match val.as_str() {
            "Super" => CoreType::Super,
            "Performance" => CoreType::Performance,
            "Efficiency" => CoreType::Efficiency,
            _ => CoreType::Performance,
        }
    }
}

impl From<CoreType> for String {
    fn from(val: CoreType) -> Self {
        let s = match val {
            CoreType::Super => "Super",
            CoreType::Performance => "Performance",
            CoreType::Efficiency => "Efficiency",
        };

        String::from(s)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct CpuCore {
    pub kind: CoreType,
    pub name: Option<String>,
    pub cache: Option<Cache>,
    pub count: usize,
}

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub cores: BTreeMap<CoreType, CpuCore>,
    pub raw: BTreeMap<String, String>,
}

impl Cpu {
    fn find_core_codename(midr: &Midr, kind: CoreType) -> Option<String> {
        let str = match (midr.part, kind) {
            // M1
            (0x008..=0x00B, CoreType::Performance) => "FireStorm",
            (0x008..=0x00B, CoreType::Efficiency) => "IceStorm",

            // M2
            (0x00C..=0x010, CoreType::Performance) => "Avalanche",
            (0x00C..=0x010, CoreType::Efficiency) => "Blizzard",

            // M3+, A18 Pro
            (0x101 | 0x011..=0x016, CoreType::Performance) => "Everest",
            (0x101 | 0x011..=0x016, CoreType::Efficiency) => "Sawtooth",

            (_, _) => UNK,
        };

        if str == UNK {
            None
        } else {
            Some(String::from(str))
        }
    }
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let mut cores: BTreeMap<CoreType, CpuCore> = BTreeMap::new();

        let raw_midr = get_synth_midr();
        let midr = Midr::new(raw_midr);
        let vendor = Vendor::from(midr.implementer);
        let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
        let values = get_sysctl_map();

        let perf_levels: usize = values.get("hw.nperflevels").unwrap().parse().unwrap();

        for i in 0..perf_levels {
            let kind_type = values.get(&format!("hw.perflevel{}.name", i));
            let kind = CoreType::from(kind_type.unwrap().clone());
            let mut cache = Cache::default();
            let mut l1 = Level1Cache::default_split();

            let cpus_per_l2: u32 = values
                .get(&format!("hw.perflevel{}.cpusperl2", i))
                .unwrap()
                .parse()
                .unwrap();
            let l1d_size: u32 = values
                .get(&format!("hw.perflevel{}.l1dcachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let l1i_size: u32 = values
                .get(&format!("hw.perflevel{}.l1icachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let l2_size: u32 = values
                .get(&format!("hw.perflevel{}.l2cachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let count: usize = values
                .get(&format!("hw.perflevel{}.physicalcpu", i))
                .unwrap()
                .parse()
                .unwrap();

            l1.set_data(l1d_size / 1024, 0);
            l1.set_data_share_count(1);
            l1.set_instruction(l1i_size / 1024, 0);
            l1.set_instruction_share_count(1);
            cache.l1 = l1;
            cache.l2 = Some(CacheLevel::new(
                l2_size / 1024,
                CacheType::Unified,
                0,
                cpus_per_l2,
            ));

            let name = Self::find_core_codename(&midr, kind);

            cores.insert(
                kind,
                CpuCore {
                    kind,
                    name,
                    cache: Some(cache),
                    count,
                },
            );
        }

        Self {
            model: values.get("machdep.cpu.brand_string").unwrap().to_string(),
            raw_midr,
            midr,
            vendor: vendor.into(),
            cpu_arch,
            cores,
            raw: values,
        }
    }
    fn debug(&self) {
        println!("{:#?}", self);
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

        println!();
        simple_line("Brand/Implementor", self.cpu_arch.implementer.into());
        simple_line("Model", &self.model);
        simple_line("Code Name", &String::from(self.cpu_arch.micro_arch));
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
        println!();
    }
}
