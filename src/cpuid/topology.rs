use super::constants::*;
use super::mp::MpTable;
use super::{has_ht, is_valid_leaf, vendor_str, x86_cpuid_count};
use crate::common::{Cache, Speed};
use alloc::vec::Vec;

#[cfg(not(target_os = "none"))]
use super::{info_source, provider::CpuidInfoSource};

impl Speed {
    /// Detects the CPU speed from available sources.
    #[must_use]
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
        if info_source() == CpuidInfoSource::DumpFile || !super::has_tsc() {
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
#[derive(Debug, Default, PartialEq, Copy, Clone)]
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

pub type DomainList = Vec<TopologyDomain>;

/// Complete CPU topology information including sockets, cores, threads, and cache.
#[derive(Debug, Default, PartialEq)]
pub struct Topology {
    /// Number of processor sockets
    pub sockets: u32,
    /// Number of dies per socket
    pub dies: u32,
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
    #[must_use]
    pub fn detect() -> Self {
        let speed = Speed::detect();
        let cache = Cache::detect();
        let domains: DomainList = Self::detect_domains();
        let (sockets, cores, threads) = Self::count_domains(&domains);

        let mut threads_per_socket = 0;
        let mut threads_per_die = 0;

        for d in &domains {
            if d.count > threads_per_socket {
                threads_per_socket = d.count;
            }
            if d.kind == TopologyType::Die {
                threads_per_die = d.count;
            }
        }

        let dies = if threads_per_die > 0 && threads_per_socket > 0 {
            (threads_per_socket / threads_per_die).max(1)
        } else {
            1
        };

        Topology {
            sockets,
            dies,
            cores,
            threads,
            speed,
            cache,
            domains,
        }
    }

    fn amd_threads() -> u32 {
        if vendor_str() != VENDOR_AMD {
            return 1;
        }

        // 1. Try modern V2 topology leaves
        // 2. Try V1 topology leaf
        for &leaf in &[EXT_LEAF_26, LEAF_1F, LEAF_0B] {
            if is_valid_leaf(leaf) {
                let mut subleaf = 0;
                let mut max_lcpus = 0;
                loop {
                    let res = x86_cpuid_count(leaf, subleaf);
                    let domain_type = (res.ecx >> 8) & 0xFF;
                    if domain_type == 0 {
                        break;
                    }
                    max_lcpus = res.ebx;
                    subleaf += 1;
                }
                if max_lcpus > 0 {
                    return max_lcpus;
                }
            }
        }

        // 3. Fallback for AMD
        if is_valid_leaf(EXT_LEAF_8) {
            let res = super::x86_cpuid(EXT_LEAF_8);
            let count = (res.ecx & 0xFF) + 1;
            if count > 1 {
                return count;
            }
        }

        // 4. Fallback for Leaf 1
        if is_valid_leaf(LEAF_1) {
            let res = super::x86_cpuid(LEAF_1);
            let count = (res.ebx >> 16) & 0xFF;
            if count > 0 {
                return count;
            }
        }

        1
    }

    /// Returns (sockets, total_cores, total_threads)
    fn count_domains(domains: &DomainList) -> (u32, u32, u32) {
        let threads_total_fallback = Self::amd_threads();

        // 1. Initial socket count detection (Platform-specific fallbacks)
        // TODO: detect socket count from domains
        #[cfg(target_os = "none")]
        let sockets_detected = MpTable::detect().socket_count();

        #[cfg(not(target_os = "none"))]
        let sockets_detected: u32 = if info_source() == CpuidInfoSource::Cpu {
            MpTable::detect().socket_count()
        } else {
            1
        };

        if domains.is_empty() {
            let (cores, threads) = match &*vendor_str() {
                VENDOR_AMD if threads_total_fallback > 1 => {
                    (threads_total_fallback, threads_total_fallback)
                }
                VENDOR_INTEL => {
                    if threads_total_fallback < 2 {
                        (1, 1)
                    } else if has_ht() {
                        (threads_total_fallback / 2, threads_total_fallback)
                    } else {
                        (threads_total_fallback, threads_total_fallback)
                    }
                }
                _ => (1, 1),
            };

            return (
                sockets_detected,
                cores * sockets_detected,
                threads * sockets_detected,
            );
        }

        // 2. Extract domain counts
        let mut threads_per_core = 1;
        let mut threads_per_package = 0;

        for d in domains {
            if d.kind == TopologyType::Thread {
                threads_per_core = d.count;
            }
            if d.count > threads_per_package {
                threads_per_package = d.count;
            }
        }

        if threads_per_package == 0 {
            threads_per_package = threads_total_fallback;
        }

        let t_per_core = threads_per_core.max(1);
        let t_per_pkg = threads_per_package.max(1);
        let c_per_pkg = t_per_pkg / t_per_core;

        (
            sockets_detected,
            c_per_pkg * sockets_detected,
            t_per_pkg * sockets_detected,
        )
    }

    fn detect_domains() -> DomainList {
        let d: DomainList = Vec::new();

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
        let mut d: DomainList = Vec::new();

        if !is_valid_leaf(leaf) {
            return d;
        }

        for subleaf in 0..16 {
            let res = x86_cpuid_count(leaf, subleaf);

            let domain_lcpus = res.ebx;
            let level = res.ecx & 0xFF;
            let domain_type = (res.ecx >> 8) & 0xFF;

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
            }
        }

        d
    }
}
