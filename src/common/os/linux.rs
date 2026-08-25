#![cfg(target_os = "linux")]

use crate::common::{
    DataSource, OS, TDetect, TOSData, TopologyCount, TopologyTier, cleanup_soc_vendor,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::common::{Cache, CacheLevel, CacheType, Level1Cache};

#[cfg(any(arm_cpu, test))]
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

fn get_soc_cpuinfo() -> Option<String> {
    let cpuinfo = get_proc_cpuinfo_data();
    if let Some(last) = cpuinfo.last()
        && (!last.contains_key("processor"))
        && let Some(raw_soc) = last.get("Hardware")
    {
        return Some(String::from(raw_soc.trim()));
    }
    None
}

pub fn get_devicetree_compatible() -> Option<Vec<Vec<String>>> {
    if let Ok(raw) = std::fs::read_to_string("/proc/device-tree/compatible") {
        let res: Vec<_> = raw
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|p| -> Vec<_> {
                // Since Mac Model strings contain commas, we don't want to split on those
                if !(p.contains("Power") || p.contains("Mac")) {
                    p.split(",").map(String::from).collect()
                } else {
                    vec![String::from(p)]
                }
            })
            .collect();

        return Some(res);
    }

    None
}

pub fn format_compatible_pair(pair: Vec<String>) -> String {
    if pair.len() < 2 {
        return pair[0].clone();
    }

    let raw_vendor = pair[0].clone();
    let raw_model = pair[1].clone();

    let vendor = cleanup_soc_vendor(raw_vendor.as_str());

    let model = if raw_model
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        && raw_model.chars().any(|c| c.is_ascii_lowercase())
        && raw_model.chars().any(|c| c.is_ascii_digit())
    {
        raw_model.to_uppercase()
    } else {
        raw_model
    };

    format!("{vendor} {model}")
}

use super::{is_generic_value, is_known_hypervisor_vendor};

/// Read a DMI field from sysfs, trying both the virtual and class mount
/// points, and return its first NUL-delimited value trimmed.
fn get_dmi_field(field: &str) -> Option<String> {
    for root in ["/sys/devices/virtual/dmi/id", "/sys/class/dmi/id"] {
        let path = format!("{root}/{field}");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            let value = raw.split('\0').next().unwrap_or("").trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Read several DMI fields and join their non-generic values with a space,
/// e.g. combining `sys_vendor` + `product_name` into "QEMU Standard PC ...".
/// When `vendor_only` is set, only produce a result for known hypervisors so
/// that real hardware strings are not prefixed with their manufacturer.
fn get_combined_dmi(fields: &[&str], vendor_only: bool) -> Option<String> {
    let mut parts = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        if vendor_only && i == 0 {
            continue;
        }
        let value = get_dmi_field(field)?;
        if is_generic_value(&value) {
            return None;
        }
        parts.push(value);
    }

    if vendor_only && !parts.is_empty() {
        let vendor = get_dmi_field(fields[0])?;
        if !is_known_hypervisor_vendor(&vendor) {
            return None;
        }
        parts.insert(0, vendor);
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn get_raw_system_name() -> Option<String> {
    // Let's look for a few possibilities that may have the formatted device name,
    // or at least the easier to use system name
    let simple_paths: Vec<_> = vec![
        "/proc/device-tree/model",
        "/proc/device-tree/smbios/smbios/system/product",
    ];

    for path in simple_paths {
        if let Ok(raw) = std::fs::read_to_string(path) {
            let raw: Vec<_> = raw.split('\0').collect();
            let raw = raw.first();
            {
                let raw = raw?;
                let trimmed = raw.trim();

                if is_generic_value(trimmed) {
                    continue;
                }

                return Some(String::from(trimmed));
            }
        }
    }

    for field in ["product_family", "product_name"] {
        if let Some(name) = get_dmi_field(field)
            && !is_generic_value(&name)
        {
            return Some(name);
        }
    }

    // For hypervisors, fold the vendor in so we get useful names like
    // "QEMU Standard PC (i440FX + PIIX, 1996)" instead of generic DMI strings.
    if let Some(name) = get_combined_dmi(&["sys_vendor", "product_name"], true) {
        return Some(name);
    }

    // When the product is a placeholder but the board is specific (common on
    // white-box ASUS/Gigabyte systems), fall back to the board identity.
    if let Some(name) = get_combined_dmi(&["board_vendor", "board_name"], false) {
        return Some(name);
    }

    // If we see nothing in sysfs, check the device tree 'compatible' value
    if let Some(raw_pairs) = get_devicetree_compatible()
        && let Some(pair) = raw_pairs.first().cloned()
    {
        return Some(format_compatible_pair(pair));
    }

    None
}

fn get_soc_devicetree() -> Option<String> {
    if let Some(raw_pairs) = get_devicetree_compatible()
        && let Some(pair) = raw_pairs.last().cloned()
    {
        return Some(format_compatible_pair(pair));
    }

    None
}

pub fn get_proc_cpuinfo_data() -> Vec<HashMap<String, String>> {
    let content = match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .map(|section| {
            let mut map = HashMap::new();
            for line in section.lines() {
                if let Some((key, val)) = line.split_once(':') {
                    map.insert(key.trim().to_string(), val.trim().to_string());
                }
            }
            map
        })
        .collect()
}

impl TOSData for OS {
    fn get_soc() -> Option<String> {
        if let Some(soc) = get_soc_cpuinfo() {
            return Some(soc);
        }

        if let Some(soc) = get_soc_devicetree() {
            return Some(soc);
        }

        None
    }

    fn get_system_name() -> Option<String> {
        // Let's try /proc/cpuinfo first, as that will be formatted nicely
        if let Some(last) = get_proc_cpuinfo_data().last()
            && (!last.contains_key("processor"))
            && let Some(raw) = last.get("Model")
            && !is_generic_value(raw.trim())
        {
            return Some(String::from(raw.trim()));
        }

        let name = get_raw_system_name();

        if name.is_some() {
            return name;
        }

        None
    }

    fn get_socket_count() -> TopologyTier {
        // Fallback: /proc/cpuinfo unique physical ids
        let cpuinfo = get_proc_cpuinfo_data();
        if !cpuinfo.is_empty() {
            let mut entries = 0;
            let mut physical_ids = HashSet::new();
            let mut core_ids = HashSet::new();

            for cpu_map in cpuinfo {
                if let Some(id) = cpu_map.get("physical id") {
                    physical_ids.insert(id.trim().to_string());
                }

                if let Some(id) = cpu_map.get("core id") {
                    core_ids.insert(id.trim().to_string());
                }

                entries += 1;
            }

            // For the Pentium Pro, all the rules seem to be broken.
            // There might be multiple entries in /proc/cpuinfo, all with identical ids
            if physical_ids.len() == 1 && core_ids.len() == 1 && entries != 1 {
                TopologyTier::new(entries, DataSource::LinuxProcCpuinfo)
            } else {
                TopologyTier::new(physical_ids.len() as u32, DataSource::LinuxProcCpuinfo)
            }
        } else {
            TopologyTier::default()
        }
    }
}

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();

        let mut topo = TopologyCount {
            sockets,
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

    pub(crate) fn from_sys_fs() -> Option<Cache> {
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
    #[cfg(any(arm_cpu, test))]
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
            if let Some(&first_cpu) = cpus_in_group.first()
                && let Some(cache) = Self::read_cpu_cache(first_cpu) {
                    cache_map.insert(midr, cache);
                }
        }

        if cache_map.is_empty() {
            None
        } else {
            Some(cache_map)
        }
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
    use super::super::normalize_for_compare;
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

    #[test]
    fn test_is_generic_value_placeholders() {
        for value in [
            "To Be Filled By O.E.M.",
            "To Be Filled",
            "System Product Name",
            "System Name",
            "Product Name",
            "All Series",
            "Default string",
            "Not Specified",
            "Not Applicable",
            "Unknown",
            "Generic",
            "OEM",
            "O.E.M.",
        ] {
            assert!(is_generic_value(value), "{value:?} should be generic");
        }
    }

    #[test]
    fn test_is_generic_value_whitespace_and_case() {
        assert!(is_generic_value("  DEFAULT   STRING  "));
        assert!(is_generic_value("to be filled by o.e.m."));
        assert!(is_generic_value("\tSystem Product Name\n"));
        assert!(is_generic_value(""));
    }

    #[test]
    fn test_is_generic_value_real_names() {
        for value in [
            "ThinkPad X1 Carbon",
            "HP Spectre x360",
            "MacBookPro18,3",
            "Dell XPS 13 9310",
            "QEMU Standard PC (i440FX + PIIX, 1996)",
            "Orange Pi 5",
        ] {
            assert!(!is_generic_value(value), "{value:?} should be real");
        }
    }

    #[test]
    fn test_is_known_hypervisor_vendor() {
        for vendor in [
            "QEMU",
            "VMware, Inc.",
            "innotek GmbH",
            "Microsoft Corporation",
        ] {
            assert!(is_known_hypervisor_vendor(vendor));
        }
        for vendor in ["Dell Inc.", "ASUSTeK COMPUTER INC.", "LENOVO"] {
            assert!(!is_known_hypervisor_vendor(vendor));
        }
    }

    #[test]
    fn test_normalize_for_compare() {
        assert_eq!(
            normalize_for_compare("  Default   String "),
            "default string"
        );
        assert_eq!(normalize_for_compare("QEMU"), "qemu");
        assert_eq!(normalize_for_compare(""), "");
    }
}
