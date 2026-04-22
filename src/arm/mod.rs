#![cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod micro_arch;
use crate::common::{CoreType, CpuDisplay};
pub use micro_arch::{CpuCore, Midr};
use std::collections::{BTreeMap, HashSet};

trait TArmCpu {
    /// Returns the CPU model name, if available
    #[allow(unused)]
    fn model(&self) -> Option<&str> {
        None
    }

    fn raw_midr(&self) -> HashSet<usize>;
    fn midr(&self) -> Option<&Midr>;
    fn vendor(&self) -> &str;
}

impl CpuDisplay {
    pub fn display(
        cpu_arch: &crate::arm::micro_arch::CpuArch,
        cores: &BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
        color: bool,
    ) {
        println!();

        Self::simple_line(
            "Brand/Implementor",
            <crate::arm::brand::Vendor as Into<&str>>::into(cpu_arch.implementer),
        );

        Self::simple_line("Model", &cpu_arch.model);

        Self::simple_line("Code Name", cpu_arch.code_name);

        if let Some(tech) = cpu_arch.technology {
            Self::simple_line("Process", tech);
        }

        for ((kind, _, _), core) in cores {
            let name = format!("{} Cores", Into::<String>::into(*kind));
            println!("{}", Self::raw_label(&name));

            if let Some(name) = core.name.clone() {
                println!("{}{}", Self::label("Name"), name);
            }

            println!("{}{}", Self::label("Count"), core.count);

            Self::display_cache(core.cache, core.count);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub use cpu::*;

// ----------------------------------------------------------------------------
// ! MacOS
// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod apple;
#[cfg(target_os = "macos")]
pub use apple::*;

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub fn get_windows_midrs() -> Vec<usize> {
    use std::mem::size_of;
    use windows::Win32::System::Registry::*;
    use windows::core::{HSTRING, w};

    let mut midrs = Vec::new();
    let mut i = 0;

    loop {
        let subkey_str = format!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\{}", i);
        let subkey = HSTRING::from(&subkey_str);
        let mut hkey = HKEY::default();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                windows::core::PCWSTR(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            )
        };

        if result.is_err() {
            break;
        }

        let mut midr = None;

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
            midr = Some(cpu_id_qword as usize);
        } else {
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

            if query_cpuid.is_ok() && dw_type == REG_DWORD {
                midr = Some(cpu_id_dword as usize);
            }
        }

        let _ = unsafe { RegCloseKey(hkey) };

        if let Some(m) = midr {
            midrs.push(m);
        } else {
            // If we can't find MIDR for this core, but it exists in registry,
            // we might have reached the end of useful info or just missing one.
            // For now, continue to see if others exist.
        }

        i += 1;
    }

    midrs
}

#[cfg(target_os = "windows")]
fn get_synth_midr() -> usize {
    let midrs = get_windows_midrs();
    if !midrs.is_empty() {
        return midrs[0];
    }

    // Fallback to GetNativeSystemInfo if registry fails
    use std::mem::zeroed;
    use windows::Win32::System::SystemInformation::*;

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
