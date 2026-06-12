//! Let's count sockets/cores/threads
use crate::common::{DataSource, TopologyTier};
use crate::cpuid::{amd_threads_per_core, cpuid_data_source, has_ht};

#[cfg(not(dos))]
use crate::common::{OS, TOSData};

use super::{amd_logical_cores, is_amd};

#[cfg(not(dos))]
use super::{info_source, provider::CpuidInfoSource};

pub fn get_platform_socket_count() -> TopologyTier {
    #[cfg(dos)]
    let sockets_detected = TopologyTier::new(
        super::mp::MpTable::detect().socket_count(),
        DataSource::MpTable,
    );

    #[cfg(not(dos))]
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
