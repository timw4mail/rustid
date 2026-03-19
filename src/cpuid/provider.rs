use super::*;
use std::sync::{LazyLock, RwLock};

/// Trait abstracting the CPUID provider, allowing for mocking in tests.
///
/// This trait enables dependency injection of CPUID providers,
/// which is useful for testing without requiring real x86 hardware.
pub trait CpuidProvider: Send + Sync {
    /// Execute CPUID with the given leaf and sub-leaf.
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid;
}

/// Real CPUID provider that executes the CPUID instruction on x86 hardware.
pub struct RealCpuid;

impl CpuidProvider for RealCpuid {
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        real_x86_cpuid_count(leaf, sub_leaf)
    }
}

pub(crate) static PROVIDER: LazyLock<RwLock<Box<dyn CpuidProvider + Send + Sync>>> =
    LazyLock::new(|| RwLock::new(Box::new(RealCpuid) as Box<dyn CpuidProvider + Send + Sync>));

/// Sets a custom CPUID provider (used primarily for testing).
pub fn set_cpuid_provider<P: CpuidProvider + Send + Sync + 'static>(provider: P) {
    let mut p = PROVIDER.write().unwrap();
    *p = Box::new(provider);
}

/// Resets the CPUID provider to the real implementation.
pub fn reset_cpuid_provider() {
    let mut p = PROVIDER.write().unwrap();
    *p = Box::new(RealCpuid);
}
