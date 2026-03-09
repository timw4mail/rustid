use super::brand::{CpuBrand, VENDOR_AMD, VENDOR_INTEL};
use crate::cpuid::{EXT_LEAF_1D, get_ht, max_extended_leaf, vendor_str};

#[allow(unused_imports)]
use super::{EXT_LEAF_5, EXT_LEAF_6, LEAF_4, LEAF_16, max_leaf, x86_cpuid, x86_cpuid_count};

const DATA_CACHE: u32 = 1;
const INSTRUCTION_CACHE: u32 = 2;
const UNIFIED_CACHE: u32 = 3;

const L1: u32 = 1;
const L2: u32 = 2;
const L3: u32 = 3;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum CacheType {
    Unified,
    Data,
    Instruction,
    #[default]
    Invalid,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CacheLevel {
    assoc: u32,
    pub(crate) size: u32,
    kind: CacheType,
}

impl CacheLevel {
    pub fn new(size: u32, kind: CacheType) -> Self {
        CacheLevel {
            size,
            kind,
            assoc: 0,
        }
    }

    pub fn new_unified(size: u32) -> Self {
        Self::new(size, CacheType::Unified)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Level1Cache {
    Unified(CacheLevel),
    Split {
        data: CacheLevel,
        instruction: CacheLevel,
    },
}

impl Level1Cache {
    pub fn new_unified(size: u32) -> Self {
        Level1Cache::Unified(CacheLevel::new_unified(size))
    }

    pub fn is_unified(&self) -> bool {
        match self {
            Level1Cache::Unified(_) => true,
            Level1Cache::Split { .. } => false,
        }
    }

    pub fn is_split(&self) -> bool {
        !self.is_unified()
    }

    pub fn set_data(&mut self, size: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.size = size;
            data.kind = CacheType::Data;
        }
    }

    pub fn set_instruction(&mut self, size: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.size = size;
            instruction.kind = CacheType::Instruction;
        }
    }

    pub fn default_split() -> Self {
        Level1Cache::Split {
            data: CacheLevel::default(),
            instruction: CacheLevel::default(),
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            Level1Cache::Unified(level) => level.size,
            Level1Cache::Split { data, instruction } => data.size + instruction.size,
        }
    }
}

impl Default for Level1Cache {
    fn default() -> Self {
        Level1Cache::Unified(CacheLevel::default())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Cache {
    pub l1: Level1Cache,
    pub l2: Option<CacheLevel>,
    pub l3: Option<CacheLevel>,
}

impl Cache {
    pub fn new(l1: Level1Cache, l2: Option<CacheLevel>, l3: Option<CacheLevel>) -> Cache {
        Cache { l1, l2, l3 }
    }

    pub fn detect() -> Option<Self> {
        // Check for support for the Intel method
        match CpuBrand::detect() {
            CpuBrand::Intel => Self::detect_intel(),
            CpuBrand::AMD => Self::detect_amd(),
            _ => Self::detect_intel(),
        }
    }

    fn detect_amd() -> Option<Self> {
        if max_extended_leaf() >= EXT_LEAF_1D {
            Cache::detect_general(EXT_LEAF_1D)
        } else {
            Cache::detect_amd_fallback()
        }
    }

    fn detect_amd_fallback() -> Option<Self> {
        let res5 = x86_cpuid(EXT_LEAF_5);
        let res6 = x86_cpuid(EXT_LEAF_6);

        let mut c = Cache {
            l1: Level1Cache::default_split(),
            ..Cache::default()
        };

        c.l1.set_data((res5.ecx >> 24) * 1024);
        c.l1.set_instruction((res5.edx >> 24) * 1024);

        let l2size = (res6.ecx >> 16) * 1024;
        let l3size = (res6.edx >> 18) * 512 * 1024;

        if l2size != 0 {
            c.l2 = Some(CacheLevel::new_unified(l2size));
        }

        if l3size != 0 {
            c.l3 = Some(CacheLevel::new_unified(l3size));
        }

        Some(c)
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

        for level in 0u32..32 {
            let res = x86_cpuid_count(leaf, level);
            let cache_type = res.eax & 0xF;

            // If cache_type is 0, the cache type is invalid
            if cache_type == 0 {
                break;
            }

            let cache_level = (res.eax >> 5) & 0x7;
            let cache_sets = res.ecx + 1;
            let cache_line_size = (res.ebx & 0xFFF) + 1;
            let cache_partitions = ((res.ebx >> 12) & 0x3FF) + 1;
            let cache_ways_of_associativity = ((res.ebx >> 22) & 0x3FF) + 1;

            let cache_size =
                cache_sets * cache_partitions * cache_ways_of_associativity * cache_line_size;

            match cache_type {
                DATA_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_data(cache_size);
                    }
                }
                INSTRUCTION_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_instruction(cache_size);
                    }
                }
                UNIFIED_CACHE => match cache_level {
                    L1 => {
                        c.l1 = Level1Cache::new_unified(cache_size);
                    }
                    L2 => {
                        c.l2 = Some(CacheLevel::new(cache_size, CacheType::Unified));
                    }
                    L3 => {
                        c.l3 = Some(CacheLevel::new(cache_size, CacheType::Unified));
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
        match vendor_str().as_str() {
            VENDOR_INTEL => {
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
            _ => Speed::measure(),
        }
    }

    fn measure() -> Self {
        if !super::has_tsc() {
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

#[derive(Debug, Default)]
pub struct Topology {
    pub cores: u32,
    pub threads: u32,

    #[cfg(not(target_os = "none"))]
    pub speed: Speed,

    pub cache: Option<Cache>,
}

impl Topology {
    pub fn detect_core_count() -> u32 {
        match vendor_str().as_str() {
            VENDOR_INTEL => {
                if max_leaf() < LEAF_4 {
                    return 1;
                }

                let res = x86_cpuid(LEAF_4);

                (res.ebx >> 26) + 1
            }
            VENDOR_AMD => {
                if get_ht() != 0 {
                    return super::logical_cores() / (get_ht() + 1);
                };

                1
            }
            _ => 1,
        }
    }

    #[cfg(not(target_os = "none"))]
    pub fn detect() -> Self {
        let threads = super::logical_cores();
        let cores = Self::detect_core_count();
        let speed = Speed::detect();
        let cache = Cache::detect();

        Topology {
            cores,
            threads,
            speed,
            cache,
        }
    }

    #[cfg(target_os = "none")]
    pub fn detect() -> Self {
        let mut t = Topology::default();
        t.cache = Cache::detect();

        return t;
    }
}
