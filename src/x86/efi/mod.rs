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

use crate::common::Speed;
use crate::x86::cpu::{Cpu, CpuCore, finalize_hybrid_cores, probe_current_core, push_or_merge_core};
use crate::x86::is_intel;
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
    cpu.topology.scale_sockets(
        efi_sockets,
        "EFI sockets * CPUID cores",
        "EFI sockets * CPUID threads",
    );

    // 3. Frequency measurement (TSC stall or SMBIOS fallback)
    if cpu.topology.speed.base == 0 {
        let measured = Speed::detect();
        if measured.base > 0 {
            cpu.topology.speed = measured;
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

    if let Some(mp) = mp::EfiMpServices::detect() {
        let proc_count = mp.processor_count();

        for cpu_idx in 0..proc_count {
            let mut core_opt = None;
            mp.run_on_processor(cpu_idx, || {
                let speed = Speed::detect();
                core_opt = probe_current_core(Some(speed));
            });

            if let Some(core) = core_opt {
                push_or_merge_core(&mut cores, core);
            }
        }
    }

    finalize_hybrid_cores(&mut cores);
    cores
}
