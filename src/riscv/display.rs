use super::*;
use crate::common::{CliFlags, CpuDisplay, UNK};
use crate::riscv::brand::format_uarch;

impl CpuDisplay {
    pub fn display(cpu_info: &Cpu, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        disp.newline();

        if let Some(system) = &cpu_info.system {
            disp.display_system(system, flags);
        }

        disp.simple_line("Architecture", &cpu_info.isa_string);

        disp.simple_line("SoC", &cpu_info.model);

        let ma = cpu_info.cpu_arch.micro_arch.as_str();
        if ma != UNK {
            disp.simple_line("CPU Core", ma);
        } else if let Some(uarch) = cpu_info.raw.get("uarch") {
            if !uarch.is_empty() {
                disp.simple_line("CPU Core", &format_uarch(uarch));
            }
        } else {
            let cpu_vendor_str: &str = cpu_info.cpu_arch.vendor.into();
            disp.simple_line("CPU Vendor", cpu_vendor_str);
        }

        if !(cpu_info.cpu_arch.code_name == UNK || cpu_info.cpu_arch.code_name == ma) {
            disp.simple_line("Codename", cpu_info.cpu_arch.code_name);
        }

        if let Some(tech) = cpu_info.cpu_arch.technology {
            disp.simple_line("Process Node", tech);
        }

        // Display topology
        if !cpu_info.cores.is_empty() {
            let total_cores: u32 = cpu_info.cores.values().map(|c| c.count).sum();
            println!("{}{} cores", disp.label("Topology"), total_cores);
            disp.newline();
        }

        // Display cache at top level
        if !cpu_info.cores.is_empty() {
            let total_cores: u32 = cpu_info.cores.values().map(|c| c.count).sum();
            let first_core = cpu_info.cores.values().next().unwrap();
            let cc =
                |share_count: u32| -> String { CpuDisplay::cache_count(share_count, total_cores) };
            disp.display_cache(first_core.cache, &cc, 0);
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
                        println!("{}{}", disp.inline_sublabel("Features", key), feat_str);
                        first = false;
                    } else {
                        println!("{}{}", disp.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
}
