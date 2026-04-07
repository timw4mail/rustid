#![cfg(not(target_os = "none"))]

/// Returns the number of logical cores (threads).
pub fn logical_cores() -> usize {
    #[cfg(not(target_arch = "x86"))]
    if let Some(cores) = core_affinity::get_core_ids() {
        cores.len()
    } else {
        1
    }

    #[cfg(target_arch = "x86")]
    1
}
