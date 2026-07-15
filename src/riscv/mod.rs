#![cfg(target_arch = "riscv64")]
//! RISC-V CPU detection.

pub mod brand;
pub mod cpu;
pub mod features;
pub mod micro_arch;
pub mod os;
use crate::common::{CliFlags, CpuDisplay};
pub use cpu::*;
pub use features::{RiscvFeatures, TRiscvFeatures};
pub use micro_arch::CpuArch;
pub use os::*;

pub(crate) trait TRiscvCpu {
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
            cpu.simple_line("System", &cpu.format_system_name(system));
        }

        cpu.simple_line("Vendor", &cpu_info.vendor);

        cpu.simple_line("Model", &cpu_info.cpu_arch.model);

        cpu.simple_line("Codename", cpu_info.cpu_arch.code_name);

        if let Some(tech) = cpu_info.cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        cpu.simple_line("ISA", &cpu_info.isa_string);

        // Display cores
        if !cpu_info.cores.is_empty() {
            println!("{}", cpu.label("Cores"));
            let keys: Vec<_> = cpu_info.cores.keys().collect();
            let core = cpu_info
                .cores
                .get(keys[0])
                .expect("There should be a core to display");

            if let Some(name) = &core.name {
                println!("{}{}", cpu.label("Name"), name);
            }

            println!("{}{}", cpu.label("Count"), core.count);

            let cc = |s| CpuDisplay::cache_count(s, core.count);
            cpu.display_cache(core.cache, &cc, 0);
        }

        // Display features
        if !cpu_info.features.is_empty() {
            let keys = [
                "Mul", "Atomic", "Float", "Compressed", "Bitmanip",
                "Vector", "Crypto", "Priv", "Cache", "Misc",
            ];
            let mut first = true;
            for key in keys {
                if let Some(feat_str) = cpu_info.features.get(key) {
                    if first {
                        println!("{}{}", cpu.inline_sublabel("Extensions", key), feat_str);
                        first = false;
                    } else {
                        println!("{}{}", cpu.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
}

/// Reads the RISC-V `misa` CSR via inline assembly.
///
/// Returns 0 on platforms without inline asm support.
pub fn get_misa() -> u64 {
    #[cfg(target_arch = "riscv64")]
    {
        let val: u64;
        unsafe {
            core::arch::asm!("csrr {rd}, misa", rd = out(reg) val, options(nomem, nostack));
        }
        val
    }
    #[cfg(not(target_arch = "riscv64"))]
    {
        0
    }
}
