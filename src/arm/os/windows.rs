//! Windows-specific ARM CPU feature detection.
//!
//! Uses `IsProcessorFeaturePresent` API and registry reads for feature detection.
//! Follows the existing Windows pattern in `src/arm/mod.rs` using the `windows` crate.

use super::OsCpuInfo;
use crate::arm::TArmFeatures;
use crate::arm::brand::{IMPL_ARM, Vendor};
use crate::arm::micro_arch::*;
use crate::common::DataSource;
use crate::common::os::TOSData;
use std::collections::{BTreeMap, HashSet};
use windows::Win32::System::Threading::*;

/// Windows-specific CPU detection via MRS and the registry.
pub fn detect() -> OsCpuInfo {
    let mut midrs: HashSet<Midr> = HashSet::new();
    let mut all_midrs: Vec<Midr> = Vec::new();
    let mut midr_source = DataSource::CpuLookupTable;

    crate::common::for_each_logical_core(|| {
        let midr_val = crate::arm::get_midr();
        let midr = Midr::new(midr_val);
        midrs.insert(midr);
        all_midrs.push(midr);
    });

    // On Windows, MRS is emulated. Try the registry for more accurate MIDRs.
    let windows_midrs = get_windows_midrs();
    if !windows_midrs.is_empty() {
        all_midrs.clear();
        midrs.clear();
        for m_val in windows_midrs {
            let midr = Midr::new(m_val);
            midrs.insert(midr);
            all_midrs.push(midr);
        }
        midr_source =
            DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0:CP 0");
    }

    let primary_midr = midrs.iter().next().copied().unwrap_or(Midr::default());
    let vendor: String = Vendor::from(primary_midr.implementer).into();
    let cpu_arch = CpuArch::find(
        primary_midr.implementer,
        primary_midr.part,
        primary_midr.variant,
    );
    let cores = super::detect_cores(&all_midrs);

    let model = crate::common::OS::get_soc()
        .and_then(|s| s.display_name())
        .unwrap_or_default();
    let raw = get_windows_raw_info();

    OsCpuInfo {
        midrs,
        vendor,
        cpu_arch,
        cores,
        model,
        raw,
        midr_source,
        features_source: DataSource::SystemCall,
    }
}

/// Query Windows registry for raw processor metadata.
pub fn get_windows_raw_info() -> BTreeMap<String, String> {
    use crate::common::os::windows::{read_reg_dword, read_reg_string};
    use windows::Win32::System::Registry::*;
    use windows::core::w;

    let mut raw = BTreeMap::new();
    let mut hkey = HKEY::default();
    let subkey = w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let result = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, None, KEY_READ, &mut hkey) };

    if result.is_ok() {
        if let Some(val) = read_reg_string(hkey, "ProcessorNameString") {
            raw.insert("ProcessorNameString".to_string(), val);
        }
        if let Some(val) = read_reg_string(hkey, "Identifier") {
            raw.insert("Identifier".to_string(), val);
        }
        if let Some(val) = read_reg_string(hkey, "VendorIdentifier") {
            raw.insert("VendorIdentifier".to_string(), val);
        }

        if let Some(mhz) = read_reg_dword(hkey, "~MHZ") {
            raw.insert("~MHZ".to_string(), format!("{mhz} MHz"));
        }

        let _ = unsafe { RegCloseKey(hkey) };
    }

    raw
}

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
// TArmFeatures implementation
// ----------------------------------------------------------------------------

impl TArmFeatures for crate::arm::ArmFeatures {
    fn has_fp(&self) -> bool {
        let features = get_features_from_api();
        features.get("fp").copied().unwrap_or(false)
    }

    fn has_asimd(&self) -> bool {
        let features = get_features_from_api();
        features.get("asimd").copied().unwrap_or(false)
    }

    fn has_aes(&self) -> bool {
        let features = get_features_from_api();
        features.get("aes").copied().unwrap_or(false)
    }

    fn has_sha1(&self) -> bool {
        let features = get_features_from_api();
        features.get("sha1").copied().unwrap_or(false)
    }

    fn has_sha2(&self) -> bool {
        let features = get_features_from_api();
        features.get("sha2").copied().unwrap_or(false)
    }

    fn has_crc32(&self) -> bool {
        let features = get_features_from_api();
        features.get("crc32").copied().unwrap_or(false)
    }

    fn has_atomics(&self) -> bool {
        let features = get_features_from_api();
        features.get("atomics").copied().unwrap_or(false)
    }
}

// ----------------------------------------------------------------------------
// Get all features as a BTreeMap (for Cpu struct)
// ----------------------------------------------------------------------------

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let src = get_features_from_api();
    let detected = crate::arm::features::populate_detected_features(&src);
    crate::arm::features::build_feature_map(&detected)
}

#[cfg(windows_os)]
pub fn get_windows_midrs() -> Vec<usize> {
    use crate::common::os::windows::{read_reg_dword, read_reg_qword};
    use windows::Win32::System::Registry::*;
    use windows::core::HSTRING;

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

        let midr = read_reg_qword(hkey, "CP 4000")
            .map(|q| q as usize)
            .or_else(|| read_reg_dword(hkey, "CPUID").map(|d| d as usize));

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
    synthetic_midr |= IMPL_ARM << IMPLEMENTER_OFFSET;
    synthetic_midr |= (sys_info.wProcessorLevel as usize & 0xFFF) << PART_OFFSET;
    synthetic_midr |= sys_info.wProcessorRevision as usize & REVISION_MASK;

    synthetic_midr
}
