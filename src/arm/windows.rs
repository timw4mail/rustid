//! Windows-specific ARM CPU feature detection.
//!
//! Uses `IsProcessorFeaturePresent` API and registry reads for feature detection.
//! Follows the existing Windows pattern in `src/arm/mod.rs` using the `windows` crate.

use std::collections::BTreeMap;
use windows::Win32::System::WindowsProgramming::*;

// ----------------------------------------------------------------------------
// Feature detection via IsProcessorFeaturePresent
// ----------------------------------------------------------------------------

/// Returns a BTreeMap of feature names to boolean values using Windows APIs.
pub fn get_features_from_api() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    // PF_ARM_NEON_INSTRUCTIONS_AVAILABLE = 19
    // PF_ARM_VFP_32_REGISTERS_AVAILABLE = 18
    // PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE = 24
    // PF_ARM_64BIT_LOAD_STORE_AVAILABLE = 25
    // PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE = 27
    // PF_ARM_FP_REGISTERS_32_AVAILABLE = 37
    // Reference: windows/Win32/System/WindowsProgramming.rs in windows crate

    const PF_ARM_NEON_INSTRUCTIONS_AVAILABLE: u32 = 19;
    const PF_ARM_VFP_32_REGISTERS_AVAILABLE: u32 = 18;
    const PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE: u32 = 24;
    const PF_ARM_64BIT_LOAD_STORE_AVAILABLE: u32 = 25;
    const PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE: u32 = 27;
    const PF_ARM_FP_REGISTERS_32_AVAILABLE: u32 = 37;

    unsafe {
        features.insert(
            "neon".to_string(),
            IsProcessorFeaturePresent(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE).as_bool(),
        );
        features.insert(
            "asimd".to_string(),
            IsProcessorFeaturePresent(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE).as_bool(),
        );
        features.insert(
            "fp".to_string(),
            IsProcessorFeaturePresent(PF_ARM_VFP_32_REGISTERS_AVAILABLE).as_bool(),
        );
        features.insert(
            "fmac".to_string(),
            IsProcessorFeaturePresent(PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE).as_bool(),
        );
        features.insert(
            "fp32regs".to_string(),
            IsProcessorFeaturePresent(PF_ARM_FP_REGISTERS_32_AVAILABLE).as_bool(),
        );
        features.insert(
            "ldst64".to_string(),
            IsProcessorFeaturePresent(PF_ARM_64BIT_LOAD_STORE_AVAILABLE).as_bool(),
        );
        features.insert(
            "div".to_string(),
            IsProcessorFeaturePresent(PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE).as_bool(),
        );
    }

    // Windows on ARM64 always has these features
    features.insert("cpuid".to_string(), true);
    features.insert("evtstrm".to_string(), true);
    features.insert("crc32".to_string(), true);
    features.insert("atomics".to_string(), true);
    features.insert("sha1".to_string(), true);
    features.insert("sha2".to_string(), true);
    features.insert("aes".to_string(), true);
    features.insert("pmull".to_string(), true);

    features
}

// ----------------------------------------------------------------------------
// Feature detection via Registry
// ----------------------------------------------------------------------------

/// Reads additional feature information from the Windows registry.
pub fn get_features_from_registry() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    use windows::Win32::System::Registry::*;
    use windows::core::w;

    let mut i = 0;
    loop {
        let subkey_str = format!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\{}", i);
        let subkey = windows::core::HSTRING::from(&subkey_str);
        let mut hkey = HKEY::default();

        let result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                windows::core::PCWSTR(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            )
        };

        if result.is_err() {
            break;
        }

        // Check for feature flags in registry
        // This is platform-specific and may vary
        // For now, just close the key and continue

        let _ = unsafe { RegCloseKey(hkey) };
        i += 1;
    }

    features
}

// ----------------------------------------------------------------------------
// Public API: has_* functions
// ----------------------------------------------------------------------------

/// Check if floating-point (fp) is supported.
pub fn has_fp() -> bool {
    let features = get_features_from_api();
    features.get("fp").copied().unwrap_or(false)
}

/// Check if Advanced SIMD (NEON/asimd) is supported.
pub fn has_simd() -> bool {
    let features = get_features_from_api();
    features.get("asimd").copied().unwrap_or(false)
}

/// Check if NEON is supported (alias for has_simd on ARM).
pub fn has_neon() -> bool {
    has_simd()
}

/// Check if AES instructions are supported.
pub fn has_aes() -> bool {
    let features = get_features_from_api();
    features.get("aes").copied().unwrap_or(false)
}

/// Check if SHA1 instructions are supported.
pub fn has_sha1() -> bool {
    let features = get_features_from_api();
    features.get("sha1").copied().unwrap_or(false)
}

/// Check if SHA2 instructions are supported.
pub fn has_sha2() -> bool {
    let features = get_features_from_api();
    features.get("sha2").copied().unwrap_or(false)
}

/// Check if SHA3 instructions are supported.
pub fn has_sha3() -> bool {
    // Windows on ARM doesn't expose SHA3 via IsProcessorFeaturePresent
    // Check registry or assume not present
    false
}

/// Check if SHA512 instructions are supported.
pub fn has_sha512() -> bool {
    // Windows on ARM doesn't expose SHA512 via IsProcessorFeaturePresent
    false
}

/// Check if CRC32 instructions are supported.
pub fn has_crc32() -> bool {
    let features = get_features_from_api();
    features.get("crc32").copied().unwrap_or(false)
}

/// Check if atomic instructions (LSE) are supported.
pub fn has_atomics() -> bool {
    let features = get_features_from_api();
    features.get("atomics").copied().unwrap_or(false)
}

// ----------------------------------------------------------------------------
// Get all features as a BTreeMap (for Cpu struct)
// ----------------------------------------------------------------------------

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();
    let api_features = get_features_from_api();
    let _registry_features = get_features_from_registry();

    // Base features
    detected.insert("fp", api_features.get("fp").copied().unwrap_or(false));
    detected.insert("asimd", api_features.get("asimd").copied().unwrap_or(false));
    detected.insert("cpuid", api_features.get("cpuid").copied().unwrap_or(false));
    detected.insert(
        "evtstrm",
        api_features.get("evtstrm").copied().unwrap_or(false),
    );

    // SIMD
    detected.insert("neon", api_features.get("neon").copied().unwrap_or(false));
    detected.insert("asimdhp", false);
    detected.insert("asimdfhm", false);
    detected.insert("asimddp", false);
    detected.insert("asimdrdm", false);

    // Crypto
    detected.insert("aes", api_features.get("aes").copied().unwrap_or(false));
    detected.insert("pmull", api_features.get("pmull").copied().unwrap_or(false));
    detected.insert("sha1", api_features.get("sha1").copied().unwrap_or(false));
    detected.insert("sha2", api_features.get("sha2").copied().unwrap_or(false));
    detected.insert("sha3", false);
    detected.insert("sha512", false);
    detected.insert("sm3", false);
    detected.insert("sm4", false);

    // Atomic
    detected.insert(
        "atomics",
        api_features.get("atomics").copied().unwrap_or(false),
    );
    detected.insert("lse", api_features.get("atomics").copied().unwrap_or(false));
    detected.insert("lse2", false);

    // FP
    detected.insert("fphp", false);
    detected.insert("fp16", false);
    detected.insert("fcma", false);
    detected.insert("jscvt", false);

    // Misc
    detected.insert("crc32", api_features.get("crc32").copied().unwrap_or(false));
    detected.insert("dcpop", false);
    detected.insert("lrcpc", false);
    detected.insert("lrcpc2", false);
    detected.insert("flagm", false);
    detected.insert("flagm2", false);
    detected.insert("dit", false);
    detected.insert("ssbs", false);
    detected.insert("bti", false);
    detected.insert("pauth", false);
    detected.insert("pauth2", false);
    detected.insert("fpac", false);
    detected.insert("specres", false);
    detected.insert("specres2", false);
    detected.insert("csv2", false);
    detected.insert("csv3", false);
    detected.insert("ecv", false);
    detected.insert("sb", false);
    detected.insert("frintts", false);
    detected.insert("dpb", false);
    detected.insert("dpb2", false);
    detected.insert("dotprod", false);
    detected.insert("bf16", false);
    detected.insert("i8mm", false);
    detected.insert("sve", false);
    detected.insert("sve2", false);
    detected.insert("sme", false);

    super::features::build_feature_map(&detected)
}
