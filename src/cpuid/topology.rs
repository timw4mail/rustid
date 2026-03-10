use super::brand::{VENDOR_AMD, VENDOR_INTEL};
use super::cache::Cache;
use super::{
    EXT_LEAF_26, LEAF_0B, LEAF_1F, LEAF_16, max_extended_leaf, max_leaf, vendor_str, x86_cpuid,
    x86_cpuid_count,
};

use heapless::Vec;

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

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct TopologyDomain {
    level: u32,
    kind: TopologyType,
    count: u32,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum TopologyType {
    #[default]
    Invalid,
    Thread,
    Core,
    Die,
    Socket,
    Module,
    Tile,
    DieGroup,
}

#[derive(Debug, Default)]
pub struct Topology {
    pub cores: u32,
    pub threads: u32,

    #[cfg(not(target_os = "none"))]
    pub speed: Speed,

    pub cache: Option<Cache>,

    #[allow(unused)]
    domains: Vec<TopologyDomain, 64>,
}

impl Topology {
    #[cfg(not(target_os = "none"))]
    pub fn detect() -> Self {
        let speed = Speed::detect();
        let cache = Cache::detect();
        let domains: Vec<TopologyDomain, 64> = Self::detect_domains();

        let raw_cores = domains
            .iter()
            .find(|d| d.kind == TopologyType::Core)
            .map(|d| d.count)
            .unwrap_or(1);
        let raw_threads = domains
            .iter()
            .find(|d| d.kind == TopologyType::Thread)
            .map(|d| d.count)
            .unwrap_or(1);

        let threads = if vendor_str().as_str() == VENDOR_AMD && raw_threads < raw_cores {
            raw_threads * raw_cores
        } else {
            // In the case of Intel/Centaur, the 'Core' count is Cores * Threads,
            raw_cores
        };

        let cores = if raw_cores == threads {
            raw_cores / raw_threads
        } else {
            raw_cores
        };

        Topology {
            cores,
            threads,
            speed,
            cache,
            domains,
        }
    }

    fn detect_domains() -> Vec<TopologyDomain, 64> {
        match vendor_str().as_str() {
            VENDOR_INTEL => Self::detect_domains_intel(),
            VENDOR_AMD => Self::detect_domains_amd(),
            _ => Self::detect_domains_fallback(),
        }
    }

    fn detect_domains_intel() -> Vec<TopologyDomain, 64> {
        if max_leaf() < LEAF_1F {
            return Self::detect_domains_fallback();
        }

        let mut d: Vec<TopologyDomain, 64> = Vec::new();

        for subleaf in 0..64 {
            let res = x86_cpuid_count(LEAF_1F, subleaf);
            let domain_lcpus = res.ebx;
            let level = res.ecx & 0x7;
            let domain_type = res.ecx >> 8;

            if domain_type == 0 {
                break;
            }

            let _ = d.push(TopologyDomain {
                level,
                kind: match domain_type {
                    1 => TopologyType::Thread,
                    2 => TopologyType::Core,
                    3 => TopologyType::Module,
                    4 => TopologyType::Tile,
                    5 => TopologyType::Die,
                    6 => TopologyType::Socket,
                    _ => TopologyType::Invalid,
                },
                count: domain_lcpus,
            });
        }

        d
    }

    fn detect_domains_amd() -> Vec<TopologyDomain, 64> {
        if max_extended_leaf() < EXT_LEAF_26 {
            return Self::detect_domains_fallback();
        }

        let mut d: Vec<TopologyDomain, 64> = Vec::new();

        for subleaf in 0..64 {
            let res = x86_cpuid_count(EXT_LEAF_26, subleaf);
            let domain_lcpus = res.ebx;
            let level = res.ecx & 0x7;
            let domain_type = res.ecx >> 8;

            if domain_type == 0 {
                break;
            }

            let _ = d.push(TopologyDomain {
                level,
                kind: match domain_type {
                    1 => TopologyType::Thread,
                    2 => TopologyType::Core,
                    3 => TopologyType::Die,
                    4 => TopologyType::Socket,
                    _ => TopologyType::Invalid,
                },
                count: domain_lcpus,
            });
        }

        d
    }

    fn detect_domains_fallback() -> Vec<TopologyDomain, 64> {
        let mut d: Vec<TopologyDomain, 64> = Vec::new();

        if max_leaf() < LEAF_0B {
            return d;
        }

        for subleaf in 0..64 {
            let res = x86_cpuid_count(LEAF_0B, subleaf);
            let domain_lcpus = res.ebx;
            let level = res.ecx & 0x7;
            let domain_type = res.ecx >> 8;

            if domain_type == 0 {
                break;
            }

            let _ = d.push(TopologyDomain {
                level,
                kind: match domain_type {
                    1 => TopologyType::Thread,
                    2 => TopologyType::Core,
                    _ => TopologyType::Invalid,
                },
                count: domain_lcpus,
            });
        }

        d
    }

    #[cfg(target_os = "none")]
    pub fn detect() -> Self {
        let mut t = Topology::default();
        t.cache = Cache::detect();

        return t;
    }
}
