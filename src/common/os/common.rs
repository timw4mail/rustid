use alloc::string::String;

/// Normalize a string for comparison: trim and collapse runs of whitespace.
pub fn normalize_for_compare(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.trim().chars() {
        if c.is_whitespace() {
            if !last_was_space && !out.is_empty() {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c.to_ascii_lowercase());
            last_was_space = false;
        }
    }
    out
}

/// Returns true when a DMI / device-tree / SMBIOS value is a firmware placeholder or
/// other generic string that does not identify the actual hardware.
pub fn is_generic_value(raw: &str) -> bool {
    const GENERIC: &[&str] = &[
        "to be filled by o.e.m.",
        "to be filled",
        "not specified by o.e.m.",
        "system product name",
        "system name",
        "product name",
        "all series",
        "default string",
        "not specified",
        "not applicable",
        "unknown",
        "none",
        "generic",
        "oem",
        "o.e.m.",
        "i386",
        "x86",
        "x64",
        "laptop",
    ];

    let normalized = normalize_for_compare(raw);
    normalized.is_empty() || GENERIC.contains(&normalized.as_str())
}

/// Vendor strings that identify a hypervisor rather than a physical machine.
pub fn is_known_hypervisor_vendor(vendor: &str) -> bool {
    let vendor = normalize_for_compare(vendor);
    const HYPERVISORS: &[&str] = &[
        "qemu",
        "bochs",
        "kvm",
        "vmware",
        "vmware, inc.",
        "innotek gmbh",
        "microsoft corporation",
        "xen",
        "parallels",
        "parallels software international inc.",
        "openstack foundation",
        "amazon ec2",
        "google",
    ];
    HYPERVISORS.contains(&vendor.as_str())
}

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
pub fn expand_cpu_list(s: &str) -> alloc::vec::Vec<u32> {
    let mut cpus = alloc::vec::Vec::new();
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

#[cfg(std_os)]
pub fn format_compatible_pair(pair: alloc::vec::Vec<String>) -> String {
    if pair.len() < 2 {
        return pair[0].clone();
    }

    let raw_vendor = pair[0].clone();
    let raw_model = pair[1].clone();

    let vendor = crate::common::cleanup_soc_vendor(raw_vendor.as_str());

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

    alloc::format!("{vendor} {model}")
}

#[cfg(std_os)]
pub fn get_devicetree_compatible() -> Option<alloc::vec::Vec<alloc::vec::Vec<String>>> {
    if let Ok(raw) = std::fs::read_to_string("/proc/device-tree/compatible") {
        let res: alloc::vec::Vec<_> = raw
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|p| -> alloc::vec::Vec<_> {
                // Since Mac Model strings contain commas, we don't want to split on those
                if !(p.contains("Power") || p.contains("Mac")) {
                    p.split(',').map(String::from).collect()
                } else {
                    alloc::vec![String::from(p)]
                }
            })
            .collect();

        return Some(res);
    }

    None
}

#[cfg(std_os)]
pub fn get_proc_cpuinfo_data() -> std::vec::Vec<std::collections::HashMap<String, String>> {
    let content = match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(c) => c,
        Err(_) => return std::vec::Vec::new(),
    };

    content
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
        .collect()
}

/// Parse a frequency string (e.g. "3.2 GHz", "800 MHz", "2400.00", "1.5GHz") into MHz as u64.
pub fn parse_frequency_mhz(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let is_ghz = value.ends_with("GHz") || value.ends_with("ghz") || value.ends_with("Ghz");
    let clean = value
        .trim_end_matches("MHz")
        .trim_end_matches("mhz")
        .trim_end_matches("Mhz")
        .trim_end_matches("GHz")
        .trim_end_matches("ghz")
        .trim_end_matches("Ghz")
        .trim();

    if let Some((whole, frac)) = clean.split_once('.') {
        let whole_val: u64 = whole.trim().parse().ok()?;
        if is_ghz {
            let frac = frac.trim();
            let mut frac_mhz = 0u64;
            if !frac.is_empty() {
                let frac_digits = &frac[..frac.len().min(3)];
                let frac_num: u64 = frac_digits.parse().ok()?;
                let mult = match frac_digits.len() {
                    1 => 100,
                    2 => 10,
                    _ => 1,
                };
                frac_mhz = frac_num * mult;
            }
            Some(whole_val * 1000 + frac_mhz)
        } else {
            Some(whole_val)
        }
    } else if let Ok(val) = clean.parse::<u64>() {
        if is_ghz { Some(val * 1000) } else { Some(val) }
    } else {
        None
    }
}

/// Reads a NUL-terminated or trimmed string property from device-tree (e.g. /proc/device-tree/model).
#[cfg(std_os)]
pub fn read_devicetree_string(path: impl AsRef<std::path::Path>) -> Option<String> {
    if let Ok(raw) = std::fs::read_to_string(path) {
        let first = raw.split('\0').next()?.trim();
        if !first.is_empty() && !is_generic_value(first) {
            return Some(first.to_string());
        }
    }
    None
}

/// Reads a big-endian 32-bit or 64-bit integer (or fallback ASCII text) from device-tree.
#[cfg(std_os)]
pub fn read_devicetree_u64(path: impl AsRef<std::path::Path>) -> Option<u64> {
    let p = path.as_ref();
    if let Ok(raw_bytes) = std::fs::read(p) {
        if raw_bytes.len() == 4 {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&raw_bytes);
            return Some(u32::from_be_bytes(arr) as u64);
        } else if raw_bytes.len() == 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&raw_bytes);
            return Some(u64::from_be_bytes(arr));
        }
    }
    if let Ok(s) = std::fs::read_to_string(p)
        && let Ok(val) = s.trim().parse::<u64>()
    {
        return Some(val);
    }
    None
}

/// Executes a closure on each available logical processor using `core_affinity`.
#[cfg(all(std_os, not(target_arch = "arm")))]
pub fn for_each_logical_core<F: FnMut()>(mut f: F) {
    if let Some(core_ids) = core_affinity::get_core_ids() {
        for core_id in core_ids {
            core_affinity::set_for_current(core_id);
            f();
        }
    } else {
        f();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_list_count() {
        assert_eq!(parse_cpu_list_count("0-7"), 8);
        assert_eq!(parse_cpu_list_count("0-3,4-7"), 8);
        assert_eq!(parse_cpu_list_count("0"), 1);
        assert_eq!(parse_cpu_list_count(""), 0);
    }

    #[test]
    fn test_expand_cpu_list() {
        assert_eq!(expand_cpu_list("0-3"), alloc::vec![0, 1, 2, 3]);
        assert_eq!(expand_cpu_list("0,4,7"), alloc::vec![0, 4, 7]);
    }

    #[test]
    #[cfg(std_os)]
    fn test_format_compatible_pair() {
        let pair = alloc::vec!["qcom".to_string(), "sm8450".to_string()];
        assert_eq!(format_compatible_pair(pair), "Qualcomm SM8450");

        let single = alloc::vec!["Apple".to_string()];
        assert_eq!(format_compatible_pair(single), "Apple");
    }

    #[test]
    fn test_parse_frequency_mhz() {
        assert_eq!(parse_frequency_mhz("800 MHz"), Some(800));
        assert_eq!(parse_frequency_mhz("800MHz"), Some(800));
        assert_eq!(parse_frequency_mhz("3.2 GHz"), Some(3200));
        assert_eq!(parse_frequency_mhz("3.20 GHz"), Some(3200));
        assert_eq!(parse_frequency_mhz("2.49 GHz"), Some(2490));
        assert_eq!(parse_frequency_mhz("2400.00"), Some(2400));
        assert_eq!(parse_frequency_mhz("1500"), Some(1500));
        assert_eq!(parse_frequency_mhz(""), None);
    }
}
