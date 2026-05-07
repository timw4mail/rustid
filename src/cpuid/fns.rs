// ------------------------------------------------------------------------
// ! CPU Feature Lookups
// ------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid_count, CpuidResult};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__cpuid_count, CpuidResult};

use super::constants::*;

#[cfg(target_arch = "x86")]
use super::quirks::get_vendor_by_quirk;

use super::is_hypervisor_guest;
use alloc::string::String;

/// Represents the result of a CPUID instruction call.
///
/// The CPUID instruction returns processor identification and feature information
/// in the EAX, EBX, ECX, and EDX registers.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Cpuid {
    /// EAX register value
    pub eax: u32,
    /// EBX register value
    pub ebx: u32,
    /// ECX register value
    pub ecx: u32,
    /// EDX register value
    pub edx: u32,
}

impl From<CpuidResult> for Cpuid {
    fn from(res: CpuidResult) -> Self {
        Self {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        }
    }
}

/// Calls CPUID with the given leaf (EAX).
pub fn x86_cpuid(leaf: u32) -> Cpuid {
    x86_cpuid_count(leaf, 0)
}

#[inline]
pub(crate) fn real_x86_cpuid_count(leaf: u32, sub_leaf: u32) -> Cpuid {
    if !has_cpuid() {
        return Cpuid::default();
    }

    #[allow(unused_unsafe)]
    unsafe {
        __cpuid_count(leaf, sub_leaf).into()
    }
}

/// Calls CPUID with the given leaf (EAX) and sub-leaf (ECX).
pub fn x86_cpuid_count(leaf: u32, sub_leaf: u32) -> Cpuid {
    #[cfg(target_os = "none")]
    return real_x86_cpuid_count(leaf, sub_leaf);

    #[cfg(not(target_os = "none"))]
    super::provider::cpuid_count(leaf, sub_leaf)
}

#[cfg(not(target_os = "none"))]
pub fn info_source() -> super::provider::CpuidInfoSource {
    super::provider::info_source()
}

/// Returns true if the CPUID instruction is supported.
///
/// Verified on real hardware
pub fn has_cpuid() -> bool {
    #[cfg(target_arch = "x86_64")]
    return true;

    #[cfg(target_arch = "x86")]
    {
        let supported: u32;
        unsafe {
            core::arch::asm!(
            "pushfd",
            "pop eax",
            "mov ecx, eax",
            "xor eax, 0x200000",
            "push eax",
            "popfd",
            "pushfd",
            "pop eax",
            "push ecx",
            "popfd",
            "xor eax, ecx",
            "and eax, 0x200000",
            out("eax") supported,
            out("ecx") _,
            );
        }
        supported != 0
    }
}

/// Returns the maximum basic CPUID leaf supported.
pub fn max_leaf() -> u32 {
    x86_cpuid(LEAF_0).eax
}

pub fn max_hypervisor_leaf() -> u32 {
    x86_cpuid(HYP_LEAF_0).eax
}

/// Returns the maximum extended CPUID leaf supported.
pub fn max_extended_leaf() -> u32 {
    x86_cpuid(EXT_LEAF_0).eax
}

/// Returns the maximum vendor-specific CPUID leaf, if one exists,
/// otherwise returns the maximum extended leaf.
pub fn max_vendor_leaf() -> u32 {
    match &*vendor_str() {
        VENDOR_CENTAUR => x86_cpuid(CENTAUR_LEAF_0).eax,
        VENDOR_TRANSMETA => x86_cpuid(TRANSMETA_LEAF_0).eax,
        _ => max_extended_leaf(),
    }
}

/// Returns true if the given leaf is valid and supported by the CPU.
///
/// Checks whether the specified CPUID leaf is within the range supported by
/// the processor (basic, extends, or vendor-specific).
pub fn is_valid_leaf(leaf: u32) -> bool {
    if !has_cpuid() {
        return false;
    }

    // Optimization: Leaf 1 is always valid if CPUID is present.
    // This avoids redundant max_leaf() -> x86_cpuid(0) calls for common features.
    if leaf == LEAF_1 {
        return true;
    }

    match leaf {
        0..EXT_LEAF_0 => leaf <= max_leaf(),
        EXT_LEAF_0..=EXT_LEAF_MAX => leaf <= max_extended_leaf(),
        _ => leaf <= max_vendor_leaf(),
    }
}

/// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
///
/// Returns a 12-character vendor string from CPUID leaf 0.
pub fn vendor_str() -> String {
    #[cfg(target_arch = "x86")]
    if !has_cpuid() {
        return String::from(get_vendor_by_quirk());
    }

    raw_vendor_str(LEAF_0)
}

pub fn hypervisor_str() -> String {
    if is_hypervisor_guest() {
        raw_vendor_str(HYP_LEAF_0)
    } else {
        String::from(UNK)
    }
}

fn raw_vendor_str(leaf: u32) -> String {
    let res = x86_cpuid(leaf);
    let mut bytes = [0u8; 12];

    bytes[0..4].copy_from_slice(&res.ebx.to_le_bytes());
    bytes[4..8].copy_from_slice(&res.edx.to_le_bytes());
    bytes[8..12].copy_from_slice(&res.ecx.to_le_bytes());

    let s = core::str::from_utf8(&bytes)
        .unwrap_or(UNK)
        .trim_matches('\0');

    String::from(s)
}

pub fn read_multi_leaf_str(min_leaf: u32, max_leaf: u32) -> String {
    if !is_valid_leaf(max_leaf) {
        return String::from(UNK);
    }

    let mut model = String::new();
    for leaf in min_leaf..=max_leaf {
        let res = x86_cpuid(leaf);
        for reg in [res.eax, res.ebx, res.ecx, res.edx] {
            let bytes = reg.to_le_bytes();
            let s = core::str::from_utf8(&bytes).unwrap_or("");
            model.push_str(s);
        }
    }

    String::from(model.trim().trim_matches('\0'))
}

fn is_vendor(v: &str) -> bool {
    vendor_str() == v
}

/// Returns true if the CPU is from AMD.
pub fn is_amd() -> bool {
    is_vendor(VENDOR_AMD)
}

/// Returns true if the CPU is from Centaur (IDT/VIA/Zhaoxin).
pub fn is_centaur() -> bool {
    is_vendor(VENDOR_CENTAUR)
}

/// Returns true if the CPU is from Cyrix.
pub fn is_cyrix() -> bool {
    is_vendor(VENDOR_CYRIX) || is_vendor(VENDOR_NSC)
}

/// Is the CPU a Vortex86 or very similar RDC chip?
pub fn is_vortex() -> bool {
    is_vendor(VENDOR_DMP) || is_vendor(VENDOR_RDC)
}

/// Returns true if the CPU is from Intel.
pub fn is_intel() -> bool {
    is_vendor(VENDOR_INTEL)
}

pub fn is_umc() -> bool {
    is_vendor(VENDOR_UMC)
}

/// Returns true if the CPU is from Zhaoxin.
pub fn is_zhaoxin() -> bool {
    is_vendor(VENDOR_ZHAOXIN)
}

/// Returns true if the CPU is an Intel Overdrive processor.
///
/// Checks the Overdrive bit (EAX bit 12) in CPUID leaf 1.
pub fn is_overdrive() -> bool {
    (x86_cpuid(LEAF_1).eax & (1 << 12)) != 0
}

/// Returns the number of logical cores.
pub fn logical_cores() -> u32 {
    // Since AMD has a handy flag for getting logical cores,
    // try that first
    if is_amd() {
        if is_valid_leaf(EXT_LEAF_8) {
            let res = x86_cpuid(EXT_LEAF_8);
            let count = (res.ecx & 0xFF) + 1;
            if count > 1 {
                return count;
            }
        }

        if max_leaf() > 0 {
            let res = x86_cpuid(1);
            let count = (res.ebx >> 16) & 0xFF;
            if count != 0 {
                return count;
            }
        }
    }

    1
}

// ------------------------------------------------------------------------
// ! Leaf 0000_0001h
// ------------------------------------------------------------------------

/// Returns the CPU brand ID from basic leaf 0x00000001.
///
/// The brand ID is a value that identifies the specific CPU brand variant.
pub fn get_brand_id() -> u32 {
    let res = x86_cpuid(LEAF_1);
    res.ebx & 0xFF
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpuid::vendor_str;

    #[test]
    fn test_from_cpuid_result_for_cpu_info() {
        let cpuid_result = CpuidResult {
            eax: 10,
            ebx: 20,
            ecx: 30,
            edx: 40,
        };
        let cpu_info: Cpuid = cpuid_result.into();
        assert_eq!(cpu_info.eax, 10);
        assert_eq!(cpu_info.ebx, 20);
        assert_eq!(cpu_info.ecx, 30);
        assert_eq!(cpu_info.edx, 40);
    }

    #[test]
    fn test_vendor_str() {
        let vendor = vendor_str();
        assert!(!vendor.is_empty());
    }
}
