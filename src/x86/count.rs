#[cfg(any(dos, dos32a, target_os = "uefi"))]
use crate::common::DataSource;
use crate::common::TopologyTier;
use crate::x86::{
    cpuid_cores_per_package, cpuid_data_source, cpuid_threads_per_core, cpuid_threads_per_package,
};

#[cfg(not(nostd_os))]
use crate::common::{OS, TOSData};

#[cfg(not(nostd_os))]
use super::{info_source, provider::CpuidInfoSource};

pub fn get_platform_socket_count() -> TopologyTier {
    #[cfg(any(dos, dos32a))]
    let mut sockets_detected = TopologyTier::new(
        crate::x86::dos::mp::MpTable::detect().socket_count(),
        DataSource::MpTable,
    );

    #[cfg(target_os = "uefi")]
    let mut sockets_detected = if let Some(mp) = crate::x86::efi::mp::EfiMpServices::detect() {
        TopologyTier::new(
            mp.socket_count() as u32,
            DataSource::Calculated("EFI MP Services"),
        )
    } else if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
        let populated_sockets = smbios
            .processors
            .iter()
            .filter(|p| p.is_populated && p.is_enabled)
            .count() as u32;
        if populated_sockets > 0 {
            TopologyTier::new(populated_sockets, DataSource::Calculated("SMBIOS"))
        } else if !smbios.processors.is_empty() {
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
    let mut sockets_detected = if info_source() == CpuidInfoSource::Cpu {
        OS::get_socket_count()
    } else {
        TopologyTier::default()
    };

    let threads_per_pkg = cpuid_threads_per_package();
    let total_threads = get_platform_thread_count().count;

    if threads_per_pkg > 0 && total_threads > 0 {
        let max_sockets = (total_threads / threads_per_pkg).max(1);
        if sockets_detected.count > max_sockets {
            sockets_detected.count = max_sockets;
        }
    } else if threads_per_pkg > 1
        && sockets_detected.count > 1
        && sockets_detected.count <= threads_per_pkg
    {
        sockets_detected.count = 1;
    }

    #[cfg(target_os = "uefi")]
    {
        if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
            if smbios.is_laptop() {
                sockets_detected.count = 1;
            }
        }
    }

    sockets_detected
}

pub fn get_thread_count() -> TopologyTier {
    let platform_threads = get_platform_thread_count();
    let pkg_threads = cpuid_threads_per_package();

    if platform_threads.count > 0 {
        TopologyTier::new(
            platform_threads.count.max(pkg_threads),
            platform_threads.source,
        )
    } else if pkg_threads > 0 {
        TopologyTier::new(pkg_threads, cpuid_data_source())
    } else {
        TopologyTier::default()
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
        let total_threads: u32 = smbios
            .processors
            .iter()
            .filter(|p| p.is_populated && p.is_enabled)
            .map(|p| {
                if p.thread_count > 0 {
                    p.thread_count
                } else {
                    1
                }
            })
            .sum();
        if total_threads > 0 {
            return TopologyTier::new(total_threads, DataSource::Calculated("SMBIOS"));
        }
    }

    TopologyTier::default()
}

pub fn get_core_count() -> TopologyTier {
    let threads_tier = get_thread_count();
    let t_count = threads_tier.count;
    let t_per_core = cpuid_threads_per_core();

    if t_per_core > 1 && t_count > 1 {
        TopologyTier::new(t_count / t_per_core, threads_tier.source)
    } else {
        let pkg_cores = cpuid_cores_per_package();
        if t_count < pkg_cores && pkg_cores > 0 {
            TopologyTier::new(pkg_cores, cpuid_data_source())
        } else {
            threads_tier
        }
    }
}
