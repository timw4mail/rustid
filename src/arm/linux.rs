//! Linux-specific ARM CPU feature detection.
//!
//! Uses text-based parsing of `/proc/cpuinfo` "Features" line and
//! `getauxval(AT_HWCAP/AT_HWCAP2/AT_HWCAP3)` when available.

use std::collections::BTreeMap;
use std::process::Command;

// ----------------------------------------------------------------------------
// Feature detection via /proc/cpuinfo (text-based, primary method)
// ----------------------------------------------------------------------------

/// Parses `/proc/cpuinfo` Features line to get a set of available features.
/// All feature names are converted to lowercase for consistency.
pub fn get_features_from_cpuinfo() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Features") {
                if let Some(rest) = line.split(':').nth(1) {
                    for feat in rest.split_whitespace() {
                        let normalized = feat.to_lowercase();
                        features.insert(normalized, true);
                    }
                }
                break;
            }
        }
    }

    features
}

// ----------------------------------------------------------------------------
// Feature detection via getauxval (when libc is available)
// ----------------------------------------------------------------------------

/// Attempts to get features from `getauxval(AT_HWCAP/AT_HWCAP2/AT_HWCAP3)`.
/// Returns a map of feature names to boolean values.
/// This is a fallback to /proc/cpuinfo parsing.
pub fn get_features_from_hwcap() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    // Only attempt if we can use libc
    #[cfg(target_os = "linux")]
    {
        use std::os::raw::c_ulong;

        // AT_HWCAP = 16, AT_HWCAP2 = 26, AT_HWCAP3 = 32 (from linux/auxvec.h)
        const AT_HWCAP: i32 = 16;
        const AT_HWCAP2: i32 = 26;
        const AT_HWCAP3: i32 = 32;

        // HWCAP flags for ARM (from linux/elf.h and kernel docs)
        const HWCAP_FP: c_ulong = 1 << 0;
        const HWCAP_ASIMD: c_ulong = 1 << 1;
        const HWCAP_EVTSTRM: c_ulong = 1 << 2;
        const HWCAP_AES: c_ulong = 1 << 3;
        const HWCAP_PMULL: c_ulong = 1 << 4;
        const HWCAP_SHA1: c_ulong = 1 << 5;
        const HWCAP_SHA2: c_ulong = 1 << 6;
        const HWCAP_CRC32: c_ulong = 1 << 7;
        const HWCAP_ATOMICS: c_ulong = 1 << 8;
        const HWCAP_FPHP: c_ulong = 1 << 9;
        const HWCAP_ASIMDHP: c_ulong = 1 << 10;
        const HWCAP_CPUID: c_ulong = 1 << 11;
        const HWCAP_ASIMDRDM: c_ulong = 1 << 12;
        const HWCAP_JSCVT: c_ulong = 1 << 13;
        const HWCAP_FCMA: c_ulong = 1 << 14;
        const HWCAP_LRCPC: c_ulong = 1 << 15;
        const HWCAP_DCPOP: c_ulong = 1 << 16;
        const HWCAP_SHA3: c_ulong = 1 << 17;
        const HWCAP_SM3: c_ulong = 1 << 18;
        const HWCAP_SM4: c_ulong = 1 << 19;
        const HWCAP_ASIMDDP: c_ulong = 1 << 20;
        const HWCAP_SHA512: c_ulong = 1 << 21;
        const HWCAP_SVE: c_ulong = 1 << 22;
        const HWCAP_ASIMDFHM: c_ulong = 1 << 23;
        const HWCAP_DIT: c_ulong = 1 << 24;
        const HWCAP_USCAT: c_ulong = 1 << 25;
        const HWCAP_ILRCPC: c_ulong = 1 << 26;
        const HWCAP_FLAGM: c_ulong = 1 << 27;
        const HWCAP_SSBS: c_ulong = 1 << 28;
        const HWCAP_SB: c_ulong = 1 << 29;
        const HWCAP_PACA: c_ulong = 1 << 30;
        const HWCAP_PACG: c_ulong = 1 << 31;

        // HWCAP2 flags
        const HWCAP2_DCPODP: c_ulong = 1 << 0;
        const HWCAP2_SVE2: c_ulong = 1 << 1;
        const HWCAP2_SVEAES: c_ulong = 1 << 2;
        const HWCAP2_SVEPMULL: c_ulong = 1 << 3;
        const HWCAP2_SVEBITPERM: c_ulong = 1 << 4;
        const HWCAP2_SVESHA3: c_ulong = 1 << 5;
        const HWCAP2_SVESHA512: c_ulong = 1 << 6;
        const HWCAP2_SVESM4: c_ulong = 1 << 7;
        const HWCAP2_FLAGM2: c_ulong = 1 << 8;
        const HWCAP2_FRINT: c_ulong = 1 << 9;
        const HWCAP2_SVEI8MM: c_ulong = 1 << 10;
        const HWCAP2_SVEF32MM: c_ulong = 1 << 11;
        const HWCAP2_SVEF64MM: c_ulong = 1 << 12;
        const HWCAP2_SVEBF16: c_ulong = 1 << 13;
        const HWCAP2_I8MM: c_ulong = 1 << 14;
        const HWCAP2_BF16: c_ulong = 1 << 15;
        const HWCAP2_DGH: c_ulong = 1 << 16;
        const HWCAP2_RNG: c_ulong = 1 << 17;
        const HWCAP2_BTI: c_ulong = 1 << 18;
        const HWCAP2_MTE: c_ulong = 1 << 19;
        const HWCAP2_ECV: c_ulong = 1 << 20;
        const HWCAP2_AFP: c_ulong = 1 << 21;
        const HWCAP2_RPRES: c_ulong = 1 << 22;
        const HWCAP2_MTE3: c_ulong = 1 << 23;
        const HWCAP2_SME: c_ulong = 1 << 24;
        const HWCAP2_SME_I16I64: c_ulong = 1 << 25;
        const HWCAP2_SME_F64F64: c_ulong = 1 << 26;
        const HWCAP2_SME_I8I32: c_ulong = 1 << 27;
        const HWCAP2_SME_F16F32: c_ulong = 1 << 28;
        const HWCAP2_SME_B16F32: c_ulong = 1 << 29;
        const HWCAP2_SME_F32F32: c_ulong = 1 << 30;
        const HWCAP2_SME_FA64: c_ulong = 1 << 31;

        unsafe {
            let hwcap = libc::getauxval(AT_HWCAP as u64);
            let hwcap2 = libc::getauxval(AT_HWCAP2 as u64);
            let hwcap3 = libc::getauxval(AT_HWCAP3 as u64);

            // Map HWCAP flags to feature names
            features.insert("fp".to_string(), (hwcap & HWCAP_FP) != 0);
            features.insert("asimd".to_string(), (hwcap & HWCAP_ASIMD) != 0);
            features.insert("evtstrm".to_string(), (hwcap & HWCAP_EVTSTRM) != 0);
            features.insert("aes".to_string(), (hwcap & HWCAP_AES) != 0);
            features.insert("pmull".to_string(), (hwcap & HWCAP_PMULL) != 0);
            features.insert("sha1".to_string(), (hwcap & HWCAP_SHA1) != 0);
            features.insert("sha2".to_string(), (hwcap & HWCAP_SHA2) != 0);
            features.insert("crc32".to_string(), (hwcap & HWCAP_CRC32) != 0);
            features.insert("atomics".to_string(), (hwcap & HWCAP_ATOMICS) != 0);
            features.insert("fphp".to_string(), (hwcap & HWCAP_FPHP) != 0);
            features.insert("asimdhp".to_string(), (hwcap & HWCAP_ASIMDHP) != 0);
            features.insert("cpuid".to_string(), (hwcap & HWCAP_CPUID) != 0);
            features.insert("asimdrdm".to_string(), (hwcap & HWCAP_ASIMDRDM) != 0);
            features.insert("jscvt".to_string(), (hwcap & HWCAP_JSCVT) != 0);
            features.insert("fcma".to_string(), (hwcap & HWCAP_FCMA) != 0);
            features.insert("lrcpc".to_string(), (hwcap & HWCAP_LRCPC) != 0);
            features.insert("dcpop".to_string(), (hwcap & HWCAP_DCPOP) != 0);
            features.insert("sha3".to_string(), (hwcap & HWCAP_SHA3) != 0);
            features.insert("sm3".to_string(), (hwcap & HWCAP_SM3) != 0);
            features.insert("sm4".to_string(), (hwcap & HWCAP_SM4) != 0);
            features.insert("asimddp".to_string(), (hwcap & HWCAP_ASIMDDP) != 0);
            features.insert("sha512".to_string(), (hwcap & HWCAP_SHA512) != 0);
            features.insert("sve".to_string(), (hwcap & HWCAP_SVE) != 0);
            features.insert("asimdfhm".to_string(), (hwcap & HWCAP_ASIMDFHM) != 0);
            features.insert("dit".to_string(), (hwcap & HWCAP_DIT) != 0);
            features.insert("ssbs".to_string(), (hwcap & HWCAP_SSBS) != 0);
            features.insert("sb".to_string(), (hwcap & HWCAP_SB) != 0);
            features.insert("pauth".to_string(), (hwcap & HWCAP_PACA) != 0);

            // HWCAP2 features
            features.insert("sve2".to_string(), (hwcap2 & HWCAP2_SVE2) != 0);
            features.insert("bf16".to_string(), (hwcap2 & HWCAP2_BF16) != 0);
            features.insert("i8mm".to_string(), (hwcap2 & HWCAP2_I8MM) != 0);
            features.insert("bti".to_string(), (hwcap2 & HWCAP2_BTI) != 0);
            features.insert("ecv".to_string(), (hwcap2 & HWCAP2_ECV) != 0);
            features.insert("sme".to_string(), (hwcap2 & HWCAP2_SME) != 0);
            features.insert("flagm2".to_string(), (hwcap2 & HWCAP2_FLAGM2) != 0);
            features.insert("frintts".to_string(), (hwcap2 & HWCAP2_FRINT) != 0);
            features.insert("dotprod".to_string(), (hwcap2 & HWCAP2_SVEAES) != 0);

            // HWCAP3 features (if available)
            if hwcap3 != 0 {
                // HWCAP3 flags would be defined here
                // For now, just mark that HWCAP3 is available
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
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("fp") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("fp").copied().unwrap_or(false)
}

/// Check if Advanced SIMD (NEON/asimd) is supported.
pub fn has_simd() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("asimd") {
        return has;
    }
    if let Some(&has) = cpuinfo_features.get("neon") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("asimd").copied().unwrap_or(false)
}

/// Check if NEON is supported (alias for has_simd on ARM).
pub fn has_neon() -> bool {
    has_simd()
}

/// Check if AES instructions are supported.
pub fn has_aes() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("aes") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("aes").copied().unwrap_or(false)
}

/// Check if SHA1 instructions are supported.
pub fn has_sha1() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("sha1") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("sha1").copied().unwrap_or(false)
}

/// Check if SHA2 instructions are supported.
pub fn has_sha2() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("sha2") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("sha2").copied().unwrap_or(false)
}

/// Check if SHA3 instructions are supported.
pub fn has_sha3() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("sha3") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("sha3").copied().unwrap_or(false)
}

/// Check if SHA512 instructions are supported.
pub fn has_sha512() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("sha512") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("sha512").copied().unwrap_or(false)
}

/// Check if CRC32 instructions are supported.
pub fn has_crc32() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("crc32") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("crc32").copied().unwrap_or(false)
}

/// Check if atomic instructions (LSE) are supported.
pub fn has_atomics() -> bool {
    let cpuinfo_features = get_features_from_cpuinfo();
    if let Some(&has) = cpuinfo_features.get("atomics") {
        return has;
    }
    if let Some(&has) = cpuinfo_features.get("lse") {
        return has;
    }

    let hwcap_features = get_features_from_hwcap();
    hwcap_features.get("atomics").copied().unwrap_or(false)
}

// ----------------------------------------------------------------------------
// Get all features as a BTreeMap (for Cpu struct)
// ----------------------------------------------------------------------------

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();

    // Combine cpuinfo and hwcap sources
    let cpuinfo = get_features_from_cpuinfo();
    let hwcap = get_features_from_hwcap();

    // Base features
    detected.insert(
        "fp",
        cpuinfo.get("fp").copied().unwrap_or(false) || hwcap.get("fp").copied().unwrap_or(false),
    );
    detected.insert(
        "asimd",
        cpuinfo.get("asimd").copied().unwrap_or(false)
            || cpuinfo.get("neon").copied().unwrap_or(false)
            || hwcap.get("asimd").copied().unwrap_or(false),
    );
    detected.insert("cpuid", hwcap.get("cpuid").copied().unwrap_or(false));
    detected.insert(
        "evtstrm",
        cpuinfo.get("evtstrm").copied().unwrap_or(false)
            || hwcap.get("evtstrm").copied().unwrap_or(false),
    );

    // SIMD
    detected.insert("neon", detected.get("asimd").copied().unwrap_or(false));
    detected.insert(
        "asimdhp",
        cpuinfo.get("asimdhp").copied().unwrap_or(false)
            || hwcap.get("asimdhp").copied().unwrap_or(false),
    );
    detected.insert(
        "asimdfhm",
        cpuinfo.get("asimdfhm").copied().unwrap_or(false)
            || hwcap.get("asimdfhm").copied().unwrap_or(false),
    );
    detected.insert(
        "asimddp",
        cpuinfo.get("asimddp").copied().unwrap_or(false)
            || hwcap.get("asimddp").copied().unwrap_or(false),
    );
    detected.insert(
        "asimdrdm",
        cpuinfo.get("asimdrdm").copied().unwrap_or(false)
            || hwcap.get("asimdrdm").copied().unwrap_or(false),
    );

    // Crypto
    detected.insert(
        "aes",
        cpuinfo.get("aes").copied().unwrap_or(false) || hwcap.get("aes").copied().unwrap_or(false),
    );
    detected.insert(
        "pmull",
        cpuinfo.get("pmull").copied().unwrap_or(false)
            || hwcap.get("pmull").copied().unwrap_or(false),
    );
    detected.insert(
        "sha1",
        cpuinfo.get("sha1").copied().unwrap_or(false)
            || hwcap.get("sha1").copied().unwrap_or(false),
    );
    detected.insert(
        "sha2",
        cpuinfo.get("sha2").copied().unwrap_or(false)
            || hwcap.get("sha2").copied().unwrap_or(false),
    );
    detected.insert(
        "sha3",
        cpuinfo.get("sha3").copied().unwrap_or(false)
            || hwcap.get("sha3").copied().unwrap_or(false),
    );
    detected.insert(
        "sha512",
        cpuinfo.get("sha512").copied().unwrap_or(false)
            || hwcap.get("sha512").copied().unwrap_or(false),
    );
    detected.insert(
        "sm3",
        cpuinfo.get("sm3").copied().unwrap_or(false) || hwcap.get("sm3").copied().unwrap_or(false),
    );
    detected.insert(
        "sm4",
        cpuinfo.get("sm4").copied().unwrap_or(false) || hwcap.get("sm4").copied().unwrap_or(false),
    );

    // Atomic
    detected.insert(
        "atomics",
        cpuinfo.get("atomics").copied().unwrap_or(false)
            || cpuinfo.get("lse").copied().unwrap_or(false)
            || hwcap.get("atomics").copied().unwrap_or(false),
    );
    detected.insert("lse", detected.get("atomics").copied().unwrap_or(false));
    detected.insert("lse2", cpuinfo.get("lse2").copied().unwrap_or(false));

    // FP
    detected.insert(
        "fphp",
        cpuinfo.get("fphp").copied().unwrap_or(false)
            || hwcap.get("fphp").copied().unwrap_or(false),
    );
    detected.insert("fp16", cpuinfo.get("fp16").copied().unwrap_or(false));
    detected.insert(
        "fcma",
        cpuinfo.get("fcma").copied().unwrap_or(false)
            || hwcap.get("fcma").copied().unwrap_or(false),
    );
    detected.insert(
        "jscvt",
        cpuinfo.get("jscvt").copied().unwrap_or(false)
            || hwcap.get("jscvt").copied().unwrap_or(false),
    );

    // Misc
    detected.insert(
        "crc32",
        cpuinfo.get("crc32").copied().unwrap_or(false)
            || hwcap.get("crc32").copied().unwrap_or(false),
    );
    detected.insert(
        "dcpop",
        cpuinfo.get("dcpop").copied().unwrap_or(false)
            || hwcap.get("dcpop").copied().unwrap_or(false),
    );
    detected.insert(
        "lrcpc",
        cpuinfo.get("lrcpc").copied().unwrap_or(false)
            || hwcap.get("lrcpc").copied().unwrap_or(false),
    );
    detected.insert("lrcpc2", cpuinfo.get("lrcpc2").copied().unwrap_or(false));
    detected.insert(
        "flagm",
        cpuinfo.get("flagm").copied().unwrap_or(false)
            || hwcap.get("flagm").copied().unwrap_or(false),
    );
    detected.insert(
        "flagm2",
        cpuinfo.get("flagm2").copied().unwrap_or(false)
            || hwcap.get("flagm2").copied().unwrap_or(false),
    );
    detected.insert(
        "dit",
        cpuinfo.get("dit").copied().unwrap_or(false) || hwcap.get("dit").copied().unwrap_or(false),
    );
    detected.insert(
        "ssbs",
        cpuinfo.get("ssbs").copied().unwrap_or(false)
            || hwcap.get("ssbs").copied().unwrap_or(false),
    );
    detected.insert(
        "bti",
        cpuinfo.get("bti").copied().unwrap_or(false) || hwcap.get("bti").copied().unwrap_or(false),
    );
    detected.insert(
        "pauth",
        cpuinfo.get("pauth").copied().unwrap_or(false)
            || hwcap.get("pauth").copied().unwrap_or(false),
    );
    detected.insert("pauth2", cpuinfo.get("pauth2").copied().unwrap_or(false));
    detected.insert("fpac", cpuinfo.get("fpac").copied().unwrap_or(false));
    detected.insert("speces", cpuinfo.get("speces").copied().unwrap_or(false));
    detected.insert(
        "specres2",
        cpuinfo.get("specres2").copied().unwrap_or(false),
    );
    detected.insert("csv2", cpuinfo.get("csv2").copied().unwrap_or(false));
    detected.insert("csv3", cpuinfo.get("csv3").copied().unwrap_or(false));
    detected.insert(
        "ecv",
        cpuinfo.get("ecv").copied().unwrap_or(false) || hwcap.get("ecv").copied().unwrap_or(false),
    );
    detected.insert(
        "sb",
        cpuinfo.get("sb").copied().unwrap_or(false) || hwcap.get("sb").copied().unwrap_or(false),
    );
    detected.insert(
        "frintts",
        cpuinfo.get("frintts").copied().unwrap_or(false)
            || hwcap.get("frintts").copied().unwrap_or(false),
    );
    detected.insert("dpb", cpuinfo.get("dpb").copied().unwrap_or(false));
    detected.insert("dpb2", cpuinfo.get("dpb2").copied().unwrap_or(false));
    detected.insert(
        "dotprod",
        cpuinfo.get("dotprod").copied().unwrap_or(false)
            || hwcap.get("dotprod").copied().unwrap_or(false),
    );
    detected.insert(
        "bf16",
        cpuinfo.get("bf16").copied().unwrap_or(false)
            || hwcap.get("bf16").copied().unwrap_or(false),
    );
    detected.insert(
        "i8mm",
        cpuinfo.get("i8mm").copied().unwrap_or(false)
            || hwcap.get("i8mm").copied().unwrap_or(false),
    );
    detected.insert(
        "sve",
        cpuinfo.get("sve").copied().unwrap_or(false) || hwcap.get("sve").copied().unwrap_or(false),
    );
    detected.insert(
        "sve2",
        cpuinfo.get("sve2").copied().unwrap_or(false)
            || hwcap.get("sve2").copied().unwrap_or(false),
    );
    detected.insert(
        "sme",
        cpuinfo.get("sme").copied().unwrap_or(false) || hwcap.get("sme").copied().unwrap_or(false),
    );

    super::features::build_feature_map(&detected)
}
