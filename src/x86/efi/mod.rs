#![cfg(uefi)]
//! Zero-dependency UEFI environment support for rustid.

pub mod display;
pub mod font;
pub mod mp;
pub mod os;
pub mod smbios;

pub use display::*;
pub use mp::*;
pub use os::*;
pub use smbios::*;

use crate::common::{Cache, CoreType, DataSource, Speed, TopologyTier, UNK};
use crate::x86::cpu::{Cpu, CpuCore, CpuSignature};
use crate::x86::micro_arch::{CpuArch, MicroArch};
use crate::x86::vendor::Intel;
use crate::x86::{
    core_type_from_cpuid, cpuid_cores_per_package, cpuid_threads_per_core,
    cpuid_threads_per_package, is_intel, vendor_str,
};
use alloc::string::String;
use alloc::vec::Vec;

/// Enriches a CPU detected via pure CPUID with live UEFI firmware/hardware information
/// (SMBIOS system name, SMBIOS/MP multi-socket counts, dynamic frequency measurement, and hybrid cores).
pub fn enrich_cpu(cpu: &mut Cpu) {
    // 1. SMBIOS System Name
    if let Some(sys_name) = smbios::detect_smbios_system_name() {
        cpu.system = Some(sys_name);
    }

    // 2. Multi-socket / multi-package topology from EFI MP Services / SMBIOS
    let efi_sockets = crate::x86::count::get_platform_socket_count();
    if efi_sockets.count > 1 {
        cpu.extra.topology.sockets = efi_sockets;
        let cores = cpu
            .extra
            .topology
            .cores
            .count
            .max(cpuid_cores_per_package() * efi_sockets.count);
        let threads = cpu
            .extra
            .topology
            .threads
            .count
            .max(cpuid_threads_per_package() * efi_sockets.count);
        cpu.extra.topology.cores =
            TopologyTier::new(cores, DataSource::Calculated("EFI sockets * CPUID cores"));
        cpu.extra.topology.threads = TopologyTier::new(
            threads,
            DataSource::Calculated("EFI sockets * CPUID threads"),
        );
        let sockets = cpu.extra.topology.sockets.count;
        if let Some(ref mut cache) = cpu.extra.topology.cache {
            cache.resolve_share_counts(cores, threads, sockets);
        }
    }

    // 3. Frequency measurement (TSC stall or SMBIOS fallback)
    if cpu.extra.topology.speed.base == 0 {
        let measured = Speed::detect();
        if measured.base > 0 {
            cpu.extra.topology.speed = measured;
            if cpu.cores.len() == 1 && cpu.cores[0].speed.is_none() {
                cpu.cores[0].speed = Some(measured);
            }
        }
    }

    // 4. Hybrid core discovery via EFI MP Services
    if is_intel() {
        let detected = detect_live_core_types();
        if detected.len() > 1 {
            cpu.cores = detected;
        }
    }
}

/// Enumerates all logical processors across APs in UEFI to discover unique core types (e.g. Intel P-cores and E-cores).
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

    if let Some(mp) = mp::EfiMpServices::detect() {
        let proc_count = mp.processor_count();

        for cpu_idx in 0..proc_count {
            let mut core_type = CoreType::default();
            let mut sig = CpuSignature::default();
            let mut raw_model = String::new();
            let mut vendor = String::new();
            let mut speed = Speed::default();

            mp.run_on_processor(cpu_idx, || {
                core_type = core_type_from_cpuid();
                sig = CpuSignature::detect();
                raw_model = Cpu::raw_model_string();
                vendor = vendor_str();
                speed = Speed::detect();
            });

            let arch = CpuArch::find(&raw_model, sig, &vendor);
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
