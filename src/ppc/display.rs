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
        disp.simple_line_opt("Process", self.cpu_arch.technology);

        let total_cores = self.total_cores();
        let total_threads = self.total_threads();
        let sockets = self.total_sockets();

        if total_cores > 1 || total_threads > 1 || sockets > 1 || flags.verbose {
            disp.display_topology_line(
                sockets,
                total_cores,
                total_threads,
                self.is_hybrid(),
                self.cores.len(),
            );
        }

        if let Some(core) = self.cores.first() {
            disp.display_frequency(core.speed, flags);

            disp.display_core_cache(core.cache, total_cores, sockets);
        }

        println!();
    }
}
