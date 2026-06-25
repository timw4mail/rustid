//! Linux-specific ARM CPU feature detection.
//!
//! Uses text-based parsing of `/proc/cpuinfo` "Features" line.

use super::OsCpuInfo;
use crate::arm::brand::Vendor;
use crate::arm::micro_arch::*;
use crate::common::DataSource;
use crate::common::get_proc_cpuinfo_data;
use std::collections::{BTreeMap, HashSet};

/// Linux-specific CPU detection via MRS, /sys, and /proc/cpuinfo.
pub fn detect() -> OsCpuInfo {
    let mut raw_midr: HashSet<usize> = HashSet::new();
    let mut midrs: HashSet<Midr> = HashSet::new();
    let mut all_midrs: Vec<Midr> = Vec::new();
    let mut midr_source = DataSource::CpuLookupTable;

    if let Some(core_ids) = core_affinity::get_core_ids() {
        for core_id in core_ids {
            core_affinity::set_for_current(core_id);
            let midr_val = crate::arm::get_midr();
            raw_midr.insert(midr_val);
            let midr = Midr::new(midr_val);
            midrs.insert(midr);
            all_midrs.push(midr);
        }
    } else {
        let midr_val = crate::arm::get_midr();
        raw_midr.insert(midr_val);
        let midr = Midr::new(midr_val);
        midrs.insert(midr);
        all_midrs.push(midr);
    }

    // If the kernel emulates a uniform MIDR for MRS, try /proc/cpuinfo or /sys.
    if midrs.len() == 1 || all_midrs.len() <= 1 {
        let linux_midrs = detect_linux_midrs();
        if !linux_midrs.is_empty() {
            all_midrs.clear();
            midrs.clear();
            raw_midr.clear();
            for m_val in linux_midrs {
                raw_midr.insert(m_val);
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
        raw_midr,
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

/// Reads MIDR values from sysfs or /proc/cpuinfo as a fallback when MRS
/// returns a uniform value for all cores.
fn detect_linux_midrs() -> Vec<usize> {
    let mut midrs = Vec::new();

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

            let m = (i << 24)
                | (var.unwrap_or(0) << 20)
                | (arch.unwrap_or(0) << 16)
                | (p << 4)
                | rev.unwrap_or(0);
            midrs.push(m);
        }
    }

    midrs
}

// ----------------------------------------------------------------------------
// Feature detection via /proc/cpuinfo (text-based, primary method)
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
// Public API: has_* functions
// ----------------------------------------------------------------------------

/// Check if floating-point (fp) is supported.
pub fn has_fp() -> bool {
    get_features_from_cpuinfo().get("fp").copied().unwrap_or(false)
}

/// Check if Advanced SIMD (NEON/asimd) is supported.
pub fn has_simd() -> bool {
    let features = get_features_from_cpuinfo();
    features.get("asimd").copied().unwrap_or(false)
        || features.get("neon").copied().unwrap_or(false)
}

/// Check if NEON is supported (alias for has_simd on ARM).
pub fn has_neon() -> bool {
    has_simd()
}

/// Check if AES instructions are supported.
pub fn has_aes() -> bool {
    get_features_from_cpuinfo().get("aes").copied().unwrap_or(false)
}

/// Check if SHA1 instructions are supported.
pub fn has_sha1() -> bool {
    get_features_from_cpuinfo().get("sha1").copied().unwrap_or(false)
}

/// Check if SHA2 instructions are supported.
pub fn has_sha2() -> bool {
    get_features_from_cpuinfo().get("sha2").copied().unwrap_or(false)
}

/// Check if SHA3 instructions are supported.
pub fn has_sha3() -> bool {
    get_features_from_cpuinfo().get("sha3").copied().unwrap_or(false)
}

/// Check if SHA512 instructions are supported.
pub fn has_sha512() -> bool {
    get_features_from_cpuinfo().get("sha512").copied().unwrap_or(false)
}

/// Check if CRC32 instructions are supported.
pub fn has_crc32() -> bool {
    get_features_from_cpuinfo().get("crc32").copied().unwrap_or(false)
}

/// Check if atomic instructions (LSE) are supported.
pub fn has_atomics() -> bool {
    let features = get_features_from_cpuinfo();
    features.get("atomics").copied().unwrap_or(false)
        || features.get("lse").copied().unwrap_or(false)
}

// ----------------------------------------------------------------------------
// Get all features as a BTreeMap (for Cpu struct)
// ----------------------------------------------------------------------------

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();

    let cpuinfo = get_features_from_cpuinfo();

    // Base features
    detected.insert("fp", cpuinfo.get("fp").copied().unwrap_or(false));
    detected.insert(
        "asimd",
        cpuinfo.get("asimd").copied().unwrap_or(false)
            || cpuinfo.get("neon").copied().unwrap_or(false),
    );
    detected.insert("cpuid", cpuinfo.get("cpuid").copied().unwrap_or(false));
    detected.insert("evtstrm", cpuinfo.get("evtstrm").copied().unwrap_or(false));

    // SIMD
    detected.insert("neon", detected.get("asimd").copied().unwrap_or(false));
    detected.insert("asimdhp", cpuinfo.get("asimdhp").copied().unwrap_or(false));
    detected.insert("asimdfhm", cpuinfo.get("asimdfhm").copied().unwrap_or(false));
    detected.insert("asimddp", cpuinfo.get("asimddp").copied().unwrap_or(false));
    detected.insert("asimdrdm", cpuinfo.get("asimdrdm").copied().unwrap_or(false));

    // Crypto
    detected.insert("aes", cpuinfo.get("aes").copied().unwrap_or(false));
    detected.insert("pmull", cpuinfo.get("pmull").copied().unwrap_or(false));
    detected.insert("sha1", cpuinfo.get("sha1").copied().unwrap_or(false));
    detected.insert("sha2", cpuinfo.get("sha2").copied().unwrap_or(false));
    detected.insert("sha3", cpuinfo.get("sha3").copied().unwrap_or(false));
    detected.insert("sha512", cpuinfo.get("sha512").copied().unwrap_or(false));
    detected.insert("sm3", cpuinfo.get("sm3").copied().unwrap_or(false));
    detected.insert("sm4", cpuinfo.get("sm4").copied().unwrap_or(false));

    // Atomic
    detected.insert(
        "atomics",
        cpuinfo.get("atomics").copied().unwrap_or(false)
            || cpuinfo.get("lse").copied().unwrap_or(false),
    );
    detected.insert("lse", detected.get("atomics").copied().unwrap_or(false));
    detected.insert("lse2", cpuinfo.get("lse2").copied().unwrap_or(false));

    // FP
    detected.insert("fphp", cpuinfo.get("fphp").copied().unwrap_or(false));
    detected.insert("fp16", cpuinfo.get("fp16").copied().unwrap_or(false));
    detected.insert("fcma", cpuinfo.get("fcma").copied().unwrap_or(false));
    detected.insert("jscvt", cpuinfo.get("jscvt").copied().unwrap_or(false));

    // Misc
    detected.insert("crc32", cpuinfo.get("crc32").copied().unwrap_or(false));
    detected.insert("dcpop", cpuinfo.get("dcpop").copied().unwrap_or(false));
    detected.insert("lrcpc", cpuinfo.get("lrcpc").copied().unwrap_or(false));
    detected.insert("lrcpc2", cpuinfo.get("lrcpc2").copied().unwrap_or(false));
    detected.insert("flagm", cpuinfo.get("flagm").copied().unwrap_or(false));
    detected.insert("flagm2", cpuinfo.get("flagm2").copied().unwrap_or(false));
    detected.insert("dit", cpuinfo.get("dit").copied().unwrap_or(false));
    detected.insert("ssbs", cpuinfo.get("ssbs").copied().unwrap_or(false));
    detected.insert("bti", cpuinfo.get("bti").copied().unwrap_or(false));
    detected.insert("pauth", cpuinfo.get("pauth").copied().unwrap_or(false));
    detected.insert("pauth2", cpuinfo.get("pauth2").copied().unwrap_or(false));
    detected.insert("fpac", cpuinfo.get("fpac").copied().unwrap_or(false));
    detected.insert("speces", cpuinfo.get("speces").copied().unwrap_or(false));
    detected.insert("specres2", cpuinfo.get("specres2").copied().unwrap_or(false));
    detected.insert("csv2", cpuinfo.get("csv2").copied().unwrap_or(false));
    detected.insert("csv3", cpuinfo.get("csv3").copied().unwrap_or(false));
    detected.insert("ecv", cpuinfo.get("ecv").copied().unwrap_or(false));
    detected.insert("sb", cpuinfo.get("sb").copied().unwrap_or(false));
    detected.insert("frintts", cpuinfo.get("frintts").copied().unwrap_or(false));
    detected.insert("dpb", cpuinfo.get("dpb").copied().unwrap_or(false));
    detected.insert("dpb2", cpuinfo.get("dpb2").copied().unwrap_or(false));
    detected.insert("dotprod", cpuinfo.get("dotprod").copied().unwrap_or(false));
    detected.insert("bf16", cpuinfo.get("bf16").copied().unwrap_or(false));
    detected.insert("i8mm", cpuinfo.get("i8mm").copied().unwrap_or(false));
    detected.insert("sve", cpuinfo.get("sve").copied().unwrap_or(false));
    detected.insert("sve2", cpuinfo.get("sve2").copied().unwrap_or(false));
    detected.insert("sme", cpuinfo.get("sme").copied().unwrap_or(false));

    crate::arm::features::build_feature_map(&detected)
}
