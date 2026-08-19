//! DOS-specific fallback implementations for pre-CPUID systems.

use crate::common::Cache;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Returns CPU features for real-mode DOS (pre-CPUID systems).
#[must_use]
pub fn dos_feature_list() -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::new();
    if crate::x86::fns::has_cpuid() && crate::x86::features::has_fpu() {
        map.insert("Base", String::from("FPU"));
    }
    map
}

/// Returns physical cores per package for DOS fallback (always 1).
#[inline(always)]
#[must_use]
pub fn dos_cores_per_package() -> u32 {
    1
}

/// Returns logical threads per package for DOS fallback (always 1).
#[inline(always)]
#[must_use]
pub fn dos_threads_per_package() -> u32 {
    1
}

/// Returns logical threads per physical core for DOS fallback (always 1).
#[inline(always)]
#[must_use]
pub fn dos_threads_per_core() -> u32 {
    1
}

/// Returns cache configuration for DOS fallback (always None on pre-CPUID).
#[inline(always)]
#[must_use]
pub fn dos_cache_detect() -> Option<Cache> {
    None
}
