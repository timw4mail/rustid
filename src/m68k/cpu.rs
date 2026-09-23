//! Contains the Cpu struct for Motorola 680x0.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;

use crate::common::cache::{Cache, CacheLevel, CacheType, Level1Cache};
use crate::common::{
    CoreType, DataSource, Speed, SystemInfo, TDetect, Topology, TopologyTier, UNK,
};
use crate::m68k::gestalt;
use crate::m68k::micro_arch::{CpuArch, CpuCore, FpuType, MicroArch, MmuType};

/// Motorola 68k architecture-specific data.
#[derive(Debug, Default, PartialEq)]
pub struct M68kData {
    pub cpu_type: u32,
    pub fpu_type: FpuType,
    pub mmu_type: MmuType,
    pub cpu_arch: CpuArch,
    pub clock_speed_source: DataSource,
    pub mach_id: u32,
    pub sys_version_bcd: u32,
}

pub type Cpu = crate::common::Cpu<M68kData, MicroArch>;

impl Cpu {
    /// Detects L1 cache sizes for 68k models with on-chip caches.
    pub fn detect_cache(arch: &CpuArch) -> Option<Cache> {
        let mut cache = Cache::default();
        match arch.micro_arch {
            MicroArch::M68020 | MicroArch::M68EC020 => {
                // 256 bytes direct-mapped instruction cache
                cache.l1 = Level1Cache::Split {
                    data: CacheLevel::default(),
                    instruction: CacheLevel::no_count(256, CacheType::Instruction, 1),
                };
                Some(cache)
            }
            MicroArch::M68030 | MicroArch::M68EC030 => {
                // 256 bytes I-cache + 256 bytes D-cache
                cache.l1 = Level1Cache::Split {
                    data: CacheLevel::no_count(256, CacheType::Data, 1),
                    instruction: CacheLevel::no_count(256, CacheType::Instruction, 1),
                };
                Some(cache)
            }
            MicroArch::M68040 | MicroArch::M68LC040 | MicroArch::M68EC040 => {
                // 4KB I-cache + 4KB D-cache, 4-way set associative
                cache.l1 = Level1Cache::Split {
                    data: CacheLevel::no_count(4096, CacheType::Data, 4),
                    instruction: CacheLevel::no_count(4096, CacheType::Instruction, 4),
                };
                Some(cache)
            }
            MicroArch::M68060 | MicroArch::M68LC060 | MicroArch::M68EC060 => {
                // 8KB I-cache + 8KB D-cache, 4-way set associative
                cache.l1 = Level1Cache::Split {
                    data: CacheLevel::no_count(8192, CacheType::Data, 4),
                    instruction: CacheLevel::no_count(8192, CacheType::Instruction, 4),
                };
                Some(cache)
            }
            _ => None,
        }
    }
}

impl TDetect for Cpu {
    fn detect() -> Self {
        #[cfg(all(target_arch = "m68k", not(test)))]
        {
            let (cpu_type, fpu_type, mmu_type, mach_id, clk_hz, sysv_bcd) =
                super::query_gestalt_m68k();
            let cpu_arch = CpuArch::find_by_gestalt(cpu_type, fpu_type, mmu_type);
            let speed_mhz = (clk_hz / 1_000_000) as u32;
            let speed = if speed_mhz > 0 {
                Some(Speed {
                    base: speed_mhz,
                    boost: speed_mhz,
                    measured: false,
                })
            } else {
                None
            };

            let cache = Self::detect_cache(&cpu_arch);

            let cores = vec![CpuCore {
                kind: CoreType::Performance,
                micro_arch: cpu_arch.micro_arch,
                name: if cpu_arch.marketing_name != UNK {
                    Some(cpu_arch.marketing_name.to_string())
                } else {
                    None
                },
                implementer: None,
                cache,
                speed,
                count: 1,
                threads: 1,
            }];

            let extra = M68kData {
                cpu_type,
                fpu_type: cpu_arch.default_fpu,
                mmu_type: cpu_arch.default_mmu,
                cpu_arch: cpu_arch.clone(),
                clock_speed_source: DataSource::DefaultValue,
                mach_id,
                sys_version_bcd: sysv_bcd,
            };

            let system = Some(SystemInfo::new(
                Some("Apple Computer, Inc.".to_string()),
                DataSource::DefaultValue,
                Some(gestalt::get_mac_model_name(mach_id).to_string()),
                DataSource::DefaultValue,
            ));

            let topology = Topology {
                sockets: TopologyTier::new(1, DataSource::DefaultValue),
                cores: TopologyTier::new(1, DataSource::DefaultValue),
                threads: TopologyTier::new(1, DataSource::DefaultValue),
                speed: speed.unwrap_or_default(),
                cache,
                ..Default::default()
            };

            Self {
                system,
                vendor: String::from("Motorola"),
                model: cpu_arch.marketing_name.to_string(),
                topology,
                cores,
                features: BTreeMap::new(),
                extra,
            }
        }
        #[cfg(not(all(target_arch = "m68k", not(test))))]
        {
            // Fallback for tests / non-m68k host builds
            let cpu_arch = CpuArch::find_by_gestalt(4, 2, 3); // Default to 68030 with 68882 FPU and MMU
            let speed = Some(Speed {
                base: 25,
                boost: 25,
                measured: false,
            });
            let cache = Self::detect_cache(&cpu_arch);

            let cores = vec![CpuCore {
                kind: CoreType::Performance,
                micro_arch: cpu_arch.micro_arch,
                name: Some(cpu_arch.marketing_name.to_string()),
                implementer: None,
                cache,
                speed,
                count: 1,
                threads: 1,
            }];

            let extra = M68kData {
                cpu_type: 4,
                fpu_type: cpu_arch.default_fpu,
                mmu_type: cpu_arch.default_mmu,
                cpu_arch: cpu_arch.clone(),
                clock_speed_source: DataSource::DefaultValue,
                mach_id: 9, // SE/30
                sys_version_bcd: 0x0755,
            };

            let system = Some(SystemInfo::new(
                Some("Apple Computer, Inc.".to_string()),
                DataSource::DefaultValue,
                Some(gestalt::get_mac_model_name(9).to_string()),
                DataSource::DefaultValue,
            ));

            let topology = Topology {
                sockets: TopologyTier::new(1, DataSource::DefaultValue),
                cores: TopologyTier::new(1, DataSource::DefaultValue),
                threads: TopologyTier::new(1, DataSource::DefaultValue),
                speed: speed.unwrap_or_default(),
                cache,
                ..Default::default()
            };

            Self {
                system,
                vendor: String::from("Motorola"),
                model: cpu_arch.marketing_name.to_string(),
                topology,
                cores,
                features: BTreeMap::new(),
                extra,
            }
        }
    }
}
