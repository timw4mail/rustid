//! ARM-specific functions.

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

#[cfg(target_os = "macos")]
mod macos_api_ffi {
    #[link(name = "c")] // sysctl is in libc
    unsafe extern "C" { // Changed to unsafe extern "C"
        pub fn sysctlbyname(
            name: *const libc::c_char,
            oldp: *mut libc::c_void,
            oldlenp: *mut libc::size_t,
            newp: *mut libc::c_void,
            newlen: libc::size_t,
        ) -> libc::c_int;
    }
}

pub fn get_features() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();

    unimplemented!("ARM: get_features()");
}

/// Gets the Main ID Register (MIDR).
///
/// The MIDR contains information about the CPU implementer, part number, and revision.
///
/// Note: Accessing system registers like MIDR requires privileged mode (e.g., kernel mode)
/// or specific architectural features. This function assumes an environment where it's
/// safe and permitted to read MIDR (e.g., Linux user space if exposed, or bare-metal).
pub fn get_midr() -> usize {
    #[cfg(target_os = "windows")]
    {
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

    #[cfg(target_os = "macos")]
    {
        use crate::arm::fns::macos_api_ffi::*;
        let mut synthetic_midr: usize = 0;

        // Try to get hw.cpufamily
        let mut cpufamily: u64 = 0; // It's often u64
        let mut len = core::mem::size_of_val(&cpufamily);
        let family_name = b"hw.cpufamily\0"; // C string

        unsafe {
            if sysctlbyname(
                family_name.as_ptr() as *const libc::c_char,
                &mut cpufamily as *mut _ as *mut libc::c_void,
                &mut len as *mut libc::size_t,
                core::ptr::null_mut(),
                0,
            ) == 0 {
                // Map CPU family to Part Number (bits 4-15) and Implementer (bits 24-31)
                // This is a rough mapping and not precise MIDR.
                synthetic_midr |= ((cpufamily as usize) & 0xFF) << 24; // Implementer: lower 8 bits of family
                synthetic_midr |= ((cpufamily as usize) & 0xFFF00) << 4; // PartNum: higher bits of family
            }
        }
        return synthetic_midr;
    }

    let mut midr: usize = 0;
    // ARMv7 and ARMv8 (AArch64) have MIDR at c0, so `mrs r0, MIDR` or `mrs x0, MIDR_EL1`
    #[cfg(not(any(target_os = "windows", target_os = "macos")))] // For non-Windows/macOS targets
    {
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
    }
    midr
}
