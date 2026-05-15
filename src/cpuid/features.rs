use super::CpuBrand;
use super::constants::{EXT_LEAF_1, LEAF_1, LEAF_7};
use super::fns::{is_amd, is_cyrix, is_valid_leaf, x86_cpuid};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// CPUID register selector for feature bit checking.
#[allow(unused)]
pub(crate) enum Reg {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

/// Checks if a specific feature bit is set in the given CPUID leaf.
pub(crate) fn has_feature(leaf: u32, register: Reg, bit: u32) -> bool {
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

#[must_use]
pub fn has_aes() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 25)
}

/// Returns true if the CPU has a Floating Point Unit (FPU).
#[must_use]
pub fn has_fpu() -> bool {
    has_feature(LEAF_1, Reg::Edx, 0)
}

/// Returns true if the CPU has a Time Stamp Counter (TSC).
#[must_use]
pub fn has_tsc() -> bool {
    has_feature(LEAF_1, Reg::Edx, 4)
}

/// Returns true if the CPU supports CMPXCHG8B instruction.
#[must_use]
pub fn has_cx8() -> bool {
    has_feature(LEAF_1, Reg::Edx, 8)
}

/// Returns true if the CPU supports CMOV instructions.
#[must_use]
pub fn has_cmov() -> bool {
    has_feature(LEAF_1, Reg::Edx, 15)
}

/// Returns true if the CPU supports MMX instructions.
#[must_use]
pub fn has_mmx() -> bool {
    has_feature(LEAF_1, Reg::Edx, 23)
}

/// Returns true if the CPU supports SSE instructions.
#[must_use]
pub fn has_sse() -> bool {
    has_feature(LEAF_1, Reg::Edx, 25)
}

/// Returns true if the CPU supports SSE2 instructions.
#[must_use]
pub fn has_sse2() -> bool {
    has_feature(LEAF_1, Reg::Edx, 26)
}

/// Returns true if the CPU supports Hyper-Threading.
#[must_use]
pub fn has_ht() -> bool {
    has_feature(LEAF_1, Reg::Edx, 28)
}

/// Returns true if the CPU supports SSE3 instructions.
#[must_use]
pub fn has_sse3() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 0)
}

/// Returns true if the CPU supports SSSE3 instructions.
#[must_use]
pub fn has_ssse3() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 9)
}

/// Returns true if the CPU supports FMA (Fused Multiply Add).
#[must_use]
pub fn has_fma() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 12)
}

/// Returns true if the CPU supports CMPXCHG16B instruction.
#[must_use]
pub fn has_cx16() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 13)
}

/// Returns true if the CPU supports SSE4.1 instructions.
#[must_use]
pub fn has_sse41() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 19)
}

/// Returns true if the CPU supports SSE4.2 instructions.
#[must_use]
pub fn has_sse42() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 20)
}

#[must_use]
pub fn has_x2apic() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 21)
}

/// Returns true if the CPU supports POPCNT instruction.
#[must_use]
pub fn has_popcnt() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 23)
}

/// Returns true if the CPU supports AVX instructions.
#[must_use]
pub fn has_avx() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 28)
}

/// Returns true if the CPU supports F16C (Half-precision floating point).
#[must_use]
pub fn has_f16c() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 29)
}

/// Returns true if the CPU supports RDRAND instruction.
#[must_use]
pub fn has_rdrand() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 30)
}

/// Is this cpu running under a hypervisor? (Virtualization)
#[must_use]
pub fn is_hypervisor_guest() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 31)
}

#[must_use]
pub fn has_apic() -> bool {
    has_feature(LEAF_1, Reg::Edx, 9)
}

// ----------------------------------------------------------------------------
// ! Leaf 0000_0007h - Extended feature flags
// ----------------------------------------------------------------------------

/// Returns true if the CPU supports BMI1 (Bit Manipulation Instructions).
#[must_use]
pub fn has_bmi1() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 3)
}

/// Returns true if the CPU supports AVX2 instructions.
#[must_use]
pub fn has_avx2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 5)
}

/// Returns true if the CPU supports BMI2 instructions.
#[must_use]
pub fn has_bmi2() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 8)
}

/// Returns true if the CPU supports AVX-512 Foundation instructions.
#[must_use]
pub fn has_avx512_f() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 16)
}

/// Returns true if the CPU supports AVX-512 DQ instructions.
#[must_use]
pub fn has_avx512_dq() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 17)
}

#[must_use]
pub fn has_rdseed() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 18)
}

/// Returns true if the CPU supports AVX-512 IFMA instructions.
#[must_use]
pub fn has_avx512_ifma() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 21)
}

/// Returns true if the CPU supports AVX-512 PF instructions (Xeon Phi).
#[must_use]
pub fn has_avx512_pf() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 26)
}

/// Returns true if the CPU supports AVX-512 ER instructions (Xeon Phi).
#[must_use]
pub fn has_avx512_er() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 27)
}

/// Returns true if the CPU supports AVX-512 CD instructions.
#[must_use]
pub fn has_avx512_cd() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 28)
}

#[must_use]
pub fn has_sha() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 29)
}

/// Returns true if the CPU supports AVX-512 BW instructions.
#[must_use]
pub fn has_avx512_bw() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 30)
}

/// Returns true if the CPU supports AVX-512 VL instructions.
#[must_use]
pub fn has_avx512_vl() -> bool {
    has_feature(LEAF_7, Reg::Ebx, 31)
}

/// Vector version of AES instruction
#[must_use]
pub fn has_vaes() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 9)
}

/// Returns true if the CPU supports VPCLMULQDQ instructions.
#[must_use]
pub fn has_vpclmulqdq() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 10)
}

/// Returns true if the CPU supports AVX-VNNI instructions.
#[must_use]
pub fn has_avx_vnni() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 11)
}

/// Returns true if the CPU supports AVX-512 BITALG instructions.
#[must_use]
pub fn has_avx512_bitalg() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 12)
}

/// Returns true if the CPU supports AVX-512 VPOPCNTDQ instructions.
#[must_use]
pub fn has_avx512_vpopcntdq() -> bool {
    has_feature(LEAF_7, Reg::Ecx, 14)
}

/// Returns true if the CPU supports AVX-512 4VNNIW instructions (Xeon Phi).
#[must_use]
pub fn has_avx512_4vnniw() -> bool {
    has_feature(LEAF_7, Reg::Edx, 2)
}

/// Returns true if the CPU supports AVX-512 4FMAPS instructions (Xeon Phi).
#[must_use]
pub fn has_avx512_4fmaps() -> bool {
    has_feature(LEAF_7, Reg::Edx, 3)
}

/// Returns true if the CPU supports AVX-512 VP2INTERSECT instructions.
#[must_use]
pub fn has_avx512_vp2intersect() -> bool {
    has_feature(LEAF_7, Reg::Edx, 8)
}

// ----------------------------------------------------------------------------
// ! Leaf 8000_0001h - Extended features
// ----------------------------------------------------------------------------

/// Returns true if the CPU supports the NX (No-Execute) bit / XD (Execute Disable).
///
/// This bit in EDX indicates that the CPU supports marking pages as non-executable,
/// which is a key security feature used by NX/XD/DEP.
#[must_use]
pub fn has_nx() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 20)
}

/// Returns true if the CPU supports Intel VT-x (VMX) hardware virtualization.
///
/// Checks ECX bit 5 in basic leaf 0x1.
#[must_use]
pub fn has_vtx() -> bool {
    has_feature(LEAF_1, Reg::Ecx, 5)
}

/// Returns true if the CPU supports AMD SVM (Secure Virtual Machine) hardware virtualization.
///
/// Checks ECX bit 2 in extended leaf 0x80000001.
#[must_use]
pub fn has_amdv() -> bool {
    has_feature(EXT_LEAF_1, Reg::Ecx, 2)
}

/// Returns true if the CPU supports hardware-assisted virtualization.
///
/// This is true if either Intel VT-x (VMX) or AMD SVM is supported.
#[must_use]
pub fn has_virtualization() -> bool {
    has_vtx() || has_amdv()
}

/// Returns true if the CPU supports SSE4A instructions (AMD-specific).
#[must_use]
pub fn has_sse4a() -> bool {
    if CpuBrand::detect() != CpuBrand::AMD {
        return false;
    }

    has_feature(EXT_LEAF_1, Reg::Ecx, 6)
}

#[must_use]
pub fn has_3dnow_prefetch() -> bool {
    has_feature(EXT_LEAF_1, Reg::Ecx, 8)
}

#[must_use]
pub fn has_mmx_plus() -> bool {
    if is_amd() {
        has_feature(EXT_LEAF_1, Reg::Edx, 22)
    } else if is_cyrix() {
        has_feature(EXT_LEAF_1, Reg::Edx, 24)
    } else {
        false
    }
}

/// Returns true if the CPU supports AMD64 (x86-64) instructions.
#[must_use]
pub fn has_amd64() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 29)
}

/// Returns true if the CPU supports 3DNow!+ instructions.
#[must_use]
pub fn has_3dnow_plus() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 30)
}

/// Returns true if the CPU supports 3DNow! instructions.
#[must_use]
pub fn has_3dnow() -> bool {
    has_feature(EXT_LEAF_1, Reg::Edx, 31)
}

// ----------------------------------------------------------------------------
// ! Feature list aggregation
// ----------------------------------------------------------------------------

type FeatureFn = fn() -> bool;
type FeatureMap<'a> = &'a [(&'static str, FeatureFn)];

#[cfg(target_os = "none")]
pub fn get_feature_list() -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::new();

    const FEATURES: FeatureMap = &[
        ("FPU", has_fpu),
        ("TSC", has_tsc),
        ("CMPXCHG8B", has_cx8),
        ("CMPXCHG16B", has_cx16),
        ("CMOV", has_cmov),
        ("MMX", has_mmx),
        ("MMX+", has_mmx_plus),
        ("3DNow!", has_3dnow),
        ("3DNow!+", has_3dnow_plus),
        ("APIC", has_apic),
        ("AMD64", has_amd64),
        ("SSE", has_sse),
        ("SSE2", has_sse2),
        ("SSE3", has_sse3),
        ("SSE4A", has_sse4a),
        ("SSE4.1", has_sse41),
        ("SSE4.2", has_sse42),
        ("SSSE3", has_ssse3),
        ("AES", has_aes),
        ("SHA", has_sha),
    ];

    let mut features: Vec<&'static str> = Vec::new();

    for (name, check) in FEATURES {
        if check() {
            features.push(name);
        }
    }

    if !features.is_empty() {
        map.insert("Base", features.join(" "));
    }

    map
}

/// Get the full list of detected features.
#[cfg(not(target_os = "none"))]
#[must_use]
pub fn get_feature_list() -> BTreeMap<&'static str, String> {
    use super::vendor::centaur;

    const BASIC_FEATURES: FeatureMap = &[
        ("FPU", has_fpu),
        ("TSC", has_tsc),
        ("CX8", has_cx8),
        ("CX16", has_cx16),
        ("CMOV", has_cmov),
        ("MMX", has_mmx),
        ("MMX+", has_mmx_plus),
        ("3DNow!", has_3dnow),
        ("3DNow!+", has_3dnow_plus),
        ("3DNow!-Prefetch", has_3dnow_prefetch),
        ("HT", has_ht),
        ("APIC", has_apic),
        ("AMD64", has_amd64),
    ];

    const SSE_FEATURES: FeatureMap = &[
        ("SSE", has_sse),
        ("SSE2", has_sse2),
        ("SSE3", has_sse3),
        ("SSE4A", has_sse4a),
        ("SSE4.1", has_sse41),
        ("SSE4.2", has_sse42),
        ("SSSE3", has_ssse3),
    ];

    const AVX_FEATURES: FeatureMap = &[
        ("AVX", has_avx),
        ("AVX2", has_avx2),
        ("AVX-VNNI", has_avx_vnni),
        ("VPCLMULQDQ", has_vpclmulqdq),
    ];

    const AVX512_FEATURES: FeatureMap = &[
        ("F", has_avx512_f),
        ("DQ", has_avx512_dq),
        ("IFMA", has_avx512_ifma),
        ("PF", has_avx512_pf),
        ("ER", has_avx512_er),
        ("CD", has_avx512_cd),
        ("BW", has_avx512_bw),
        ("VL", has_avx512_vl),
        ("BITALG", has_avx512_bitalg),
        ("VPOPCNTDQ", has_avx512_vpopcntdq),
        ("4VNNIW", has_avx512_4vnniw),
        ("4FMAPS", has_avx512_4fmaps),
        ("VP2INTERSECT", has_avx512_vp2intersect),
    ];

    const SECURITY_FEATURES: FeatureMap = &[
        ("NX", has_nx),
        ("RDSEED", has_rdseed),
        ("RDRAND", has_rdrand),
        ("AES", has_aes),
        ("VAES", has_vaes),
        ("SHA", has_sha),
        ("VT-x", has_vtx),
        ("AMD-V", has_amdv),
    ];

    const MATH_FEATURES: FeatureMap = &[
        ("FMA", has_fma),
        ("BMI1", has_bmi1),
        ("BMI2", has_bmi2),
        ("F16C", has_f16c),
    ];

    const OTHER_FEATURES: FeatureMap = &[("x2apic", has_x2apic), ("POPCNT", has_popcnt)];

    const CENTAUR_FEATURES: FeatureMap = &[
        ("RNG", centaur::has_rng),
        ("RNG2", centaur::has_rng2),
        ("ACE", centaur::has_ace),
        ("ACE2", centaur::has_ace2),
        ("PHE", centaur::has_phe),
        ("PHE2", centaur::has_phe2),
        ("PMM", centaur::has_pmm),
    ];

    let mut map = BTreeMap::new();

    let mut basic: Vec<&'static str> = Vec::new();
    let mut sse: Vec<&'static str> = Vec::new();
    let mut avx: Vec<&'static str> = Vec::new();
    let mut avx512: Vec<&'static str> = Vec::new();
    let mut encryption: Vec<&'static str> = Vec::new();
    let mut math: Vec<&'static str> = Vec::new();
    let mut other: Vec<&'static str> = Vec::new();
    let mut centaur: Vec<&'static str> = Vec::new();

    for (v, key, checks) in [
        (&mut basic, "Base", BASIC_FEATURES),
        (&mut sse, "SSE", SSE_FEATURES),
        (&mut avx, "AVX", AVX_FEATURES),
        (&mut avx512, "AVX512", AVX512_FEATURES),
        (&mut encryption, "Security", SECURITY_FEATURES),
        (&mut math, "Math", MATH_FEATURES),
        (&mut other, "Other", OTHER_FEATURES),
        (&mut centaur, "Centaur", CENTAUR_FEATURES),
    ] {
        for (name, check) in checks {
            if check() {
                v.push(name);
            }
        }

        if !v.is_empty() {
            let s = v.join(" ");
            map.insert(key, s);
        }
    }

    map
}
