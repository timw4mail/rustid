//! Let's count sockets/cores/threads
use crate::common::DataSource;
use crate::cpuid::{amd_threads_per_core, has_ht};

use super::{amd_logical_cores, is_amd};

#[cfg(not(dos))]
use super::{info_source, provider::CpuidInfoSource};

pub fn get_socket_count() -> (u32, DataSource) {
    #[cfg(dos)]
    let sockets_detected = (
        super::mp::MpTable::detect().socket_count(),
        DataSource::MpTable,
    );

    #[cfg(not(dos))]
    let sockets_detected = if info_source() == CpuidInfoSource::Cpu {
        crate::common::os::get_socket_count()
    } else {
        (1u32, DataSource::DefaultValue)
    };

    sockets_detected
}

pub fn get_thread_count() -> u32 {
    if is_amd() {
        amd_logical_cores()
    } else {
        get_platform_thread_count()
    }
}

fn get_platform_thread_count() -> u32 {
    1
}

pub fn get_core_count() -> u32 {
    if is_amd() {
        amd_logical_cores() / amd_threads_per_core()
    } else {
        get_platform_core_count()
    }
}

fn get_platform_core_count() -> u32 {
    let thread_count = get_platform_thread_count();

    if !is_amd() && has_ht() && thread_count > 1 {
        thread_count / 2
    } else {
        thread_count
    }
}
