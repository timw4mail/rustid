//! Contains the Cpu struct for PowerPC.

use crate::common::cache::Cache;
#[cfg(target_os = "linux")]
use crate::common::get_proc_cpuinfo_data;
use crate::common::os::TOSData;
use crate::common::{CoreType, DataSource, Speed, TDetect, Topology, TopologyTier, UNK};
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
    fn detect_topology() -> (u32, u32, u32) {
        #[cfg(target_os = "linux")]
        {
            let sysfs_topo = crate::common::detect_sysfs_topology();
            let cpuinfo = get_proc_cpuinfo_data();
            let proc_count = cpuinfo
                .iter()
                .filter(|m| m.contains_key("processor"))
                .count() as u32;

            let thread_count = if proc_count > 0 {
                proc_count
            } else if sysfs_topo.threads > 0 {
                sysfs_topo.threads
            } else {
                1
            };

            let sockets = if sysfs_topo.sockets.count > 0 {
                sysfs_topo.sockets.count
            } else {
                1
            };

            // Check sysfs for SMT thread siblings per core
            let path = "/sys/devices/system/cpu/cpu0/topology/thread_siblings_list";
            if let Ok(content) = fs::read_to_string(path) {
                let threads_per_core = crate::common::expand_cpu_list(&content).len().max(1) as u32;
                let core_count = (thread_count / threads_per_core).max(1);
                return (sockets, core_count, thread_count);
            }

            (sockets, thread_count, thread_count)
        }
        #[cfg(not(target_os = "linux"))]
        {
            (1, 1, 1)
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
        // 1. Try /proc/cpuinfo first ("clock" or "cpu MHz")
        if let Some(speed) = Self::detect_clock_speed_from_cpuinfo() {
            return (Some(speed), DataSource::LinuxProcCpuinfo);
        }

        // 2. Try sysfs cpufreq
        if let Some(speed) = Self::detect_clock_speed_from_cpufreq() {
            return (Some(speed), DataSource::LinuxSysFs);
        }

        // 3. Try device tree CPU nodes (/proc/device-tree/cpus/*/clock-frequency)
        if let Some(speed) = Self::detect_clock_speed_from_device_tree() {
            return (Some(speed), DataSource::DeviceTree);
        }

        // 4. Try lscpu for clock speed
        if let Some(speed) = Self::detect_clock_speed_from_lscpu() {
            return (Some(speed), DataSource::Lscpu);
        }

        (None, DataSource::DefaultValue)
    }

    fn detect_clock_speed_from_cpufreq() -> Option<u64> {
        let paths = [
            "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq",
        ];

        for path in paths {
            if let Ok(content) = fs::read_to_string(path)
                && let Ok(khz) = content.trim().parse::<u64>()
            {
                let mhz = khz / 1000;
                if mhz > 0 {
                    return Some(mhz);
                }
            }
        }

        None
    }

    fn detect_clock_speed_from_device_tree() -> Option<u64> {
        let cpus_roots = [
            Path::new("/proc/device-tree/cpus"),
            Path::new("/sys/firmware/devicetree/base/cpus"),
        ];

        for cpus_dir in cpus_roots {
            if let Ok(entries) = fs::read_dir(cpus_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let clock_path = path.join("clock-frequency");
                    if let Some(freq_hz) = crate::common::read_devicetree_u64(&clock_path)
                        && freq_hz > 0
                    {
                        return Some(freq_hz / 1_000_000);
                    }
                }
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
            if let Some(val) = map.get("clock").or_else(|| map.get("cpu MHz"))
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
        let (socket_count, core_count, thread_count) = Self::detect_topology();
        let mut cache = Self::detect_cache();
        if let Some(c) = &mut cache {
            c.resolve_share_counts(core_count, thread_count, socket_count);
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

        let topology = Topology {
            sockets: TopologyTier::new(socket_count, DataSource::LinuxProcCpuinfo),
            cores: TopologyTier::new(core_count, DataSource::LinuxProcCpuinfo),
            threads: TopologyTier::new(thread_count, DataSource::LinuxProcCpuinfo),
            speed: speed.unwrap_or_default(),
            cache,
            ..Default::default()
        };

        Self {
            system,
            vendor,
            model: extra.cpu_arch.marketing_name.to_string(),
            topology,
            cores,
            features: std::collections::BTreeMap::new(),
            extra,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_ppc_clock_speed() {
        assert_eq!(
            crate::common::parse_frequency_mhz("1250.000000MHz"),
            Some(1250)
        );
        assert_eq!(
            crate::common::parse_frequency_mhz("166.666666MHz"),
            Some(166)
        );
        assert_eq!(crate::common::parse_frequency_mhz("1.25GHz"), Some(1250));
        assert_eq!(crate::common::parse_frequency_mhz("1.25 GHz"), Some(1250));
        assert_eq!(crate::common::parse_frequency_mhz("1.42 GHz"), Some(1420));
        assert_eq!(crate::common::parse_frequency_mhz("800 MHz"), Some(800));
        assert_eq!(crate::common::parse_frequency_mhz("1600.00"), Some(1600));
    }

    #[test]
    fn test_ppc_cpuinfo_processor_filter() {
        let cpuinfo_sample = "processor\t: 0\ncpu\t\t: 7447/7457\nclock\t\t: 1250.000000MHz\nrevision\t: 1.1 (pvr 8002 0101)\n\nplatform\t: PowerBook\nmodel\t\t: PowerBook5,2\nmachine\t\t: PowerBook5,2\n";
        let sections: Vec<std::collections::HashMap<String, String>> = cpuinfo_sample
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .map(|section| {
                let mut map = std::collections::HashMap::new();
                for line in section.lines() {
                    if let Some((key, val)) = line.split_once(':') {
                        map.insert(key.trim().to_string(), val.trim().to_string());
                    }
                }
                map
            })
            .collect();

        // 2 sections total: 1 processor section, 1 platform/system section
        assert_eq!(sections.len(), 2);

        // Processor section count should be exactly 1
        let proc_count = sections
            .iter()
            .filter(|m| m.contains_key("processor"))
            .count();
        assert_eq!(proc_count, 1);

        // Clock speed from processor section
        let clock = sections[0]
            .get("clock")
            .and_then(|v| crate::common::parse_frequency_mhz(v));
        assert_eq!(clock, Some(1250));
    }
}
