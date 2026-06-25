use super::micro_arch::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

/// Platform-specific CPU detection result, used by `cpu::Cpu::detect()`.
pub struct OsCpuInfo {
    pub raw_midr: HashSet<usize>,
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
    pub model: String,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

/// Shared helper used by Linux and Windows detection.
/// Iterates over MIDRs, assigning core types/names via `CpuArch::find()`
/// and merging cache data from the runtime or sysfs.
#[cfg(not(target_os = "macos"))]
pub(crate) fn detect_cores(midrs: &[Midr]) -> BTreeMap<(CoreType, Option<String>, Midr), CpuCore> {
    let mut cores: BTreeMap<(CoreType, Option<String>, Midr), CpuCore> = BTreeMap::new();

    let runtime_cache = Cache::detect();

    #[cfg(target_os = "linux")]
    let sysfs_per_type = Cache::from_sys_fs_per_type();

    let mut core_cache_map: BTreeMap<usize, Option<Cache>> = BTreeMap::new();

    let mut unique_midrs: Vec<Midr> = midrs.to_vec();
    unique_midrs.sort();
    unique_midrs.dedup();

    for midr in &unique_midrs {
        #[cfg(target_os = "linux")]
        let cache = sysfs_per_type
            .as_ref()
            .and_then(|m| m.get(&midr.to_bits()).copied())
            .or_else(|| runtime_cache)
            .or(None);

        #[cfg(not(target_os = "linux"))]
        let cache = runtime_cache.or(None);

        core_cache_map.insert(midr.to_bits(), cache);
    }

    for midr in midrs {
        let arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
        let core_type = arch.micro_arch.core_type();
        let core_name: String = arch.micro_arch.into();

        let name = if core_name != super::micro_arch::UNK {
            Some(core_name)
        } else {
            None
        };

        let cache = core_cache_map.get(&midr.to_bits()).cloned().flatten();

        cores
            .entry((core_type, name.clone(), *midr))
            .and_modify(|c| c.count += 1)
            .or_insert(CpuCore {
                kind: core_type,
                name,
                cache,
                count: 1,
            });
    }

    cores
}

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod apple;
#[cfg(target_os = "macos")]
pub use apple::*;

// ----------------------------------------------------------------------------
// ! Linux
// ----------------------------------------------------------------------------

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;
