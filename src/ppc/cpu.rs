//! Contains the Cpu struct for PowerPC.

use crate::common::cache::Cache;
#[cfg(target_os = "linux")]
use crate::common::get_proc_cpuinfo_data;
use crate::common::{CliFlags, CpuDisplay, DataSource, TCpuDisplay, TDetect};
use crate::ppc::micro_arch::CpuArch;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    pub system: Option<String>,
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
    pub cpu_arch: CpuArch,
    pub cache: Option<Cache>,
    pub clock_speed: Option<u64>,
    pub clock_speed_source: DataSource,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::detect()
    }
}

impl Cpu {
    fn detect_cache() -> Option<Cache> {
        #[cfg(any(target_os = "linux", target_family = "unix"))]
        if let Some(cache) = Cache::detect() {
            return Some(cache);
        }

        // Let's just fall back to no cache, rather than
        // potentially returning incorrect values
        None
    }

    fn detect_clock_speed() -> (Option<u64>, DataSource) {
        // Try to get clock speed from device tree first
        if let Some(speed) = Self::detect_clock_speed_from_device_tree() {
            return (Some(speed), DataSource::DeviceTree);
        }

        // Try lscpu for clock speed
        if let Some(speed) = Self::detect_clock_speed_from_lscpu() {
            return (Some(speed), DataSource::Lscpu);
        }

        // Fallback to /proc/cpuinfo
        let speed = Self::detect_clock_speed_from_cpuinfo();
        let source = if speed.is_some() {
            DataSource::LinuxProcCpuinfo
        } else {
            DataSource::DefaultValue
        };
        (speed, source)
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
                if let Some(value) = line.split(':').nth(1) {
                    if let Some(freq) = Self::parse_mhz_value(value) {
                        return Some(freq);
                    }
                }
            }
        }

        None
    }

    fn detect_clock_speed_from_cpuinfo() -> Option<u64> {
        let cpuinfo = get_proc_cpuinfo_data();
        for map in &cpuinfo {
            if let Some(val) = map.get("cpu MHz").or_else(|| map.get("clock")) {
                if let Some(freq) = Self::parse_mhz_value(val) {
                    return Some(freq);
                }
            }
        }

        None
    }

    fn parse_mhz_value(value: &str) -> Option<u64> {
        let value = value.trim();
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

impl TDetect for Cpu {
    fn detect() -> Self {
        let pvr = super::get_pvr();
        let version = (pvr >> 16) as u16;
        let revision = (pvr & 0xFFFF) as u16;
        let cpu_arch = CpuArch::find(pvr);
        let cache = Self::detect_cache();
        let (clock_speed, clock_speed_source) = Self::detect_clock_speed();

        Self {
            pvr,
            version,
            revision,
            cpu_arch,
            cache,
            clock_speed,
            clock_speed_source,
        }
    }
}

impl TCpuDisplay for Cpu {
    fn debug(&self) {
        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        println!();

        let cpu = CpuDisplay { flags };

        if let Some(system) = self.system {
            cpu.simple_line("System", &cpu.format_system_name(system));
        }

        cpu.simple_line("Model", self.cpu_arch.marketing_name);
        cpu.simple_line("MicroArch", self.cpu_arch.micro_arch.into());
        cpu.simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        if let Some(clock_mhz) = self.clock_speed {
            println!(
                "{}{}",
                cpu.label("Frequency"),
                CpuDisplay::format_frequency(clock_mhz)
            );
            CpuDisplay::newline();
        }

        // TODO handle multiple cores/sockets
        let cc = |s| CpuDisplay::cache_count(s, 1);
        cpu.display_cache(self.cache, &cc, 0);

        println!();
    }
}
