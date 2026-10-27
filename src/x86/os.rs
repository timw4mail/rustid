//! OS and platform-specific data gathering for x86 processors.
//!
//! This module encapsulates all host OS and platform queries:
//! - Host system / machine name (from DMI, sysfs, registry, sysctl, Haiku)
//! - Platform multi-socket counts (from /proc/cpuinfo, sysfs, Windows registry)
//! - Live core enumeration via thread affinity (`core_affinity`)
//! - Dynamic TSC frequency measurement via OS timer

use super::cpu::{Cpu, CpuCore, finalize_hybrid_cores, probe_current_core, push_or_merge_core};
use super::is_intel;
use crate::common::{Speed, SystemInfo, TopologyTier};
use alloc::vec::Vec;

#[cfg(not(dos_os))]
use crate::common::{OS, TOSData};

/// Returns the host system name reported by the operating system.
#[must_use]
pub fn get_system_name() -> Option<SystemInfo> {
    #[cfg(not(dos_os))]
    {
        OS::get_system_name()
    }

    #[cfg(dos_os)]
    {
        None
    }
}

/// Returns the number of physical sockets reported by the operating system.
#[must_use]
pub fn get_socket_count() -> TopologyTier {
    #[cfg(not(dos_os))]
    {
        OS::get_socket_count()
    }

    #[cfg(dos_os)]
    {
        TopologyTier::default()
    }
}

/// Dynamically measures the CPU clock frequency using RDTSC and OS timer.
#[must_use]
pub fn measure_frequency() -> Speed {
    if !super::has_tsc() {
        return Speed::default();
    }

    let freq = measure_frequency_tsc();
    if freq == 0 {
        return Speed::default();
    }

    Speed {
        base: freq,
        boost: freq,
        measured: true,
    }
}

fn measure_frequency_tsc() -> u32 {
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

/// Discovers hybrid core types by pinning the current thread to each logical core.
#[must_use]
pub fn detect_live_core_types() -> Vec<CpuCore> {
    let mut cores: Vec<CpuCore> = Vec::new();

    crate::common::for_each_logical_core(|| {
        let speed = Speed::detect_cpuid();
        if let Some(core) = probe_current_core(Some(speed)) {
            push_or_merge_core(&mut cores, core);
        }
    });

    finalize_hybrid_cores(&mut cores);
    cores
}

/// Enriches a pure CPUID detection result with host operating system information.
pub fn enrich_cpu(cpu: &mut Cpu) {
    // 1. Host system / machine name
    cpu.system = get_system_name();

    // 2. Physical platform sockets
    let os_sockets = get_socket_count();
    cpu.topology.scale_sockets(
        os_sockets,
        "OS sockets * CPUID cores",
        "OS sockets * CPUID threads",
    );

    // 3. Dynamic speed measurement if CPUID did not report frequency
    if cpu.topology.speed.base == 0 {
        let measured = measure_frequency();
        if measured.base > 0 {
            cpu.topology.speed = measured;
            for core in &mut cpu.cores {
                if core.speed.is_none() {
                    core.speed = Some(measured);
                }
            }
        }
    }

    // 4. Hybrid core discovery via thread pinning on Intel processors
    if is_intel() {
        let live_cores = detect_live_core_types();
        if live_cores.len() > 1 {
            cpu.cores = live_cores;
        }
    }
}
