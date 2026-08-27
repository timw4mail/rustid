//! PowerPC CPU detection.

#[cfg(not(any(ppc_cpu, test)))]
compile_error!("This crate only supports PowerPC architectures.");

pub mod cpu;
pub mod display;
pub mod micro_arch;

/// Gets the Processor Version Register (PVR).
///
/// The PVR contains information about the CPU version and revision.
pub fn get_pvr() -> u32 {
    #[cfg(ppc_cpu)]
    {
        let mut pvr: u32 = 0;
        // PVR is SPR 287 on classic PowerPC
        unsafe {
            core::arch::asm!("mfspr {pvr}, 287", pvr = out(reg) pvr, options(nomem, nostack));
        }
        pvr
    }
    #[cfg(not(ppc_cpu))]
    {
        0
    }
}
