#![cfg(linux_os)]

//! Linux and Android ARM CPU feature detection.
//!
//! Uses text-based parsing of `/proc/cpuinfo` "Features" line,
//! core affinity pinning + MRS (`MIDR_EL1` via `HWCAP_CPUID` kernel trap),
//! and sysfs `/sys/devices/system/cpu/` topology and MIDR parsing.

use super::OsCpuInfo;
use crate::arm::brand::Vendor;
use crate::arm::micro_arch::*;
use crate::common::DataSource;
use crate::common::get_proc_cpuinfo_data;
use std::collections::{BTreeMap, HashSet};

/// Linux and Android CPU detection via /sys, /proc/cpuinfo, and inline asm / MRS fallback.
pub fn detect() -> OsCpuInfo {
    let mut midrs: HashSet<Midr> = HashSet::new();
    let mut all_midrs: Vec<Midr> = Vec::new();
    let mut midr_source = DataSource::CpuLookupTable;

    #[cfg(not(target_arch = "arm"))]
    crate::common::for_each_logical_core(|| {
        let midr_val = crate::arm::get_midr();
        let midr = Midr::new(midr_val);
        midrs.insert(midr);
        all_midrs.push(midr);
    });

    // Prefer sysfs for reading the MIDR on 32-bit ARM to avoid
    // inline asm (`mrc p15, ...`) which may cause SIGILL on
    // systems where the coprocessor access is trapped or on
    // older CPUs with LLVM codegen issues.
    #[cfg(target_arch = "arm")]
    {
        let linux_midrs = detect_linux_midrs();
        if !linux_midrs.is_empty() {
            for m_val in linux_midrs {
                let midr = Midr::new(m_val);
                midrs.insert(midr);
                all_midrs.push(midr);
            }
            midr_source = DataSource::LinuxSysFs;
        } else {
            panic!("Could not get midr value from sysfs");
        }
    }

    // For AArch64 (where MRS is always available), check sysfs / proc cpuinfo
    // if MRS returned a uniform value on big.LITTLE or if sysfs / cpuinfo finds more cores.
    #[cfg(not(target_arch = "arm"))]
    {
        let linux_midrs = detect_linux_midrs();
        if !linux_midrs.is_empty() && (linux_midrs.len() > all_midrs.len() || midrs.len() <= 1) {
            all_midrs.clear();
            midrs.clear();
            for m_val in linux_midrs {
                let midr = Midr::new(m_val);
                midrs.insert(midr);
                all_midrs.push(midr);
            }
            midr_source = DataSource::LinuxSysFs;
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

/// Reads MIDR values from sysfs or /proc/cpuinfo across all CPU cores.
/// Handles big.LITTLE / DynamIQ heterogeneous topologies and offline cores.
pub fn detect_linux_midrs() -> Vec<usize> {
    // 1. Determine all expected CPUs from sysfs /possible or /present
    let mut possible_cpus = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/possible") {
        possible_cpus = crate::common::expand_cpu_list(&content);
    }
    if possible_cpus.is_empty()
        && let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/present")
    {
        possible_cpus = crate::common::expand_cpu_list(&content);
    }
    if possible_cpus.is_empty() {
        let mut missing_streak = 0;
        for i in 0..256 {
            let cpu_dir = format!("/sys/devices/system/cpu/cpu{}", i);
            if std::path::Path::new(&cpu_dir).exists() {
                possible_cpus.push(i);
                missing_streak = 0;
            } else {
                missing_streak += 1;
                if missing_streak >= 8 && i > 8 {
                    break;
                }
            }
        }
    }

    // 2. Read sysfs midr_el1 for each possible CPU
    let mut sysfs_midrs: BTreeMap<u32, usize> = BTreeMap::new();
    for &cpu_id in &possible_cpus {
        let path = format!(
            "/sys/devices/system/cpu/cpu{}/regs/identification/midr_el1",
            cpu_id
        );
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(midr) = usize::from_str_radix(content.trim().trim_start_matches("0x"), 16)
        {
            sysfs_midrs.insert(cpu_id, midr);
        }
    }

    // 3. For any offline CPU missing sysfs midr_el1, infer from cluster siblings
    for &cpu_id in &possible_cpus {
        if !sysfs_midrs.contains_key(&cpu_id) {
            let rel_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/related_cpus", cpu_id);
            let sibling_cpus = std::fs::read_to_string(&rel_path)
                .ok()
                .map(|s| crate::common::expand_cpu_list(&s))
                .or_else(|| {
                    let sib_path = format!(
                        "/sys/devices/system/cpu/cpu{}/topology/core_siblings_list",
                        cpu_id
                    );
                    std::fs::read_to_string(&sib_path)
                        .ok()
                        .map(|s| crate::common::expand_cpu_list(&s))
                });

            if let Some(siblings) = sibling_cpus {
                for sib in siblings {
                    if let Some(&known_midr) = sysfs_midrs.get(&sib) {
                        sysfs_midrs.insert(cpu_id, known_midr);
                        break;
                    }
                }
            }
        }
    }

    // 4. Parse /proc/cpuinfo per-processor blocks
    let mut cpuinfo_midrs: BTreeMap<u32, usize> = BTreeMap::new();
    let mut cpuinfo_list: Vec<usize> = Vec::new();
    let cpuinfo = get_proc_cpuinfo_data();
    for (idx, map) in cpuinfo.iter().enumerate() {
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

            let proc_id = map
                .get("processor")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(idx as u32);

            cpuinfo_midrs.insert(proc_id, m);
            cpuinfo_list.push(m);
        }
    }

    // 5. Fill any remaining gaps in sysfs_midrs from cpuinfo_midrs
    for &cpu_id in &possible_cpus {
        if !sysfs_midrs.contains_key(&cpu_id)
            && let Some(&m) = cpuinfo_midrs.get(&cpu_id)
        {
            sysfs_midrs.insert(cpu_id, m);
        }
    }

    // 6. Return the most complete and accurate list of MIDRs
    if sysfs_midrs.len() >= possible_cpus.len() && !sysfs_midrs.is_empty() {
        return sysfs_midrs.into_values().collect();
    }

    if cpuinfo_list.len() >= sysfs_midrs.len() && !cpuinfo_list.is_empty() {
        return cpuinfo_list;
    }

    if !sysfs_midrs.is_empty() {
        return sysfs_midrs.into_values().collect();
    }

    cpuinfo_list
}

// ----------------------------------------------------------------------------
// Feature detection via /proc/cpuinfo (text-based, primary method)
// ----------------------------------------------------------------------------

/// Parses `/proc/cpuinfo` Features line to get a set of available features.
/// All feature names are converted to lowercase for consistency.
pub fn get_features_from_cpuinfo() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    let cpuinfo = get_proc_cpuinfo_data();
    if let Some(first) = cpuinfo.first()
        && let Some(features_str) = first.get("Features")
    {
        for feat in features_str.split_whitespace() {
            features.insert(feat.to_lowercase(), true);
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
