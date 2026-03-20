//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod micro_arch;

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn get_synth_midr() -> usize {
    use std::collections::HashMap;
    use std::process::Command;

    let raw_sysctl: String = Command::new("sysctl")
        .arg("-a")
        .output()
        .expect("Failed to load cpu details from sysctl")
        .stdout
        .try_into()
        .unwrap();

    let mut values: HashMap<&str, &str> = HashMap::new();
    raw_sysctl
        .split('\n')
        .filter(|l| l.len() > 0)
        .for_each(|x| {
            let line: Vec<_> = x.split(": ").collect();
            if let Some(key) = line.get(0)
                && let Some(val) = line.get(1)
            {
                if key.starts_with("machdep.cpu")
                    || (key.starts_with("hw") && (key.contains("cpu") || key.contains("cache")))
                {
                    values.insert(key, val);
                }
            }
        });

    // println!("{:#?}", values);

    let cpufamily = if let Some(family) = values.get("hw.cpufamily") {
        match family.parse::<u64>() {
            Ok(f) => Some(f),
            Err(_) => None,
        }
    } else {
        None
    };

    let brand_string = if let Some(brand_string) = values.get("machdep.cpu.brand_string") {
        Some(brand_string)
    } else {
        None
    };

    if let (Some(family), Some(brand)) = (cpufamily, brand_string) {
        cpufamily_to_midr(family, &brand)
    } else {
        0
    }
}

#[cfg(target_os = "macos")]
fn cpufamily_to_midr(cpufamily: u64, brand_string: &str) -> usize {
    const APPLE_IMPLEMENTER: usize = 0x61;
    let midr_base = APPLE_IMPLEMENTER << 24;

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

        // Apple M1 family (0x1b588bb3) - need brand string to distinguish variants
        0x0000_1B58_8BB3 => {
            if brand_string.contains("M1 Pro") || brand_string.contains("M1 Max") {
                midr_base | (0x009 << 4)
            } else if brand_string.contains("M1 Ultra") {
                midr_base | (0x00A << 4)
            } else {
                midr_base | (0x008 << 4) // M1 base
            }
        }

        // Apple A15 / M2 family (0xda33d83d) - Avalanche/Blizzard
        0x0000_DA33_D83D => {
            if brand_string.contains("M2 Pro") || brand_string.contains("M2 Max") {
                midr_base | (0x00B << 4)
            } else if brand_string.contains("M2 Ultra") {
                midr_base | (0x00C << 4)
            } else {
                midr_base | (0x00D << 4) // A15, M2 base
            }
        }

        // Apple A16 / M3 family (0x8765edea) - Everest/Sawtooth
        0x0000_8765_EDEA => {
            if brand_string.contains("M3 Pro") || brand_string.contains("M3 Max") {
                midr_base | (0x00E << 4)
            } else {
                midr_base | (0x00F << 4) // A16, M3 base
            }
        }

        // Apple A18 / A18 Pro (0x75D4ACB9)
        0x0000_75D4_ACB9 => {
            if brand_string.contains("A18 Pro") {
                midr_base | (0x101 << 4)
            } else {
                midr_base | (0x100 << 4) // A18
            }
        }

        // Apple M4 family
        0x0000_4B4F_AE0A => {
            if brand_string.contains("M4 Pro") || brand_string.contains("M4 Max") {
                midr_base | (0x011 << 4)
            } else if brand_string.contains("M4 Ultra") {
                midr_base | (0x012 << 4)
            } else {
                midr_base | (0x010 << 4) // M4 base
            }
        }

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
    _unused_dwActiveProcessorMask: usize,
    _unused_dwNumberOfProcessors: u32,
    _unused_dwProcessorType: u32,
    _unused_dwAllocationGranularity: u32,
    pub wProcessorLevel: u16,
    pub wProcessorRevision: u16,
}

#[cfg(target_os = "windows")]
fn get_synth_midr() -> usize {
    use windows_api_ffi::*;
    let mut sys_info: SYSTEM_INFO = unsafe { core::mem::zeroed() };
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
