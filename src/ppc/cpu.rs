//! Contains the Cpu struct for PowerPC.

use crate::common::cache::{Cache, CacheLevel, CacheType, Level1Cache};
use crate::common::{CpuDisplay, TCpu};
use crate::ppc::micro_arch::CpuArch;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
    pub cpu_arch: CpuArch,
    pub cache: Option<Cache>,
    pub clock_speed: Option<u64>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::detect()
    }
}

impl Cpu {
    fn detect_cache(pvr: u32) -> Option<Cache> {
        #[cfg(any(target_os = "linux", target_family = "unix"))]
        if let Some(cache) = Cache::detect() {
            return Some(cache);
        }

        // Fallback to hardcoded values based on PVR
        Self::detect_cache_from_pvr(pvr)
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
            0x0308 | 0x0309 | 0x030C | 0x0351..=0x0354 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8, 0)), // 256KB unified L2, 8-way
                l3: None,
            }),

            // G4s with more cache
            0x030D => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32 * 1024, CacheType::Data, 8, 0), // 32KB data, 8-way
                    instruction: CacheLevel::new(32 * 1024, CacheType::Instruction, 8, 0), // 32KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 0)),
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

            // PowerPC 970 / G5 series
            0x0039 | 0x003C | 0x0044 | 0x0045 | 0x0052 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(64 * 1024, CacheType::Data, 8, 0), // 64KB data, 8-way
                    instruction: CacheLevel::new(64 * 1024, CacheType::Instruction, 8, 0), // 64KB instruction, 8-way
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 0)), // 512KB unified L2, 8-way
                l3: None,
            }),

            _ => None,
        }
    }

    fn detect_clock_speed() -> Option<u64> {
        // Try to get clock speed from device tree first
        if let Some(speed) = Self::detect_clock_speed_from_device_tree() {
            return Some(speed);
        }

        // Try lscpu for clock speed
        if let Some(speed) = Self::detect_clock_speed_from_lscpu() {
            return Some(speed);
        }

        // Fallback to /proc/cpuinfo
        Self::detect_clock_speed_from_cpuinfo()
    }

    fn detect_clock_speed_from_device_tree() -> Option<u64> {
        let dt_root = Path::new("/proc/device-tree");
        if !dt_root.exists() {
            return None;
        }

        if let Ok(freq_str) = fs::read_to_string(dt_root.join("clock-frequency")) {
            if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                return Some(freq_hz / 1_000_000);
            }
        }

        if let Ok(freq_str) = fs::read_to_string(dt_root.join("timebase-frequency")) {
            if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                return Some(freq_hz / 1_000_000);
            }
        }

        None
    }

    fn detect_clock_speed_from_lscpu() -> Option<u64> {
        let output = match std::process::Command::new("lscpu").output() {
            Ok(o) => o.stdout,
            Err(_) => return None,
        };

        let output_str = match String::from_utf8(output) {
            Ok(s) => s,
            Err(_) => return None,
        };

        for line in output_str.lines() {
            if line.starts_with("CPU max MHz") || line.starts_with("CPU MHz") {
                if let Some(freq) = Self::parse_mhz(line) {
                    return Some(freq);
                }
            }
        }

        None
    }

    fn detect_clock_speed_from_cpuinfo() -> Option<u64> {
        let output = match fs::read_to_string("/proc/cpuinfo") {
            Ok(o) => o,
            Err(_) => return None,
        };

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("cpu MHz") || line.starts_with("clock") {
                if let Some(freq) = Self::parse_mhz(line) {
                    return Some(freq as u64);
                }
            }
        }

        None
    }

    fn parse_mhz(line: &str) -> Option<u64> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1].trim();
        let value = value.trim_end_matches("MHz").trim().trim_end_matches("MHz");
        let value = value.trim_end_matches("GHz");

        if value.contains('.') {
            let parts: Vec<&str> = value.split('.').collect();
            if let Ok(mhz) = parts[0].parse::<u64>() {
                if value.ends_with("GHz") {
                    return Some(mhz * 1000);
                }
                return Some(mhz);
            }
        }

        value.parse::<u64>().ok()
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

    fn display_table(&self, color: bool) {
        println!();

        let cpu = CpuDisplay { color };

        cpu.simple_line("Model", self.cpu_arch.marketing_name);
        cpu.simple_line("MicroArch", self.cpu_arch.micro_arch.into());
        cpu.simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        if let Some(clock_mhz) = self.clock_speed {
            if clock_mhz >= 1000 {
                let whole = clock_mhz / 1000;
                let fract = (clock_mhz % 1000) / 10;
                println!("{}{}.{:02} GHz", cpu.label("Frequency"), whole, fract);
            } else {
                println!("{}{}.00 MHz", cpu.label("Frequency"), clock_mhz);
            }
            println!();
        }

        // TODO handle multiple cores/sockets
        cpu.display_cache(self.cache, 1);

        println!();
    }
}
