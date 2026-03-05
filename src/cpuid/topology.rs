use crate::cpuid;
use crate::cpuid::brand::CpuBrand;
use crate::cpuid::{EXT_LEAF_1D, max_leaf, x86_cpuid, x86_cpuid_count};
use crate::cpuid::{LEAF_4, LEAF_16};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(not(target_os = "none"))]
#[allow(unused)]
pub enum CacheType {
    Unified,
    Data,
    Instruction,
    Invalid,
}

#[cfg(not(target_os = "none"))]
impl Default for CacheType {
    fn default() -> Self {
        CacheType::Invalid
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg(not(target_os = "none"))]
#[allow(unused)]
pub struct CacheLevel {
    size: u32,
    kind: CacheType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(not(target_os = "none"))]
pub enum Level1Cache {
    Unified(CacheLevel),
    Split {
        data: CacheLevel,
        instruction: CacheLevel,
    },
}

#[cfg(not(target_os = "none"))]
impl Level1Cache {
    pub fn new_unified(size: u32) -> Self {
        Level1Cache::Unified(CacheLevel {
            size,
            kind: CacheType::Unified,
        })
    }

    pub fn is_unified(&self) -> bool {
        match self {
            Level1Cache::Unified(_) => true,
            Level1Cache::Split { .. } => false,
        }
    }

    pub fn set_data(&mut self, size: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.size = size;
        }
    }

    pub fn set_instruction(&mut self, size: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.size = size;
        }
    }

    pub fn default_split() -> Self {
        Level1Cache::Split {
            data: CacheLevel {
                size: 0,
                kind: CacheType::Data,
            },
            instruction: CacheLevel {
                size: 0,
                kind: CacheType::Instruction,
            },
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            Level1Cache::Unified(level) => level.size,
            Level1Cache::Split { data, instruction } => data.size + instruction.size,
        }
    }
}

#[cfg(not(target_os = "none"))]
impl Default for Level1Cache {
    fn default() -> Self {
        Level1Cache::Unified(CacheLevel::default())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg(not(target_os = "none"))]
#[allow(unused)]
pub struct Cache {
    l1: Level1Cache,
    l2: Option<CacheLevel>,
    l3: Option<CacheLevel>,
}

#[cfg(not(target_os = "none"))]
#[allow(unused)]
impl Cache {
    pub fn new(l1: Level1Cache, l2: Option<CacheLevel>, l3: Option<CacheLevel>) -> Cache {
        Cache { l1, l2, l3 }
    }

    pub fn detect() -> Option<Self> {
        // Check for support for the Intel method
        match CpuBrand::detect() {
            CpuBrand::Intel => Self::detect_intel(),
            CpuBrand::AMD => Self::detect_amd(),
            _ => {
                unimplemented!();
            }
        }
    }

    fn detect_amd() -> Option<Self> {
        if EXT_LEAF_1D < max_leaf() {
            Cache::detect_general(EXT_LEAF_1D)
        } else {
            Cache::detect_amd_fallback()
        }
    }

    fn detect_amd_fallback() -> Option<Self> {
        unimplemented!();
    }

    fn detect_intel() -> Option<Self> {
        if LEAF_4 < max_leaf() {
            Cache::detect_general(LEAF_4)
        } else {
            None
        }
    }

    fn detect_general(leaf: u32) -> Option<Self> {
        let mut c = Cache::default();

        for level in 0u32..5 {
            let res = x86_cpuid_count(LEAF_4, level);
            let cache_type = res.eax & 0xF;

            // If cache_type is 0, the cache type is invalid
            if cache_type == 0 {
                break;
            }

            let cache_level = (res.eax >> 5) & 0x7;
            let cache_sets = res.ecx + 1;
            let cache_line_size = (res.ebx & 0xFFF) + 1;
            let cache_partitions = ((res.ebx >> 12) & 0x3FF) + 1;
            let cache_ways_of_associativity = ((res.ebx >> 10) & 0x3FF) + 1;

            let cache_size =
                cache_sets * cache_partitions * cache_ways_of_associativity * cache_line_size;

            match cache_type {
                // Data cache
                1 => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_data(cache_size);
                    }
                }
                // Instruction cache
                2 => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_instruction(cache_size);
                    }
                }
                // Unified cache
                3 => match cache_level {
                    1 => {
                        c.l1 = Level1Cache::new_unified(cache_size);
                    }
                    2 => {
                        c.l2 = Some(CacheLevel {
                            size: cache_size,
                            kind: CacheType::Unified,
                        });
                    }
                    3 => {
                        c.l3 = Some(CacheLevel {
                            size: cache_size,
                            kind: CacheType::Unified,
                        });
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if c == Cache::default() { None } else { Some(c) }
    }
}

#[derive(Debug, Default)]
#[cfg(not(target_os = "none"))]
pub struct Speed {
    pub base: u32,
    pub boost: u32,
    pub measured: bool,
}

#[cfg(not(target_os = "none"))]
impl Speed {
    pub fn detect() -> Self {
        if max_leaf() < LEAF_16 {
            return Speed::measure();
        }

        let res = x86_cpuid(LEAF_16);

        let base = res.eax;
        let boost = res.ebx;

        if base == 0 {
            return Speed::measure();
        }

        Speed {
            base,
            boost,
            measured: false,
        }
    }
    fn measure() -> Self {
        if !cpuid::has_tsc() {
            return Speed::default();
        }

        let freq = measure_tsc_frequency();
        if freq == 0 {
            return Speed::default();
        }

        Speed {
            base: freq,
            boost: freq,
            measured: true,
        }
    }
}

#[cfg(not(target_os = "none"))]
fn measure_tsc_frequency() -> u32 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::_rdtsc as rdtsc;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::_rdtsc as rdtsc;

    const MHZ_DIVISOR: u64 = 1_000_000;

    use core::time::Duration;

    let start_tsc = unsafe { rdtsc() };
    let start_time = std::time::Instant::now();

    let end_time = start_time + Duration::from_millis(10);

    while std::time::Instant::now() < end_time {
        core::hint::spin_loop();
    }

    let end_tsc = unsafe { rdtsc() };

    let elapsed = start_time.elapsed().as_nanos() as u64;
    let tsc_delta = end_tsc - start_tsc;

    if elapsed == 0 {
        return 0;
    }

    let freq_mhz = (tsc_delta * MHZ_DIVISOR) / elapsed;

    (freq_mhz / 1000) as u32
}

#[cfg(target_os = "none")]
#[derive(Debug, Default)]
pub struct Topology;

#[cfg(target_os = "none")]
impl Topology {
    pub fn detect() -> Self {
        Topology::default()
    }
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Default)]
pub struct Topology {
    pub cores: u32,
    pub threads: u32,
    pub speed: Speed,
    pub cache: Option<Cache>,
}

#[cfg(not(target_os = "none"))]
impl Topology {
    pub fn detect() -> Self {
        let threads = cpuid::logical_cores();
        let cores = 1;
        let speed = Speed::detect();
        let cache = None;
        // let cache = Cache::detect();

        Topology::new(cores, threads, speed, cache)
    }
    pub fn new(cores: u32, threads: u32, speed: Speed, cache: Option<Cache>) -> Topology {
        Topology {
            cores,
            threads,
            speed,
            cache,
        }
    }
}
