#![cfg(target_os = "android")]

//! Android-specific ARM CPU feature detection.
//!
//! Uses core affinity pinning + MRS (`MIDR_EL1` via `HWCAP_CPUID` kernel trap)
//! and text-based parsing of `/proc/cpuinfo` "Features" line.

use super::OsCpuInfo;
use crate::arm::brand::Vendor;
use crate::arm::micro_arch::*;
use crate::common::DataSource;
use crate::common::get_proc_cpuinfo_data;
use std::collections::{BTreeMap, HashSet};

/// Android-specific CPU detection via core affinity MRS, sysfs, and /proc/cpuinfo fallback.
pub fn detect() -> OsCpuInfo {
    let mut midrs: HashSet<Midr> = HashSet::new();
    let mut all_midrs: Vec<Midr> = Vec::new();
    let mut midr_source = DataSource::CpuLookupTable;

    #[cfg(not(target_arch = "arm"))]
    if let Some(core_ids) = core_affinity::get_core_ids() {
        for core_id in core_ids {
            core_affinity::set_for_current(core_id);
            let midr_val = crate::arm::get_midr();
            let midr = Midr::new(midr_val);
            midrs.insert(midr);
            all_midrs.push(midr);
        }
    } else {
        let midr_val = crate::arm::get_midr();
        let midr = Midr::new(midr_val);
        midrs.insert(midr);
        all_midrs.push(midr);
    }

    // On 32-bit ARM or when MRS returned uniform value on big.LITTLE,
    // check sysfs or /proc/cpuinfo
    #[cfg(target_arch = "arm")]
    {
        let fallback_midrs = detect_android_midrs();
        if !fallback_midrs.is_empty() {
            for m_val in fallback_midrs {
                let midr = Midr::new(m_val);
                midrs.insert(midr);
                all_midrs.push(midr);
            }
            midr_source = DataSource::LinuxProcCpuinfo;
        }
    }

    #[cfg(not(target_arch = "arm"))]
    if midrs.len() == 1 || all_midrs.len() <= 1 {
        let fallback_midrs = detect_android_midrs();
        if !fallback_midrs.is_empty() && fallback_midrs.len() > all_midrs.len() {
            all_midrs.clear();
            midrs.clear();
            for m_val in fallback_midrs {
                let midr = Midr::new(m_val);
                midrs.insert(midr);
                all_midrs.push(midr);
            }
            midr_source = DataSource::LinuxProcCpuinfo;
        }
    }

    let primary_midr = midrs.iter().next().copied().unwrap_or(Midr::default());
    let vendor: String = Vendor::from(primary_midr.implementer).into();
    let cpu_arch = CpuArch::find(
        primary_midr.implementer,
        primary_midr.part,
        primary_midr.variant,
    );
    let cores = super::detect_cores(&all_midrs);

    OsCpuInfo {
        midrs,
        vendor,
        cpu_arch,
        cores,
        model: String::new(),
        raw: BTreeMap::new(),
        midr_source,
        features_source: DataSource::LinuxProcCpuinfo,
    }
}

/// Reads MIDR values from sysfs or /proc/cpuinfo as a fallback.
fn detect_android_midrs() -> Vec<usize> {
    let mut midrs = Vec::new();

    // 1. Try sysfs if readable
    let mut i = 0;
    loop {
        let path = format!(
            "/sys/devices/system/cpu/cpu{}/regs/identification/midr_el1",
            i
        );
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(midr) = usize::from_str_radix(content.trim().trim_start_matches("0x"), 16) {
                midrs.push(midr);
            }
        } else {
            break;
        }
        i += 1;
    }

    if !midrs.is_empty() {
        return midrs;
    }

    // 2. Parse /proc/cpuinfo per-processor blocks
    let cpuinfo = get_proc_cpuinfo_data();
    for map in &cpuinfo {
        let impl_ = map.get("CPU implementer").and_then(|s| {
            usize::from_str_radix(
                s.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches("0x"),
                16,
            )
            .ok()
        });
        let part = map.get("CPU part").and_then(|s| {
            usize::from_str_radix(
                s.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches("0x"),
                16,
            )
            .ok()
        });
        if let (Some(i), Some(p)) = (impl_, part) {
            let var = map.get("CPU variant").and_then(|s| {
                usize::from_str_radix(
                    s.split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("0x"),
                    16,
                )
                .ok()
            });
            let arch = map
                .get("CPU architecture")
                .and_then(|s| s.split_whitespace().next().unwrap_or("").parse().ok());
            let rev = map
                .get("CPU revision")
                .and_then(|s| s.split_whitespace().next().unwrap_or("").parse().ok());

            let m = (i << IMPLEMENTER_OFFSET)
                | (var.unwrap_or(0) << VARIANT_OFFSET)
                | (arch.unwrap_or(0) << ARCHITECTURE_OFFSET)
                | (p << PART_OFFSET)
                | rev.unwrap_or(0);
            midrs.push(m);
        }
    }

    midrs
}

// ----------------------------------------------------------------------------
// Feature detection via /proc/cpuinfo
// ----------------------------------------------------------------------------

/// Parses `/proc/cpuinfo` Features line to get a set of available features.
/// All feature names are converted to lowercase for consistency.
pub fn get_features_from_cpuinfo() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    let cpuinfo = get_proc_cpuinfo_data();
    if let Some(first) = cpuinfo.first() {
        if let Some(features_str) = first.get("Features") {
            for feat in features_str.split_whitespace() {
                features.insert(feat.to_lowercase(), true);
            }
        }
    }

    features
}

// ----------------------------------------------------------------------------
// TArmFeatures implementation
// ----------------------------------------------------------------------------

use crate::arm::TArmFeatures;

impl TArmFeatures for crate::arm::ArmFeatures {
    fn has_fp(&self) -> bool {
        get_features_from_cpuinfo()
            .get("fp")
            .copied()
            .unwrap_or(false)
    }

    fn has_asimd(&self) -> bool {
        let features = get_features_from_cpuinfo();
        features.get("asimd").copied().unwrap_or(false)
            || features.get("neon").copied().unwrap_or(false)
    }

    fn has_aes(&self) -> bool {
        get_features_from_cpuinfo()
            .get("aes")
            .copied()
            .unwrap_or(false)
    }

    fn has_sha1(&self) -> bool {
        get_features_from_cpuinfo()
            .get("sha1")
            .copied()
            .unwrap_or(false)
    }

    fn has_sha2(&self) -> bool {
        get_features_from_cpuinfo()
            .get("sha2")
            .copied()
            .unwrap_or(false)
    }

    fn has_sha3(&self) -> bool {
        get_features_from_cpuinfo()
            .get("sha3")
            .copied()
            .unwrap_or(false)
    }

    fn has_sha512(&self) -> bool {
        get_features_from_cpuinfo()
            .get("sha512")
            .copied()
            .unwrap_or(false)
    }

    fn has_crc32(&self) -> bool {
        get_features_from_cpuinfo()
            .get("crc32")
            .copied()
            .unwrap_or(false)
    }

    fn has_atomics(&self) -> bool {
        let features = get_features_from_cpuinfo();
        features.get("atomics").copied().unwrap_or(false)
            || features.get("lse").copied().unwrap_or(false)
    }
}

// ----------------------------------------------------------------------------
// Get all features as a BTreeMap (for Cpu struct)
// ----------------------------------------------------------------------------

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let src = get_features_from_cpuinfo();
    let detected = crate::arm::features::populate_detected_features(&src);
    crate::arm::features::build_feature_map(&detected)
}
