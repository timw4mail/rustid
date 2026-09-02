//! OS and platform-specific data gathering for x86 processors.
//!
//! This module encapsulates all host OS and platform queries:
//! - Host system / machine name (from DMI, sysfs, registry, sysctl, Haiku)
//! - Platform multi-socket counts (from /proc/cpuinfo, sysfs, Windows registry)
//! - Live core enumeration via thread affinity (`core_affinity`)
//! - Dynamic TSC frequency measurement via OS timer

use super::cpu::{Cpu, CpuCore, CpuSignature};
use super::micro_arch::{CpuArch, MicroArch};
use super::vendor::Intel;
use super::{
    core_type_from_cpuid, cpuid_cores_per_package, cpuid_threads_per_core,
    cpuid_threads_per_package, is_intel, vendor_str,
};
use crate::common::{Cache, CoreType, DataSource, Speed, SystemInfo, TopologyTier, UNK};
use alloc::string::String;
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

    fn find_or_push(cores: &mut Vec<CpuCore>, core: CpuCore) {
        if let Some(c) = cores
            .iter_mut()
            .find(|c| c.kind == core.kind && c.micro_arch == core.micro_arch && c.name == core.name)
        {
            c.count += core.count;
            c.threads += core.threads;
            if c.speed.is_none() && core.speed.is_some() {
                c.speed = core.speed;
            }
        } else {
            cores.push(core);
        }
    }

    if let Some(core_ids) = core_affinity::get_core_ids() {
        for core_id in core_ids {
            core_affinity::set_for_current(core_id);

            let core_type = core_type_from_cpuid();
            let sig = CpuSignature::detect();
            let arch = CpuArch::find(&Cpu::raw_model_string(), sig, &vendor_str());
            let micro_arch = if is_intel() {
                Intel::core_micro_arch(arch.micro_arch, core_type)
            } else {
                arch.micro_arch
            };

            if micro_arch == MicroArch::Unknown {
                continue;
            }

            let name_str = micro_arch.as_str();
            let name = if name_str != UNK {
                Some(String::from(name_str))
            } else {
                None
            };

            let cache = Cache::detect();
            let speed = Speed::detect_cpuid();
            let speed_opt = if speed.base > 0 { Some(speed) } else { None };

            find_or_push(
                &mut cores,
                CpuCore {
                    kind: core_type,
                    micro_arch,
                    name,
                    implementer: None,
                    cache,
                    speed: speed_opt,
                    count: 1,
                    threads: 1,
                },
            );
        }
    }

    for c in &mut cores {
        let smt = if c.kind == CoreType::Efficiency {
            1
        } else {
            cpuid_threads_per_core().max(1)
        };
        c.count = (c.threads / smt).max(1);
        if let Some(ref mut cache) = c.cache {
            cache.resolve_share_counts(c.count, c.threads, 1);
        }
    }

    cores
}

/// Enriches a pure CPUID detection result with host operating system information.
pub fn enrich_cpu(cpu: &mut Cpu) {
    // 1. Host system / machine name
    cpu.system = get_system_name();

    // 2. Physical platform sockets
    let os_sockets = get_socket_count();
    if os_sockets.count > 1 {
        cpu.topology.sockets = os_sockets;
        let cores = cpu
            .topology
            .cores
            .count
            .max(cpuid_cores_per_package() * os_sockets.count);
        let threads = cpu
            .topology
            .threads
            .count
            .max(cpuid_threads_per_package() * os_sockets.count);
        cpu.topology.cores =
            TopologyTier::new(cores, DataSource::Calculated("OS sockets * CPUID cores"));
        cpu.topology.threads = TopologyTier::new(
            threads,
            DataSource::Calculated("OS sockets * CPUID threads"),
        );
        let sockets = cpu.topology.sockets.count;
        if let Some(ref mut cache) = cpu.topology.cache {
            cache.resolve_share_counts(cores, threads, sockets);
        }
    }

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
