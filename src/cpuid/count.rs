//! Let's count sockets/cores/threads
#[cfg(dos)]
use crate::common::DataSource;
use crate::common::os::OS;
use crate::common::{DataSource, TOSData, TopologyTier};
use crate::cpuid::{amd_threads_per_core, has_ht};

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
        TopologyTier::new(amd_logical_cores(), DataSource::Cpuid)
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
            DataSource::Calculated("AMD Cpujd"),
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
