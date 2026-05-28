#![cfg(target_os = "macos")]

use super::sysctl::*;

pub fn get_socket_count() -> u32 {
    let map = get_int_sysctl_map("macdep.cpu", "macdep.cpu.");
    let cores_per_package = map.get("cores_per_package");
    let core_count = map.get("core_count");

    if let Some(cores_per) = cores_per_package
        && let Some(core_count) = core_count
    {
        if cores_per >= core_count {
            return 1;
        } else {
            return core_count / cores_per;
        }
    }

    1
}

pub fn get_core_count() -> u32 {
    get_sysctl_int_value("machdep.cpu.core_count").unwrap_or(1)
}

pub fn get_thread_count() -> u32 {
    get_sysctl_int_value("machdep.cpu.thread_count").unwrap_or(1)
}
