//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

/// Represents the result of a CPUID instruction call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuInfo {
    /// EAX register value
    pub eax: u32,
    /// EBX register value
    pub ebx: u32,
    /// ECX register value
    pub ecx: u32,
    /// EDX register value
    pub edx: u32,
}

impl From<CpuidResult> for CpuInfo {
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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn native_cpuid(leaf: u32) -> CpuInfo {
    unsafe { __cpuid(leaf).into() }
}

/// Calls CPUID with the given leaf (EAX) and sub-leaf (ECX).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn native_cpuid_count(leaf: u32, sub_leaf: u32) -> CpuInfo {
    unsafe { __cpuid_count(leaf, sub_leaf).into() }
}

pub struct Cpu {
    // We can cache basic info here if needed
}

impl Cpu {
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
    pub fn vendor_id(&self) -> String {
        let res = native_cpuid(0);
        let mut bytes = Vec::with_capacity(12);
        for &reg in &[res.ebx, res.edx, res.ecx] {
            bytes.extend_from_slice(&reg.to_le_bytes());
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// Gets the CPU brand string.
    pub fn brand_string(&self) -> String {
        let mut brand = String::new();
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
                        brand.push(b as char);
                    }
                }
            }
        }
        brand.trim().to_string()
    }

    pub fn has_sse3(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 0)) != 0
    }

    pub fn has_pclmulqdq(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 1)) != 0
    }

    pub fn has_ssse3(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 9)) != 0
    }

    pub fn has_fma(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 12)) != 0
    }

    pub fn has_sse41(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 19)) != 0
    }

    pub fn has_sse42(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 20)) != 0
    }

    pub fn has_avx(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 28)) != 0
    }

    pub fn has_f16c(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 29)) != 0
    }

    pub fn has_rdrand(&self) -> bool {
        (native_cpuid(1).ecx & (1 << 30)) != 0
    }

    pub fn has_sse(&self) -> bool {
        (native_cpuid(1).edx & (1 << 25)) != 0
    }

    pub fn has_sse2(&self) -> bool {
        (native_cpuid(1).edx & (1 << 26)) != 0
    }

    pub fn has_avx2(&self) -> bool {
        (native_cpuid_count(7, 0).ebx & (1 << 5)) != 0
    }

    pub fn has_bmi1(&self) -> bool {
        (native_cpuid_count(7, 0).ebx & (1 << 3)) != 0
    }

    pub fn has_bmi2(&self) -> bool {
        (native_cpuid_count(7, 0).ebx & (1 << 8)) != 0
    }

    pub fn has_avx512f(&self) -> bool {
        (native_cpuid_count(7, 0).ebx & (1 << 16)) != 0
    }

    /// Returns the number of logical cores.
    pub fn logical_cores(&self) -> u32 {
        native_cpuid(1).ebx >> 16 & 0xFF
    }

    /// Returns the CPU's stepping, model, and family.
    pub fn signature(&self) -> (u32, u32, u32) {
        let res = native_cpuid(1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

        let display_family = if family == 0xF {
            family + extended_family
        } else {
            family
        };

        let display_model = if family == 0x6 || family == 0xF {
            (extended_model << 4) + model
        } else {
            model
        };

        (stepping, display_model, display_family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id() {
        let cpu = Cpu::new();
        let vendor = cpu.vendor_id();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_brand_string() {
        let cpu = Cpu::new();
        let brand = cpu.brand_string();
        println!("Brand: {}", brand);
    }
}
