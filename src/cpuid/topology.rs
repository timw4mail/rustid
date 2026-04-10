use super::brand::*;
use super::{
    EXT_LEAF_26, LEAF_0B, LEAF_1F, StaticVec, has_ht, is_valid_leaf, logical_cores, vendor_str,
    x86_cpuid_count,
};
use crate::common::cache::Cache;

/// CPU speed information (base and boost frequencies).
#[derive(Debug, Default, PartialEq)]
pub struct Speed {
    /// Base frequency in MHz
    pub base: u32,
    /// Boost frequency in MHz
    pub boost: u32,
    /// Whether the frequency was measured (vs reported by CPU)
    pub measured: bool,
}

impl Speed {
    /// Detects the CPU speed from available sources.
    pub fn detect() -> Self {
        use super::{LEAF_16, x86_cpuid};
        match &*vendor_str() {
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
            #[cfg(target_arch = "x86")]
            VENDOR_TRANSMETA => {
                use crate::cpuid::TRANSMETA_LEAF_1;

                if !is_valid_leaf(TRANSMETA_LEAF_1) {
                    return Speed::measure();
                }

                let res = x86_cpuid(TRANSMETA_LEAF_1);
                let base = res.ecx;
                let boost = res.ecx;

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
        #[cfg(not(target_os = "none"))]
        if !super::has_tsc() {
            return Speed::default();
        }

        let freq = Self::measure_frequency();
        if freq == 0 {
            return Speed::default();
        }

        Speed {
            base: freq,
            boost: freq,
            measured: true,
        }
    }

    #[cfg(not(target_os = "none"))]
    fn measure_frequency() -> u32 {
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

pub type DomainList = StaticVec<TopologyDomain, 8>;

/// Complete CPU topology information including sockets, cores, threads, and cache.
#[derive(Debug, Default, PartialEq)]
pub struct Topology {
    /// Number of processor sockets
    pub sockets: u32,
    /// Number of physical cores
    pub cores: u32,
    /// Number of logical threads (includes SMT)
    pub threads: u32,
    /// CPU speed information
    pub speed: Speed,
    /// Cache hierarchy information
    pub cache: Option<Cache>,

    #[allow(unused)]
    domains: DomainList,
}

impl Topology {
    /// Detects and returns the CPU topology.
    pub fn detect() -> Self {
        let speed = Speed::detect();

        let cache = Cache::detect();
        let domains: DomainList = Self::detect_domains();
        let (sockets, cores, threads) = Self::count_domains(&domains);

        Topology {
            sockets,
            cores: cores * sockets,
            threads: threads * sockets,
            speed,
            cache,
            domains,
        }
    }

    /// Returns (sockets, cores, threads)
    // TODO: verify socket count from domains
    fn count_domains(domains: &DomainList) -> (u32, u32, u32) {
        let threads = logical_cores();

        // Old school socket counts
        let sockets: u32 = {
            #[cfg(any(target_os = "none", target_os = "linux"))]
            {
                super::mp::MpTable::detect().socket_count()
            }
            #[cfg(not(any(target_os = "none", target_os = "linux")))]
            {
                1
            }
        };

        if domains.is_empty() {
            let (cores, threads) = match &*vendor_str() {
                VENDOR_AMD => {
                    // Logical cpus = cores before Zen, when
                    // Topology domains start returning results
                    if threads > 1 {
                        (threads, threads)
                    } else {
                        (1, 1)
                    }
                }
                VENDOR_INTEL => {
                    if threads < 2 {
                        (1, 1)
                    } else if has_ht() {
                        (threads, threads / 2)
                    } else {
                        (threads, threads)
                    }
                }
                _ => (1, 1),
            };

            return (sockets, cores, threads);
        }

        // Get domain counts in a single iteration
        let mut raw_sockets = 1;
        let mut raw_cores = 1;
        let mut raw_threads = 1;
        for d in domains.iter() {
            match d.kind {
                TopologyType::Socket => raw_sockets = d.count,
                TopologyType::Core => raw_cores = d.count,
                TopologyType::Thread => raw_threads = d.count,
                _ => {}
            }
        }

        // TODO: Socket/Die/Core/Thread

        // Thread/core
        if raw_sockets == 1 && raw_cores > raw_threads {
            return (raw_sockets, raw_cores / raw_threads, raw_cores);
        }

        // AMD has literal core count for 'Core' type domain
        // other have Cores * Threads
        match &*vendor_str() {
            // AMD has literal core count
            VENDOR_AMD => (sockets, raw_cores, raw_threads * raw_cores),
            // Others have 'Core' as Threads * Cores
            _ => (sockets, raw_cores / raw_threads, raw_cores),
        }
    }

    fn detect_domains() -> DomainList {
        let d: DomainList = StaticVec::new();

        if !is_valid_leaf(LEAF_0B) {
            return d;
        }

        let v2_leaf = match &*vendor_str() {
            VENDOR_INTEL => LEAF_1F,
            VENDOR_AMD => EXT_LEAF_26,
            _ => 0,
        };

        if v2_leaf > 0 && is_valid_leaf(v2_leaf) {
            Self::detect_domains_leaf(v2_leaf)
        } else {
            Self::detect_domains_leaf(LEAF_0B)
        }
    }

    fn detect_domains_leaf(leaf: u32) -> DomainList {
        let mut d: DomainList = StaticVec::new();

        if !is_valid_leaf(leaf) {
            return d;
        }

        for subleaf in 0..16 {
            let res = x86_cpuid_count(leaf, subleaf);

            // let x2apic_id_shift = res.eax & 0b1111;
            let domain_lcpus = res.ebx;
            let level = res.ecx & 0x7;
            let domain_type = res.ecx >> 8;

            if domain_type == 0 {
                break;
            }

            match leaf {
                // Topology v1
                LEAF_0B => {
                    d.push(TopologyDomain {
                        level,
                        kind: match domain_type {
                            1 => TopologyType::Thread,
                            2 => TopologyType::Core,
                            _ => TopologyType::Invalid,
                        },
                        count: domain_lcpus,
                    });
                }
                // Intel Topology V2
                LEAF_1F => {
                    d.push(TopologyDomain {
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
                // AMD Topology V2
                EXT_LEAF_26 => {
                    d.push(TopologyDomain {
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
                _ => return d,
            };
        }

        d
    }
}
