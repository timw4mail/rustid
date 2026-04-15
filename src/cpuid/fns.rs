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

use crate::cpuid::{CpuBrand, FeatureList, Str};

/// Represents the result of a CPUID instruction call.
///
/// The CPUID instruction returns processor identification and feature information
/// in the EAX, EBX, ECX, and EDX registers.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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
    super::provider::PROVIDER
        .read()
        .unwrap()
        .cpuid_count(leaf, sub_leaf)
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

    match leaf {
        0..EXT_LEAF_0 => leaf <= max_leaf(),
        EXT_LEAF_0..=EXT_LEAF_MAX => leaf <= max_extended_leaf(),
        _ => leaf <= max_vendor_leaf(),
    }
}

/// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
///
/// Returns a 12-character vendor string from CPUID leaf 0.
pub fn vendor_str() -> Str<20> {
    #[cfg(target_arch = "x86")]
    if !has_cpuid() {
        let v = get_vendor_by_quirk();

        return Str::from(v);
    }

    let mut s: Str<20> = Str::new();

    let res = x86_cpuid(0);
    let mut bytes = [0u8; 12];

    bytes[0..4].copy_from_slice(&res.ebx.to_le_bytes());
    bytes[4..8].copy_from_slice(&res.edx.to_le_bytes());
    bytes[8..12].copy_from_slice(&res.ecx.to_le_bytes());

    for &b in &bytes {
        if b != 0 {
            s.push(b as char);
        }
    }

    s
}

pub fn read_multi_leaf_str(min_leaf: u32, max_leaf: u32) -> Str<70> {
    let mut model: Str<70> = Str::new();
    if !is_valid_leaf(max_leaf) {
        return Str::from(UNK);
    }

    for leaf in min_leaf..=max_leaf {
        let res = x86_cpuid(leaf);
        for reg in &[res.eax, res.ebx, res.ecx, res.edx] {
            for &b in &reg.to_le_bytes() {
                if b != 0 {
                    model.push(b as char);
                }
            }
        }
    }

    let trimmed = model.trim();

    Str::from(trimmed)
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
    is_vendor(VENDOR_CYRIX)
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

    //     #[cfg(not(target_os = "none"))]
    //     return crate::common::logical_cores() as u32;
    //
    //     #[cfg(target_os = "none")]
    1
}

/// CPUID register selector for feature bit checking.
#[allow(unused)]
enum Reg {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

/// Checks if a specific feature bit is set in the given CPUID leaf.
fn has_feature(leaf: u32, register: Reg, bit: u32) -> bool {
    if !is_valid_leaf(leaf) {
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

/// Returns the CPU brand ID from basic leaf 0x00000001.
///
/// The brand ID is a value that identifies the specific CPU brand variant.
pub fn get_brand_id() -> u32 {
    let res = x86_cpuid(LEAF_1);
    res.ebx & 0xFF
}

pub fn has_aes() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 25)
}

/// Returns true if the CPU has a Floating Point Unit (FPU).
pub fn has_fpu() -> bool {
    has_feature(LEAF_1, Reg::Edx, 0)
}

/// Returns true if the CPU has a Time Stamp Counter (TSC).
pub fn has_tsc() -> bool {
    has_feature(LEAF_1, Reg::Edx, 4)
}

/// Returns true if the CPU supports CMPXCHG8B instruction.
pub fn has_cx8() -> bool {
    has_feature(LEAF_1, Reg::Edx, 8)
}

/// Returns true if the CPU supports CMOV instructions.
pub fn has_cmov() -> bool {
    has_feature(LEAF_1, Reg::Edx, 15)
}

/// Returns true if the CPU supports MMX instructions.
pub fn has_mmx() -> bool {
    has_feature(LEAF_1, Reg::Edx, 23)
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

pub fn has_x2apic() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 21)
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

/// Returns true if the CPU supports BMI1 (Bit Manipulation Instructions).
pub fn has_bmi1() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 3)
}

/// Returns true if the CPU supports AVX2 instructions.
pub fn has_avx2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 5)
}

/// Returns true if the CPU supports BMI2 instructions.
pub fn has_bmi2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 8)
}

/// Returns true if the CPU supports AVX-512 Foundation instructions.
pub fn has_avx512f() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 16)
}

pub fn has_sha() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 29)
}

pub fn has_vaes() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 9)
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
    type FeatureFn = fn() -> bool;

    #[cfg(target_os = "none")]
    const FEATURES: &[(&str, FeatureFn)] = &[
        ("FPU", has_fpu),
        ("TSC", has_tsc),
        ("CMPXCHG8B", has_cx8),
        ("CMPXCHG16B", has_cx16),
        ("CMOV", has_cmov),
        ("MMX", has_mmx),
        ("3DNow!", has_3dnow),
        ("3DNow!+", has_3dnow_plus),
        ("AMD64", has_amd64),
        ("SSE", has_sse),
        ("SSE2", has_sse2),
        ("SSE3", has_sse3),
        ("SSE4A", has_sse4a),
        ("SSE4.1", has_sse41),
        ("SSE4.2", has_sse42),
        ("SSSE3", has_ssse3),
    ];
    #[cfg(not(target_os = "none"))]
    const FEATURES: &[(&str, FeatureFn)] = &[
        #[cfg(target_arch = "x86")]
        ("FPU", has_fpu),
        ("TSC", has_tsc),
        ("CMPXCHG8B", has_cx8),
        ("CMPXCHG16B", has_cx16),
        ("CMOV", has_cmov),
        ("MMX", has_mmx),
        ("3DNow!", has_3dnow),
        ("3DNow!+", has_3dnow_plus),
        ("HT", has_ht),
        ("x2apic", has_x2apic),
        ("AMD64", has_amd64),
        ("SSE", has_sse),
        ("SSE2", has_sse2),
        ("SSE3", has_sse3),
        ("SSE4A", has_sse4a),
        ("SSE4.1", has_sse41),
        ("SSE4.2", has_sse42),
        ("SSSE3", has_ssse3),
        ("AES", has_aes),
        ("VAES", has_vaes),
        ("AVX", has_avx),
        ("AVX2", has_avx2),
        ("AVX512F", has_avx512f),
        ("FMA", has_fma),
        ("BMI1", has_bmi1),
        ("BMI2", has_bmi2),
        ("RDRAND", has_rdrand),
        ("POPCNT", has_popcnt),
        ("F16C", has_f16c),
        ("SHA", has_sha),
    ];

    let mut out: FeatureList = FeatureList::new();
    for (name, check) in FEATURES {
        if check() {
            out.push(name);
        }
    }
    out
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
