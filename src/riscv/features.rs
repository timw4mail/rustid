//! RISC-V CPU feature detection.
//!
//! Features are derived from RISC-V ISA extension letters parsed from
//! `/proc/cpuinfo` or the `misa` CSR.

use std::collections::BTreeMap;

// ----------------------------------------------------------------------------
// Feature categories (RISC-V ISA extensions)
// ----------------------------------------------------------------------------

/// Base integer ISA (always present)
pub const BASE_FEATURES: &[&str] = &["rv32i", "rv64i"];

/// Multiply/divide extension
pub const MUL_FEATURES: &[&str] = &["m"];

/// Atomic extension
pub const ATOMIC_FEATURES: &[&str] = &["a"];

/// Single-precision floating point
pub const FP_FEATURES: &[&str] = &["f", "d", "q", "zfh", "zfhmin"];

/// Compressed instruction extension
pub const COMPRESSED_FEATURES: &[&str] = &["c"];

/// Compressed/pointer extensions (Zc*)
pub const ZC_FEATURES: &[&str] = &["zca", "zcb", "zcmp", "zcmt"];

/// Bit manipulation extensions
pub const BITMANIP_FEATURES: &[&str] = &["zba", "zbb", "zbc", "zbs"];

/// Vector extension
pub const VECTOR_FEATURES: &[&str] = &["v", "zvfh", "zvfhmin", "zvbb", "zvbc"];

/// Scalar crypto extensions
pub const CRYPTO_FEATURES: &[&str] = &[
    "zkne", "zknd", "zksed", "zksh", "zknh", "zkn", "zks", "zbkb", "zkc", "zbkx",
];

/// Atomic max/min extension
pub const AMO_FEATURES: &[&str] = &["zacas"];

/// Supervisor/user/hypervisor extensions
pub const PRIV_FEATURES: &[&str] = &["s", "u", "h"];

/// Cache/block management extensions
pub const CACHE_FEATURES: &[&str] = &["zicbom", "zicbop", "zicboz"];

/// Memory model and virtual memory extensions
pub const MEM_FEATURES: &[&str] = &["ztso", "zama16b", "svinval", "svnapot", "svpbmt"];

/// Miscellaneous extensions
pub const MISC_FEATURES: &[&str] = &["zicsr", "zifencei", "zicntr", "zihintpause", "zihintntl"];

/// Populate a detected features map from a platform source map.
/// Handles multi-letter extensions (e.g. "zba", "zkne") directly.
pub fn populate_detected_features(src: &BTreeMap<String, bool>) -> BTreeMap<&'static str, bool> {
    let mut d: BTreeMap<&'static str, bool> = BTreeMap::new();

    let all_features: &[&str] = &[
        "m",
        "a",
        "f",
        "d",
        "q",
        "c",
        "v",
        "zfh",
        "zfhmin",
        "zvfh",
        "zvfhmin",
        "zvbb",
        "zvbc",
        "zba",
        "zbb",
        "zbc",
        "zbs",
        "zkne",
        "zknd",
        "zksed",
        "zksh",
        "zknh",
        "zkn",
        "zks",
        "zbkb",
        "zkc",
        "zbkx",
        "zca",
        "zcb",
        "zcmp",
        "zcmt",
        "zacas",
        "ztso",
        "zama16b",
        "svinval",
        "svnapot",
        "svpbmt",
        "s",
        "u",
        "h",
        "zicbom",
        "zicbop",
        "zicboz",
        "zicsr",
        "zifencei",
        "zicntr",
        "zihintpause",
        "zihintntl",
    ];

    for feat in all_features {
        d.insert(feat, src.get(*feat).copied().unwrap_or(false));
    }

    d
}

/// Returns a BTreeMap of feature categories to space-separated feature strings.
pub fn build_feature_map(
    detected: &BTreeMap<&'static str, bool>,
) -> BTreeMap<&'static str, String> {
    let mut result: BTreeMap<&'static str, String> = BTreeMap::new();

    let categories: &[(&str, &[&str])] = &[
        ("Mul", MUL_FEATURES),
        ("Atomic", ATOMIC_FEATURES),
        ("Float", FP_FEATURES),
        ("Compressed", COMPRESSED_FEATURES),
        ("Zc", ZC_FEATURES),
        ("Bitmanip", BITMANIP_FEATURES),
        ("Vector", VECTOR_FEATURES),
        ("ScalarCrypto", CRYPTO_FEATURES),
        ("AmoMaxMin", AMO_FEATURES),
        ("Priv", PRIV_FEATURES),
        ("Cache", CACHE_FEATURES),
        ("MemModel", MEM_FEATURES),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_detected_features_empty() {
        let src = BTreeMap::new();
        let d = populate_detected_features(&src);
        assert!(!d["m"]);
        assert!(!d["a"]);
        assert!(!d["f"]);
    }

    #[test]
    fn test_populate_detected_features_full() {
        let mut src = BTreeMap::new();
        for k in &["m", "a", "f", "d", "c", "v", "zba", "zbb"] {
            src.insert(k.to_string(), true);
        }
        let d = populate_detected_features(&src);
        assert!(d["m"]);
        assert!(d["a"]);
        assert!(d["f"]);
        assert!(d["d"]);
        assert!(d["c"]);
        assert!(d["v"]);
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
        for f in MUL_FEATURES.iter().chain(ATOMIC_FEATURES.iter()) {
            detected.insert(*f, false);
        }
        let map = build_feature_map(&detected);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_feature_map_mul_only() {
        let mut detected = BTreeMap::new();
        detected.insert("m", true);
        let map = build_feature_map(&detected);
        assert!(map.contains_key("Mul"));
        assert_eq!(map.get("Mul"), Some(&String::from("m")));
    }

    #[test]
    fn test_build_feature_map_multiple_categories() {
        let mut detected = BTreeMap::new();
        detected.insert("m", true);
        detected.insert("a", true);
        detected.insert("f", true);
        detected.insert("c", true);
        let map = build_feature_map(&detected);
        assert!(map.contains_key("Mul"));
        assert!(map.contains_key("Atomic"));
        assert!(map.contains_key("Float"));
        assert!(map.contains_key("Compressed"));
    }
}
