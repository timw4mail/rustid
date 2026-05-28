#![cfg(not(dos))]

/// Returns the number of logical cores (threads).
#[must_use]
pub fn logical_cores() -> usize {
    #[cfg(not(x86_cpu))]
    if let Some(cores) = core_affinity::get_core_ids() {
        cores.len()
    } else {
        1
    }

    #[cfg(x86_cpu)]
    1
}
