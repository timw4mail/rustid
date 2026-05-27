//! Let's count sockets/cores/threads
use crate::cpuid::{amd_threads_per_core, has_ht};

use super::mp::MpTable;
use super::{amd_logical_cores, is_amd};

#[cfg(not(target_os = "none"))]
use super::{info_source, provider::CpuidInfoSource};

pub fn get_socket_count() -> u32 {
    #[cfg(target_os = "none")]
    let sockets_detected = MpTable::detect().socket_count();

    #[cfg(not(target_os = "none"))]
    let sockets_detected: u32 = if info_source() == CpuidInfoSource::Cpu {
        MpTable::detect().socket_count()
    } else {
        1
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
