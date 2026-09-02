#![cfg(target_os = "linux")]

use crate::common::{
    DataSource, OS, SystemInfo, TOSData, TopologyTier, format_compatible_pair,
    get_devicetree_compatible, get_proc_cpuinfo_data, get_soc_from_devicetree,
    get_soc_from_proc_cpuinfo, get_system_name_from_proc_cpuinfo, read_devicetree_string,
};
use std::collections::HashSet;

use super::{is_generic_value, parse_apple_model};

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

fn get_raw_system_name() -> Option<SystemInfo> {
    for path in [
        "/proc/device-tree/model",
        "/proc/device-tree/smbios/smbios/system/product",
    ] {
        if let Some(name) = read_devicetree_string(path) {
            return Some(SystemInfo::from_model(
                name,
                DataSource::DeviceTree("/proc/device-tree/model"),
            ));
        }
    }

    // 1. Apple model identification
    for field in [
        "product_name",
        "bios_version",
        "product_family",
        "board_name",
    ] {
        if let Some(val) = get_dmi_field(field)
            && let Some(mac) = parse_apple_model(&val)
        {
            return Some(SystemInfo::new(
                Some("Apple Inc.".to_string()),
                DataSource::LinuxSysFs("/sys/class/dmi/id/sys_vendor"),
                Some(mac),
                DataSource::LinuxSysFs("/sys/class/dmi/id/product_name"),
            ));
        }
    }

    let prod = get_dmi_field("product_name");
    let family = get_dmi_field("product_family");

    if let Some(f) = family
        && !is_generic_value(&f)
    {
        let sys_vendor = get_dmi_field("sys_vendor");
        return Some(SystemInfo::new(
            sys_vendor,
            DataSource::LinuxSysFs("/sys/class/dmi/id/sys_vendor"),
            Some(f),
            DataSource::LinuxSysFs("/sys/class/dmi/id/product_family"),
        ));
    }

    if let Some(p) = prod
        && !is_generic_value(&p)
    {
        let sys_vendor = get_dmi_field("sys_vendor");
        return Some(SystemInfo::new(
            sys_vendor,
            DataSource::LinuxSysFs("/sys/class/dmi/id/sys_vendor"),
            Some(p),
            DataSource::LinuxSysFs("/sys/class/dmi/id/product_name"),
        ));
    }

    if let Some(board) = get_dmi_field("board_name")
        && !is_generic_value(&board)
    {
        let board_vendor = get_dmi_field("board_vendor");
        return Some(SystemInfo::new(
            board_vendor,
            DataSource::LinuxSysFs("/sys/class/dmi/id/board_vendor"),
            Some(board),
            DataSource::LinuxSysFs("/sys/class/dmi/id/board_name"),
        ));
    }

    // If we see nothing in sysfs, check the device tree 'compatible' value
    if let Some(raw_pairs) = get_devicetree_compatible()
        && let Some(pair) = raw_pairs.first().cloned()
    {
        return Some(SystemInfo::from_model(
            format_compatible_pair(pair),
            DataSource::DeviceTree("/proc/device-tree/compatible"),
        ));
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

    fn get_system_name() -> Option<SystemInfo> {
        if let Some(name) = get_system_name_from_proc_cpuinfo() {
            return Some(SystemInfo::from_model(name, DataSource::LinuxProcCpuinfo));
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
    use super::super::{is_known_hypervisor_vendor, normalize_for_compare};
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
