use super::brand::{VENDOR_AMD, VENDOR_INTEL};
use super::cache::Cache;
use super::{
    EXT_LEAF_26, LEAF_0B, LEAF_1F, has_ht, is_amd, is_valid_leaf, logical_cores, vendor_str,
    x86_cpuid_count,
};

use heapless::Vec;

/// CPU speed information (base and boost frequencies).
#[derive(Debug, Default, PartialEq)]
#[cfg(not(target_os = "none"))]
pub struct Speed {
    /// Base frequency in MHz
    pub base: u32,
    /// Boost frequency in MHz
    pub boost: u32,
    /// Whether the frequency was measured (vs reported by CPU)
    pub measured: bool,
}

#[cfg(not(target_os = "none"))]
impl Speed {
    /// Detects the CPU speed from available sources.
    pub fn detect() -> Self {
        use super::{LEAF_16, x86_cpuid};
        match vendor_str().as_str() {
            VENDOR_INTEL => {
                if !is_valid_leaf(LEAF_16) {
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

        let freq = Self::measure_tsc_frequency();
        if freq == 0 {
            return Speed::default();
        }

        Speed {
            base: freq,
            boost: freq,
            measured: true,
        }
    }

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
}

/// Represents a topology domain (thread, core, die, socket, etc.).
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct TopologyDomain {
    level: u32,
    kind: TopologyType,
    count: u32,
}

/// CPU topology domain type.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum TopologyType {
    /// Invalid or unknown topology level
    #[default]
    Invalid,
    /// Thread level (logical processor)
    Thread,
    /// Core level (physical processor)
    Core,
    /// Die level
    Die,
    /// Socket level (processor package)
    Socket,
    /// Module level
    Module,
    /// Tile level
    Tile,
    /// Die group level
    DieGroup,
}

/// Complete CPU topology information including sockets, cores, threads, and cache.
#[derive(Debug, Default, PartialEq)]
pub struct Topology {
    /// Number of processor sockets
    pub sockets: usize,
    /// Number of physical cores
    pub cores: u32,
    /// Number of logical threads (includes SMT)
    pub threads: u32,

    /// CPU speed information (not available on bare-metal/no_std)
    #[cfg(not(target_os = "none"))]
    pub speed: Speed,

    /// Cache hierarchy information
    pub cache: Option<Cache>,

    #[allow(unused)]
    domains: Vec<TopologyDomain, 16>,
}

impl Topology {
    /// Detects and returns the CPU topology.
    pub fn detect() -> Self {
        #[cfg(not(target_os = "none"))]
        let speed = Speed::detect();

        let cache = Cache::detect();
        let domains: Vec<TopologyDomain, 16> = Self::detect_domains();
        let (cores, threads) = Self::count_domains(&domains);

        let sockets = {
            #[cfg(any(target_os = "none", target_os = "linux", target_os = "windows"))]
            {
                super::mp::MpTable::detect().socket_count()
            }
            #[cfg(not(any(target_os = "none", target_os = "linux", target_os = "windows")))]
            {
                1usize
            }
        };

        Topology {
            sockets,
            cores,
            threads,
            #[cfg(not(target_os = "none"))]
            speed,
            cache,
            domains,
        }
    }

    fn count_domains(domains: &Vec<TopologyDomain, 16>) -> (u32, u32) {
        let amd_threads = if is_amd() { logical_cores() } else { 0 };

        // TODO: determine cores/threads for Intel if domains are empty
        if domains.is_empty() {
            return match vendor_str().as_str() {
                VENDOR_AMD => {
                    if amd_threads > 0 {
                        let cores = if has_ht() {
                            amd_threads / 2
                        } else {
                            amd_threads
                        };

                        (cores, amd_threads)
                    } else {
                        (1, 1)
                    }
                }
                _ => (1, 1),
            };
        }

        // Get the highest count from the domains
        // We'll assume this is the total thread count
        let raw_sockets = domains
            .iter()
            .find(|d| d.kind == TopologyType::Socket)
            .map(|d| d.count)
            .unwrap_or(1);
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

        // Socket/Die/Core/Thread
        if raw_sockets > 1 && raw_threads > 1 && raw_cores > 1 {
            return (raw_sockets / raw_threads, raw_sockets);
        }

        // Thread/core
        if raw_sockets == 1 && raw_cores > raw_threads {
            return (raw_cores / raw_threads, raw_cores);
        }

        // AMD has literal core count for 'Core' type domain
        // other have Cores * Threads
        match vendor_str().as_str() {
            // AMD has literal core count
            VENDOR_AMD => (raw_cores, raw_threads * raw_cores),
            // Others have 'Core' as Threads * Cores
            _ => (raw_cores / raw_threads, raw_cores),
        }
    }

    fn detect_domains() -> Vec<TopologyDomain, 16> {
        match vendor_str().as_str() {
            VENDOR_INTEL => Self::detect_domains_intel(),
            VENDOR_AMD => Self::detect_domains_amd(),
            _ => Self::detect_domains_fallback(),
        }
    }

    fn detect_domains_intel() -> Vec<TopologyDomain, 16> {
        if !is_valid_leaf(LEAF_1F) {
            return Self::detect_domains_fallback();
        }

        let mut d: Vec<TopologyDomain, 16> = Vec::new();

        for subleaf in 0..16 {
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

    fn detect_domains_amd() -> Vec<TopologyDomain, 16> {
        if !is_valid_leaf(EXT_LEAF_26) {
            return Self::detect_domains_fallback();
        }

        let mut d: Vec<TopologyDomain, 16> = Vec::new();

        for subleaf in 0..16 {
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

    fn detect_domains_fallback() -> Vec<TopologyDomain, 16> {
        let mut d: Vec<TopologyDomain, 16> = Vec::new();

        if !is_valid_leaf(LEAF_0B) {
            return d;
        }

        for subleaf in 0..16 {
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
}
