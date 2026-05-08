#![cfg(not(target_os = "none"))]

/// Returns the number of logical cores (threads).
#[must_use]
pub fn logical_cores() -> usize {
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    if let Some(cores) = core_affinity::get_core_ids() {
        cores.len()
    } else {
        1
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    1
}
