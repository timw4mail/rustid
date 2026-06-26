//! ARM CPU feature detection.
//!
//! Provides feature detection similar to `src/cpuid/fns.rs` but for ARM CPUs.
//! Features are grouped into ARM-specific categories and returned in a consistent
//! lowercase format.

use std::collections::BTreeMap;

// ----------------------------------------------------------------------------
// Feature categories (ARM-specific, all lowercase)
// ----------------------------------------------------------------------------

/// Base CPU features (always or commonly present)
pub const BASE_FEATURES: &[&str] = &["fp", "asimd", "evtstrm", "cpuid"];

/// SIMD (Advanced SIMD / NEON) features
pub const SIMD_FEATURES: &[&str] = &["neon", "asimdhp", "asimdfhm", "asimddp", "asimdrdm"];

/// Crypto/hash features
pub const CRYPTO_FEATURES: &[&str] = &[
    "aes", "pmull", "sha1", "sha2", "sha3", "sha512", "sm3", "sm4",
];

/// Atomic and memory ordering features
pub const ATOMIC_FEATURES: &[&str] = &["atomics", "lse", "lse2"];

/// Floating point features
pub const FP_FEATURES: &[&str] = &["fphp", "fp16", "fcma", "jscvt"];

/// Miscellaneous features
pub const MISC_FEATURES: &[&str] = &[
    "crc32", "dcpop", "lrpc", "lrpc2", "flagm", "flagm2", "dit", "ssbs", "bti", "pauth", "pauth2",
    "fpac", "specres", "specres2", "csv2", "csv3", "ecv", "sb", "frintts", "dpb", "dpb2",
    "dotprod", "bf16", "i8mm", "sve", "sve2", "sve2p1", "sme", "sme2", "sme2p1", "hbc", "mops",
    "the", "smep", "smap", "5lvl",
];

/// Populate a detected features map from a platform source map.
/// Handles common aliases (asimd/neon, atomics/lse) and defaults unknown features to false.
pub fn populate_detected_features(src: &BTreeMap<String, bool>) -> BTreeMap<&'static str, bool> {
    let mut d: BTreeMap<&'static str, bool> = BTreeMap::new();

    let has_asimd =
        src.get("asimd").copied().unwrap_or(false) || src.get("neon").copied().unwrap_or(false);
    let has_atomics =
        src.get("atomics").copied().unwrap_or(false) || src.get("lse").copied().unwrap_or(false);

    d.insert("fp", src.get("fp").copied().unwrap_or(false));
    d.insert("asimd", has_asimd);
    d.insert("cpuid", src.get("cpuid").copied().unwrap_or(false));
    d.insert("evtstrm", src.get("evtstrm").copied().unwrap_or(false));
    d.insert("neon", has_asimd);
    d.insert("asimdhp", src.get("asimdhp").copied().unwrap_or(false));
    d.insert("asimdfhm", src.get("asimdfhm").copied().unwrap_or(false));
    d.insert("asimddp", src.get("asimddp").copied().unwrap_or(false));
    d.insert("asimdrdm", src.get("asimdrdm").copied().unwrap_or(false));
    d.insert("aes", src.get("aes").copied().unwrap_or(false));
    d.insert("pmull", src.get("pmull").copied().unwrap_or(false));
    d.insert("sha1", src.get("sha1").copied().unwrap_or(false));
    d.insert("sha2", src.get("sha2").copied().unwrap_or(false));
    d.insert("sha3", src.get("sha3").copied().unwrap_or(false));
    d.insert("sha512", src.get("sha512").copied().unwrap_or(false));
    d.insert("sm3", src.get("sm3").copied().unwrap_or(false));
    d.insert("sm4", src.get("sm4").copied().unwrap_or(false));
    d.insert("atomics", has_atomics);
    d.insert("lse", has_atomics);
    d.insert("lse2", src.get("lse2").copied().unwrap_or(false));
    d.insert("fphp", src.get("fphp").copied().unwrap_or(false));
    d.insert("fp16", src.get("fp16").copied().unwrap_or(false));
    d.insert("fcma", src.get("fcma").copied().unwrap_or(false));
    d.insert("jscvt", src.get("jscvt").copied().unwrap_or(false));
    d.insert("crc32", src.get("crc32").copied().unwrap_or(false));
    d.insert("dcpop", src.get("dcpop").copied().unwrap_or(false));
    d.insert("lrcpc", src.get("lrcpc").copied().unwrap_or(false));
    d.insert("lrcpc2", src.get("lrcpc2").copied().unwrap_or(false));
    d.insert("flagm", src.get("flagm").copied().unwrap_or(false));
    d.insert("flagm2", src.get("flagm2").copied().unwrap_or(false));
    d.insert("dit", src.get("dit").copied().unwrap_or(false));
    d.insert("ssbs", src.get("ssbs").copied().unwrap_or(false));
    d.insert("bti", src.get("bti").copied().unwrap_or(false));
    d.insert("pauth", src.get("pauth").copied().unwrap_or(false));
    d.insert("pauth2", src.get("pauth2").copied().unwrap_or(false));
    d.insert("fpac", src.get("fpac").copied().unwrap_or(false));
    d.insert("specres", src.get("specres").copied().unwrap_or(false));
    d.insert("specres2", src.get("specres2").copied().unwrap_or(false));
    d.insert("csv2", src.get("csv2").copied().unwrap_or(false));
    d.insert("csv3", src.get("csv3").copied().unwrap_or(false));
    d.insert("ecv", src.get("ecv").copied().unwrap_or(false));
    d.insert("sb", src.get("sb").copied().unwrap_or(false));
    d.insert("frintts", src.get("frintts").copied().unwrap_or(false));
    d.insert("dpb", src.get("dpb").copied().unwrap_or(false));
    d.insert("dpb2", src.get("dpb2").copied().unwrap_or(false));
    d.insert("dotprod", src.get("dotprod").copied().unwrap_or(false));
    d.insert("bf16", src.get("bf16").copied().unwrap_or(false));
    d.insert("i8mm", src.get("i8mm").copied().unwrap_or(false));
    d.insert("sve", src.get("sve").copied().unwrap_or(false));
    d.insert("sve2", src.get("sve2").copied().unwrap_or(false));
    d.insert("sme", src.get("sme").copied().unwrap_or(false));

    d
}

// ----------------------------------------------------------------------------
// Common feature check helpers (to be used by platform modules)
// ----------------------------------------------------------------------------

/// Returns a BTreeMap of feature categories to space-separated feature strings.
/// All feature names are lowercase for consistency.
pub fn build_feature_map(
    detected: &BTreeMap<&'static str, bool>,
) -> BTreeMap<&'static str, String> {
    let mut result: BTreeMap<&'static str, String> = BTreeMap::new();

    let categories: &[(&str, &[&str])] = &[
        ("Base", BASE_FEATURES),
        ("SIMD", SIMD_FEATURES),
        ("Security", CRYPTO_FEATURES),
        ("Atomics", ATOMIC_FEATURES),
        ("Fp", FP_FEATURES),
        ("Misc", MISC_FEATURES),
    ];

    for (cat_name, cat_features) in categories {
        let features: Vec<&str> = cat_features
            .iter()
            .filter(|f| detected.get(**f).copied().unwrap_or(false))
            .copied()
            .collect();
        if !features.is_empty() {
            result.insert(cat_name, features.join(" "));
        }
    }

    result
}

/// Normalize a feature name to lowercase.
pub fn normalize_feature_name(name: &str) -> String {
    name.to_lowercase()
}

// ----------------------------------------------------------------------------
// Aggregator: get_feature_list()
// ----------------------------------------------------------------------------

/// Returns a map of feature categories to space-separated feature strings.
/// Mirrors `src/cpuid/fns.rs::get_feature_list()` but with ARM-specific groups.
pub fn get_feature_list() -> BTreeMap<&'static str, String> {
    use crate::arm::TArmFeatures;

    let f = crate::arm::ArmFeatures;
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();

    // Base
    detected.insert("fp", f.has_fp());
    detected.insert("asimd", f.has_asimd());
    detected.insert("cpuid", cfg!(target_os = "linux")); // Only on Linux via HWCAP_CPUID

    // SIMD
    detected.insert("neon", f.has_neon());
    detected.insert("asimdhp", f.has_asimdhp());
    detected.insert("asimdfhm", f.has_asimdfhm());
    detected.insert("asimddp", f.has_asimddp());
    detected.insert("asimdrdm", f.has_asimdrdm());

    // Crypto
    detected.insert("aes", f.has_aes());
    detected.insert("sha1", f.has_sha1());
    detected.insert("sha2", f.has_sha2());
    detected.insert("sha3", f.has_sha3());
    detected.insert("sha512", f.has_sha512());
    detected.insert("pmull", f.has_pmull());
    detected.insert("sm3", f.has_sm3());
    detected.insert("sm4", f.has_sm4());

    // Atomic
    detected.insert("atomics", f.has_atomics());
    detected.insert("lse", f.has_lse());
    detected.insert("lse2", f.has_lse2());

    // FP
    detected.insert("fphp", f.has_fphp());
    detected.insert("fp16", f.has_fp16());
    detected.insert("fcma", f.has_fcma());
    detected.insert("jscvt", f.has_jscvt());

    build_feature_map(&detected)
}
