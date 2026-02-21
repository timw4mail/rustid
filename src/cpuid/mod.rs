//! A library for querying CPU information using the x86 CPUID instruction.
//!
//! This crate provides a high-level interface to query CPU vendor, brand string,
//! supported features (like SSE, AVX), and other hardware details.

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__cpuid, __cpuid_count, CpuidResult};

pub mod brand;
pub mod cpu;
pub mod fns;
pub mod micro_arch;

pub use cpu::*;

/// Represents the result of a CPUID instruction call.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

impl ufmt::uDebug for CpuInfo {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.debug_struct("CpuInfo")?
            .field("eax", &self.eax)?
            .field("ebx", &self.ebx)?
            .field("ecx", &self.ecx)?
            .field("edx", &self.edx)?
            .finish()
    }
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

/// Check for Cyrix CPUs and enable CPUID on them.
pub fn init() {
    #[cfg(target_arch = "x86")]
    {
        #[cfg(target_os = "none")]
        use crate::println;
        #[cfg(not(target_os = "none"))]
        use std::println;

        let has_cpuid = fns::has_cpuid();

        if !has_cpuid {
            println!("The CPU does not appear to have CPUID, or it is disabled.");
            println!("Checking for Cyrix CPUs...");
        }

        if (!has_cpuid) && fns::is_cyrix() {
            println!("Attempting to enable CPUID on Cyrix CPU...");
            unsafe {
                fns::enable_cyrix_cpuid();
            }

            let has_cpuid = fns::has_cpuid();

            if has_cpuid {
                println!("CPUID is now enabled on Cyrix CPU.");
            } else {
                println!("Failed to enable CPUID on Cyrix CPU.");
            }
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
