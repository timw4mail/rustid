//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.
/// Compile-time check to ensure this crate is only used on x86/x86_64.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid_count, CpuidResult};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__cpuid_count, CpuidResult};

pub mod brand;

pub mod cache;

pub mod cpu;

#[cfg(target_os = "none")]
pub mod dos;

pub mod dump;

pub mod micro_arch;

pub mod mp;

#[cfg(feature = "file_mock")]
pub mod provider;

pub mod topology;

pub mod vendor;

#[cfg(target_arch = "x86")]
pub mod quirks;

#[macro_use]
pub mod string;

pub use brand::*;
pub use cpu::*;
pub use string::*;

#[cfg(target_arch = "x86")]
pub use quirks::*;

pub use crate::common::UNK;

pub type FeatureList = heapless::Vec<&'static str, 64>;

/// CPUID leaf 0x00000000 - Maximum basic leaf
pub const LEAF_0: u32 = 0x0;

/// CPUID leaf 0x00000001 - Processor info and feature flags
pub const LEAF_1: u32 = 0x1;

/// CPUID leaf 0x00000002 - Cache descriptors
pub const LEAF_2: u32 = 0x2;

/// Intel deterministic cache parameters
pub const LEAF_4: u32 = 0x4;

/// CPUID leaf 0x00000007 - Extended feature flags
pub const LEAF_7: u32 = 0x7;

/// CPU extended topology v1
pub const LEAF_0B: u32 = 0xB;

/// CPUID leaf 0x00000016 - Intel Processor Frequency
pub const LEAF_16: u32 = 0x16;

/// Intel extended topology v2
pub const LEAF_1F: u32 = 0x1F;

/// Extended CPUID leaf 0x80000000 - Maximum extended leaf
pub const EXT_LEAF_0: u32 = 0x8000_0000;

/// Extended CPUID leaf 0x80000001 - Extended processor info
pub const EXT_LEAF_1: u32 = 0x8000_0001;

/// Cpu model string start
pub const EXT_LEAF_2: u32 = 0x8000_0002;

/// Cpu model string end
pub const EXT_LEAF_4: u32 = 0x8000_0004;

/// AMD/Transmeta L1 cache and TLB
pub const EXT_LEAF_5: u32 = 0x8000_0005;

/// AMD L2/L3 cache parameters
pub const EXT_LEAF_6: u32 = 0x8000_0006;

/// AMD deterministic cache parameters
pub const EXT_LEAF_1D: u32 = 0x8000_001D;

/// AMD extended CPU topology
pub const EXT_LEAF_26: u32 = 0x8000_0026;

/// The max value of the extended CPUID leaf
pub const EXT_LEAF_MAX: u32 = 0x8000_FFFF;

/// Centaur/Zhaoxin vendor leaf base
pub const CENTAUR_LEAF_0: u32 = 0xC000_0000;

/// Centaur/Zhaoxin extended CPU features
pub const CENTAUR_LEAF_1: u32 = 0xC000_0001;

pub const CENTAUR_LEAF_2: u32 = 0xC000_0002;

/// Transmeta vendor leaf base
pub const TRANSMETA_LEAF_0: u32 = 0x8086_0000;

/// Transmeta extended CPU features
pub const TRANSMETA_LEAF_1: u32 = 0x8086_0001;

/// Transmeta CMS (Code Morphing Software)
pub const TRANSMETA_LEAF_2: u32 = 0x8086_0002;

/// Transmeta Cpu model string start
pub const TRANSMETA_LEAF_3: u32 = 0x8086_0003;

/// Transmeta Cpu model string end
pub const TRANSMETA_LEAF_6: u32 = 0x8086_0006;

/// Transmeta live CPU information
pub const TRANSMETA_LEAF_7: u32 = 0x8086_0007;

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

    // I think the latest version of rust just made this a "safe" function.
    // For now, so the very latest version isn't required, I'll wrap in the unsafe block
    #[allow(unused_unsafe)]
    unsafe {
        __cpuid_count(leaf, sub_leaf).into()
    }
}

/// Calls CPUID with the given leaf (EAX) and sub-leaf (ECX).
pub fn x86_cpuid_count(leaf: u32, sub_leaf: u32) -> Cpuid {
    #[cfg(not(feature = "file_mock"))]
    return real_x86_cpuid_count(leaf, sub_leaf);

    #[cfg(feature = "file_mock")]
    provider::PROVIDER
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
pub fn vendor_str() -> Str<12> {
    #[cfg(target_arch = "x86")]
    if !has_cpuid() {
        let v = get_vendor_by_quirk();

        return Str::from(v);
    }

    let mut s = Str::new();

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

pub fn read_multi_leaf_str(min_leaf: u32, max_leaf: u32) -> Str<64> {
    let mut model: Str<64> = Str::new();
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

/// Returns true if the CPU is from Intel.
pub fn is_intel() -> bool {
    is_vendor(VENDOR_INTEL)
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
    if is_amd() && max_leaf() > 0 {
        let res = x86_cpuid(1);
        let count = (res.ebx >> 16) & 0xFF;
        if count != 0 {
            return count;
        }
    }

    #[cfg(not(target_os = "none"))]
    return crate::common::logical_cores() as u32;

    #[cfg(target_os = "none")]
    1
}

// ------------------------------------------------------------------------
// ! CPU Feature Lookups
// ------------------------------------------------------------------------

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

/// Returns the CPU brand ID from extended leaf 0x80000001.
///
/// The brand ID is a value that identifies the specific CPU brand variant.
pub fn get_brand_id() -> u32 {
    let res = x86_cpuid(EXT_LEAF_1);
    res.ebx >> 8
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
    #[cfg(target_arch = "x86_64")]
    return true;

    #[cfg(target_arch = "x86")]
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
        let _ = out.push("3DNow!+");
    };
    if has_ht() {
        let _ = out.push("HT");
    };
    if has_x2apic() {
        let _ = out.push("x2apic");
    };
    if has_amd64() {
        if is_intel() {
            let _ = out.push("EM64T");
        } else {
            let _ = out.push("AMD64");
        }
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
    if has_aes() {
        let _ = out.push("AES");
    };
    if has_vaes() {
        let _ = out.push("VAES");
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
    if has_sha() {
        let _ = out.push("SHA");
    };

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
