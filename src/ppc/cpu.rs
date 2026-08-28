//! Contains the Cpu struct for PowerPC.

use crate::common::cache::Cache;
#[cfg(target_os = "linux")]
use crate::common::get_proc_cpuinfo_data;
use crate::common::os::TOSData;
use crate::common::{CoreType, DataSource, Speed, TDetect, UNK};
use crate::ppc::micro_arch::{CpuArch, CpuCore, MicroArch};
use std::fs;
use std::path::Path;

/// PowerPC architecture-specific data.
#[derive(Debug, Default, PartialEq)]
pub struct PpcData {
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
    pub cpu_arch: CpuArch,
    pub clock_speed_source: DataSource,
}

pub type Cpu = crate::common::Cpu<PpcData, MicroArch>;

impl Cpu {
    fn detect_topology() -> (u32, u32) {
        #[cfg(target_os = "linux")]
        {
            let cpuinfo = get_proc_cpuinfo_data();
            let thread_count = cpuinfo.len().max(1) as u32;

            // Check sysfs for SMT thread siblings per core
            let path = "/sys/devices/system/cpu/cpu0/topology/thread_siblings_list";
            if let Ok(content) = fs::read_to_string(path) {
                let threads_per_core = crate::common::expand_cpu_list(&content).len().max(1) as u32;
                let core_count = (thread_count / threads_per_core).max(1);
                return (core_count, thread_count);
            }

            (thread_count, thread_count)
        }
        #[cfg(not(target_os = "linux"))]
        {
            (1, 1)
        }
    }

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

        if let Some(freq_hz) = crate::common::read_devicetree_u64(dt_root.join("clock-frequency")) {
            return Some(freq_hz / 1_000_000);
        }

        if let Some(freq_hz) =
            crate::common::read_devicetree_u64(dt_root.join("timebase-frequency"))
        {
            return Some(freq_hz / 1_000_000);
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
            if (line.starts_with("CPU max MHz") || line.starts_with("CPU MHz"))
                && let Some(value) = line.split(':').nth(1)
                && let Some(freq) = crate::common::parse_frequency_mhz(value)
            {
                return Some(freq);
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn detect_clock_speed_from_cpuinfo() -> Option<u64> {
        let cpuinfo = get_proc_cpuinfo_data();
        for map in &cpuinfo {
            if let Some(val) = map.get("cpu MHz").or_else(|| map.get("clock"))
                && let Some(freq) = crate::common::parse_frequency_mhz(val)
            {
                return Some(freq);
            }
        }

        None
    }

    #[cfg(not(target_os = "linux"))]
    fn detect_clock_speed_from_cpuinfo() -> Option<u64> {
        None
    }
}

impl TDetect for Cpu {
    fn detect() -> Self {
        let system = crate::common::OS::get_system_name();
        let pvr = super::get_pvr();
        let version = (pvr >> 16) as u16;
        let revision = (pvr & 0xFFFF) as u16;
        let cpu_arch = CpuArch::find(pvr);
        let (core_count, thread_count) = Self::detect_topology();
        let mut cache = Self::detect_cache();
        if let Some(c) = &mut cache {
            c.resolve_share_counts(core_count, thread_count, 1);
        }
        let (clock_speed, clock_speed_source) = Self::detect_clock_speed();
        let speed = clock_speed.map(|mhz| Speed {
            base: mhz as u32,
            boost: mhz as u32,
            measured: false,
        });

        let cores = vec![CpuCore {
            kind: CoreType::Performance,
            micro_arch: cpu_arch.micro_arch,
            name: if cpu_arch.marketing_name != UNK {
                Some(cpu_arch.marketing_name.to_string())
            } else {
                None
            },
            implementer: None,
            cache,
            speed,
            count: core_count,
            threads: thread_count,
        }];

        let extra = PpcData {
            pvr,
            version,
            revision,
            cpu_arch,
            clock_speed_source,
        };

        let vendor = String::from(if extra.cpu_arch.marketing_name != UNK {
            "IBM"
        } else {
            UNK
        });

        Self {
            system,
            vendor,
            model: extra.cpu_arch.marketing_name.to_string(),
            cores,
            features: std::collections::BTreeMap::new(),
            extra,
        }
    }
}
