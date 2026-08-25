use super::micro_arch::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

/// Platform-specific CPU detection result, used by `cpu::Cpu::detect()`.
pub struct OsCpuInfo {
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub cores: BTreeMap<(CoreType, Midr), CpuCore>,
    pub model: String,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

/// Shared helper used by Linux and Windows detection.
/// Iterates over MIDRs, assigning core types/names via `CpuArch::find()`
/// and merging cache data from the runtime or sysfs.
#[cfg(not(target_os = "macos"))]
pub(crate) fn detect_cores(midrs: &[Midr]) -> BTreeMap<(CoreType, Midr), CpuCore> {
    let mut cores: BTreeMap<(CoreType, Midr), CpuCore> = BTreeMap::new();

    let runtime_cache = Cache::detect();

    #[cfg(any(target_os = "android", target_os = "linux"))]
    let sysfs_per_type = Cache::from_sys_fs_per_type();

    let mut core_cache_map: BTreeMap<usize, Option<Cache>> = BTreeMap::new();

    let mut unique_midrs: Vec<Midr> = midrs.to_vec();
    unique_midrs.sort();
    unique_midrs.dedup();

    for midr in &unique_midrs {
        #[cfg(any(target_os = "android", target_os = "linux"))]
        let cache = sysfs_per_type
            .as_ref()
            .and_then(|m| m.get(&midr.to_bits()).copied())
            .or(runtime_cache)
            .or(None);

        #[cfg(not(any(target_os = "android", target_os = "linux")))]
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

        cores
            .entry((core_type, *midr))
            .and_modify(|c| c.count += 1)
            .or_insert(CpuCore {
                implementer,
                kind: core_type,
                micro_arch,
                code_name,
                cache,
                count: 1,
            });
    }

    for core in cores.values_mut() {
        if let Some(c) = &mut core.cache {
            c.resolve_share_counts(core.count, core.count, 1);
        }
    }

    cores
}

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

// ----------------------------------------------------------------------------
// ! Android
// ----------------------------------------------------------------------------

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub use android::*;

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
    use crate::arm::brand::{IMPL_ARM, IMPL_QUALCOMM, Vendor};

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
            .get(&(CoreType::Efficiency, silver_midr))
            .expect("silver core missing");
        assert_eq!(silver_core.count, 6);
        assert_eq!(silver_core.kind, CoreType::Efficiency);
        assert_eq!(silver_core.implementer, Vendor::Arm);
        assert_eq!(silver_core.micro_arch, MicroArch::ArmCortexA55);

        let gold_core = cores
            .get(&(CoreType::Performance, gold_midr))
            .expect("gold core missing");
        assert_eq!(gold_core.count, 2);
        assert_eq!(gold_core.kind, CoreType::Performance);
        assert_eq!(gold_core.implementer, Vendor::Arm);
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
            .get(&(CoreType::Efficiency, silver_midr))
            .expect("silver core missing");
        assert_eq!(silver.count, 4);
        assert_eq!(silver.implementer, Vendor::Qualcomm);

        let gold = cores
            .get(&(CoreType::Performance, gold_midr))
            .expect("gold core missing");
        assert_eq!(gold.count, 4);
        assert_eq!(gold.implementer, Vendor::Qualcomm);
    }
}
