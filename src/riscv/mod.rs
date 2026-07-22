#![cfg(target_arch = "riscv64")]
//! RISC-V CPU detection.

pub mod brand;
pub mod cpu;
pub mod features;
pub mod micro_arch;
pub mod os;
use crate::common::{CliFlags, CpuDisplay};
pub use cpu::*;
pub use micro_arch::{CpuArch, CpuCore};
pub use os::*;

pub(crate) trait TRiscvCpu {
    /// Returns the CPU model name, if available
    #[allow(unused)]
    fn model(&self) -> Option<&str> {
        None
    }

    #[allow(unused)]
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
            let total_cores: u32 = cpu_info.cores.values().map(|c| c.count).sum();
            println!(
                "{}{} cores across {} core types",
                cpu.label("Cpu Topology"),
                total_cores,
                cpu_info.cores.len()
            );
            CpuDisplay::newline();

            for (i, core) in cpu_info.cores.iter().enumerate() {
                let core_label = alloc::format!("Core #{}", i + 1);
                println!("{}", cpu.label(&core_label));

                let type_str: &str = core.1.kind.into();
                println!("{}{}", cpu.label("Type"), type_str);

                if let Some(name) = &core.1.name {
                    println!("{}{}", cpu.label("Codename"), name);
                }

                println!("{}{} cores", cpu.label("Topology"), core.1.count);

                let cc = |s| CpuDisplay::cache_count(s, core.1.count);
                cpu.display_cache(core.1.cache, &cc, 0);
            }
        }

        // Display features
        if !cpu_info.features.is_empty() {
            let keys = [
                "Mul",
                "Atomic",
                "Float",
                "Compressed",
                "Bitmanip",
                "Vector",
                "Crypto",
                "Priv",
                "Cache",
                "Misc",
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
