#![cfg(not(target_os = "none"))]

/// Returns the number of logical cores (threads).
pub fn logical_cores() -> usize {
    if let Some(cores) = core_affinity::get_core_ids() {
        cores.len()
    } else {
        1
    }
}
