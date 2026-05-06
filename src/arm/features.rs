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
    "fpac", "speces", "specres2", "csv2", "csv3", "ecv", "sb", "frintts", "dpb", "dpb2", "dotprod",
    "bf16", "i8mm", "sve", "sve2", "sve2p1", "sme", "sme2", "sme2p1", "hbc", "mops", "the", "smep",
    "smap", "5lvl",
];

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
// Public API (individual has_* functions)
// ----------------------------------------------------------------------------

/// Check if a feature is present (platform-specific, implemented per platform)
#[cfg(target_os = "macos")]
pub use super::apple::{
    has_aes, has_atomics, has_crc32, has_fp, has_neon, has_sha1, has_sha2, has_sha3, has_sha512,
    has_simd,
};

#[cfg(target_os = "linux")]
pub use super::linux::{
    has_aes, has_atomics, has_crc32, has_fp, has_neon, has_sha1, has_sha2, has_sha3, has_sha512,
    has_simd,
};

#[cfg(target_os = "windows")]
pub use super::windows::{
    has_aes, has_atomics, has_crc32, has_fp, has_neon, has_sha1, has_sha2, has_sha3, has_sha512,
    has_simd,
};

// ----------------------------------------------------------------------------
// Aggregator: get_feature_list()
// ----------------------------------------------------------------------------

/// Returns a map of feature categories to space-separated feature strings.
/// Mirrors `src/cpuid/fns.rs::get_feature_list()` but with ARM-specific groups.
pub fn get_feature_list() -> BTreeMap<&'static str, String> {
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();

    // Base
    detected.insert("fp", has_fp());
    detected.insert("asimd", has_simd());
    detected.insert("cpuid", cfg!(target_os = "linux")); // Only on Linux via HWCAP_CPUID

    // SIMD
    detected.insert("neon", has_neon());
    detected.insert("asimdhp", false); // Platform-specific detection needed
    detected.insert("asimdfhm", false);
    detected.insert("asimddp", false);
    detected.insert("asimdrdm", false);

    // Crypto
    detected.insert("aes", has_aes());
    detected.insert("sha1", has_sha1());
    detected.insert("sha2", has_sha2());
    detected.insert("sha3", has_sha3());
    detected.insert("sha512", has_sha512());
    detected.insert("pmull", false);
    detected.insert("sm3", false);
    detected.insert("sm4", false);

    // Atomic
    detected.insert("atomics", has_atomics());
    detected.insert("lse", has_atomics()); // LSE is the ARMv8.1 atomics
    detected.insert("lse2", false);

    // FP
    detected.insert("fphp", false);
    detected.insert("fp16", false);
    detected.insert("fcma", false);
    detected.insert("jscvt", false);

    build_feature_map(&detected)
}
