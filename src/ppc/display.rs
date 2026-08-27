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
        disp.display_topology_line(
            total_cores,
            total_threads,
            self.is_hybrid(),
            self.cores.len(),
        );

        if let Some(core) = self.cores.first() {
            disp.display_frequency(core.speed, flags);

            let cc = |s| CpuDisplay::cache_count(s, total_cores);
            disp.display_cache(core.cache, &cc, 0);
        }

        println!();
    }
}
