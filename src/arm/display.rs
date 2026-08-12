use super::cpu::Cpu;
use super::*;
use crate::common::TCpuDisplay;
use crate::common::{CliFlags, CpuDisplay};

impl CpuDisplay {
    pub fn display(cpu_info: &Cpu, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        disp.newline();

        if let Some(system) = &cpu_info.system {
            disp.simple_line("System", &disp.format_system_name(system));
        }

        if let Some(soc_model) = &cpu_info.soc_model {
            disp.simple_line("SoC", soc_model);
        }

        disp.simple_line(
            "Implementer",
            <brand::Vendor as Into<&str>>::into(cpu_info.cpu_arch.implementer),
        );

        disp.simple_line("Model", &cpu_info.cpu_arch.model);

        disp.simple_line("Codename", cpu_info.cpu_arch.code_name);

        if let Some(tech) = cpu_info.cpu_arch.technology {
            disp.simple_line("Process", tech);
        }

        #[allow(clippy::explicit_counter_loop)]
        if cpu_info.cores.len() > 1 {
            let mut i = 1;
            for core in cpu_info.cores.values() {
                let core_num = format!("Core #{i}");
                println!("{}", disp.label(&core_num));
                println!("{}{}", disp.label("Count"), core.count);
                let name = Into::<&str>::into(core.kind);
                println!("{}{}", disp.label("Type"), name);

                if let Some(name) = core.name.clone() {
                    println!("{}{}", disp.label("Codename"), name);
                }

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, 0);

                if core.cache.is_none() {
                    disp.newline();
                }

                i += 1;
            }
        } else {
            println!("{}", disp.label("Cores"));
            let keys: Vec<_> = cpu_info.cores.keys().collect();
            let core = cpu_info
                .cores
                .get(keys[0])
                .expect("There should be a core to display");

            if let Some(name) = core.name.clone() {
                println!("{}{}", disp.label("Name"), name);
            }

            println!("{}{}", disp.label("Count"), core.count);

            let cc = |s| CpuDisplay::cache_count(s, core.count);
            disp.display_cache(core.cache, &cc, 0);
        }

        // Display features
        if !cpu_info.features.is_empty() {
            let keys = ["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"];
            for key in keys {
                if let Some(feat_str) = cpu_info.features.get(key) {
                    if key == "Base" {
                        println!("{}{}", disp.inline_sublabel("Features", "Base"), feat_str);
                    } else {
                        println!("{}{}", disp.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
}

impl TCpuDisplay for Cpu {
    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        if !self.midrs.is_empty() {
            println!("Main ID Register (MIDR) values:");
            for (i, midr) in self.midrs.iter().enumerate() {
                println!("Midr {i}:");
                println!("    Raw: 0x{:X}", midr.raw);
                println!(
                    "    Implementer: 0x{:X} ({})",
                    midr.implementer,
                    self.vendor()
                );
                println!("    Variant: 0x{:X}", midr.variant);
                println!("    Part Number: 0x{:X}", midr.part);
                println!("    Revision: 0x{:X}", midr.revision);
            }

            println!();
        }

        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        CpuDisplay::display(self, flags);
    }
}
