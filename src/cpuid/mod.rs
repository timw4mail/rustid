//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__cpuid, __cpuid_count, CpuidResult};
use ufmt::derive::uDebug;

pub mod brand;
pub mod cpu;
pub mod fns;
pub mod micro_arch;
pub mod topology;

pub use cpu::*;

pub const UNK: &str = "Unknown";

/// Represents the result of a CPUID instruction call.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, uDebug)]
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

/// Check for CPUID
pub fn init() {
    #[cfg(target_arch = "x86")]
    {
        #[cfg(target_os = "none")]
        use crate::println;
        #[cfg(not(target_os = "none"))]
        use std::println;

        if !fns::has_cpuid() && fns::is_cyrix() {
            println!("This CPU might have CPUID support, but it is disabled.");
            println!("Some BIOSes have an option to enable CPUID for Cyrix chips.");
            println!("For DOS, you can download a utility from ");
            println!("  https://www.deinmeister.de/cypower.com");
            println!("If run before rustid, CPUID should be enabled");
        }
    }

    #[cfg(target_arch = "x86_64")]
    {}

    // TODO: Implement CPU detection for ARM.
    #[cfg(target_arch = "aarch64")]
    {}
}

/// Calls CPUID with the given leaf (EAX).
#[allow(unused_unsafe)]
pub fn x86_cpuid(leaf: u32) -> CpuInfo {
    if !fns::has_cpuid() {
        return CpuInfo::default();
    }
    unsafe { __cpuid(leaf).into() }
}

/// Calls CPUID with the given leaf (EAX) and sub-leaf (ECX).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(unused_unsafe)]
pub fn x86_cpuid_count(leaf: u32, sub_leaf: u32) -> CpuInfo {
    if !fns::has_cpuid() {
        return CpuInfo::default();
    }
    unsafe { __cpuid_count(leaf, sub_leaf).into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::println;
    use ufmt::uwrite;

    #[test]
    fn test_cpu_info_udebug() {
        let info = CpuInfo {
            eax: 1,
            ebx: 2,
            ecx: 3,
            edx: 4,
        };
        let mut s = heapless::String::<64>::new();
        uwrite!(&mut s, "{:?}", info).unwrap();
        assert_eq!(s.as_str(), "CpuInfo { eax: 1, ebx: 2, ecx: 3, edx: 4 }");
    }

    #[test]
    fn test_from_cpuid_result_for_cpu_info() {
        let cpuid_result = CpuidResult {
            eax: 10,
            ebx: 20,
            ecx: 30,
            edx: 40,
        };
        let cpu_info: CpuInfo = cpuid_result.into();
        assert_eq!(cpu_info.eax, 10);
        assert_eq!(cpu_info.ebx, 20);
        assert_eq!(cpu_info.ecx, 30);
        assert_eq!(cpu_info.edx, 40);
    }

    #[test]
    fn test_init() {
        // Ensure init does not panic. Output is console dependent.
        init();
    }

    #[test]
    fn test_x86_cpuid_leaf_0() {
        let cpu_info = x86_cpuid(0);
        // Cannot assert specific values, but can ensure it doesn't panic
        // and that some fields might be non-zero for vendor ID.
        println!("CPUID Leaf 0: {:?}", cpu_info);
        assert!(cpu_info.eax > 0); // EAX for leaf 0 is max_leaf, should be > 0
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_x86_cpuid_count_leaf_1_subleaf_0() {
        let cpu_info = x86_cpuid_count(1, 0);
        // Cannot assert specific values, but can ensure it doesn't panic.
        println!("CPUID Leaf 1, Subleaf 0: {:?}", cpu_info);
        // EAX for leaf 1 contains processor signature, should be non-zero for most CPUs
        assert!(cpu_info.eax > 0);
    }
}
