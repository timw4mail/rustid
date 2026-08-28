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
        } else if let Some(uarch) = cpu_info.raw.get("uarch")
            && !uarch.is_empty()
        {
            disp.simple_line("CPU Core", &format_uarch(uarch));
        } else {
            let cpu_vendor_str: &str = cpu_info.cpu_arch.vendor.into();
            disp.simple_line("CPU Vendor", cpu_vendor_str);
        }

        if !(cpu_info.cpu_arch.code_name == UNK || cpu_info.cpu_arch.code_name == ma) {
            disp.simple_line("Codename", cpu_info.cpu_arch.code_name);
        }

        disp.simple_line_opt("Process Node", cpu_info.cpu_arch.technology);

        // Display topology & per-core details
        if cpu_info.is_hybrid() {
            disp.display_topology_line(
                cpu_info.total_cores(),
                cpu_info.total_threads(),
                true,
                cpu_info.cores.len(),
            );

            for (i, core) in cpu_info.cores.iter().enumerate() {
                disp.core_heading(i);

                let type_str: &str = core.kind.into();
                disp.section_line("Type", type_str);

                disp.section_line_opt("MicroArch", core.name.as_deref());

                disp.section_line("Count", &core.count.to_string());

                disp.display_frequency(
                    core.speed,
                    CliFlags {
                        compact: true,
                        ..flags
                    },
                );

                disp.display_core_cache(core.cache, core.count, 0);

                if core.cache.is_none() {
                    disp.newline();
                }
            }
        } else if let Some(core) = cpu_info.cores.first() {
            disp.display_topology_line(core.count, core.threads, false, 1);

            disp.display_core_cache(core.cache, core.count, 0);

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
