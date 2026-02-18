//! CPUID Function Wrappers

use crate::cpuid::brand::CpuBrand;

use super::x86_cpuid;

/// Returns the number of logical cores.
pub fn logical_cores() -> u32 {
    x86_cpuid(1).ebx >> 16 & 0xFF
}

// ------------------------------------------------------------------------
// ! CPU Feature Lookups
// ------------------------------------------------------------------------

pub fn has_fpu() -> bool {
    (x86_cpuid(1).edx & (1 << 0)) != 0
}

pub fn has_amd64() -> bool {
    (x86_cpuid(0x8000_0001).edx & (1 << 29)) != 0
}

pub fn has_mmx() -> bool {
    (x86_cpuid(1).edx & (1 << 23)) != 0
}

pub fn has_3dnow() -> bool {
    if CpuBrand::detect() == CpuBrand::Intel {
        return false;
    }

    (x86_cpuid(1).edx & (1 << 31)) != 0
}

pub fn has_sse() -> bool {
    (x86_cpuid(1).edx & (1 << 25)) != 0
}

pub fn has_sse2() -> bool {
    (x86_cpuid(1).edx & (1 << 26)) != 0
}

pub fn has_sse3() -> bool {
    (x86_cpuid(1).ecx & (1 << 0)) != 0
}

pub fn has_ssse3() -> bool {
    (x86_cpuid(1).ecx & (1 << 9)) != 0
}

pub fn has_sse41() -> bool {
    (x86_cpuid(1).ecx & (1 << 19)) != 0
}

pub fn has_sse42() -> bool {
    (x86_cpuid(1).ecx & (1 << 20)) != 0
}

pub fn has_avx() -> bool {
    (x86_cpuid(1).ecx & (1 << 28)) != 0
}

pub fn has_avx2() -> bool {
    (x86_cpuid(7).ebx & (1 << 5)) != 0
}

pub fn has_avx512f() -> bool {
    (x86_cpuid(7).ebx & (1 << 16)) != 0
}

pub fn has_fma() -> bool {
    (x86_cpuid(1).ecx & (1 << 12)) != 0
}

pub fn has_f16c() -> bool {
    (x86_cpuid(1).ecx & (1 << 29)) != 0
}

pub fn has_bmi1() -> bool {
    (x86_cpuid(7).ebx & (1 << 3)) != 0
}

pub fn has_bmi2() -> bool {
    (x86_cpuid(7).ebx & (1 << 8)) != 0
}

pub fn has_pclmulqdq() -> bool {
    (x86_cpuid(1).ecx & (1 << 1)) != 0
}

pub fn has_rdrand() -> bool {
    (x86_cpuid(1).ecx & (1 << 30)) != 0
}
