#![cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod micro_arch;
use crate::common::{CoreType, Level1Cache};
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

pub struct CpuDisplay;

impl CpuDisplay {
    fn label(s: &str) -> String {
        #[cfg(not(target_family = "unix"))]
        return format!("{:>17}: ", s);
        #[cfg(target_family = "unix")]
        return format!("\x1b[32m{:>17}\x1b[0m: ", s);
    }

    fn sublabel(s: &str) -> String {
        #[cfg(not(target_family = "unix"))]
        return format!("{:>19} : ", s);
        #[cfg(target_family = "unix")]
        return format!("\x1b[94m{:>19}{}\x1b[0m: ", "", s);
    }

    fn inline_sublabel(label: &str, sub: &str) -> String {
        #[cfg(not(target_family = "unix"))]
        return format!("{:>17}: {:1}: ", label, sub);
        #[cfg(target_family = "unix")]
        return format!("\x1b[32m{:>17}\x1b[0m: \x1b[94m{:1}\x1b[0m: ", label, sub);
    }

    fn cache_count(share_count: u32, core_count: u32) -> String {
        if share_count == 0 || (core_count / share_count) <= 1 {
            String::new()
        } else {
            format!("{}x ", core_count / share_count)
        }
    }

    pub fn display(
        cpu_arch: &crate::arm::micro_arch::CpuArch,
        cores: &BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
    ) {
        println!();
        println!(
            "{}{}",
            Self::label("Brand/Implementor"),
            <crate::arm::brand::Vendor as Into<&str>>::into(cpu_arch.implementer)
        );
        println!();

        println!("{}{}", Self::label("Model"), cpu_arch.model);
        println!();

        println!("{}{}", Self::label("Code Name"), cpu_arch.code_name);
        println!();

        if let Some(tech) = cpu_arch.technology {
            println!("{}{}", Self::label("Process"), tech);
            println!();
        }

        for ((kind, _, _), core) in cores {
            let name = format!("{} Cores", Into::<String>::into(*kind));
            println!("{}", Self::label(&name));

            if let Some(name) = core.name.clone() {
                println!("{}{}", Self::label("Name"), name);
            }

            println!("{}{}", Self::label("Count"), core.count);

            if let Some(cache) = core.cache {
                #[inline]
                fn cache_size(raw_size: u32) -> (u32, &'static str) {
                    let mut num = raw_size / 1024;
                    let unit = if num >= 1024 { "MB" } else { "KB" };

                    if num >= 1024 {
                        num /= 1024;
                    }

                    (num, unit)
                }

                match cache.l1 {
                    Level1Cache::Unified(cache) => {
                        println!("{}L1: Unified {:>4} KB", Self::label("Cache"), cache.size);
                    }
                    Level1Cache::Split { data, instruction } => {
                        let data_count: String =
                            Self::cache_count(data.share_count, core.count as u32);
                        let instruction_count =
                            Self::cache_count(instruction.share_count, core.count as u32);

                        if data.assoc > 0 {
                            println!(
                                "{}{}{} KB, {}-way",
                                Self::inline_sublabel("Cache", "L1d"),
                                &data_count,
                                data.size / 1024,
                                data.assoc
                            );
                        } else {
                            println!(
                                "{}{}{} KB",
                                Self::inline_sublabel("Cache", "L1d"),
                                &data_count,
                                data.size / 1024
                            );
                        }

                        if instruction.assoc > 0 {
                            println!(
                                "{}{}{} KB, {}-way",
                                Self::sublabel("L1i"),
                                &instruction_count,
                                instruction.size / 1024,
                                instruction.assoc
                            );
                        } else {
                            println!(
                                "{}{}{} KB",
                                Self::sublabel("L1i"),
                                &instruction_count,
                                instruction.size / 1024,
                            );
                        }
                    }
                }

                if let Some(cache) = cache.l2 {
                    let count = Self::cache_count(cache.share_count, core.count as u32);
                    let (num, unit) = cache_size(cache.size);

                    if cache.assoc > 0 {
                        println!(
                            "{} {}{} {}, {}-way",
                            Self::sublabel("L2"),
                            &count,
                            num,
                            unit,
                            cache.assoc
                        );
                    } else {
                        println!("{} {}{} {}", Self::sublabel("L2"), &count, num, unit);
                    }
                }

                if let Some(cache) = cache.l3 {
                    let (num, unit) = cache_size(cache.size);
                    let cache_count = Self::cache_count(cache.share_count, core.count as u32);

                    if cache.assoc > 0 {
                        println!(
                            "{} {}{} {}, {}-way",
                            Self::sublabel("L3"),
                            &cache_count,
                            num,
                            unit,
                            cache.assoc
                        );
                    } else {
                        println!("{} {}{} {}", Self::sublabel("L3"), &cache_count, num, unit);
                    }
                }
            }
            println!();
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
