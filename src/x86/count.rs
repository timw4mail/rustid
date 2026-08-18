//! Let's count sockets/cores/threads
use crate::common::{DataSource, TopologyTier};
use crate::x86::{amd_threads_per_core, cpuid_data_source, has_ht};

#[cfg(not(nostd_os))]
use crate::common::{OS, TOSData};

use super::{amd_logical_cores, is_amd};

#[cfg(not(nostd_os))]
use super::{info_source, provider::CpuidInfoSource};

pub fn get_platform_socket_count() -> TopologyTier {
    #[cfg(any(dos, dos32a))]
    let sockets_detected = TopologyTier::new(
        crate::x86::dos::mp::MpTable::detect().socket_count(),
        DataSource::MpTable,
    );

    #[cfg(target_os = "uefi")]
    let sockets_detected = if let Some(mp) = crate::x86::efi::mp::EfiMpServices::detect() {
        TopologyTier::new(
            mp.socket_count() as u32,
            DataSource::Calculated("EFI MP Services"),
        )
    } else if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
        if !smbios.processors.is_empty() {
            TopologyTier::new(
                smbios.processors.len() as u32,
                DataSource::Calculated("SMBIOS"),
            )
        } else {
            TopologyTier::default()
        }
    } else {
        TopologyTier::default()
    };

    #[cfg(not(nostd_os))]
    let sockets_detected = if info_source() == CpuidInfoSource::Cpu {
        OS::get_socket_count()
    } else {
        TopologyTier::default()
    };

    sockets_detected
}

pub fn get_thread_count() -> TopologyTier {
    if is_amd() {
        TopologyTier::new(amd_logical_cores(), cpuid_data_source())
    } else {
        get_platform_thread_count()
    }
}

fn get_platform_thread_count() -> TopologyTier {
    #[cfg(target_os = "uefi")]
    if let Some(mp) = crate::x86::efi::mp::EfiMpServices::detect() {
        return TopologyTier::new(
            mp.processor_count() as u32,
            DataSource::Calculated("EFI MP Services"),
        );
    } else if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
        let total_threads: u32 = smbios.processors.iter().map(|p| p.thread_count).sum();
        if total_threads > 0 {
            return TopologyTier::new(total_threads, DataSource::Calculated("SMBIOS"));
        }
    }

    TopologyTier::default()
}

pub fn get_core_count() -> TopologyTier {
    if is_amd() {
        TopologyTier::new(
            amd_logical_cores() / amd_threads_per_core(),
            DataSource::Calculated("AMD Cpuid"),
        )
    } else {
        get_platform_core_count()
    }
}

fn get_platform_core_count() -> TopologyTier {
    let thread_tier = get_platform_thread_count();
    let thread_count = thread_tier.count;

    if !is_amd() && has_ht() && thread_count > 1 {
        TopologyTier::new(thread_count / 2, DataSource::Calculated("AMD Cpuid"))
    } else {
        thread_tier
    }
}
