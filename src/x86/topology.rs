use super::constants::*;
use super::{is_valid_leaf, vendor_str, x86_cpuid_count};
use crate::common::{Cache, DataSource, Speed, TopologyTier};
use crate::x86::count::{get_core_count, get_platform_socket_count, get_thread_count};
use alloc::vec::Vec;

#[cfg(not(nostd_os))]
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
                use crate::x86::TRANSMETA_LEAF_1;

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
        #[cfg(nostd_os)]
        return Speed::default();

        #[cfg(not(nostd_os))]
        {
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
    }

    #[cfg(not(nostd_os))]
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
    pub sockets: TopologyTier,
    /// Number of dies per socket
    pub dies: TopologyTier,
    /// Number of physical cores
    pub cores: TopologyTier,
    /// Number of logical threads (includes SMT)
    pub threads: TopologyTier,
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

        let mut threads_per_socket = 0u32;
        let mut threads_per_die = 0u32;

        for d in &domains {
            if d.count > threads_per_socket {
                threads_per_socket = d.count;
            }
            if d.kind == TopologyType::Die {
                threads_per_die = d.count;
            }
        }

        let dies = if threads_per_die > 0 && threads_per_socket > 0 {
            TopologyTier::new(
                (threads_per_socket / threads_per_die).max(1),
                DataSource::Calculated("Cpuid"),
            )
        } else {
            TopologyTier::default()
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

    /// Returns (sockets, total_cores, total_threads)
    fn count_domains(domains: &DomainList) -> (TopologyTier, TopologyTier, TopologyTier) {
        // 1. Get raw counts from fallback sources
        let sockets = if domains.is_empty() {
            get_platform_socket_count()
        } else {
            TopologyTier::default()
        };
        let threads = get_thread_count();
        let cores = get_core_count();

        if domains.is_empty() {
            return (
                sockets,
                TopologyTier::new(cores.count * sockets.count, cores.source),
                TopologyTier::new(threads.count * sockets.count, threads.source),
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
            threads_per_package = threads.count;
        }

        let t_per_core = threads_per_core.max(1);
        let t_per_pkg = threads_per_package.max(1);
        let c_per_pkg = t_per_pkg / t_per_core;

        (
            sockets,
            TopologyTier::new(c_per_pkg * sockets.count, DataSource::Calculated("Cpuid")),
            TopologyTier::new(t_per_pkg * sockets.count, DataSource::Calculated("Cpuid")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_domain_default() {
        let d = TopologyDomain::default();
        assert_eq!(d.level, 0);
        assert_eq!(d.kind, TopologyType::Invalid);
        assert_eq!(d.count, 0);
    }

    #[test]
    fn test_topology_domain_new() {
        let d = TopologyDomain {
            level: 1,
            kind: TopologyType::Core,
            count: 4,
        };
        assert_eq!(d.level, 1);
        assert_eq!(d.kind, TopologyType::Core);
        assert_eq!(d.count, 4);
    }

    #[test]
    fn test_topology_type_variants() {
        assert_eq!(format!("{:?}", TopologyType::Invalid), "Invalid");
        assert_eq!(format!("{:?}", TopologyType::Thread), "Thread");
        assert_eq!(format!("{:?}", TopologyType::Core), "Core");
        assert_eq!(format!("{:?}", TopologyType::Die), "Die");
        assert_eq!(format!("{:?}", TopologyType::Socket), "Socket");
        assert_eq!(format!("{:?}", TopologyType::Module), "Module");
        assert_eq!(format!("{:?}", TopologyType::Tile), "Tile");
        assert_eq!(format!("{:?}", TopologyType::DieGroup), "DieGroup");
    }

    #[test]
    fn test_topology_default() {
        let t = Topology::default();
        assert_eq!(t.sockets, TopologyTier::default());
        assert_eq!(t.dies, TopologyTier::default());
        assert_eq!(t.cores, TopologyTier::default());
        assert_eq!(t.threads, TopologyTier::default());
        assert_eq!(t.speed, Speed::default());
        assert!(t.cache.is_none());
    }

    #[test]
    fn test_speed_default() {
        let s = Speed::default();
        assert_eq!(s.base, 0);
        assert_eq!(s.boost, 0);
        assert!(!s.measured);
    }

    #[test]
    fn test_speed_new() {
        let s = Speed {
            base: 2400,
            boost: 3200,
            measured: true,
        };
        assert_eq!(s.base, 2400);
        assert_eq!(s.boost, 3200);
        assert!(s.measured);
    }

    #[test]
    fn test_topology_debug() {
        let t = Topology::default();
        let debug = format!("{t:?}");
        assert!(debug.contains("sockets"));
        assert!(debug.contains("cores"));
        assert!(debug.contains("threads"));
    }
}
