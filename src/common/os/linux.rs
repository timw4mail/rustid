#![cfg(target_os = "linux")]

use crate::common::{TDetect, TopologyCount};
use std::fs;
use std::path::Path;

#[cfg(not(x86_cpu))]
use crate::common::{Cache, CacheLevel, CacheType, DataSource, Level1Cache};

#[cfg(arm_cpu)]
use std::collections::BTreeMap;

/// Parse a Linux CPU list string (e.g., "0-3", "0-3,8-11", "0") and return
/// the total number of CPUs it represents.
fn parse_cpu_list_count(s: &str) -> u32 {
    let mut count = 0;
    for part in s.trim().split(',') {
        let part = part.trim();
        if let Some(dash) = part.find('-') {
            if let (Ok(start), Ok(end)) =
                (part[..dash].parse::<u32>(), part[dash + 1..].parse::<u32>())
            {
                count += end.saturating_sub(start) + 1;
            }
        } else if part.parse::<u32>().is_ok() {
            count += 1;
        }
    }
    count
}

/// Expand a Linux CPU list string into a vector of individual CPU IDs.
fn expand_cpu_list(s: &str) -> Vec<u32> {
    let mut cpus = Vec::new();
    for part in s.trim().split(',') {
        let part = part.trim();
        if let Some(dash) = part.find('-') {
            if let (Ok(start), Ok(end)) =
                (part[..dash].parse::<u32>(), part[dash + 1..].parse::<u32>())
            {
                for cpu in start..=end {
                    cpus.push(cpu);
                }
            }
        } else if let Ok(cpu) = part.parse::<u32>() {
            cpus.push(cpu);
        }
    }
    cpus
}

impl TopologyCount {
    pub fn get_socket_count() -> u32 {
        use std::collections::HashSet;

        // Fallback: /proc/cpuinfo unique physical ids
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut entries = 0;
            let mut physical_ids = HashSet::new();
            let mut core_ids = HashSet::new();

            for line in content.lines() {
                if line.starts_with("physical id")
                    && let Some(id) = line.split(':').nth(1)
                {
                    physical_ids.insert(id.trim());
                    entries += 1;
                }

                if line.starts_with("core id")
                    && let Some(id) = line.split(':').nth(1)
                {
                    core_ids.insert(id.trim());
                }
            }

            // For the Pentium Pro, all the rules seem to be broken.
            // There might be multiple entries in /proc/cpuinfo, all with identical ids
            if physical_ids.len() == 1 && core_ids.len() == 1 && entries != 1 {
                entries
            } else {
                physical_ids.len() as u32
            }
        } else {
            1
        }
    }
}

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let mut topo = TopologyCount {
            sockets: Self::get_socket_count(),
            ..Default::default()
        };

        let cpu_root = Path::new("/sys/devices/system/cpu");
        if !cpu_root.exists() {
            return topo;
        }

        if let Ok(online) = fs::read_to_string(cpu_root.join("online")) {
            topo.threads = parse_cpu_list_count(&online);

            let cpus = expand_cpu_list(&online);
            let mut core_ids = std::collections::HashSet::new();
            for cpu_id in cpus {
                let core_id_path = cpu_root
                    .join(format!("cpu{}", cpu_id))
                    .join("topology")
                    .join("core_id");
                if let Ok(id_str) = fs::read_to_string(&core_id_path) {
                    core_ids.insert(id_str.trim().to_string());
                }
            }
            topo.cores = core_ids.len() as u32;
        }

        topo
    }
}

#[cfg(not(x86_cpu))]
impl Cache {
    pub fn detect() -> Option<Cache> {
        if let Some(cache) = Self::from_sys_fs() {
            return Some(cache);
        }

        if let Some(cache) = Self::from_lscpu_command() {
            return Some(cache);
        }

        None
    }

    fn from_sys_fs() -> Option<Cache> {
        Self::read_cpu_cache(0)
    }

    /// Read the full cache hierarchy for a single CPU from sysfs.
    fn read_cpu_cache(cpu_num: u32) -> Option<Cache> {
        let root = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", cpu_num))
            .join("cache");
        if !root.exists() {
            return None;
        }

        let mut cache = Cache::default();
        cache.source = DataSource::LinuxSysFs;
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

            let share_count =
                if let Ok(shared_str) = fs::read_to_string(path.join("shared_cpu_list")) {
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
                                    data: CacheLevel::new(
                                        size_bytes,
                                        CacheType::Data,
                                        assoc,
                                        share_count,
                                    ),
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
                                        CacheType::Instruction,
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

    /// Read cache info for each distinct CPU type (MIDR group).
    ///
    /// On heterogeneous ARM systems (big.LITTLE / DynamIQ), each core type may
    /// have a different cache hierarchy. This method reads per-CPU cache info
    /// from sysfs and returns a map keyed by MIDR value.
    ///
    /// Returns `None` if `midr_el1` is unavailable (non-ARM or older kernel).
    #[cfg(arm_cpu)]
    pub(crate) fn from_sys_fs_per_type() -> Option<BTreeMap<usize, Cache>> {
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
                if let Ok(midr) = usize::from_str_radix(content.trim().trim_start_matches("0x"), 16)
                {
                    midr_map.entry(midr).or_default().push(cpu_id);
                }
            } else {
                // No midr_el1 → not an ARM system, can't do per-type
                return None;
            }
        }

        // Read cache config from first CPU of each MIDR group
        let mut cache_map: BTreeMap<usize, Cache> = BTreeMap::new();
        for (&midr, cpus_in_group) in &midr_map {
            if let Some(&first_cpu) = cpus_in_group.first() {
                if let Some(cache) = Self::read_cpu_cache(first_cpu) {
                    cache_map.insert(midr, cache);
                }
            }
        }

        if cache_map.is_empty() {
            None
        } else {
            Some(cache_map)
        }
    }

    fn from_lscpu_command() -> Option<Cache> {
        let output = match std::process::Command::new("lscpu").arg("-C").output() {
            Ok(o) => o.stdout,
            Err(_) => return None,
        };

        let output_str = match String::from_utf8(output) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let mut cache = Cache::default();
        cache.source = DataSource::Lscpu;
        let mut found_cache = false;

        let lines: Vec<&str> = output_str.lines().collect();

        // No output from lscpu -C
        if lines.len() < 2 {
            return None;
        }

        let table_keys: Vec<&str> = lines[0].split_whitespace().collect();

        for line in lines.into_iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
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

        // Handle case where L1 is split but L1i wasn't in the output
        if let Level1Cache::Split { data, instruction } = &cache.l1
            && instruction.size == 0
            && data.size > 0
        {
            // Copy data settings to instruction
            cache.l1 = Level1Cache::Split {
                data: *data,
                instruction: CacheLevel::new(data.size, CacheType::Instruction, data.assoc, 0),
            };
        }

        if found_cache { Some(cache) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_list_count_single() {
        assert_eq!(parse_cpu_list_count("0"), 1);
        assert_eq!(parse_cpu_list_count("5"), 1);
    }

    #[test]
    fn test_parse_cpu_list_count_range() {
        assert_eq!(parse_cpu_list_count("0-3"), 4);
        assert_eq!(parse_cpu_list_count("4-7"), 4);
        assert_eq!(parse_cpu_list_count("0-0"), 1);
    }

    #[test]
    fn test_parse_cpu_list_count_mixed() {
        assert_eq!(parse_cpu_list_count("0-3,8-11"), 8);
        assert_eq!(parse_cpu_list_count("0,2,4"), 3);
        assert_eq!(parse_cpu_list_count("0-1,4,8-9"), 5);
    }

    #[test]
    fn test_parse_cpu_list_count_whitespace() {
        assert_eq!(parse_cpu_list_count(" 0-3, 8-11 "), 8);
    }

    #[test]
    fn test_parse_cpu_list_count_empty() {
        assert_eq!(parse_cpu_list_count(""), 0);
    }

    #[test]
    fn test_expand_cpu_list_single() {
        assert_eq!(expand_cpu_list("0"), vec![0]);
        assert_eq!(expand_cpu_list("5"), vec![5]);
    }

    #[test]
    fn test_expand_cpu_list_range() {
        assert_eq!(expand_cpu_list("0-3"), vec![0, 1, 2, 3]);
        assert_eq!(expand_cpu_list("4-7"), vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_expand_cpu_list_mixed() {
        assert_eq!(expand_cpu_list("0-3,8-11"), vec![0, 1, 2, 3, 8, 9, 10, 11]);
        assert_eq!(expand_cpu_list("0,2,4"), vec![0, 2, 4]);
    }

    #[test]
    fn test_expand_cpu_list_empty() {
        let empty: Vec<u32> = Vec::new();
        assert_eq!(expand_cpu_list(""), empty);
    }
}
