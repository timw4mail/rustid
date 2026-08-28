use super::constants::*;
use super::{cpuid_data_source, is_valid_leaf, vendor_str, x86_cpuid_count};
use crate::common::{Cache, DataSource, Speed, TopologyTier};
use crate::x86::{cpuid_cores_per_package, cpuid_threads_per_package};
use alloc::vec::Vec;

#[cfg(std_os)]
use super::{info_source, provider::CpuidInfoSource};

impl Speed {
    /// Detects CPU speed purely from CPUID leaves (Intel Leaf 16 or Transmeta Leaf 0x80860001).
    ///
    /// Returns default (0 MHz, unmeasured) if frequency is not reported in CPUID.
    #[must_use]
    pub fn detect_cpuid() -> Self {
        use super::{LEAF_16, x86_cpuid};
        match &*vendor_str() {
            VENDOR_INTEL => {
                if !is_valid_leaf(LEAF_16) {
                    return Speed::default();
                }

                let res = x86_cpuid(LEAF_16);

                let base = res.eax;
                let boost = res.ebx;

                if base == 0 {
                    return Speed::default();
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
                    return Speed::default();
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
            _ => Speed::default(),
        }
    }

    /// Detects the CPU speed from available sources.
    #[must_use]
    pub fn detect() -> Self {
        let speed = Self::detect_cpuid();
        if speed.base > 0 {
            return speed;
        }

        #[cfg(std_os)]
        {
            if info_source() == CpuidInfoSource::Cpu {
                super::os::measure_frequency()
            } else {
                Speed::default()
            }
        }

        #[cfg(uefi)]
        {
            Self::measure_uefi()
        }

        #[cfg(dos_os)]
        {
            let freq = Self::measure_frequency();
            if freq > 0 {
                Speed {
                    base: freq,
                    boost: freq,
                    measured: true,
                }
            } else {
                Speed::default()
            }
        }
    }

    #[cfg(uefi)]
    fn measure_uefi() -> Self {
        if !super::has_tsc() {
            return Speed::default();
        }

        let freq = Self::measure_frequency_uefi();
        if freq == 0 {
            if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
                if let Some(proc) = smbios.processors.first() {
                    let speed = if proc.current_speed_mhz > 0 {
                        proc.current_speed_mhz as u32
                    } else if proc.max_speed_mhz > 0 {
                        proc.max_speed_mhz as u32
                    } else {
                        0
                    };
                    if speed > 0 {
                        let max_speed = proc.max_speed_mhz as u32;
                        return Speed {
                            base: speed,
                            boost: if max_speed > speed { max_speed } else { speed },
                            measured: false,
                        };
                    }
                }
            }
            return Speed::default();
        }

        Speed {
            base: freq,
            boost: freq,
            measured: true,
        }
    }

    #[cfg(uefi)]
    fn measure_frequency_uefi() -> u32 {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdtsc as rdtsc;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdtsc as rdtsc;

        let st = crate::x86::efi::os::get_system_table();
        if st.is_null() {
            return 0;
        }
        let bs = unsafe { (*st).boot_services };
        if bs.is_null() {
            return 0;
        }

        let start_tsc = unsafe { rdtsc() };
        let status = unsafe { ((*bs).stall)(10_000) };
        if status != 0 {
            return 0;
        }
        let end_tsc = unsafe { rdtsc() };

        let tsc_delta = end_tsc.saturating_sub(start_tsc);
        (tsc_delta / 10_000) as u32
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
    /// Detects CPU topology purely from CPUID leaves without touching OS information.
    #[must_use]
    pub fn detect_cpuid() -> Self {
        let speed = Speed::detect_cpuid();
        let mut cache = Cache::detect();
        let domains: DomainList = Self::detect_domains();
        let (sockets, cores, threads) = Self::count_cpuid_domains(&domains);

        if let Some(c) = &mut cache {
            c.resolve_share_counts(cores.count, threads.count, sockets.count);
        }

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

    /// Detects and returns the CPU topology, enriching with OS information on live hardware.
    #[must_use]
    pub fn detect() -> Self {
        let mut topo = Self::detect_cpuid();

        #[cfg(std_os)]
        if info_source() == CpuidInfoSource::Cpu {
            let os_sockets = super::os::get_socket_count();
            if os_sockets.count > 1 {
                topo.sockets = os_sockets;
                topo.cores = TopologyTier::new(
                    topo.cores
                        .count
                        .max(cpuid_cores_per_package() * os_sockets.count),
                    DataSource::Calculated("OS sockets * CPUID cores"),
                );
                topo.threads = TopologyTier::new(
                    topo.threads
                        .count
                        .max(cpuid_threads_per_package() * os_sockets.count),
                    DataSource::Calculated("OS sockets * CPUID threads"),
                );
                if let Some(c) = &mut topo.cache {
                    c.resolve_share_counts(
                        topo.cores.count,
                        topo.threads.count,
                        topo.sockets.count,
                    );
                }
            }

            if topo.speed.base == 0 {
                let measured = super::os::measure_frequency();
                if measured.base > 0 {
                    topo.speed = measured;
                }
            }
        }

        #[cfg(uefi)]
        {
            let os_sockets = crate::x86::count::get_platform_socket_count();
            if os_sockets.count > 1 {
                topo.sockets = os_sockets;
                topo.cores = TopologyTier::new(
                    topo.cores
                        .count
                        .max(cpuid_cores_per_package() * os_sockets.count),
                    DataSource::Calculated("EFI sockets * CPUID cores"),
                );
                topo.threads = TopologyTier::new(
                    topo.threads
                        .count
                        .max(cpuid_threads_per_package() * os_sockets.count),
                    DataSource::Calculated("EFI sockets * CPUID threads"),
                );
                if let Some(c) = &mut topo.cache {
                    c.resolve_share_counts(
                        topo.cores.count,
                        topo.threads.count,
                        topo.sockets.count,
                    );
                }
            }
            if topo.speed.base == 0 {
                let measured = Speed::measure_uefi();
                if measured.base > 0 {
                    topo.speed = measured;
                }
            }
        }

        #[cfg(dos_os)]
        {
            let mp_table = crate::x86::dos::mp::MpTable::detect();
            let mp_sockets = mp_table.socket_count();
            let total_cores = mp_table.total_cores();
            let total_threads = mp_table.total_threads();

            if mp_sockets > 1 || total_threads > topo.threads.count {
                topo.sockets = TopologyTier::new(mp_sockets, DataSource::MpTable);
                topo.cores = TopologyTier::new(
                    topo.cores.count.max(total_cores),
                    DataSource::Calculated("MP Table * CPUID cores"),
                );
                topo.threads = TopologyTier::new(
                    topo.threads.count.max(total_threads),
                    DataSource::Calculated("MP Table logical processors"),
                );
                if let Some(c) = &mut topo.cache {
                    c.resolve_share_counts(topo.cores.count, topo.threads.count, mp_sockets);
                }
            }
            if topo.speed.base == 0 {
                let measured = Speed::detect();
                if measured.base > 0 {
                    topo.speed = measured;
                }
            }
        }

        topo
    }

    /// Returns (sockets, total_cores, total_threads) from pure CPUID queries
    fn count_cpuid_domains(domains: &DomainList) -> (TopologyTier, TopologyTier, TopologyTier) {
        let sockets = TopologyTier::new(1, cpuid_data_source());
        let threads = cpuid_threads_per_package();
        let cores = cpuid_cores_per_package();

        if domains.is_empty() {
            return (
                sockets,
                TopologyTier::new(cores.max(1), cpuid_data_source()),
                TopologyTier::new(threads.max(1), cpuid_data_source()),
            );
        }

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
            threads_per_package = threads;
        }

        let t_per_core = threads_per_core.max(1);
        let t_per_pkg = threads_per_package.max(1);
        let c_per_pkg = t_per_pkg / t_per_core;

        (
            sockets,
            TopologyTier::new(c_per_pkg.max(1), DataSource::Calculated("Cpuid")),
            TopologyTier::new(t_per_pkg.max(1), DataSource::Calculated("Cpuid")),
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
