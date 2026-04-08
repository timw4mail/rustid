use super::brand::{VENDOR_AMD, VENDOR_INTEL};
use super::{
    EXT_LEAF_26, LEAF_0B, LEAF_1F, has_ht, is_valid_leaf, logical_cores, vendor_str,
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

    #[cfg(target_os = "none")]
    fn measure_frequency() -> u32 {
        use super::is_386;
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
            // We'll count how many times we can run a loop in 8 ticks (~440ms).
            // We also use the PIT Channel 0 for sub-tick precision.

            let mut iterations: u32 = 0;
            let target_ticks = t1.wrapping_add(8);
            let mut start_pit: u16 = 0;
            let mut end_pit: u16 = 0;

            unsafe {
                core::arch::asm!(
                    "push es",
                    "mov ax, 0x40",
                    "mov es, ax",

                    // Latch and read start PIT
                    "xor al, al",
                    "out 0x43, al",
                    "in al, 0x40",
                    "mov ah, al",
                    "in al, 0x40",
                    "xchg al, ah",
                    "mov {2:x}, ax",

                    "2:",
                    "add {0:e}, 1",
                    "push ax", // Extra work to slow down the loop and be more consistent
                    "pop ax",
                    "mov ax, es:[0x6C]",
                    "cmp ax, {1:x}",
                    "jne 2b",

                    // Latch and read end PIT
                    "xor al, al",
                    "out 0x43, al",
                    "in al, 0x40",
                    "mov ah, al",
                    "in al, 0x40",
                    "xchg al, ah",
                    "mov {3:x}, ax",

                    "pop es",
                    inout(reg) iterations,
                    in(reg) target_ticks,
                    out(reg) start_pit,
                    out(reg) end_pit,
                    out("ax") _,
                );
            }

            // PIT runs at 1.193182 MHz. Each tick is 65536 PIT cycles.
            // Total pulses = (8 * 65536) + (start_pit - end_pit)
            let elapsed_pulses = (8u64 * 65536) + (start_pit as i32 - end_pit as i32) as u64;

            // Calibration:
            // 486 loop: add(2) + push(1) + pop(1) + mov mem(3) + cmp(1) + jne(3) = 11 cycles
            // 386 loop: add(4) + push(2) + pop(4) + mov mem(6) + cmp(2) + jne(7) = 25 cycles
            // RapidCAD (486 core in 386 package): ~20 cycles
            let cycles_per_loop = match &*vendor_str() {
                super::brand::VENDOR_CYRIX => 14,
                super::brand::VENDOR_UMC => 12,
                _ => {
                    if is_386() {
                        let sig = super::cpu::CpuSignature::detect();
                        match (sig.family, sig.model) {
                            // RapidCAD
                            (3, 4) => 20,
                            // 'Regular' 386 Chips
                            _ => 25,
                        }
                    } else {
                        // 'Classic' 486
                        11
                    }
                }
            };

            // freq_hz = (iterations * cycles_per_loop * 1193182) / elapsed_pulses
            // freq_mhz = freq_hz / 1_000_000
            // We use rounded division: (numerator + denominator / 2) / denominator
            let denom = elapsed_pulses * 1000000;
            let freq_mhz =
                (iterations as u64 * cycles_per_loop as u64 * 1193182 + (denom / 2)) / denom;
            freq_mhz as u32
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
        let threads = logical_cores();

        // TODO: determine cores/threads for Intel if domains are empty
        // Perhaps via x2APIC?
        if domains.is_empty() {
            return match &*vendor_str() {
                VENDOR_AMD => {
                    // Logical cpus = cores before Bulldozer
                    if threads > 1 {
                        (threads, threads)
                    } else {
                        (1, 1)
                    }
                }
                VENDOR_INTEL => {
                    if threads > 1 {
                        if has_ht() {
                            (threads, threads / 2)
                        } else {
                            (threads, threads)
                        }
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
        match &*vendor_str() {
            // AMD has literal core count
            VENDOR_AMD => (raw_cores, raw_threads * raw_cores),
            // Others have 'Core' as Threads * Cores
            _ => (raw_cores / raw_threads, raw_cores),
        }
    }

    fn detect_domains() -> Vec<TopologyDomain, 16> {
        let d: Vec<TopologyDomain, 16> = Vec::new();

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

    fn detect_domains_leaf(leaf: u32) -> Vec<TopologyDomain, 16> {
        let mut d: Vec<TopologyDomain, 16> = Vec::new();

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
                // Intel Topology V2
                LEAF_1F => {
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
                // AMD Topology V2
                EXT_LEAF_26 => {
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
                _ => return d,
            };
        }

        d
    }
}
