//! CPUID Function Wrappers
use super::{native_cpuid, native_cpuid_count};

/// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
pub fn vendor_id() -> String {
    let res = native_cpuid(0);
    let mut bytes = Vec::with_capacity(12);
    for &reg in &[res.ebx, res.edx, res.ecx] {
        bytes.extend_from_slice(&reg.to_le_bytes());
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Gets the CPU model string.
pub fn model_string() -> String {
    let mut model = String::new();
    // Check if extended functions are supported
    let max_extended_leaf = native_cpuid(0x8000_0000).eax;
    if max_extended_leaf < 0x8000_0004 {
        return "Unknown".to_string();
    }

    for leaf in 0x8000_0002..=0x8000_0004 {
        let res = native_cpuid(leaf);
        for &reg in &[res.eax, res.ebx, res.ecx, res.edx] {
            let bytes = reg.to_le_bytes();
            for &b in &bytes {
                if b != 0 {
                    model.push(b as char);
                }
            }
        }
    }
    model.trim().to_string()
}

pub fn easter_egg() -> Option<String> {
    let mut out = String::new();

    let addr = match vendor_id().as_str() {
        "AuthenticAMD" => 0x8FFF_FFFF,
        "Rise Rise Rise" => 0x0000_5A4E,
        _ => 1,
    };

    if addr != 1 {
        let res = native_cpuid(addr);

        for &reg in &[res.eax, res.ebx, res.ecx, res.edx] {
            let bytes = reg.to_le_bytes();
            for &b in &bytes {
                if b != 0 {
                    out.push(b as char)
                }
            }
        }
    }

    let out = out.trim().to_string();
    let has_easter_egg = out.len() > 0;

    if has_easter_egg {
        Some(out)
    } else {
        None
    }
}

/// Returns the number of logical cores.
pub fn logical_cores() -> u32 {
    native_cpuid(1).ebx >> 16 & 0xFF
}

// ------------------------------------------------------------------------
// ! CPU Feature Lookups
// ------------------------------------------------------------------------

pub fn has_fpu() -> bool {
    (native_cpuid(1).edx & (1 << 0)) != 0
}

pub fn has_amd64() -> bool {
    (native_cpuid(0x8000_0001).edx & (1 << 29)) != 0
}

pub fn has_mmx() -> bool {
    (native_cpuid(1).edx & (1 << 23)) != 0
}

pub fn has_3dnow() -> bool {
    (native_cpuid(1).edx & (1 << 31)) != 0
}

pub fn has_sse() -> bool {
    (native_cpuid(1).edx & (1 << 25)) != 0
}

pub fn has_sse2() -> bool {
    (native_cpuid(1).edx & (1 << 26)) != 0
}

pub fn has_sse3() -> bool {
    (native_cpuid(1).ecx & (1 << 0)) != 0
}

pub fn has_ssse3() -> bool {
    (native_cpuid(1).ecx & (1 << 9)) != 0
}

pub fn has_sse41() -> bool {
    (native_cpuid(1).ecx & (1 << 19)) != 0
}

pub fn has_sse42() -> bool {
    (native_cpuid(1).ecx & (1 << 20)) != 0
}

pub fn has_avx() -> bool {
    (native_cpuid(1).ecx & (1 << 28)) != 0
}

pub fn has_avx2() -> bool {
    (native_cpuid_count(7, 0).ebx & (1 << 5)) != 0
}

pub fn has_avx512f() -> bool {
    (native_cpuid_count(7, 0).ebx & (1 << 16)) != 0
}

pub fn has_fma() -> bool {
    (native_cpuid(1).ecx & (1 << 12)) != 0
}

pub fn has_f16c() -> bool {
    (native_cpuid(1).ecx & (1 << 29)) != 0
}

pub fn has_bmi1() -> bool {
    (native_cpuid_count(7, 0).ebx & (1 << 3)) != 0
}

pub fn has_bmi2() -> bool {
    (native_cpuid_count(7, 0).ebx & (1 << 8)) != 0
}

pub fn has_pclmulqdq() -> bool {
    (native_cpuid(1).ecx & (1 << 1)) != 0
}

pub fn has_rdrand() -> bool {
    (native_cpuid(1).ecx & (1 << 30)) != 0
}
