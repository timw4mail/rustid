#![cfg(arm_cpu)]
//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod features;
pub mod micro_arch;
pub mod os;
use crate::common::{CliFlags, CpuDisplay};
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

impl CpuDisplay {
    pub fn display(cpu_info: &Cpu, flags: CliFlags) {
        let cpu = CpuDisplay { flags };

        println!();

        if let Some(system) = &cpu_info.system {
            cpu.simple_line("System", cpu.format_system_name(system));
        }

        if let Some(soc_model) = &cpu_info.soc_model {
            cpu.simple_line("SoC", soc_model);
        }

        cpu.simple_line(
            "Implementer",
            <brand::Vendor as Into<&str>>::into(cpu_info.cpu_arch.implementer),
        );

        cpu.simple_line("Model", &cpu_info.cpu_arch.model);

        cpu.simple_line("Codename", cpu_info.cpu_arch.code_name);

        if let Some(tech) = cpu_info.cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        #[allow(clippy::explicit_counter_loop)]
        if cpu_info.cores.len() > 1 {
            let mut i = 1;
            for core in cpu_info.cores.values() {
                let core_num = format!("Core #{i}");
                println!("{}", cpu.label(&core_num));
                println!("{}{}", cpu.label("Count"), core.count);
                let name = Into::<&str>::into(core.kind);
                println!("{}{}", cpu.label("Type"), name);

                if let Some(name) = core.name.clone() {
                    println!("{}{}", cpu.label("Codename"), name);
                }

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                cpu.display_cache(core.cache, &cc, 0);

                if core.cache.is_none() {
                    CpuDisplay::newline();
                }

                i += 1;
            }
        } else {
            println!("{}", cpu.label("Cores"));
            let keys: Vec<_> = cpu_info.cores.keys().collect();
            let core = cpu_info
                .cores
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
        if !cpu_info.features.is_empty() {
            let keys = ["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"];
            for key in keys {
                if let Some(feat_str) = cpu_info.features.get(key) {
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
        #[cfg(all(target_arch = "arm", not(target_os = "linux")))]
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
