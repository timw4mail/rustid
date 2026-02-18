//! CPUID Function Wrappers

use super::x86_cpuid;

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

pub fn has_fpu() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 0)) != 0
}

pub fn has_amd64() -> bool {
    if max_extended_leaf() < 0x8000_0001 {
        return false;
    }
    (x86_cpuid(0x8000_0001).edx & (1 << 29)) != 0
}

pub fn has_mmx() -> bool {
    if max_leaf() < 1 {
        return false;
    }
    (x86_cpuid(1).edx & (1 << 23)) != 0
}

pub fn has_3dnow() -> bool {
    if max_extended_leaf() < 0x8000_0001 {
        return false;
    }

    (x86_cpuid(0x8000_0001).edx & (1 << 31)) != 0
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
