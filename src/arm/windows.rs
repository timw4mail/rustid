//! Windows-specific ARM CPU feature detection.
//!
//! Uses `IsProcessorFeaturePresent` API and registry reads for feature detection.
//! Follows the existing Windows pattern in `src/arm/mod.rs` using the `windows` crate.

use std::collections::BTreeMap;
use windows::Win32::System::Threading::*;

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
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "asimd".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "fp".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_VFP_32_REGISTERS_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "fmac".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "fp32regs".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_FP_REGISTERS_32_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "ldst64".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_64BIT_LOAD_STORE_AVAILABLE))
                .as_bool(),
        );
        features.insert(
            "div".to_string(),
            IsProcessorFeaturePresent(PROCESSOR_FEATURE_ID(PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE))
                .as_bool(),
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

#[cfg(target_os = "windows")]
pub fn get_windows_midrs() -> Vec<usize> {
    use std::mem::size_of;
    use windows::Win32::System::Registry::*;
    use windows::core::{HSTRING, w};

    let mut midrs = Vec::new();
    let mut i = 0;

    loop {
        let subkey_str = format!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\{}", i);
        let subkey = HSTRING::from(&subkey_str);
        let mut hkey = HKEY::default();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                windows::core::PCWSTR(subkey.as_ptr()),
                None,
                KEY_READ,
                &mut hkey,
            )
        };

        if result.is_err() {
            break;
        }

        let mut midr = None;

        // 1. Try 'CP 4000' (REG_QWORD)
        let mut cpu_id_qword: u64 = 0;
        let mut size_qword = size_of::<u64>() as u32;
        let mut dw_type = REG_NONE;
        let value_name_4000 = w!("CP 4000");
        let query_4000 = unsafe {
            RegQueryValueExW(
                hkey,
                value_name_4000,
                None,
                Some(&mut dw_type),
                Some(&mut cpu_id_qword as *mut u64 as *mut u8),
                Some(&mut size_qword),
            )
        };

        if query_4000.is_ok() && dw_type == REG_QWORD {
            midr = Some(cpu_id_qword as usize);
        } else {
            // 2. Fallback to 'CPUID' (REG_DWORD)
            let mut cpu_id_dword: u32 = 0;
            let mut size_dword = size_of::<u32>() as u32;
            let value_name_cpuid = w!("CPUID");
            let query_cpuid = unsafe {
                RegQueryValueExW(
                    hkey,
                    value_name_cpuid,
                    None,
                    Some(&mut dw_type),
                    Some(&mut cpu_id_dword as *mut u32 as *mut u8),
                    Some(&mut size_dword),
                )
            };

            if query_cpuid.is_ok() && dw_type == REG_DWORD {
                midr = Some(cpu_id_dword as usize);
            }
        }

        let _ = unsafe { RegCloseKey(hkey) };

        if let Some(m) = midr {
            midrs.push(m);
        } else {
            // If we can't find MIDR for this core, but it exists in registry,
            // we might have reached the end of useful info or just missing one.
            // For now, continue to see if others exist.
        }

        i += 1;
    }

    midrs
}

pub fn get_synth_midr() -> usize {
    let midrs = get_windows_midrs();
    if !midrs.is_empty() {
        return midrs[0];
    }

    // Fallback to GetNativeSystemInfo if registry fails
    use std::mem::zeroed;
    use windows::Win32::System::SystemInformation::*;

    let mut sys_info: SYSTEM_INFO = unsafe { zeroed() };
    unsafe {
        GetNativeSystemInfo(&mut sys_info);
    }

    let mut synthetic_midr: usize = 0;
    synthetic_midr |= 0x41_usize << 24;
    synthetic_midr |= (sys_info.wProcessorLevel as usize & 0xFFF) << 4;
    synthetic_midr |= sys_info.wProcessorRevision as usize & 0xF;

    synthetic_midr
}
