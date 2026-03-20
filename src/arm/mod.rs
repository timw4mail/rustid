//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod micro_arch;

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos_api_ffi {
    #[link(name = "c")] // sysctl is in libc
    unsafe extern "C" {
        // Changed to unsafe extern "C"
        pub fn sysctlbyname(
            name: *const libc::c_char,
            oldp: *mut libc::c_void,
            oldlenp: *mut libc::size_t,
            newp: *mut libc::c_void,
            newlen: libc::size_t,
        ) -> libc::c_int;
    }
}

#[cfg(target_os = "macos")]
fn get_synth_midr() -> usize {
    use crate::arm::macos_api_ffi::*;

    let mut cpufamily: u64 = 0;
    let mut len = core::mem::size_of_val(&cpufamily);
    let family_name = b"hw.cpufamily\0";

    unsafe {
        if sysctlbyname(
            family_name.as_ptr() as *const libc::c_char,
            &mut cpufamily as *mut _ as *mut libc::c_void,
            &mut len as *mut libc::size_t,
            core::ptr::null_mut(),
            0,
        ) == 0
        {
            return cpufamily_to_midr(cpufamily);
        }
    }

    0
}

#[cfg(target_os = "macos")]
fn cpufamily_to_midr(cpufamily: u64) -> usize {
    const APPLE_IMPLEMENTER: usize = 0x61;

    let midr_base = APPLE_IMPLEMENTER << 24;

    // Apple Silicon hw.cpufamily values to MIDR part number mappings
    // Part numbers are based on cpufetch and Apple's MIDR definitions
    match cpufamily {
        // Apple A7 - Cyclone
        0x000C_0C0C_0C0E => midr_base | (0x001 << 4),
        // Apple A8 - Typhoon
        0x0000_1F2F_0E08 => midr_base | (0x002 << 4),
        // Apple A9 - Twister
        0x0000_0021_0C0A => midr_base | (0x003 << 4),
        // Apple A10 - Hurricane
        0x0000_0022_0C0A => midr_base | (0x004 << 4),
        // Apple A11 - Monsoon
        0x0000_0023_0C0A => midr_base | (0x005 << 4),
        // Apple A12 - Vortex
        0x0000_0024_0C0A => midr_base | (0x006 << 4),
        // Apple A13 - Lightning
        0x0000_0025_0C0A => midr_base | (0x007 << 4),
        // Apple A14 / M1 (Firestorm/Icestorm)
        0x0000_1B58_8BB3 => midr_base | (0x008 << 4),
        // Apple M1 Pro/Max/Ultra (Firestorm/Icestorm)
        0x0000_312F_8C0A => midr_base | (0x009 << 4),
        // Apple A15 / M1 (Avalanche/Blizzard)
        0x0000_323F_6C0A => midr_base | (0x00C << 4),
        // Apple M2 (Avalanche/Blizzard)
        0x0000_373F_7C0A => midr_base | (0x00A << 4),
        // Apple A16 / M1 (Everest/Sawtooth)
        0x0000_330F_7C0A => midr_base | (0x00F << 4),
        // Apple M3 (Gibraltar/Hull)
        0x0000_384F_8C0A => midr_base | (0x00D << 4),
        // Apple M3 Pro/Max/Ultra (Gibraltar/Hull)
        0x0000_3B4F_9C0A => midr_base | (0x00E << 4),
        // Apple A18 / A18 Pro (Ice/Dawn) - 0x75D4ACB9 is A18 Pro
        0x0000_75D4_ACB9 => midr_base | (0x101 << 4),
        // Apple M4 (Ice/Dawn)
        0x0000_4B4F_AE0A => midr_base | (0x010 << 4),
        // Apple M4 Pro/Max/Ultra (Ice/Dawn)
        0x0000_4F4F_AE0A => midr_base | (0x011 << 4),
        _ => 0,
    }
}

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod windows_api_ffi {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetNativeSystemInfo(lpSystemInfo: *mut super::SYSTEM_INFO);
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_INFO {
    _unused_dwOemId: u32,
    _unused_dwPageSize: u32,
    _unused_lpMinimumApplicationAddress: *mut core::ffi::c_void,
    _unused_lpMaximumApplicationAddress: *mut core::ffi::c_void,
    _unused_dwActiveProcessorMask: usize, // DWORD_PTR
    _unused_dwNumberOfProcessors: u32,
    _unused_dwProcessorType: u32,
    _unused_dwAllocationGranularity: u32,
    pub wProcessorLevel: u16,
    pub wProcessorRevision: u16,
}

#[cfg(target_os = "windows")]
fn get_synth_midr() -> usize {
    use crate::arm::fns::windows_api_ffi::*; // Explicitly import from the module
    let mut sys_info: SYSTEM_INFO = unsafe { core::mem::zeroed() };
    unsafe {
        GetNativeSystemInfo(&mut sys_info);
    }

    let mut synthetic_midr: usize = 0;
    synthetic_midr |= (0x41 as usize) << 24; // Implementer: ARM
    synthetic_midr |= (sys_info.wProcessorLevel as usize & 0xFFF) << 4;
    synthetic_midr |= sys_info.wProcessorRevision as usize & 0xF;

    return synthetic_midr;
}

// ----------------------------------------------------------------------------
// Linux
// ----------------------------------------------------------------------------

/// Gets the Main ID Register (MIDR).
///
/// The MIDR contains information about the CPU implementer, part number, and revision.
///
/// Note: Accessing system registers like MIDR requires privileged mode (e.g., kernel mode)
/// or specific architectural features. This function assumes an environment where it's
/// safe and permitted to read MIDR (e.g., Linux user space if exposed, or bare-metal).
pub fn get_midr() -> usize {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return get_synth_midr();

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
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
}
