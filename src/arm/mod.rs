#![cfg(arm_cpu)]
//! ARM CPU detection.

mod brand;
pub mod cpu;
mod display;
pub mod features;
pub mod micro_arch;
pub mod os;

pub use cpu::*;
pub use features::{ArmFeatures, TArmFeatures};
pub use micro_arch::{CpuCore, Midr};
pub use os::*;

trait TArmCpu {
    /// Returns the CPU model name, if available
    #[allow(unused)]
    fn model(&self) -> Option<&str> {
        None
    }

    fn vendor(&self) -> &str;
}

/// Gets the Main ID Register (MIDR).
///
/// The MIDR contains information about the CPU implementer, part number, and revision.
pub fn get_midr() -> usize {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return get_synth_midr();

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut midr: usize = 0;
        // ARMv7 and ARMv8 (AArch64) have MIDR at c0, so `mrs r0, MIDR` or `mrs x0, MIDR_EL1`
        #[cfg(all(
            target_arch = "arm",
            not(any(target_os = "android", target_os = "linux"))
        ))]
        {
            // For ARMv7-A and earlier, MIDR is c0, c0, 0
            unsafe {
                core::arch::asm!("mrc p15, 0, {midr}, c0, c0, 0", midr = out(reg) midr, options(nomem, nostack));
            }
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
        {
            // For AArch64, MIDR_EL1 (EL1)
            unsafe {
                core::arch::asm!("mrs {midr}, midr_el1", midr = out(reg) midr, options(nomem, nostack));
            }
        }
        midr
    }
}
