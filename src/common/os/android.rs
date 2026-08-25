#![cfg(target_os = "android")]

use crate::common::{
    DataSource, OS, TDetect, TOSData, TopologyCount, TopologyTier, cleanup_soc_vendor,
    is_generic_value,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::common::{Cache, CacheLevel, CacheType, Level1Cache};

#[cfg(any(arm_cpu, test))]
use std::collections::BTreeMap;

// ----------------------------------------------------------------------------
// Android System Properties (Text-parsing of getprop)
// ----------------------------------------------------------------------------

/// Runs `getprop` or `/system/bin/getprop` with optional property key argument.
pub fn run_getprop_cmd(key: Option<&str>) -> Option<String> {
    let mut cmd = std::process::Command::new("getprop");
    if let Some(k) = key {
        cmd.arg(k);
    }
    let output = cmd
        .output()
        .or_else(|_| {
            let mut fallback = std::process::Command::new("/system/bin/getprop");
            if let Some(k) = key {
                fallback.arg(k);
            }
            fallback.output()
        })
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

/// Parses the output of `getprop` into a `HashMap<String, String>`.
/// Expected format per line: `[key]: [value]`
pub fn parse_getprop_output(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('[') {
            if let Some((key, val_part)) = rest.split_once("]: [") {
                let val = val_part.strip_suffix(']').unwrap_or(val_part);
                map.insert(key.trim().to_string(), val.trim().to_string());
            }
        }
    }
    map
}

/// Retrieves all Android system properties by calling `getprop`.
pub fn get_props() -> HashMap<String, String> {
    if let Some(out) = run_getprop_cmd(None) {
        let map = parse_getprop_output(&out);
        if !map.is_empty() {
            return map;
        }
    }
    HashMap::new()
}

/// Retrieves a specific property from the given map, or falls back to `getprop <key>`.
pub fn get_property(props: &HashMap<String, String>, key: &str) -> Option<String> {
    if let Some(val) = props.get(key) {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(out) = run_getprop_cmd(Some(key)) {
        let trimmed = out.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    None
}

/// Extracts a friendly system/device name from Android system properties.
pub fn extract_system_name(props: &HashMap<String, String>) -> Option<String> {
    // 1. Market name (e.g., "Pixel 8 Pro", "Galaxy S24 Ultra")
    if let Some(market) = props.get("ro.product.marketname") {
        let trimmed = market.trim();
        if !trimmed.is_empty() && !is_generic_value(trimmed) {
            return Some(trimmed.to_string());
        }
    }

    // 2. Manufacturer / Brand + Model
    let model = props
        .get("ro.product.model")
        .or_else(|| props.get("ro.product.odm.model"))
        .or_else(|| props.get("ro.product.vendor.model"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !is_generic_value(s));

    let manufacturer = props
        .get("ro.product.manufacturer")
        .or_else(|| props.get("ro.product.brand"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !is_generic_value(s));

    if let (Some(mfg), Some(mdl)) = (manufacturer, model) {
        let mfg_clean = cleanup_soc_vendor(mfg);
        let mdl_lower = mdl.to_ascii_lowercase();
        let mfg_lower = mfg.to_ascii_lowercase();
        let mfg_clean_lower = mfg_clean.to_ascii_lowercase();

        if mdl_lower.starts_with(&mfg_lower) || mdl_lower.starts_with(&mfg_clean_lower) {
            return Some(mdl.to_string());
        } else {
            return Some(format!("{mfg_clean} {mdl}"));
        }
    }

    if let Some(mdl) = model {
        return Some(mdl.to_string());
    }

    // 3. Device or product code name
    if let Some(device) = props
        .get("ro.product.device")
        .or_else(|| props.get("ro.product.name"))
    {
        let trimmed = device.trim();
        if !trimmed.is_empty() && !is_generic_value(trimmed) {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Extracts SoC manufacturer and model name from Android system properties.
pub fn extract_soc(props: &HashMap<String, String>) -> Option<String> {
    // 1. ro.soc.manufacturer + ro.soc.model (Android 12+ / API 31+)
    let soc_mfg = props
        .get("ro.soc.manufacturer")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !is_generic_value(s));
    let soc_model = props
        .get("ro.soc.model")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !is_generic_value(s));

    if let (Some(mfg), Some(model)) = (soc_mfg, soc_model) {
        let mfg_clean = cleanup_soc_vendor(mfg);
        let model_lower = model.to_ascii_lowercase();
        let mfg_lower = mfg.to_ascii_lowercase();
        let mfg_clean_lower = mfg_clean.to_ascii_lowercase();

        if model_lower.starts_with(&mfg_lower) || model_lower.starts_with(&mfg_clean_lower) {
            return Some(model.to_string());
        } else {
            return Some(format!("{mfg_clean} {model}"));
        }
    }

    if let Some(model) = soc_model {
        return Some(model.to_string());
    }

    // 2. Vendor-specific chip name properties
    for key in ["ro.chipname", "ro.hardware.chipname", "ro.boot.hardware"] {
        if let Some(val) = props.get(key) {
            let trimmed = val.trim();
            if !trimmed.is_empty() && !is_generic_value(trimmed) {
                return Some(cleanup_soc_vendor(trimmed));
            }
        }
    }

    // 3. ro.board.platform (e.g. "taro", "lahaina", "sm8450", "exynos990", "mt6893")
    if let Some(platform) = props.get("ro.board.platform") {
        let trimmed = platform.trim();
        if !trimmed.is_empty() && !is_generic_value(trimmed) && !trimmed.eq_ignore_ascii_case("gki")
        {
            return Some(trimmed.to_string());
        }
    }

    // 4. ro.hardware (e.g. "qcom", "exynos", "tensor")
    if let Some(hw) = props.get("ro.hardware") {
        let trimmed = hw.trim();
        if !trimmed.is_empty() && !is_generic_value(trimmed) {
            return Some(cleanup_soc_vendor(trimmed));
        }
    }

    None
}

// ----------------------------------------------------------------------------
// Helpers for CPU Lists & /proc/cpuinfo
// ----------------------------------------------------------------------------

/// Parse a Linux/Android CPU list string (e.g., "0-3", "0-3,8-11", "0") and return
/// the total number of CPUs it represents.
pub fn parse_cpu_list_count(s: &str) -> u32 {
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

/// Expand a Linux/Android CPU list string into a vector of individual CPU IDs.
pub fn expand_cpu_list(s: &str) -> Vec<u32> {
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

fn get_devicetree_compatible() -> Option<Vec<Vec<String>>> {
    if let Ok(raw) = std::fs::read_to_string("/proc/device-tree/compatible") {
        let res: Vec<_> = raw
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|p| -> Vec<_> { p.split(',').map(String::from).collect() })
            .collect();

        return Some(res);
    }
    None
}

fn format_compatible_pair(pair: Vec<String>) -> String {
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

// ----------------------------------------------------------------------------
// TOSData Implementation
// ----------------------------------------------------------------------------

impl TOSData for OS {
    fn get_soc() -> Option<String> {
        let props = get_props();
        if let Some(soc) = extract_soc(&props) {
            return Some(soc);
        }

        if let Some(soc) = get_soc_cpuinfo() {
            return Some(soc);
        }

        if let Some(soc) = get_soc_devicetree() {
            return Some(soc);
        }

        None
    }

    fn get_system_name() -> Option<String> {
        let props = get_props();
        if let Some(name) = extract_system_name(&props) {
            return Some(name);
        }

        if let Some(last) = get_proc_cpuinfo_data().last()
            && (!last.contains_key("processor"))
            && let Some(raw) = last.get("Model")
            && !is_generic_value(raw.trim())
        {
            return Some(String::from(raw.trim()));
        }

        if let Ok(raw) = std::fs::read_to_string("/proc/device-tree/model") {
            let raw: Vec<_> = raw.split('\0').collect();
            if let Some(first) = raw.first() {
                let trimmed = first.trim();
                if !is_generic_value(trimmed) {
                    return Some(String::from(trimmed));
                }
            }
        }

        None
    }

    fn get_socket_count() -> TopologyTier {
        TopologyTier::new(1, DataSource::AndroidGetprop)
    }
}

// ----------------------------------------------------------------------------
// TopologyCount Detection
// ----------------------------------------------------------------------------

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();

        let mut topo = TopologyCount {
            sockets,
            ..Default::default()
        };

        let cpu_root = Path::new("/sys/devices/system/cpu");
        if cpu_root.exists() {
            if let Ok(online) = fs::read_to_string(cpu_root.join("online")) {
                topo.threads = parse_cpu_list_count(&online);

                let cpus = expand_cpu_list(&online);
                let mut core_ids = HashSet::new();
                for cpu_id in cpus {
                    let core_id_path = cpu_root
                        .join(format!("cpu{}", cpu_id))
                        .join("topology")
                        .join("core_id");
                    if let Ok(id_str) = fs::read_to_string(&core_id_path) {
                        core_ids.insert(id_str.trim().to_string());
                    }
                }
                if !core_ids.is_empty() {
                    topo.cores = core_ids.len() as u32;
                }
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

        topo
    }
}

// ----------------------------------------------------------------------------
// Cache Detection
// ----------------------------------------------------------------------------

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

    /// Read the full cache hierarchy for a single CPU from sysfs if accessible.
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

    /// Read cache info for each distinct CPU type (MIDR group) from sysfs.
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
                return None;
            }
        }

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
        if lines.len() < 2 {
            return None;
        }

        let table_keys: Vec<&str> = lines[0].split_whitespace().collect();

        for line in lines.into_iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() <= 3 {
                continue;
            }

            let name = parts[table_keys.iter().position(|&x| x == "NAME")?];
            let size_str = parts[table_keys.iter().position(|&x| x == "ONE-SIZE")?];
            let ways_str = parts[table_keys.iter().position(|&x| x == "WAYS")?];

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

// ----------------------------------------------------------------------------
// Unit Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const PIXEL_GETPROP: &str = r#"
[ro.board.platform]: [zuma]
[ro.boot.hardware]: [ripcurrent]
[ro.build.version.release]: [14]
[ro.build.version.sdk]: [34]
[ro.hardware]: [ripcurrent]
[ro.product.brand]: [google]
[ro.product.device]: [husky]
[ro.product.manufacturer]: [Google]
[ro.product.marketname]: [Pixel 8 Pro]
[ro.product.model]: [Pixel 8 Pro]
[ro.product.name]: [husky]
[ro.soc.manufacturer]: [Google]
[ro.soc.model]: [Tensor G3]
"#;

    const GALAXY_GETPROP: &str = r#"
[ro.board.platform]: [exynos2400]
[ro.chipname]: [exynos2400]
[ro.hardware]: [samsungexynos2400]
[ro.hardware.chipname]: [exynos2400]
[ro.product.brand]: [samsung]
[ro.product.device]: [e3q]
[ro.product.manufacturer]: [Samsung]
[ro.product.model]: [SM-S928B]
[ro.product.name]: [e3qxeea]
"#;

    const QUALCOMM_GETPROP: &str = r#"
[ro.board.platform]: [taro]
[ro.boot.hardware]: [qcom]
[ro.hardware]: [qcom]
[ro.hardware.chipname]: [SM8450]
[ro.product.brand]: [Xiaomi]
[ro.product.device]: [zeus]
[ro.product.manufacturer]: [Xiaomi]
[ro.product.marketname]: [Xiaomi 12 Pro]
[ro.product.model]: [2201122G]
[ro.soc.manufacturer]: [Qualcomm]
[ro.soc.model]: [Snapdragon 8 Gen 1]
"#;

    #[test]
    fn test_parse_getprop_output() {
        let props = parse_getprop_output(PIXEL_GETPROP);
        assert_eq!(
            props.get("ro.product.marketname").map(|s| s.as_str()),
            Some("Pixel 8 Pro")
        );
        assert_eq!(
            props.get("ro.soc.model").map(|s| s.as_str()),
            Some("Tensor G3")
        );
        assert_eq!(
            props.get("ro.soc.manufacturer").map(|s| s.as_str()),
            Some("Google")
        );
        assert_eq!(
            props.get("ro.product.manufacturer").map(|s| s.as_str()),
            Some("Google")
        );
    }

    #[test]
    fn test_extract_system_name_pixel() {
        let props = parse_getprop_output(PIXEL_GETPROP);
        assert_eq!(extract_system_name(&props), Some("Pixel 8 Pro".to_string()));
    }

    #[test]
    fn test_extract_system_name_galaxy() {
        let props = parse_getprop_output(GALAXY_GETPROP);
        assert_eq!(
            extract_system_name(&props),
            Some("Samsung SM-S928B".to_string())
        );
    }

    #[test]
    fn test_extract_system_name_xiaomi() {
        let props = parse_getprop_output(QUALCOMM_GETPROP);
        assert_eq!(
            extract_system_name(&props),
            Some("Xiaomi 12 Pro".to_string())
        );
    }

    #[test]
    fn test_extract_soc_pixel() {
        let props = parse_getprop_output(PIXEL_GETPROP);
        assert_eq!(extract_soc(&props), Some("Google Tensor G3".to_string()));
    }

    #[test]
    fn test_extract_soc_galaxy() {
        let props = parse_getprop_output(GALAXY_GETPROP);
        assert_eq!(extract_soc(&props), Some("Exynos2400".to_string()));
    }

    #[test]
    fn test_extract_soc_qualcomm() {
        let props = parse_getprop_output(QUALCOMM_GETPROP);
        assert_eq!(
            extract_soc(&props),
            Some("Qualcomm Snapdragon 8 Gen 1".to_string())
        );
    }

    #[test]
    fn test_parse_cpu_list_count() {
        assert_eq!(parse_cpu_list_count("0-7"), 8);
        assert_eq!(parse_cpu_list_count("0-3,4-7"), 8);
        assert_eq!(parse_cpu_list_count("0"), 1);
        assert_eq!(parse_cpu_list_count(""), 0);
    }

    #[test]
    fn test_expand_cpu_list() {
        assert_eq!(expand_cpu_list("0-3"), vec![0, 1, 2, 3]);
        assert_eq!(expand_cpu_list("0,4,7"), vec![0, 4, 7]);
    }
}
