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

pub struct ArmFeatures;

#[allow(unused)]
pub trait TArmFeatures {
    // Base features
    fn has_fp(&self) -> bool {
        false
    }
    fn has_asimd(&self) -> bool {
        false
    }
    fn has_evtstrm(&self) -> bool {
        false
    }
    fn has_cpuid(&self) -> bool {
        false
    }

    // SIMD/NEON features
    fn has_neon(&self) -> bool {
        self.has_asimd()
    }
    fn has_asimdhp(&self) -> bool {
        false
    }
    fn has_asimdfhm(&self) -> bool {
        false
    }
    fn has_asimddp(&self) -> bool {
        false
    }
    fn has_asimdrdm(&self) -> bool {
        false
    }

    // Crypto features
    fn has_aes(&self) -> bool {
        false
    }
    fn has_pmull(&self) -> bool {
        false
    }
    fn has_sha1(&self) -> bool {
        false
    }
    fn has_sha2(&self) -> bool {
        false
    }
    fn has_sha3(&self) -> bool {
        false
    }
    fn has_sha512(&self) -> bool {
        false
    }
    fn has_sm3(&self) -> bool {
        false
    }
    fn has_sm4(&self) -> bool {
        false
    }

    // Atomics
    fn has_atomics(&self) -> bool {
        false
    }
    fn has_lse(&self) -> bool {
        self.has_atomics()
    }
    fn has_lse2(&self) -> bool {
        false
    }

    // Floating-point features
    fn has_fphp(&self) -> bool {
        false
    }
    fn has_fp16(&self) -> bool {
        false
    }
    fn has_fcma(&self) -> bool {
        false
    }
    fn has_jscvt(&self) -> bool {
        false
    }

    // Misc features
    fn has_crc32(&self) -> bool {
        false
    }
    fn has_dcpop(&self) -> bool {
        false
    }
    fn has_lrcpc(&self) -> bool {
        false
    }
    fn has_lrcpc2(&self) -> bool {
        false
    }
    fn has_flagm(&self) -> bool {
        false
    }
    fn has_flagm2(&self) -> bool {
        false
    }
    fn has_dit(&self) -> bool {
        false
    }
    fn has_ssbs(&self) -> bool {
        false
    }
    fn has_bti(&self) -> bool {
        false
    }
    fn has_pauth(&self) -> bool {
        false
    }
    fn has_pauth2(&self) -> bool {
        false
    }
    fn has_fpac(&self) -> bool {
        false
    }
    fn has_specres(&self) -> bool {
        false
    }
    fn has_specres2(&self) -> bool {
        false
    }
    fn has_csv2(&self) -> bool {
        false
    }
    fn has_csv3(&self) -> bool {
        false
    }
    fn has_ecv(&self) -> bool {
        false
    }
    fn has_sb(&self) -> bool {
        false
    }
    fn has_frintts(&self) -> bool {
        false
    }
    fn has_dpb(&self) -> bool {
        false
    }
    fn has_dpb2(&self) -> bool {
        false
    }
    fn has_dotprod(&self) -> bool {
        false
    }
    fn has_bf16(&self) -> bool {
        false
    }
    fn has_i8mm(&self) -> bool {
        false
    }
    fn has_sve(&self) -> bool {
        false
    }
    fn has_sve2(&self) -> bool {
        false
    }
    fn has_sme(&self) -> bool {
        false
    }
}

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
    detected.insert(
        "cpuid",
        cfg!(any(target_os = "android", target_os = "linux")),
    ); // Only on Linux/Android via HWCAP_CPUID

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_detected_features_empty() {
        let src = BTreeMap::new();
        let d = populate_detected_features(&src);
        assert!(!d["fp"]);
        assert!(!d["asimd"]);
        assert!(!d["aes"]);
    }

    #[test]
    fn test_populate_detected_features_full() {
        let mut src = BTreeMap::new();
        for k in BASE_FEATURES
            .iter()
            .chain(SIMD_FEATURES.iter())
            .chain(CRYPTO_FEATURES.iter())
            .chain(ATOMIC_FEATURES.iter())
            .chain(FP_FEATURES.iter())
            .chain(MISC_FEATURES.iter())
        {
            src.insert(k.to_string(), true);
        }
        let d = populate_detected_features(&src);
        assert!(d["fp"]);
        assert!(d["asimd"]);
        assert!(d["neon"]);
        assert!(d["aes"]);
        assert!(d["atomics"]);
        assert!(d["lse"]);
    }

    #[test]
    fn test_populate_detected_asimd_neon_alias() {
        let mut src = BTreeMap::new();
        src.insert("asimd".to_string(), true);
        let d = populate_detected_features(&src);
        assert!(d["asimd"]);
        assert!(d["neon"]);
    }

    #[test]
    fn test_populate_detected_neon_alias() {
        let mut src = BTreeMap::new();
        src.insert("neon".to_string(), true);
        let d = populate_detected_features(&src);
        assert!(d["asimd"]);
        assert!(d["neon"]);
    }

    #[test]
    fn test_populate_detected_atomics_lse_alias() {
        let mut src = BTreeMap::new();
        src.insert("atomics".to_string(), true);
        let d = populate_detected_features(&src);
        assert!(d["atomics"]);
        assert!(d["lse"]);
    }

    #[test]
    fn test_populate_detected_lse_alias() {
        let mut src = BTreeMap::new();
        src.insert("lse".to_string(), true);
        let d = populate_detected_features(&src);
        assert!(d["atomics"]);
        assert!(d["lse"]);
    }

    #[test]
    fn test_build_feature_map_empty() {
        let detected = BTreeMap::new();
        let map = build_feature_map(&detected);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_feature_map_all_false() {
        let mut detected = BTreeMap::new();
        for f in BASE_FEATURES.iter().chain(SIMD_FEATURES.iter()) {
            detected.insert(*f, false);
        }
        let map = build_feature_map(&detected);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_feature_map_base_only() {
        let mut detected = BTreeMap::new();
        detected.insert("fp", true);
        detected.insert("asimd", true);
        detected.insert("evtstrm", false);
        detected.insert("cpuid", false);
        let map = build_feature_map(&detected);
        assert!(map.contains_key("Base"));
        assert_eq!(map.get("Base"), Some(&String::from("fp asimd")));
    }

    #[test]
    fn test_build_feature_map_all_categories() {
        let mut detected = BTreeMap::new();
        detected.insert("fp", true);
        detected.insert("asimd", true);
        detected.insert("neon", true);
        detected.insert("aes", true);
        detected.insert("atomics", true);
        detected.insert("fphp", true);
        detected.insert("crc32", true);
        let map = build_feature_map(&detected);
        assert!(map.contains_key("Base"));
        assert!(map.contains_key("SIMD"));
        assert!(map.contains_key("Security"));
        assert!(map.contains_key("Atomics"));
        assert!(map.contains_key("Fp"));
        assert!(map.contains_key("Misc"));
    }

    #[test]
    fn test_normalize_feature_name() {
        assert_eq!(normalize_feature_name("FP"), "fp");
        assert_eq!(normalize_feature_name("Asimd"), "asimd");
        assert_eq!(normalize_feature_name("CRC32"), "crc32");
    }

    #[test]
    fn test_normalize_feature_name_already_lower() {
        assert_eq!(normalize_feature_name("aes"), "aes");
    }

    #[test]
    fn test_populate_detected_misc_features() {
        let mut src = BTreeMap::new();
        src.insert("crc32".to_string(), true);
        src.insert("dcpop".to_string(), false);
        src.insert("bti".to_string(), true);
        let d = populate_detected_features(&src);
        assert!(d["crc32"]);
        assert!(!d["dcpop"]);
        assert!(d["bti"]);
    }

    #[test]
    fn test_all_feature_constants_not_empty() {
        assert!(!BASE_FEATURES.is_empty());
        assert!(!SIMD_FEATURES.is_empty());
        assert!(!CRYPTO_FEATURES.is_empty());
        assert!(!ATOMIC_FEATURES.is_empty());
        assert!(!FP_FEATURES.is_empty());
        assert!(!MISC_FEATURES.is_empty());
    }
}
