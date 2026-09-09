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
