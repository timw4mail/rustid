//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod micro_arch;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
pub use cpu::*;

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod apple;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use apple::*;

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn get_synth_midr() -> usize {
    use std::mem::{size_of, zeroed};
    use windows::Win32::System::Registry::*;
    use windows::Win32::System::SystemInformation::*;
    use windows::core::w;

    // Try registry first: 'CP 4000' is the standard for ARM64 MIDR on Windows
    let mut hkey = HKEY::default();
    let subkey = w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let result = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, 0, KEY_READ, &mut hkey) };

    if result.is_ok() {
        // 1. Try 'CP 4000' (REG_QWORD)
        let mut cpu_id_qword: u64 = 0;
        let mut size_qword = size_of::<u64>() as u32;
        let mut dw_type = REG_NONE;
        let value_name_4000 = w!("CP 4000");
        let query_4000 = unsafe {
            RegQueryValueExW(
                hkey,
                value_name_4000,
                None,
                Some(&mut dw_type),
                Some(&mut cpu_id_qword as *mut u64 as *mut u8),
                Some(&mut size_qword),
            )
        };

        if query_4000.is_ok() && dw_type == REG_QWORD {
            let _ = unsafe { RegCloseKey(hkey) };
            return cpu_id_qword as usize;
        }

        // 2. Fallback to 'CPUID' (REG_DWORD)
        let mut cpu_id_dword: u32 = 0;
        let mut size_dword = size_of::<u32>() as u32;
        let value_name_cpuid = w!("CPUID");
        let query_cpuid = unsafe {
            RegQueryValueExW(
                hkey,
                value_name_cpuid,
                None,
                Some(&mut dw_type),
                Some(&mut cpu_id_dword as *mut u32 as *mut u8),
                Some(&mut size_dword),
            )
        };
        let _ = unsafe { RegCloseKey(hkey) };

        if query_cpuid.is_ok() && dw_type == REG_DWORD {
            return cpu_id_dword as usize;
        }
    }

    // Fallback to GetNativeSystemInfo if registry fails
    let mut sys_info: SYSTEM_INFO = unsafe { zeroed() };
    unsafe {
        GetNativeSystemInfo(&mut sys_info);
    }

    let mut synthetic_midr: usize = 0;
    synthetic_midr |= (0x41 as usize) << 24;
    synthetic_midr |= (sys_info.wProcessorLevel as usize & 0xFFF) << 4;
    synthetic_midr |= sys_info.wProcessorRevision as usize & 0xF;

    synthetic_midr
}

// ----------------------------------------------------------------------------
// ! Linux
// ----------------------------------------------------------------------------

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
