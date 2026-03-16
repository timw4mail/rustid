//! PowerPC CPU detection.

#[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64")))]
compile_error!("This crate only supports PowerPC architectures.");

pub mod cpu;

/// Gets the Processor Version Register (PVR).
///
/// The PVR contains information about the CPU version and revision.
pub fn get_pvr() -> u32 {
    let mut pvr: u32 = 0;
    #[cfg(target_arch = "powerpc")]
    {
        // PVR is SPR 287 on classic PowerPC
        unsafe {
            core::arch::asm!("mfspr {pvr}, 287", pvr = out(reg) pvr, options(nomem, nostack));
        }
    }
    pvr
}
