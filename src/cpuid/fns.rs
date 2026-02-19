//! CPUID Function Wrappers

use super::x86_cpuid;

#[allow(unused_imports)]
use core::arch::asm;

/// Returns true if the CPUID instruction is supported.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
}

/// Returns true if the CPU is a Cyrix processor (detected without CPUID).
///
/// Cyrix processors are unique in that they do not modify flags during a `div`
/// instruction, whereas other x86 processors do.
pub fn is_cyrix() -> bool {
    #[cfg(target_arch = "x86_64")]
    return false;

    #[cfg(target_arch = "x86")]
    {
        let flags: i8;
        unsafe {
            asm!(
                "xor ax, ax",
                "sahf",         // Clear flags (ZF, PF, AF, CF, SF)
                "mov ax, 5",
                "mov bx, 2",
                "div bl",       // On non-Cyrix, this modifies flags
                "lahf",         // Load flags into AH
                out("ah") flags,
                out("al") _,
                out("bx") _,
            );
        }
        // Check if flags (ZF, PF, etc.) are still 0
        (flags & 0xF) == 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    false
}

/// Enables CPUID on Cyrix 5x86 and 6x86 processors.
///
/// This sets bit 7 of CCR4 (Configuration Control Register 4).
#[cfg(target_arch = "x86")]
pub unsafe fn enable_cyrix_cpuid() {
    unsafe {
        // Port 0x22 is the index port, 0x23 is the data port.
        // CCR4 is index 0xE8.
        asm!(
        "out 0x22, al",
        "in al, 0x23",
        "or al, 0x80",
        "mov ah, al",
        "mov al, 0xE8",
        "out 0x22, al",
        "mov al, ah",
        "out 0x23, al",
        inout("al") 0xE8_u8 => _,
        out("ah") _,
        );
    }
}

pub fn max_leaf() -> u32 {
    x86_cpuid(0).eax
}

pub fn max_extended_leaf() -> u32 {
    x86_cpuid(0x8000_0000).eax
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

// ------------------------------------------------------------------------
// ! Leaf 0000_0001h
// ------------------------------------------------------------------------

pub fn has_fpu() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 0)) != 0
}

pub fn has_mmx() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 23)) != 0
}

pub fn has_cmov() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 15)) != 0
}

pub fn has_fcmov() -> bool {
    has_fpu() && has_cmov()
}

pub fn has_cx8() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 8)) != 0
}

pub fn has_sse() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 25)) != 0
}

pub fn has_sse2() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 26)) != 0
}

pub fn has_sse3() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 0)) != 0
}

pub fn has_ssse3() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 9)) != 0
}

pub fn has_sse41() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 19)) != 0
}

pub fn has_sse42() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 20)) != 0
}

pub fn has_avx() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 28)) != 0
}

pub fn has_fma() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 12)) != 0
}

pub fn has_f16c() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 29)) != 0
}

pub fn has_pclmulqdq() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 1)) != 0
}

pub fn has_rdrand() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).ecx & (1 << 30)) != 0
}

// ----------------------------------------------------------------------------
// ! Leaf 0000_0007h
// ----------------------------------------------------------------------------

pub fn has_avx2() -> bool {
    if max_leaf() < 7 {
        return false;
    }
    (x86_cpuid(7).ebx & (1 << 5)) != 0
}

pub fn has_avx512f() -> bool {
    if max_leaf() < 7 {
        return false;
    }
    (x86_cpuid(7).ebx & (1 << 16)) != 0
}

pub fn has_bmi1() -> bool {
    if max_leaf() < 7 {
        return false;
    }
    (x86_cpuid(7).ebx & (1 << 3)) != 0
}

pub fn has_bmi2() -> bool {
    if max_leaf() < 7 {
        return false;
    }
    (x86_cpuid(7).ebx & (1 << 8)) != 0
}

// ----------------------------------------------------------------------------
// ! Leaf 8000_0001h
// ----------------------------------------------------------------------------

pub fn has_3dnow() -> bool {
    if max_extended_leaf() < 0x8000_0001 {
        return false;
    }

    (x86_cpuid(0x8000_0001).edx & (1 << 31)) != 0
}

pub fn has_amd64() -> bool {
    if max_extended_leaf() < 0x8000_0001 {
        return false;
    }
    (x86_cpuid(0x8000_0001).edx & (1 << 29)) != 0
}
