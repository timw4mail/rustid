use super::*;
use crate::common::{CliFlags, CpuDisplay, UNK};
use crate::riscv::brand::format_uarch;

impl CpuDisplay {
    pub fn display_riscv(cpu_info: &Cpu, flags: CliFlags) {
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

        // Display topology & per-core details
        if cpu_info.is_hybrid() {
            disp.display_topology_line(
                cpu_info.total_cores(),
                cpu_info.total_threads(),
                true,
                cpu_info.cores.len(),
            );

            for (i, core) in cpu_info.cores.iter().enumerate() {
                let core_label = format!("Core #{}", i + 1);
                println!("{}", disp.label(&core_label));

                let type_str: &str = core.kind.into();
                disp.section_line("Type", type_str);

                if let Some(name) = &core.name {
                    disp.section_line("MicroArch", name);
                }

                disp.section_line("Count", &core.count.to_string());

                disp.display_frequency(
                    core.speed,
                    CliFlags {
                        compact: true,
                        ..flags
                    },
                );

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, 0);

                if core.cache.is_none() {
                    disp.newline();
                }
            }
        } else if let Some(core) = cpu_info.cores.first() {
            disp.display_topology_line(core.count, core.threads, false, 1);

            let cc =
                |share_count: u32| -> String { CpuDisplay::cache_count(share_count, core.count) };
            disp.display_cache(core.cache, &cc, 0);

            disp.display_frequency(core.speed, flags);
        }

        // Display features
        disp.display_features(
            &cpu_info.features,
            &[
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
            ],
        );
    }
}
