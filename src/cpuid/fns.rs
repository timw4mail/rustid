//! CPUID Function Wrappers

use super::x86_cpuid;

use crate::cpuid::brand::CpuBrand;
#[allow(unused_imports)]
use core::arch::asm;

pub const LEAF_0: u32 = 0;
pub const LEAF_1: u32 = 1;
pub const LEAF_7: u32 = 7;

pub const EXT_LEAF_0: u32 = 0x8000_0000;
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

pub fn max_leaf() -> u32 {
    x86_cpuid(LEAF_0).eax
}

pub fn max_extended_leaf() -> u32 {
    x86_cpuid(EXT_LEAF_0).eax
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

fn has_feature(leaf: u32, register: char, bit: u32) -> bool {
    if leaf >= EXT_LEAF_0 && max_extended_leaf() < leaf {
        return false
    }

    if max_leaf() < leaf {
        return false;
    }

    match register {
        'b' => (x86_cpuid(leaf).ebx & (1 << bit)) != 0,
        'd' => (x86_cpuid(leaf).edx & (1 << bit)) != 0,
        'c' => (x86_cpuid(leaf).ecx & (1 << bit)) != 0,
        _ => (x86_cpuid(leaf).eax & (1 << bit)) != 0,
    }
}

// ------------------------------------------------------------------------
// ! Leaf 0000_0001h
// ------------------------------------------------------------------------

pub fn has_fpu() -> bool {
    has_feature(LEAF_1, 'd', 0)
}

pub fn has_tsc() -> bool {
    has_feature(LEAF_1, 'd', 4)
}

pub fn has_mmx() -> bool {
    has_feature(LEAF_1, 'd', 23)
}

pub fn has_cmov() -> bool {
    has_feature(LEAF_1, 'd', 15)
}

pub fn has_fcmov() -> bool {
    has_fpu() && has_cmov()
}

pub fn has_cx8() -> bool {
    has_feature(LEAF_1, 'd', 8)
}

pub fn has_sse() -> bool {
    has_feature(LEAF_1, 'd', 25)
}

pub fn has_sse2() -> bool {
    has_feature(LEAF_1, 'd', 26)
}

pub fn has_ht() -> bool {
    has_feature(LEAF_1, 'd', 28)
}

pub fn has_sse3() -> bool {
    has_feature(LEAF_1, 'c', 0)
}

pub fn has_pclmulqdq() -> bool {
    has_feature(LEAF_1, 'c', 1)
}

pub fn has_ssse3() -> bool {
    has_feature(LEAF_1, 'c', 9)
}

pub fn has_fma() -> bool {
    has_feature(LEAF_1, 'c', 12)
}

pub fn has_cx16() -> bool {
    has_feature(LEAF_1, 'c', 13)
}

pub fn has_sse41() -> bool {
    has_feature(LEAF_1, 'c', 19)
}

pub fn has_sse42() -> bool {
    has_feature(LEAF_1, 'c', 20)
}

pub fn has_popcnt() -> bool {
    has_feature(LEAF_1, 'c', 23)
}

pub fn has_avx() -> bool {
    has_feature(LEAF_1, 'c', 28)
}

pub fn has_f16c() -> bool {
    has_feature(LEAF_1, 'c', 29)
}

pub fn has_rdrand() -> bool {
    has_feature(LEAF_1, 'c', 30)
}

// ----------------------------------------------------------------------------
// ! Leaf 0000_0007h
// ----------------------------------------------------------------------------

pub fn has_avx2() -> bool {
    has_feature(LEAF_7, 'b', 5)
}

pub fn has_avx512f() -> bool {
    has_feature(LEAF_7, 'b', 16)
}

pub fn has_bmi1() -> bool {
    has_feature(LEAF_7, 'b', 3)
}

pub fn has_bmi2() -> bool {
    has_feature(LEAF_7, 'b', 8)
}

// ----------------------------------------------------------------------------
// ! Leaf 8000_0001h
// ----------------------------------------------------------------------------
pub fn has_sse4a() -> bool {
    if CpuBrand::detect() != CpuBrand::AMD {
        return false;
    }

    has_feature(EXT_LEAF_1, 'c', 6)
}
pub fn has_amd64() -> bool {
    has_feature(EXT_LEAF_1, 'd', 29)
}
pub fn has_3dnow_plus() -> bool {
    has_feature(EXT_LEAF_1, 'd', 30)
}
pub fn has_3dnow() -> bool {
    has_feature(EXT_LEAF_1, 'd', 31)
}
