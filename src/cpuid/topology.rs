use super::brand::{VENDOR_AMD, VENDOR_INTEL};
use super::{
    EXT_LEAF_26, LEAF_0B, LEAF_1F, is_amd, is_valid_leaf, logical_cores, vendor_str,
    x86_cpuid_count,
};
use crate::common::cache::Cache;

use heapless::Vec;

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
        #[cfg(target_arch = "x86")]
        use crate::cpuid::VENDOR_TRANSMETA;

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

    #[cfg(target_os = "none")]
    fn measure_frequency() -> u32 {
        use crate::cpuid::dos::peek_u16;

        // Use BIOS timer ticks at 0040:006C
        // 1 tick = 65536 / 1193182 seconds (~54.9 ms)

        let start_ticks = peek_u16(0x0040, 0x006C);
        let mut t1 = start_ticks;

        // Wait for a fresh tick
        while t1 == start_ticks {
            t1 = peek_u16(0x0040, 0x006C);
        }

        if super::has_tsc() {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::_rdtsc as rdtsc;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::_rdtsc as rdtsc;

            let start_tsc = unsafe { rdtsc() };

            // Wait for 2 ticks (~110ms)
            let target_ticks = t1.wrapping_add(2);
            while peek_u16(0x0040, 0x006C) != target_ticks {
                core::hint::spin_loop();
            }

            let end_tsc = unsafe { rdtsc() };
            let tsc_delta = end_tsc - start_tsc;

            // freq_mhz = (tsc_delta * 1193182) / (2 * 65536 * 1_000_000)
            let freq_mhz = (tsc_delta * 1193182) / 131072000000u64;
            freq_mhz as u32
        } else {
            // No TSC (386/486). Use a calibrated instruction loop.
            // We'll count how many times we can run a loop in 2 ticks.

            let mut iterations: u32 = 0;
            let target_ticks = t1.wrapping_add(2);

            unsafe {
                core::arch::asm!(
                    "push es",
                    "mov ax, 0x40",
                    "mov es, ax",
                    "2:",
                    "add {0:e}, 1",
                    "push ax", // Extra work to slow down the loop and be more consistent
                    "pop ax",
                    "mov ax, es:[0x6C]",
                    "cmp ax, {1:x}",
                    "jne 2b",
                    "pop es",
                    inout(reg) iterations,
                    in(reg) target_ticks,
                    out("ax") _,
                );
            }

            // Calibration:
            // 486 loop: add(1) + push(1) + pop(4) + mov mem(1) + cmp(1) + jne(3) = 11 cycles
            // 386 loop: add(2) + push(2) + pop(4) + mov mem(4) + cmp(2) + jne(7) = 21 cycles
            // Time: 2 ticks = ~0.1098 seconds
            // freq_mhz = (iterations * cycles_per_loop) / (0.1098 * 1,000,000)
            // freq_mhz = (iterations * cycles_per_loop) / 109800

            let factor = if crate::cpuid::is_386() { 24 } else { 12 };
            (iterations * factor) / 109800
        }
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

    /// CPU speed information
    pub speed: Speed,

    /// Cache hierarchy information
    pub cache: Option<Cache>,

    #[allow(unused)]
    domains: Vec<TopologyDomain, 16>,
}

impl Topology {
    /// Detects and returns the CPU topology.
    pub fn detect() -> Self {
        let speed = Speed::detect();

        let cache = Cache::detect();
        let domains: Vec<TopologyDomain, 16> = Self::detect_domains();
        let (cores, threads) = Self::count_domains(&domains);

        let sockets = {
            #[cfg(any(target_os = "none", target_os = "linux"))]
            {
                super::mp::MpTable::detect().socket_count()
            }
            #[cfg(not(any(target_os = "none", target_os = "linux")))]
            {
                1usize
            }
        };

        Topology {
            sockets,
            cores,
            threads,
            speed,
            cache,
            domains,
        }
    }

    fn count_domains(domains: &Vec<TopologyDomain, 16>) -> (u32, u32) {
        let amd_threads = if is_amd() { logical_cores() } else { 0 };

        // TODO: determine cores/threads for Intel if domains are empty
        // Perhaps via x2APIC?
        if domains.is_empty() {
            return match vendor_str().as_str() {
                VENDOR_AMD => {
                    // Logical cpus = cores before Bulldozer
                    if amd_threads > 0 {
                        (amd_threads, amd_threads)
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
