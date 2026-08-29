#![cfg(linux_os)]

use crate::common::{
    Cache, CacheLevel, CacheType, DataSource, Level1Cache, OS, TDetect, TOSData, TopologyCount,
    TopologyTier, expand_cpu_list, get_proc_cpuinfo_data, parse_cpu_list_count,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

/// Detects topology counts (sockets, threads and cores) from `/sys/devices/system/cpu`.
pub fn detect_sysfs_topology() -> TopologyCount {
    let mut topo = TopologyCount::default();

    let cpu_root = Path::new("/sys/devices/system/cpu");
    if cpu_root.exists()
        && let Ok(online) = fs::read_to_string(cpu_root.join("online"))
    {
        topo.threads = parse_cpu_list_count(&online);

        let cpus = expand_cpu_list(&online);
        let mut core_ids = HashSet::new();
        let mut package_ids = HashSet::new();
        for cpu_id in cpus {
            let topo_dir = cpu_root.join(format!("cpu{}", cpu_id)).join("topology");
            let core_id_path = topo_dir.join("core_id");
            if let Ok(id_str) = fs::read_to_string(&core_id_path) {
                core_ids.insert(id_str.trim().to_string());
            }
            let pkg_path = topo_dir.join("physical_package_id");
            if let Ok(id_str) = fs::read_to_string(&pkg_path) {
                package_ids.insert(id_str.trim().to_string());
            }
        }
        if !core_ids.is_empty() {
            topo.cores = core_ids.len() as u32;
        }
        if !package_ids.is_empty() {
            topo.sockets = TopologyTier::new(package_ids.len() as u32, DataSource::LinuxSysFs);
        }
    }

    if topo.threads == 0 {
        let cpuinfo = get_proc_cpuinfo_data();
        let proc_count = cpuinfo
            .iter()
            .filter(|m| m.contains_key("processor"))
            .count() as u32;
        if proc_count > 0 {
            topo.threads = proc_count;
            topo.cores = proc_count;
        } else if let Ok(n) = std::thread::available_parallelism() {
            topo.threads = n.get() as u32;
            topo.cores = n.get() as u32;
        } else {
            topo.threads = 1;
            topo.cores = 1;
        }
    } else if topo.cores == 0 {
        topo.cores = topo.threads;
    }

    if topo.sockets.count == 0 {
        let cpuinfo = get_proc_cpuinfo_data();
        let mut physical_ids = HashSet::new();
        for cpu_map in &cpuinfo {
            if let Some(id) = cpu_map.get("physical id") {
                physical_ids.insert(id.trim().to_string());
            }
        }
        if !physical_ids.is_empty() {
            topo.sockets =
                TopologyTier::new(physical_ids.len() as u32, DataSource::LinuxProcCpuinfo);
        } else {
            topo.sockets = TopologyTier::new(1, DataSource::DefaultValue);
        }
    }

    topo
}

/// Read the full cache hierarchy for a single CPU from sysfs if accessible.
pub fn read_sysfs_cpu_cache(cpu_num: u32) -> Option<Cache> {
    let root = Path::new("/sys/devices/system/cpu")
        .join(format!("cpu{}", cpu_num))
        .join("cache");
    if !root.exists() {
        return None;
    }

    let mut cache = Cache {
        source: DataSource::LinuxSysFs,
        ..Default::default()
    };
    let mut found_cache = false;

    let dir = fs::read_dir(&root).ok()?;
    for entry in dir {
        let entry = entry.ok()?;
        let path = entry.path();
        let dir_name = entry.file_name();
        let dir_name = dir_name.to_str()?;
        if !dir_name.starts_with("index") {
            continue;
        }

        let level_str = fs::read_to_string(path.join("level")).ok()?;
        let level: u32 = level_str.trim().parse().ok()?;

        let type_str = fs::read_to_string(path.join("type")).ok()?;
        let cache_type = match type_str.trim() {
            "Data" => CacheType::Data,
            "Instruction" => CacheType::Instruction,
            "Unified" => CacheType::Unified,
            _ => continue,
        };

        let size_str = fs::read_to_string(path.join("size")).ok()?;
        let size_str = size_str.trim().trim_end_matches('K');
        let size_kb: u32 = size_str.parse().ok()?;
        let size_bytes = size_kb * 1024;

        let assoc_str = fs::read_to_string(path.join("ways_of_associativity")).ok()?;
        let assoc: u32 = assoc_str.trim().parse().unwrap_or(0);

        let share_count = if let Ok(shared_str) = fs::read_to_string(path.join("shared_cpu_list")) {
            parse_cpu_list_count(shared_str.trim())
        } else {
            0
        };

        match level {
            1 => match cache_type {
                CacheType::Unified => {
                    cache.l1 = Level1Cache::Unified(CacheLevel::new(
                        size_bytes,
                        cache_type,
                        assoc,
                        share_count,
                    ));
                    found_cache = true;
                }
                CacheType::Data => {
                    match &mut cache.l1 {
                        Level1Cache::Split { data, .. } => {
                            *data = CacheLevel::new(size_bytes, cache_type, assoc, share_count);
                        }
                        _ => {
                            cache.l1 = Level1Cache::Split {
                                data: CacheLevel::new(size_bytes, cache_type, assoc, share_count),
                                instruction: CacheLevel::default(),
                            };
                        }
                    }
                    found_cache = true;
                }
                CacheType::Instruction => {
                    match &mut cache.l1 {
                        Level1Cache::Split { instruction, .. } => {
                            *instruction =
                                CacheLevel::new(size_bytes, cache_type, assoc, share_count);
                        }
                        _ => {
                            cache.l1 = Level1Cache::Split {
                                data: CacheLevel::default(),
                                instruction: CacheLevel::new(
                                    size_bytes,
                                    cache_type,
                                    assoc,
                                    share_count,
                                ),
                            };
                        }
                    }
                    found_cache = true;
                }
                _ => {}
            },
            2 => {
                cache.l2 = Some(CacheLevel::new(size_bytes, cache_type, assoc, share_count));
                found_cache = true;
            }
            3 => {
                cache.l3 = Some(CacheLevel::new(size_bytes, cache_type, assoc, share_count));
                found_cache = true;
            }
            _ => {}
        }
    }

    if found_cache { Some(cache) } else { None }
}

/// Read cache info for each distinct CPU type (MIDR group) on heterogeneous ARM systems.
pub fn read_sysfs_cache_per_type() -> Option<BTreeMap<usize, Cache>> {
    let cpu_root = Path::new("/sys/devices/system/cpu");
    if !cpu_root.exists() {
        return None;
    }

    let online = fs::read_to_string(cpu_root.join("online")).ok()?;
    let cpus = expand_cpu_list(&online);
    if cpus.is_empty() {
        return None;
    }

    // Read MIDRs for all online CPUs, group by value
    let mut midr_map: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for &cpu_id in &cpus {
        let midr_path = cpu_root
            .join(format!("cpu{}", cpu_id))
            .join("regs/identification/midr_el1");
        if let Ok(content) = fs::read_to_string(&midr_path) {
            if let Ok(midr) = usize::from_str_radix(content.trim().trim_start_matches("0x"), 16) {
                midr_map.entry(midr).or_default().push(cpu_id);
            }
        } else {
            return None;
        }
    }

    // Read cache config from first CPU of each MIDR group
    let mut cache_map: BTreeMap<usize, Cache> = BTreeMap::new();
    for (&midr, cpus_in_group) in &midr_map {
        if let Some(&first_cpu) = cpus_in_group.first()
            && let Some(cache) = read_sysfs_cpu_cache(first_cpu)
        {
            cache_map.insert(midr, cache);
        }
    }

    if cache_map.is_empty() {
        None
    } else {
        Some(cache_map)
    }
}

// ----------------------------------------------------------------------------
// TopologyCount Detection
// ----------------------------------------------------------------------------

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();
        let mut topo = detect_sysfs_topology();
        topo.sockets = sockets;
        topo
    }
}

// ----------------------------------------------------------------------------
// Cache Detection
// ----------------------------------------------------------------------------

#[cfg(any(not(x86_cpu), test))]
impl Cache {
    #[cfg(not(x86_cpu))]
    pub fn detect() -> Option<Cache> {
        if let Some(cache) = Self::from_sys_fs() {
            return Some(cache);
        }

        if let Some(cache) = Self::from_lscpu_command() {
            return Some(cache);
        }

        None
    }

    #[cfg(not(x86_cpu))]
    pub(crate) fn from_sys_fs() -> Option<Cache> {
        read_sysfs_cpu_cache(0)
    }

    /// Read cache info for each distinct CPU type (MIDR group).
    #[cfg(any(arm_cpu, test))]
    pub(crate) fn from_sys_fs_per_type() -> Option<BTreeMap<usize, Cache>> {
        read_sysfs_cache_per_type()
    }

    #[cfg(not(x86_cpu))]
    fn from_lscpu_command() -> Option<Cache> {
        let output = match std::process::Command::new("lscpu").arg("-C").output() {
            Ok(o) => o.stdout,
            Err(_) => return None,
        };

        let output_str = match String::from_utf8(output) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let mut cache = Cache {
            source: DataSource::Lscpu,
            ..Default::default()
        };
        let mut found_cache = false;

        let lines: Vec<&str> = output_str.lines().collect();

        // No output from lscpu -C
        if lines.len() < 2 {
            return None;
        }

        let table_keys: Vec<&str> = lines[0].split_whitespace().collect();

        // @TODO: Properly parse table to account for missing values
        for line in lines.into_iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() <= 3 {
                continue;
            }

            let name = parts[table_keys.iter().position(|&x| x == "NAME")?];
            let size_str = parts[table_keys.iter().position(|&x| x == "ONE-SIZE")?];
            let ways_str = parts[table_keys.iter().position(|&x| x == "WAYS")?];

            // Parse size (e.g., "32K", "256K", "4M")
            let size_bytes: u32 = if let Some(stripped) = size_str.strip_suffix('K') {
                stripped.parse::<u32>().ok()? * 1024
            } else if let Some(stripped) = size_str.strip_suffix('M') {
                stripped.parse::<u32>().ok()? * 1024 * 1024
            } else {
                size_str.parse::<u32>().ok()? * 1024
            };

            let ways: u32 = ways_str.parse().unwrap_or(0);

            match name {
                "L1d" => {
                    cache.l1 = Level1Cache::Split {
                        data: CacheLevel::new(size_bytes, CacheType::Data, ways, 0),
                        instruction: CacheLevel::default(),
                    };
                    found_cache = true;
                }
                "L1i" => {
                    if let Level1Cache::Split { instruction, .. } = &mut cache.l1 {
                        instruction.size = size_bytes;
                        instruction.kind = CacheType::Instruction;
                        instruction.assoc = ways;
                    }
                }
                "L1" => {
                    cache.l1 = Level1Cache::Unified(CacheLevel::new_unified(size_bytes, ways));
                    found_cache = true;
                }
                "L2" => {
                    cache.l2 = Some(CacheLevel::new(size_bytes, CacheType::Unified, ways, 0));
                    found_cache = true;
                }
                "L3" => {
                    cache.l3 = Some(CacheLevel::new(size_bytes, CacheType::Unified, ways, 0));
                    found_cache = true;
                }
                _ => {}
            }
        }

        if let Level1Cache::Split { data, instruction } = &cache.l1
            && instruction.size == 0
            && data.size > 0
        {
            cache.l1 = Level1Cache::Split {
                data: *data,
                instruction: CacheLevel::new(data.size, CacheType::Instruction, data.assoc, 0),
            };
        }

        if found_cache { Some(cache) } else { None }
    }
}
