use crate::cpuid;
use crate::cpuid::{max_leaf, x86_cpuid};
use crate::cpuid::{LEAF_4, LEAF_16};
use crate::cpuid::brand::CpuBrand;

#[derive(Debug, Default)]
#[cfg(not(target_os = "none"))]
#[allow(unused)]
pub struct CacheLevel {
    size: u32,
    exists: bool,
}

#[derive(Debug, Default)]
#[cfg(not(target_os = "none"))]
#[allow(unused)]
pub struct Cache {
    l1i: CacheLevel,
    l1d: CacheLevel,
    l2: Option<CacheLevel>,
    l3: Option<CacheLevel>,
}

#[cfg(not(target_os = "none"))]
#[allow(unused)]
impl Cache {
    pub fn new(
        l1i: CacheLevel,
        l1d: CacheLevel,
        l2: Option<CacheLevel>,
        l3: Option<CacheLevel>,
    ) -> Cache {
        Cache { l1i, l1d, l2, l3 }
    }

    pub fn detect() -> Option<Self> {
        let mut level = LEAF_4;
        // Check for support for the Intel method
        match CpuBrand::detect() {
            CpuBrand::Intel => {
                return if level < max_leaf() {
                    Cache::detect_intel()
                } else {
                    None
                };
            }
            CpuBrand::AMD => {
                unimplemented!();
            }
            _ => {
                unimplemented!();
            }
        }
    }

    fn detect_intel() -> Option<Self> {
        unimplemented!();
    }

    fn detect_amd() -> Option<Self> {
        unimplemented!();
    }

    fn detect_amd_fallback() -> Option<Self> {
        unimplemented!();
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
