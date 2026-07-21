//! Linux-specific RISC-V CPU detection.
//!
//! Uses `/proc/cpuinfo` for ISA string and vendor info, and sysfs for CSR values.

use super::OsCpuInfo;
use crate::common::get_proc_cpuinfo_data;
use crate::common::{CoreType, DataSource};
use crate::riscv::brand::Vendor;
use crate::riscv::micro_arch::*;
use std::collections::BTreeMap;

/// Linux-specific CPU detection via /proc/cpuinfo and sysfs.
pub fn detect() -> OsCpuInfo {
    let cpuinfo = get_proc_cpuinfo_data();
    let first = cpuinfo.first();

    let vendor_id = first
        .and_then(|m| m.get("mvendorid"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let model_name = first
        .and_then(|m| m.get("model name"))
        .or_else(|| first.and_then(|m| m.get("model")))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let isa_string = first
        .and_then(|m| m.get("isa"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Read CSR values from sysfs
    let mvendorid = read_sysfs_csr("mvendorid");
    let marchid = read_sysfs_csr("marchid");
    let mimpid = read_sysfs_csr("mimpid");
    let mconfigptr = read_sysfs_csr("mconfigptr");

    let vendor_name = if !vendor_id.is_empty() {
        let trimmed = vendor_id.trim().trim_start_matches("0x");
        let parsed = usize::from_str_radix(trimmed, 16).unwrap_or(mvendorid);
        let v: String = Vendor::from(parsed).into();
        v
    } else {
        let v: String = Vendor::from(mvendorid).into();
        v
    };

    let cpu_arch = CpuArch::find(mvendorid, marchid);

    let mut raw: BTreeMap<String, String> = BTreeMap::new();
    if let Some(first_map) = first {
        for (k, v) in first_map {
            raw.insert(k.clone(), v.clone());
        }
    }
    if mvendorid != 0 {
        raw.insert("mvendorid".to_string(), format!("0x{:x}", mvendorid));
    }
    if marchid != 0 {
        raw.insert("marchid".to_string(), format!("0x{:x}", marchid));
    }
    if mimpid != 0 {
        raw.insert("mimpid".to_string(), format!("0x{:x}", mimpid));
    }
    if mconfigptr != 0 {
        raw.insert("mconfigptr".to_string(), format!("0x{:x}", mconfigptr));
    }

    // Read device tree CPU properties
    if let Some(dt_compat) = read_dt_cpu_prop("compatible") {
        raw.insert("dt-compatible".to_string(), dt_compat);
    }
    if let Some(dt_isa) = read_dt_cpu_prop("riscv,isa") {
        raw.insert("dt-isa".to_string(), dt_isa);
    }
    if let Some(freq) = read_dt_cpu_freq() {
        raw.insert("clock-frequency".to_string(), freq);
    }

    // Count cores from cpuinfo entries
    let core_count = cpuinfo.len() as u32;
    let core_type = CoreType::Performance;
    let cores = if core_count > 0 {
        let mut map = BTreeMap::new();
        map.insert(
            core_type,
            CpuCore {
                kind: core_type,
                name: Some(String::from(cpu_arch.micro_arch)),
                cache: None,
                count: core_count,
            },
        );
        map
    } else {
        BTreeMap::new()
    };

    OsCpuInfo {
        vendor: vendor_name,
        cpu_arch,
        model: model_name,
        isa_string,
        cores,
        raw,
        midr_source: DataSource::LinuxProcCpuinfo,
        features_source: DataSource::LinuxProcCpuinfo,
    }
}

/// Read a RISC-V CSR value from sysfs.
fn read_sysfs_csr(name: &str) -> usize {
    let path = format!("/sys/devices/system/cpu/cpu0/regs/identification/{}", name);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| {
            let trimmed = s.trim().trim_start_matches("0x");
            usize::from_str_radix(trimmed, 16).ok()
        })
        .unwrap_or(0)
}

/// Read a property from the device tree CPU node.
///
/// Reads from `/sys/firmware/devicetree/base/cpus/cpu@0/{prop}`.
/// Binary properties (like `compatible`, `riscv,isa`) are null-terminated strings.
fn read_dt_cpu_prop(prop: &str) -> Option<String> {
    let path = format!("/sys/firmware/devicetree/base/cpus/cpu@0/{}", prop);
    let bytes = std::fs::read(&path).ok()?;
    // Strip trailing null bytes
    let len = bytes
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    if len == 0 {
        return None;
    }
    String::from_utf8(bytes[..len].to_vec()).ok()
}

/// Read the clock-frequency property from the device tree CPU node.
///
/// The value is a big-endian u32 in Hz. Returns a human-readable string.
fn read_dt_cpu_freq() -> Option<String> {
    let path = "/sys/firmware/devicetree/base/cpus/cpu@0/clock-frequency";
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 4 {
        return None;
    }
    // Big-endian u32
    let freq = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if freq == 0 {
        return None;
    }
    if freq >= 1_000_000_000 {
        Some(format!("{:.2} GHz", freq as f64 / 1_000_000_000.0))
    } else if freq >= 1_000_000 {
        Some(format!("{} MHz", freq / 1_000_000))
    } else {
        Some(format!("{} Hz", freq))
    }
}

// ----------------------------------------------------------------------------
// Feature detection via /proc/cpuinfo
// ----------------------------------------------------------------------------

/// Parses the ISA string from /proc/cpuinfo to extract extension letters.
/// E.g., "rv64gcv" -> vec!['i','m','a','f','d','c','v']
pub fn get_extensions_from_isa(isa: &str) -> Vec<char> {
    let mut exts = Vec::new();
    // Strip the prefix (rv32/rv64/rv128) and process remaining letters
    let remainder = if let Some(pos) = isa.find(|c: char| c.is_ascii_alphabetic() && c != 'r') {
        &isa[pos..]
    } else {
        return exts;
    };

    for ch in remainder.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_uppercase() {
            exts.push(ch);
        }
    }
    exts
}

/// Returns a map of extension letters to booleans from the ISA string.
pub fn get_features_from_isa(isa: &str) -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    // The ISA string contains single-letter extensions directly.
    // Multi-letter extensions (zba, zbb, etc.) appear after single-letter ones.
    let exts = get_extensions_from_isa(isa);
    for ch in &exts {
        features.insert(ch.to_string(), true);
    }

    // Also check for multi-letter extensions in the original string
    // (e.g., "rv64gcv_zba_zbb" or "rv64gcv_zicbom")
    let lower = isa.to_lowercase();
    let multi_letter = [
        "zba",
        "zbb",
        "zbc",
        "zbs",
        "zvfh",
        "zvbb",
        "zvbc",
        "zkne",
        "zknd",
        "zksed",
        "zksh",
        "zknh",
        "zicbom",
        "zicbop",
        "zicboz",
        "zicsr",
        "zifencei",
        "zicntr",
        "zihintpause",
        "zihintntl",
    ];
    for ext in &multi_letter {
        if lower.contains(ext) {
            features.insert(ext.to_string(), true);
        }
    }

    features
}

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features(isa: &str) -> BTreeMap<&'static str, String> {
    let src = get_features_from_isa(isa);
    let detected = crate::riscv::features::populate_detected_features(&src);
    crate::riscv::features::build_feature_map(&detected)
}
