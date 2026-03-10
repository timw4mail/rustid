use super::brand::{VENDOR_AMD, VENDOR_INTEL};
use super::cache::Cache;
use super::{LEAF_4, LEAF_16, get_ht, has_ht, max_leaf, vendor_str, x86_cpuid};

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
            _ => {
                if has_ht() {
                    return super::logical_cores() / (get_ht() + 1);
                };

                1
            }
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
