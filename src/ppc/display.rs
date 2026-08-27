use super::cpu::Cpu;
use crate::common::{CliFlags, CpuDisplay, TCpuDisplay};

impl TCpuDisplay for Cpu {
    fn debug(&self) {
        println!("PVR: {:x}", self.pvr);
        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        disp.newline();

        if let Some(system) = &self.system {
            disp.display_system(system, flags);
        }

        disp.simple_line("Model", self.cpu_arch.marketing_name);
        disp.simple_line("MicroArch", self.cpu_arch.micro_arch.into());
        disp.simple_line("Codename", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            disp.simple_line("Process", tech);
        }

        let total_cores = self.total_cores();
        let total_threads = self.total_threads();
        if total_cores > 0 {
            if total_threads != total_cores {
                disp.simple_line(
                    "Topology",
                    &alloc::format!("{} cores ({} threads)", total_cores, total_threads),
                );
            } else {
                disp.simple_line("Topology", &alloc::format!("{} cores", total_cores));
            }
        }

        if let Some(core) = self.cores.first() {
            if let Some(speed) = &core.speed
                && speed.base > 0 {
                    if speed.boost > speed.base {
                        println!(
                            "{}{}",
                            disp.inline_sublabel("Frequency", "Base"),
                            CpuDisplay::format_frequency(speed.base)
                        );
                        println!(
                            "{}{}",
                            disp.sublabel("Boost"),
                            CpuDisplay::format_frequency(speed.boost)
                        );
                        disp.newline();
                    } else {
                        disp.simple_line("Frequency", &CpuDisplay::format_frequency(speed.base));
                    }
                }

            let cc = |s| CpuDisplay::cache_count(s, total_cores);
            disp.display_cache(core.cache, &cc, 0);
        }

        println!();
    }
}
