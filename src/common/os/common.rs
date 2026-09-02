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
    // Reject strings with ASCII control characters or non-printable garbage (e.g. "4\u{8}4\u{8}A\u{4}\u{5}")
    if raw.chars().any(|c| {
        ((c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r')
            || (c as u32) == 0x7F
            || c == '\u{FFFD}'
    }) {
        return true;
    }

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
        "apple - c1",
        "apple - c2",
        "apple - 1",
        "apple - 2",
        "- c1",
        "- c2",
        "c1",
        "c2",
        "4 4 a",
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

/// Returns true if the string matches an Apple hardware model identifier (e.g. "MacBook4,1", "Mac14,2", "MacPro7,1").
pub fn is_apple_model_name(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("MacBook")
        || s.starts_with("iMac")
        || s.starts_with("Macmini")
        || s.starts_with("MacPro")
        || s.starts_with("MacStudio")
        || s.starts_with("Mac")
        || s.starts_with("PowerMac")
        || s.starts_with("PowerBook")
        || s.starts_with("iBook")
        || s.starts_with("Xserve"))
        && s.contains(',')
}

/// Attempts to extract or normalize an Apple Mac model identifier (e.g. "MacBook4,1", "MacBookPro15,2")
/// from SMBIOS / BIOS strings (such as "MB41.88Z.00C1.B00.0802091544", "Mac-F4208CC8", or "MacBook4,1").
pub fn parse_apple_model(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if is_apple_model_name(raw) {
        return Some(String::from(raw));
    }

    // 1. Check Apple Board IDs (commonly reported by Windows SMBIOS Type 1 Product Name / Baseboard Product)
    const BOARD_IDS: &[(&str, &str)] = &[
        ("Mac-F4208CC8", "MacBook4,1"),
        ("Mac-F42D86C8", "MacBook5,1"),
        ("Mac-F42D88C8", "MacBook6,1"),
        ("Mac-F22C86C8", "MacBook7,1"),
        ("Mac-F42187C8", "MacBookPro3,1"),
        ("Mac-F42C86C8", "MacBookPro4,1"),
        ("Mac-F22587C8", "MacBookPro5,1"),
        ("Mac-F22587A1", "MacBookPro5,2"),
        ("Mac-F222BEC8", "MacBookPro5,3"),
        ("Mac-F22589C6", "MacBookPro5,4"),
        ("Mac-F2268DC8", "MacBookPro5,5"),
        ("Mac-F22586C8", "MacBookPro6,1"),
        ("Mac-F22589C8", "MacBookPro6,2"),
        ("Mac-F2268EC8", "MacBookPro7,1"),
        ("Mac-94245B3640C91C81", "MacBookPro8,1"),
        ("Mac-94245A3940C91C80", "MacBookPro8,2"),
        ("Mac-942459F5819B171B", "MacBookPro8,3"),
        ("Mac-C3EC7CD22292981F", "MacBookPro10,1"),
        ("Mac-AFD82502A00C3304", "MacBookPro10,2"),
        ("Mac-189A3D4F975D5FFC", "MacBookPro11,1"),
        ("Mac-2BD1B31983FE1663", "MacBookPro11,2"),
        ("Mac-3CBD00234E554E41", "MacBookPro11,3"),
        ("Mac-F42C88C8", "Macmini2,1"),
        ("Mac-F2208EC8", "Macmini4,1"),
        ("Mac-F42189C8", "Macmini3,1"),
        ("Mac-F4218EC8", "Macmini1,1"),
        ("Mac-F4208DC8", "MacBookAir1,1"),
        ("Mac-942452F5819B1C1B", "MacBookAir3,1"),
        ("Mac-94245A3640C91C81", "MacBookAir3,2"),
        ("Mac-C08A6BB70A942AC2", "MacBookAir4,1"),
        ("Mac-742912EFDBEE19B3", "MacBookAir4,2"),
        ("Mac-F4218FC8", "MacPro1,1"),
        ("Mac-F4208AC8", "MacPro2,1"),
        ("Mac-F221BEC8", "MacPro4,1"),
        ("Mac-F4218BC8", "iMac5,1"),
        ("Mac-F4228EC8", "iMac6,1"),
        ("Mac-F42388C8", "iMac7,1"),
        ("Mac-F226BEC8", "iMac9,1"),
        ("Mac-F2238BAE", "iMac10,1"),
        ("Mac-F2238AC8", "iMac11,1"),
        ("Mac-942B59F58194171B", "iMac12,1"),
        ("Mac-942B5BF58194151B", "iMac12,2"),
    ];

    for (board_id, model_name) in BOARD_IDS {
        if raw.eq_ignore_ascii_case(board_id) {
            return Some(String::from(*model_name));
        }
    }

    // 2. Check EFI BIOS Version tokens (e.g. "MB41.88Z...", "MBP31.88Z...")
    let token = raw
        .split(['.', ' ', '\t', '\0', '-'])
        .find(|s| !s.is_empty())
        .unwrap_or(raw);

    const PREFIXES: &[(&str, &str)] = &[
        ("MBP", "MacBookPro"),
        ("MBA", "MacBookAir"),
        ("MB", "MacBook"),
        ("IM", "iMac"),
        ("MM", "Macmini"),
        ("MP", "MacPro"),
        ("XS", "Xserve"),
    ];

    for (code, name) in PREFIXES {
        if let Some(rest) = token.strip_prefix(code)
            && rest.len() >= 2
            && rest.chars().all(|c| c.is_ascii_digit())
        {
            let major = &rest[..rest.len() - 1];
            let minor = &rest[rest.len() - 1..];
            return Some(alloc::format!("{name}{major},{minor}"));
        }
    }

    None
}

/// Combines vendor name and model/system name, prepending the vendor unless it is
/// already present in the model or the model is an Apple model identifier.
pub fn combine_vendor_and_model(vendor: Option<&str>, model: &str) -> String {
    let model = model.trim();
    if is_apple_model_name(model) {
        return String::from(model);
    }
    if let Some(v) = vendor {
        let v = v.trim();
        if !v.is_empty() && !is_generic_value(v) {
            let v_lower = v.to_ascii_lowercase();
            let m_lower = model.to_ascii_lowercase();
            if !m_lower.contains(&v_lower) {
                return alloc::format!("{v} {model}");
            }
        }
    }
    String::from(model)
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

/// Reads the `Hardware` SoC name from the trailing non-processor block of `/proc/cpuinfo`.
#[cfg(std_os)]
pub fn get_soc_from_proc_cpuinfo() -> Option<String> {
    let cpuinfo = get_proc_cpuinfo_data();
    if let Some(last) = cpuinfo.last()
        && (!last.contains_key("processor"))
        && let Some(raw_soc) = last.get("Hardware")
    {
        return Some(String::from(raw_soc.trim()));
    }
    None
}

/// Reads the `Model` system name from the trailing non-processor block of `/proc/cpuinfo`.
#[cfg(std_os)]
pub fn get_system_name_from_proc_cpuinfo() -> Option<String> {
    if let Some(last) = get_proc_cpuinfo_data().last()
        && (!last.contains_key("processor"))
        && let Some(raw) = last.get("Model")
        && !is_generic_value(raw.trim())
    {
        return Some(String::from(raw.trim()));
    }
    None
}

/// Resolves SoC identity from the last entry of `/proc/device-tree/compatible`.
#[cfg(std_os)]
pub fn get_soc_from_devicetree() -> Option<String> {
    if let Some(raw_pairs) = get_devicetree_compatible()
        && let Some(pair) = raw_pairs.last().cloned()
    {
        return Some(format_compatible_pair(pair));
    }
    None
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

    #[test]
    fn test_is_apple_model_name() {
        assert!(is_apple_model_name("MacBook4,1"));
        assert!(is_apple_model_name("MacBookPro18,3"));
        assert!(is_apple_model_name("Macmini9,1"));
        assert!(is_apple_model_name("MacPro7,1"));
        assert!(is_apple_model_name("MacStudio1,1"));
        assert!(is_apple_model_name("Mac14,2"));
        assert!(is_apple_model_name("PowerMac11,2"));
        assert!(is_apple_model_name("Xserve3,1"));
        assert!(!is_apple_model_name("MacBook"));
        assert!(!is_apple_model_name("ThinkPad T480"));
        assert!(!is_apple_model_name("CustomPC"));
    }

    #[test]
    fn test_combine_vendor_and_model() {
        // Apple models should remain unchanged
        assert_eq!(
            combine_vendor_and_model(Some("Apple Inc."), "MacBook4,1"),
            "MacBook4,1"
        );
        assert_eq!(
            combine_vendor_and_model(Some("Apple"), "Mac14,2"),
            "Mac14,2"
        );

        // Standard PC: vendor prepended
        assert_eq!(
            combine_vendor_and_model(Some("Dell Inc."), "Latitude 7490"),
            "Dell Inc. Latitude 7490"
        );
        assert_eq!(
            combine_vendor_and_model(Some("LENOVO"), "ThinkPad T480"),
            "LENOVO ThinkPad T480"
        );

        // Vendor already in model: not duplicated
        assert_eq!(
            combine_vendor_and_model(Some("Dell Inc."), "Dell Inc. Latitude 7490"),
            "Dell Inc. Latitude 7490"
        );
        assert_eq!(
            combine_vendor_and_model(Some("Lenovo"), "Lenovo ThinkPad T480"),
            "Lenovo ThinkPad T480"
        );

        // Motherboard vendor & board
        assert_eq!(
            combine_vendor_and_model(Some("ASUSTeK COMPUTER INC."), "ROG STRIX B550-F"),
            "ASUSTeK COMPUTER INC. ROG STRIX B550-F"
        );

        // Generic vendor or None
        assert_eq!(
            combine_vendor_and_model(Some("Default string"), "CustomPC"),
            "CustomPC"
        );
        assert_eq!(combine_vendor_and_model(None, "CustomPC"), "CustomPC");
    }

    #[test]
    fn test_parse_apple_model() {
        assert_eq!(
            parse_apple_model("MB41.88Z.00C1.B00.0802091544"),
            Some("MacBook4,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MBP31.88Z.0070.B00.0706281432"),
            Some("MacBookPro3,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MBA11.88Z.00BB.B00.0803171226"),
            Some("MacBookAir1,1".to_string())
        );
        assert_eq!(
            parse_apple_model("IM81.88Z.00C1.B00.0802091544"),
            Some("iMac8,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MM21.88Z.009A.B00.0706281359"),
            Some("Macmini2,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MP31.88Z.006C.B05.0802291410"),
            Some("MacPro3,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MacBook4,1"),
            Some("MacBook4,1".to_string())
        );
        assert_eq!(
            parse_apple_model("Mac-F4208CC8"),
            Some("MacBook4,1".to_string())
        );
        assert_eq!(parse_apple_model("APPLE  - c1"), None);
        assert_eq!(parse_apple_model("American Megatrends Inc."), None);
    }

    #[test]
    fn test_is_generic_value_control_chars_and_firmware_garbage() {
        assert!(is_generic_value("4\u{8}4\u{8}A\u{4}\u{5}"));
        assert!(is_generic_value("4 4 A  "));
        assert!(is_generic_value("4 4 a"));
        assert!(is_generic_value("test\x00garbage"));
        assert!(is_generic_value("bad\x1Fstring"));
    }
}
