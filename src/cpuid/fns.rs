//! CPUID Helpers
//!
//! This module provides low-level CPUID functions for x86/x86_64 processors.

use super::FeatureList;
use super::x86_cpuid;

use crate::cpuid::brand::CpuBrand;
#[allow(unused_imports)]
use core::arch::asm;
use heapless::String;

/// CPUID leaf 0x00000000 - Maximum basic leaf
pub const LEAF_0: u32 = 0;
/// CPUID leaf 0x00000001 - Processor info and feature flags
pub const LEAF_1: u32 = 1;
/// CPUID leaf 0x00000002 - Cache descriptors
pub const LEAF_2: u32 = 2;
/// CPUID leaf 0x00000007 - Extended feature flags
pub const LEAF_7: u32 = 7;

/// Extended CPUID leaf 0x80000000 - Maximum extended leaf
pub const EXT_LEAF_0: u32 = 0x8000_0000;
/// Extended CPUID leaf 0x80000001 - Extended processor info
pub const EXT_LEAF_1: u32 = 0x8000_0001;

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
            asm!(
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

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    false
}

/// Returns true if the CPU is a Cyrix processor (detected without CPUID).
///
/// Cyrix processors are unique in that they do not modify flags during a `div`
/// instruction, whereas other x86 processors do.
///
/// Verified on real hardware
pub fn is_cyrix() -> bool {
    #[cfg(target_arch = "x86_64")]
    return false;

    #[cfg(target_arch = "x86")]
    {
        let flags: u8;
        unsafe {
            asm!(
                "xor ax, ax",
                "sahf",         // Clear flags (SF, ZF, AF, PF, CF)
                "mov ax, 5",
                "mov bx, 2",
                "div bl",       // Cyrix CPUs do not modify flags on 'div'
                "lahf",         // Load flags into AH
                out("ah") flags,
                out("al") _,
                out("bx") _,
            );
        }
        // Cyrix: flags (SF, ZF, AF, PF, CF) remain unchanged (0).
        // Mask 0xD5 (11010101b) selects these flags.
        (flags & 0xD5) == 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    false
}

/// Returns true if the CPU is a 386-class processor.
///
/// This is determined by checking if the AC (Alignment Check) flag in EFLAGS
/// can be toggled. 386 CPUs do not support this, while 486 and newer do.
///
/// Verified on real hardware
pub fn is_386() -> bool {
    !is_ac_flag_supported()
}

/// Returns true if the CPU is a 486-class processor, without CPUID support
///
/// Verified on real hardware
pub fn is_486() -> bool {
    is_ac_flag_supported() && !has_cpuid()
}

/// Helper to check for AC flag support in EFLAGS register.
fn is_ac_flag_supported() -> bool {
    #[cfg(target_arch = "x86_64")]
    return true; // 64-bit CPUs are much newer than 486

    #[cfg(target_arch = "x86")]
    {
        let supported: u32;
        unsafe {
            asm!(
                "pushfd",
                "pop eax",
                "mov ecx, eax",
                "xor eax, 0x40000", // Toggle AC flag (bit 18)
                "push eax",
                "popfd",
                "pushfd",
                "pop eax",
                "push ecx",
                "popfd",
                "xor eax, ecx",
                "and eax, 0x40000",
                out("eax") supported,
                out("ecx") _,
            );
        }
        supported != 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    false
}

/// Returns the maximum basic CPUID leaf supported.
pub fn max_leaf() -> u32 {
    x86_cpuid(LEAF_0).eax
}

/// Returns the maximum extended CPUID leaf supported.
pub fn max_extended_leaf() -> u32 {
    x86_cpuid(EXT_LEAF_0).eax
}

/// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
/// Returns a 12-character vendor string from CPUID leaf 0.
pub fn vendor_str() -> String<12> {
    let mut s = String::new();
    if !has_cpuid() && is_cyrix() {
        let _ = s.push_str(crate::cpuid::brand::VENDOR_CYRIX);
        return s;
    }

    let res = x86_cpuid(0);
    let mut bytes = [0u8; 12];

    bytes[0..4].copy_from_slice(&res.ebx.to_le_bytes());
    bytes[4..8].copy_from_slice(&res.edx.to_le_bytes());
    bytes[8..12].copy_from_slice(&res.ecx.to_le_bytes());

    for &b in &bytes {
        if b != 0 {
            let _ = s.push(b as char);
        }
    }

    s
}

/// Returns the number of logical cores.
pub fn logical_cores() -> u32 {
    if max_leaf() < 1 {
        return 1;
    }

    let res = x86_cpuid(1);
    let count = (res.ebx >> 16) & 0xFF;
    if count == 0 { 1 } else { count }
}

// ------------------------------------------------------------------------
// ! CPU Feature Lookups
// ------------------------------------------------------------------------

/// Represents a CPUID register.
#[allow(unused)]
enum Reg {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

/// Checks if a specific feature bit is set in the given CPUID leaf.
fn has_feature(leaf: u32, register: Reg, bit: u32) -> bool {
    if leaf >= EXT_LEAF_0 && max_extended_leaf() < leaf {
        return false;
    }

    if max_leaf() < leaf {
        return false;
    }

    match register {
        Reg::Eax => (x86_cpuid(leaf).eax & (1 << bit)) != 0,
        Reg::Ebx => (x86_cpuid(leaf).ebx & (1 << bit)) != 0,
        Reg::Ecx => (x86_cpuid(leaf).ecx & (1 << bit)) != 0,
        Reg::Edx => (x86_cpuid(leaf).edx & (1 << bit)) != 0,
    }
}

// ------------------------------------------------------------------------
// ! Leaf 0000_0001h
// ------------------------------------------------------------------------

/// Returns true if the CPU has a Floating Point Unit (FPU).
pub fn has_fpu() -> bool {
    has_feature(LEAF_1, Reg::Edx, 0)
}

/// Returns true if the CPU has a Time Stamp Counter (TSC).
pub fn has_tsc() -> bool {
    has_feature(LEAF_1, Reg::Edx, 4)
}

/// Returns true if the CPU supports MMX instructions.
pub fn has_mmx() -> bool {
    has_feature(LEAF_1, Reg::Edx, 23)
}

/// Returns true if the CPU supports CMOV instructions.
pub fn has_cmov() -> bool {
    has_feature(LEAF_1, Reg::Edx, 15)
}

/// Returns true if the CPU supports CMPXCHG8B instruction.
pub fn has_cx8() -> bool {
    has_feature(LEAF_1, Reg::Edx, 8)
}

/// Returns true if the CPU supports SSE instructions.
pub fn has_sse() -> bool {
    has_feature(LEAF_1, Reg::Edx, 25)
}

/// Returns true if the CPU supports SSE2 instructions.
pub fn has_sse2() -> bool {
    has_feature(LEAF_1, Reg::Edx, 26)
}

/// Returns true if the CPU supports Hyper-Threading.
pub fn has_ht() -> bool {
    has_feature(LEAF_1, Reg::Edx, 28)
}

/// Returns true if the CPU supports SSE3 instructions.
pub fn has_sse3() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 0)
}

/// Returns true if the CPU supports SSSE3 instructions.
pub fn has_ssse3() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 9)
}

/// Returns true if the CPU supports FMA (Fused Multiply Add).
pub fn has_fma() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 12)
}

/// Returns true if the CPU supports CMPXCHG16B instruction.
pub fn has_cx16() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 13)
}

/// Returns true if the CPU supports SSE4.1 instructions.
pub fn has_sse41() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 19)
}

/// Returns true if the CPU supports SSE4.2 instructions.
pub fn has_sse42() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 20)
}

/// Returns true if the CPU supports POPCNT instruction.
pub fn has_popcnt() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 23)
}

/// Returns true if the CPU supports AVX instructions.
pub fn has_avx() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 28)
}

/// Returns true if the CPU supports F16C (Half-precision floating point).
pub fn has_f16c() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 29)
}

/// Returns true if the CPU supports RDRAND instruction.
pub fn has_rdrand() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 30)
}

// ----------------------------------------------------------------------------
// ! Leaf 0000_0007h - Extended feature flags
// ----------------------------------------------------------------------------

/// Returns true if the CPU supports AVX2 instructions.
pub fn has_avx2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 5)
}

/// Returns true if the CPU supports AVX-512 Foundation instructions.
pub fn has_avx512f() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 16)
}

/// Returns true if the CPU supports BMI1 (Bit Manipulation Instructions).
pub fn has_bmi1() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 3)
}

/// Returns true if the CPU supports BMI2 instructions.
pub fn has_bmi2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 8)
}

// ----------------------------------------------------------------------------
// ! Leaf 8000_0001h - Extended features
// ----------------------------------------------------------------------------

/// Returns true if the CPU supports SSE4A instructions (AMD-specific).
pub fn has_sse4a() -> bool {
    if CpuBrand::detect() != CpuBrand::AMD {
        return false;
    }

    has_feature(EXT_LEAF_1, Reg::Ecx, 6)
}

/// Returns true if the CPU supports AMD64 (x86-64) instructions.
pub fn has_amd64() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 29)
}

/// Returns true if the CPU supports 3DNow!+ instructions.
pub fn has_3dnow_plus() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 30)
}

/// Returns true if the CPU supports 3DNow! instructions.
pub fn has_3dnow() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 31)
}

/// Get the full list of detected features.
pub fn get_feature_list() -> FeatureList {
    use heapless::Vec;

    let mut out: Vec<&'static str, 64> = Vec::new();

    if has_fpu() {
        let _ = out.push("FPU");
    };
    if has_tsc() {
        let _ = out.push("TSC");
    }
    if has_cx8() {
        let _ = out.push("CMPXCHG8B");
    };
    if has_cx16() {
        let _ = out.push("CMPXCHG16B");
    }
    if has_cmov() {
        let _ = out.push("CMOV");
    };
    if has_mmx() {
        let _ = out.push("MMX");
    };
    if has_3dnow() {
        let _ = out.push("3DNow!");
    };
    if has_3dnow_plus() {
        let _ = out.push("3DNow+");
    };
    if has_ht() {
        let _ = out.push("HT");
    };
    if has_amd64() {
        let _ = out.push("AMD64");
    };
    if has_sse() {
        let _ = out.push("SSE");
    };
    if has_sse2() {
        let _ = out.push("SSE2");
    };
    if has_sse3() {
        let _ = out.push("SSE3");
    };
    if has_sse4a() {
        let _ = out.push("SSE4A");
    };
    if has_sse41() {
        let _ = out.push("SSE4.1");
    };
    if has_sse42() {
        let _ = out.push("SSE4.2");
    };
    if has_ssse3() {
        let _ = out.push("SSSE3");
    };
    if has_avx() {
        let _ = out.push("AVX");
    };
    if has_avx2() {
        let _ = out.push("AVX2");
    };
    if has_avx512f() {
        let _ = out.push("AVX512F");
    };
    if has_fma() {
        let _ = out.push("FMA");
    };
    if has_bmi1() {
        let _ = out.push("BMI1");
    };
    if has_bmi2() {
        let _ = out.push("BMI2");
    };
    if has_rdrand() {
        let _ = out.push("RDRAND");
    };
    if has_popcnt() {
        let _ = out.push("POPCNT");
    };
    if has_f16c() {
        let _ = out.push("F16C");
    };

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_str() {
        let vendor = vendor_str();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }
}
