#![cfg(target_os = "linux")]

use crate::common::{
    DataSource, OS, TOSData, TopologyTier, format_compatible_pair, get_devicetree_compatible,
    get_proc_cpuinfo_data, get_soc_from_devicetree, get_soc_from_proc_cpuinfo,
    get_system_name_from_proc_cpuinfo, read_devicetree_string,
};
use std::collections::HashSet;

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
    for path in [
        "/proc/device-tree/model",
        "/proc/device-tree/smbios/smbios/system/product",
    ] {
        if let Some(name) = read_devicetree_string(path) {
            return Some(name);
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

impl TOSData for OS {
    fn get_soc() -> Option<String> {
        if let Some(soc) = get_soc_from_proc_cpuinfo() {
            return Some(soc);
        }

        if let Some(soc) = get_soc_from_devicetree() {
            return Some(soc);
        }

        None
    }

    fn get_system_name() -> Option<String> {
        if let Some(name) = get_system_name_from_proc_cpuinfo() {
            return Some(name);
        }

        get_raw_system_name()
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

#[cfg(test)]
mod tests {
    use super::super::normalize_for_compare;
    use super::*;
    use crate::common::{expand_cpu_list, parse_cpu_list_count};

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
