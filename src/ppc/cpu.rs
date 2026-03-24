//! Contains the Cpu struct for PowerPC.

use crate::TCpu;
use crate::common::cache::{Cache, CacheLevel, CacheType, Level1Cache};
use crate::ppc::micro_arch::CpuArch;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
    pub cpu_arch: CpuArch,
    pub cache: Option<crate::common::cache::Cache>,
    pub clock_speed: Option<u64>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::detect()
    }
}

impl Cpu {
    fn detect_cache(pvr: u32) -> Option<Cache> {
        // Try to get cache info from device tree first
        if let Some(cache) = Self::detect_cache_from_device_tree() {
            return Some(cache);
        }

        // Fallback to hardcoded values based on PVR
        Self::detect_cache_from_pvr(pvr)
    }

    fn detect_cache_from_device_tree() -> Option<Cache> {
        // Try to read cache information from device tree
        let dt_root = Path::new("/proc/device-tree");
        if !dt_root.exists() {
            return None;
        }

        // Try to read cache properties
        let mut cache = Cache::default();
        let mut found_cache = false;

        // Try to read L1 cache
        if let Ok(l1_size) = fs::read_to_string(dt_root.join("l1-cache-size")) {
            if let Ok(size) = l1_size.trim().parse::<u32>() {
                if let Ok(l1_assoc) = fs::read_to_string(dt_root.join("l1-cache-associativity")) {
                    if let Ok(assoc) = l1_assoc.trim().parse::<u32>() {
                        cache.l1 = Level1Cache::new_unified(size, assoc);
                        found_cache = true;
                    }
                }
            }
        }

        // Try to read L2 cache
        if let Ok(l2_size) = fs::read_to_string(dt_root.join("l2-cache-size")) {
            if let Ok(size) = l2_size.trim().parse::<u32>() {
                let mut l2_assoc = 0;
                if let Ok(assoc_str) = fs::read_to_string(dt_root.join("l2-cache-associativity")) {
                    if let Ok(assoc) = assoc_str.trim().parse::<u32>() {
                        l2_assoc = assoc;
                    }
                }
                cache.l2 = Some(CacheLevel::new(size, CacheType::Unified, l2_assoc, 0));
                found_cache = true;
            }
        }

        // Try to read L3 cache
        if let Ok(l3_size) = fs::read_to_string(dt_root.join("l3-cache-size")) {
            if let Ok(size) = l3_size.trim().parse::<u32>() {
                let mut l3_assoc = 0;
                if let Ok(assoc_str) = fs::read_to_string(dt_root.join("l3-cache-associativity")) {
                    if let Ok(assoc) = assoc_str.trim().parse::<u32>() {
                        l3_assoc = assoc;
                    }
                }
                cache.l3 = Some(CacheLevel::new(size, CacheType::Unified, l3_assoc, 0));
                found_cache = true;
            }
        }

        if found_cache { Some(cache) } else { None }
    }

    fn detect_cache_from_pvr(pvr: u32) -> Option<Cache> {
        // Hardcoded cache values based on known PowerPC implementations
        let version = (pvr >> 16) as u16;

        match version {
            // IBM/Motorola PowerPC 601
            0x0001 => Some(Cache {
                l1: Level1Cache::new_unified(32 * 1024, 8), // 32KB unified L1, 8-way
                l2: None,
                l3: None,
            }),

            // IBM/Motorola PowerPC 603/603e/603ev
            0x0003 | 0x0004 | 0x0006 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(16 * 1024, CacheType::Data, 4, 0), // 16KB data, 4-way
                    instruction: CacheLevel::new(16 * 1024, CacheType::Instruction, 4, 0), // 16KB instruction, 4-way
                },
                l2: None,
                l3: None,
            }),

            // IBM/Motorola PowerPC 604/604e/604r
            0x0007 | 0x0009 | 0x000A => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(16 * 1024, CacheType::Data, 4, 0), // 16KB data, 4-way
                    instruction: CacheLevel::new(16 * 1024, CacheType::Instruction, 4, 0), // 16KB instruction, 4-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            // IBM/Motorola PowerPC 620
            0x0013 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 0)), // 512KB unified L2, 8-way
                l3: None,
            }),

            // PowerPC 750 (G3) series
            0x0200..=0x0205 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            // PowerPC 7400 (G4) series
            0x0308 | 0x0309 | 0x030C | 0x030D | 0x0351..=0x0354 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            // PowerPC 970 / G5 series
            0x0039 | 0x003C | 0x0044 | 0x0045 | 0x0052 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(64 * 1024, CacheType::Data, 8, 0), // 64KB data, 8-way
                    instruction: CacheLevel::new(64 * 1024, CacheType::Instruction, 8, 0), // 64KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 0)), // 512KB unified L2, 8-way
                l3: None,
            }),

            // Apple G4 variants
            0x0033 | 0x8000..=0x8002 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            // Apple RISC Reduced (Titan/Apollo/Diana)
            0x8003..=0x8007 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            _ => None,
        }
    }

    fn detect_clock_speed() -> Option<u64> {
        // Try to get clock speed from device tree
        let dt_root = Path::new("/proc/device-tree");
        if dt_root.exists() {
            // Try clock-frequency property
            if let Ok(freq_str) = fs::read_to_string(dt_root.join("clock-frequency")) {
                if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                    return Some(freq_hz / 1_000_000); // Convert to MHz
                }
            }

            // Try timebase-frequency property (alternative on some systems)
            if let Ok(freq_str) = fs::read_to_string(dt_root.join("timebase-frequency")) {
                if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                    // Timebase frequency is often different from CPU clock
                    // This is just an approximation - real implementation would need to check CPU specific registers
                    return Some(freq_hz / 1_000_000); // Convert to MHz
                }
            }
        }

        None
    }
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let pvr = super::get_pvr();
        let version = (pvr >> 16) as u16;
        let revision = (pvr & 0xFFFF) as u16;
        let cpu_arch = CpuArch::find(pvr);
        let cache = Self::detect_cache(pvr);
        let clock_speed = Self::detect_clock_speed();

        Self {
            pvr,
            version,
            revision,
            cpu_arch,
            cache,
            clock_speed,
        }
    }

    fn debug(&self) {
        println!("{:#?}", self);
    }

    fn display_table(&self) {
        let label: fn(&str) -> String = |label| format!("{:>17}:{:1}", label, "");
        let simple_line = |l, v: &str| {
            let l = label(l);
            println!("{}{}", l, v);
            println!();
        };

        println!();
        simple_line("Model", self.cpu_arch.marketing_name.as_str());
        simple_line("Microarchitecture", &String::from(self.cpu_arch.micro_arch));
        simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            simple_line("Process", tech);
        }

        // Display clock speed if available
        if let Some(clock_mhz) = self.clock_speed {
            simple_line("Clock Speed", &format!("{} MHz", clock_mhz));
        }

        // Display cache information if available
        if let Some(ref cache) = self.cache {
            let cache_label = |l: &str| {
                let l = label(l);
                print!("{}{}", l, "");
            };

            let cache_sub_label = |l: &str| format!("{:>21}", l);

            println!();
            println!("{}", label("Cache"));

            // L1 Cache
            match &cache.l1 {
                Level1Cache::Unified(l1) => {
                    cache_label("L1:");
                    println!("Unified {:>4} KB", l1.size);
                }
                Level1Cache::Split { data, instruction } => {
                    cache_label("L1d:");
                    println!("{:>4} KB", data.size);
                    cache_label("L1i:");
                    println!("{:>4} KB", instruction.size);
                }
            }

            // L2 Cache
            if let Some(l2) = &cache.l2 {
                cache_label("L2:");
                println!("Unified {:>4} KB", l2.size);
            }

            // L3 Cache
            if let Some(l3) = &cache.l3 {
                cache_label("L3:");
                println!("Unified {:>4} KB", l3.size);
            }
        }

        crate::println!();
    }
}
