//! ARM-specific functions.

/// Gets the Main ID Register (MIDR).
///
/// The MIDR contains information about the CPU implementer, part number, and revision.
///
/// Note: Accessing system registers like MIDR requires privileged mode (e.g., kernel mode)
/// or specific architectural features. This function assumes an environment where it's
/// safe and permitted to read MIDR (e.g., Linux user space if exposed, or bare-metal).
pub fn get_midr() -> usize {
    let mut midr: usize = 0;
    // ARMv7 and ARMv8 (AArch64) have MIDR at c0, so `mrs r0, MIDR` or `mrs x0, MIDR_EL1`
    #[cfg(target_arch = "arm")]
    {
        // For ARMv7-A and earlier, MIDR is c0, c0, 0
        unsafe {
            core::arch::asm!("mrc p15, 0, {midr}, c0, c0, 0", midr = out(reg) midr, options(nomem, nostack));
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // For AArch64, MIDR_EL1 (EL1)
        unsafe {
            core::arch::asm!("mrs {midr}, midr_el1", midr = out(reg) midr, options(nomem, nostack));
        }
    }
    midr
}
