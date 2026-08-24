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
    let mut sockets_detected = {
        let threads_per_pkg = cpuid_threads_per_package().max(1);
        let cores_per_pkg = cpuid_cores_per_package().max(1);

        // 1. Check MP Services (Authoritative for active CPU hardware packages)
        if let Some(mp) = crate::x86::efi::mp::EfiMpServices::detect() {
            let total_threads = mp.processor_count() as u32;
            let mp_sockets = mp.socket_count() as u32;
            if total_threads > 0 && threads_per_pkg > 0 {
                let calc_sockets = (total_threads / threads_per_pkg).max(1);
                TopologyTier::new(
                    mp_sockets.max(calc_sockets),
                    DataSource::Calculated("EFI MP Services"),
                )
            } else {
                TopologyTier::new(mp_sockets.max(1), DataSource::Calculated("EFI MP Services"))
            }
        } else if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
            if smbios.is_laptop() {
                TopologyTier::new(1, DataSource::Calculated("SMBIOS"))
            } else {
                let populated = smbios
                    .processors
                    .iter()
                    .filter(|p| p.is_populated && p.is_enabled)
                    .collect::<alloc::vec::Vec<_>>();

                let has_multi_core_field = populated.iter().any(|p| p.core_count > 1);

                if has_multi_core_field {
                    TopologyTier::new(
                        populated.len().max(1) as u32,
                        DataSource::Calculated("SMBIOS"),
                    )
                } else {
                    // Legacy SMBIOS (e.g. SMBIOS 2.4 where each core/thread is a separate Type 4 record)
                    let mut unique_sockets = alloc::vec::Vec::new();
                    for p in &populated {
                        if let Some(desig) = &p.socket_designation {
                            let trimmed = desig.trim();
                            if !trimmed.is_empty() && !unique_sockets.contains(&trimmed) {
                                unique_sockets.push(trimmed);
                            }
                        }
                    }

                    if unique_sockets.len() > 1 && unique_sockets.len() < populated.len() {
                        TopologyTier::new(
                            unique_sockets.len() as u32,
                            DataSource::Calculated("SMBIOS"),
                        )
                    } else if populated.len() > 1
                        && cores_per_pkg > 1
                        && populated.len() as u32 >= cores_per_pkg
                    {
                        TopologyTier::new(
                            (populated.len() as u32 / cores_per_pkg).max(1),
                            DataSource::Calculated("SMBIOS"),
                        )
                    } else {
                        TopologyTier::new(
                            populated.len().max(1) as u32,
                            DataSource::Calculated("SMBIOS"),
                        )
                    }
                }
            }
        } else {
            TopologyTier::default()
        }
    };

    #[cfg(not(nostd_os))]
    let mut sockets_detected = if info_source() == CpuidInfoSource::Cpu {
        OS::get_socket_count()
    } else {
        TopologyTier::default()
    };

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
        let count = mp.processor_count() as u32;
        if count > 0 {
            return TopologyTier::new(count, DataSource::Calculated("EFI MP Services"));
        }
    }

    #[cfg(target_os = "uefi")]
    if let Some(smbios) = crate::x86::efi::smbios::detect_smbios() {
        let threads_per_pkg = cpuid_threads_per_package().max(1);
        let populated = smbios
            .processors
            .iter()
            .filter(|p| p.is_populated && p.is_enabled)
            .collect::<alloc::vec::Vec<_>>();

        let has_multi_core_field = populated
            .iter()
            .any(|p| p.core_count > 1 || p.thread_count > 1);

        let total_threads: u32 = if has_multi_core_field {
            populated
                .iter()
                .map(|p| {
                    if p.thread_count > 0 {
                        p.thread_count
                    } else {
                        threads_per_pkg
                    }
                })
                .sum()
        } else {
            // If legacy SMBIOS has multiple Type 4 entries (one per core), populated.len() is already the core/thread count
            populated.len() as u32
        };

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
