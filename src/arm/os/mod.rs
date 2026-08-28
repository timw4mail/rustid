use super::micro_arch::CpuCore;
use super::micro_arch::*;
use crate::arm::brand::Vendor;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

/// Platform-specific CPU detection result, used by `cpu::Cpu::detect()`.
pub struct OsCpuInfo {
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: Vec<CpuCore>,
    pub model: String,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

/// Shared helper used by Linux and Windows detection.
/// Iterates over MIDRs, assigning core types/names via `CpuArch::find()`
/// and merging cache data from the runtime or sysfs.
#[cfg(any(not(target_os = "macos"), test))]
pub(crate) fn detect_cores(midrs: &[Midr]) -> Vec<CpuCore> {
    let mut cores: BTreeMap<(CoreType, Midr), CpuCore> = BTreeMap::new();

    let runtime_cache = Cache::detect();

    #[cfg(linux_os)]
    let sysfs_per_type = Cache::from_sys_fs_per_type();

    let mut core_cache_map: BTreeMap<usize, Option<Cache>> = BTreeMap::new();

    let mut unique_midrs: Vec<Midr> = midrs.to_vec();
    unique_midrs.sort();
    unique_midrs.dedup();

    for midr in &unique_midrs {
        #[cfg(linux_os)]
        let cache = sysfs_per_type
            .as_ref()
            .and_then(|m| m.get(&midr.to_bits()).copied())
            .or(runtime_cache)
            .or(None);

        #[cfg(not(linux_os))]
        let cache = runtime_cache.or(None);

        core_cache_map.insert(midr.to_bits(), cache);
    }

    for midr in midrs {
        let arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
        let core_type = arch.micro_arch.core_type();
        let implementer = arch.implementer;
        let micro_arch = arch.micro_arch;
        let code_name = if arch.code_name != UNK && !arch.code_name.is_empty() {
            Some(arch.code_name.to_string())
        } else {
            None
        };

        let cache = core_cache_map.get(&midr.to_bits()).cloned().flatten();

        let impl_str = if implementer != Vendor::Unknown {
            Some(Into::<&str>::into(implementer).to_string())
        } else {
            None
        };

        cores
            .entry((core_type, *midr))
            .and_modify(|c| {
                c.count += 1;
                c.threads += 1;
            })
            .or_insert(CpuCore {
                kind: core_type,
                micro_arch,
                name: code_name,
                implementer: impl_str,
                cache,
                speed: None,
                count: 1,
                threads: 1,
            });
    }

    for core in cores.values_mut() {
        if let Some(c) = &mut core.cache {
            c.resolve_share_counts(core.count, core.count, 1);
        }
    }

    cores.into_values().collect()
}

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

// ----------------------------------------------------------------------------
// ! Linux / Android
// ----------------------------------------------------------------------------

#[cfg(linux_os)]
pub mod linux;
#[cfg(linux_os)]
pub use linux::*;

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(windows_os)]
pub mod windows;
#[cfg(windows_os)]
pub use windows::*;

// ----------------------------------------------------------------------------
// ! BSD (NetBSD, FreeBSD, OpenBSD)
// ----------------------------------------------------------------------------

#[cfg(bsd)]
pub mod bsd;
#[cfg(bsd)]
pub use bsd::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arm::brand::{IMPL_ARM, IMPL_QUALCOMM};

    #[test]
    fn test_snapdragon_750g_detect_cores() {
        // Snapdragon 750G: 2x Kryo 570 Gold (Cortex-A77) + 6x Kryo 570 Silver (Cortex-A55)
        let gold_midr = Midr {
            implementer: IMPL_ARM,
            variant: 1,
            architecture: 8,
            part: 0xD0D, // Cortex-A77
            revision: 0,
            raw: 0x411FD0D0,
        };
        let silver_midr = Midr {
            implementer: IMPL_ARM,
            variant: 1,
            architecture: 8,
            part: 0xD05, // Cortex-A55
            revision: 0,
            raw: 0x411FD050,
        };

        // 6 silver cores followed by 2 gold cores
        let midrs = vec![
            silver_midr,
            silver_midr,
            silver_midr,
            silver_midr,
            silver_midr,
            silver_midr,
            gold_midr,
            gold_midr,
        ];

        let cores = detect_cores(&midrs);
        assert_eq!(cores.len(), 2);

        let silver_core = cores
            .iter()
            .find(|c| c.kind == CoreType::Efficiency)
            .expect("silver core missing");
        assert_eq!(silver_core.count, 6);
        assert_eq!(silver_core.kind, CoreType::Efficiency);
        assert_eq!(silver_core.implementer.as_deref(), Some("ARM"));
        assert_eq!(silver_core.micro_arch, MicroArch::ArmCortexA55);

        let gold_core = cores
            .iter()
            .find(|c| c.kind == CoreType::Performance)
            .expect("gold core missing");
        assert_eq!(gold_core.count, 2);
        assert_eq!(gold_core.kind, CoreType::Performance);
        assert_eq!(gold_core.implementer.as_deref(), Some("ARM"));
        assert_eq!(gold_core.micro_arch, MicroArch::ArmCortexA77);
    }

    #[test]
    fn test_snapdragon_855_detect_cores() {
        // Snapdragon 855: 4x Kryo 485 Gold (part 0x804, Qualcomm) + 4x Kryo 485 Silver (part 0x805, Qualcomm)
        let gold_midr = Midr {
            implementer: IMPL_QUALCOMM,
            variant: 0,
            architecture: 8,
            part: 0x804,
            revision: 0,
            raw: 0x510F8040,
        };
        let silver_midr = Midr {
            implementer: IMPL_QUALCOMM,
            variant: 0,
            architecture: 8,
            part: 0x805,
            revision: 0,
            raw: 0x510F8050,
        };

        let midrs = vec![
            silver_midr,
            silver_midr,
            silver_midr,
            silver_midr,
            gold_midr,
            gold_midr,
            gold_midr,
            gold_midr,
        ];

        let cores = detect_cores(&midrs);
        assert_eq!(cores.len(), 2);

        let silver = cores
            .iter()
            .find(|c| c.kind == CoreType::Efficiency)
            .expect("silver core missing");
        assert_eq!(silver.count, 4);
        assert_eq!(silver.implementer.as_deref(), Some("Qualcomm"));

        let gold = cores
            .iter()
            .find(|c| c.kind == CoreType::Performance)
            .expect("gold core missing");
        assert_eq!(gold.count, 4);
        assert_eq!(gold.implementer.as_deref(), Some("Qualcomm"));
    }
}
