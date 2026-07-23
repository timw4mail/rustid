use super::*;

impl CpuDisplay {
    pub fn display(cpu_info: &Cpu, flags: CliFlags) {
        let cpu = CpuDisplay { flags };

        println!();

        if let Some(system) = &cpu_info.system {
            cpu.simple_line("System", &cpu.format_system_name(system));
        }

        cpu.simple_line("Architecture", &cpu_info.isa_string);

        cpu.simple_line("Vendor", &cpu_info.vendor);

        cpu.simple_line("SoC", &cpu_info.model);

        let ma = cpu_info.cpu_arch.micro_arch.as_str();
        if ma != UNK {
            cpu.simple_line("MicroArch", ma);
        }

        if !(cpu_info.cpu_arch.code_name == UNK || cpu_info.cpu_arch.code_name == ma) {
            cpu.simple_line("Codename", cpu_info.cpu_arch.code_name);
        }

        if let Some(tech) = cpu_info.cpu_arch.technology {
            cpu.simple_line("Process Node", tech);
        }

        // Display topology
        if !cpu_info.cores.is_empty() {
            let total_cores: u32 = cpu_info.cores.values().map(|c| c.count).sum();
            println!("{}{} cores", cpu.label("Topology"), total_cores);
            CpuDisplay::newline();
        }

        // Display cache at top level
        if !cpu_info.cores.is_empty() {
            let total_cores: u32 = cpu_info.cores.values().map(|c| c.count).sum();
            let first_core = cpu_info.cores.values().next().unwrap();
            let cc =
                |share_count: u32| -> String { CpuDisplay::cache_count(share_count, total_cores) };
            cpu.display_cache(first_core.cache, &cc, 0);
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
                        println!("{}{}", cpu.inline_sublabel("Features", key), feat_str);
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
