#![cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod features;
pub mod micro_arch;
use crate::common::{CoreType, CpuDisplay};
pub use micro_arch::{CpuCore, Midr};
use std::collections::{BTreeMap, HashSet};

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
// ! Linux
// ----------------------------------------------------------------------------

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

// ----------------------------------------------------------------------------
// ! Windows
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

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
        cpu_arch: &micro_arch::CpuArch,
        cores: &BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
        features: &BTreeMap<&'static str, String>,
        color: bool,
    ) {
        let cpu = CpuDisplay { color };

        println!();

        cpu.simple_line(
            "Brand",
            <brand::Vendor as Into<&str>>::into(cpu_arch.implementer),
        );

        cpu.simple_line("Model", &cpu_arch.model);

        cpu.simple_line("Codename", cpu_arch.code_name);

        if let Some(tech) = cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        #[allow(clippy::explicit_counter_loop)]
        if cores.len() > 1 {
            let mut i = 1;
            for ((kind, _, _), core) in cores {
                let core_num = format!("Core #{i}");
                println!("{}", cpu.label(&core_num));
                println!("{}{}", cpu.label("Count"), core.count);
                let name = Into::<String>::into(*kind);
                println!("{}{}", cpu.label("Type"), &name);

                if let Some(name) = core.name.clone() {
                    println!("{}{}", cpu.label("Codename"), name);
                }

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                cpu.display_cache(core.cache, &cc, 0);

                i += 1;
            }
        } else {
            println!("{}", cpu.label("Cores"));
            let keys: Vec<_> = cores.keys().collect();
            let core = cores
                .get(keys[0])
                .expect("There should be a core to display");

            if let Some(name) = core.name.clone() {
                println!("{}{}", cpu.label("Name"), name);
            }

            println!("{}{}", cpu.label("Count"), core.count);

            let cc = |s| CpuDisplay::cache_count(s, core.count);
            cpu.display_cache(core.cache, &cc, 0);
        }

        // Display features
        if !features.is_empty() {
            let keys = ["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"];
            for key in keys {
                if let Some(feat_str) = features.get(key) {
                    if key == "Base" {
                        println!("{}{}", cpu.inline_sublabel("Features", "Base"), feat_str);
                    } else {
                        println!("{}{}", cpu.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
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
