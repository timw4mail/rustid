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

        if let Some(clock_mhz) = self.clock_speed {
            println!(
                "{}{}",
                disp.label("Frequency"),
                CpuDisplay::format_frequency(clock_mhz)
            );
            disp.newline();
        }

        // TODO handle multiple cores/sockets
        let cc = |s| CpuDisplay::cache_count(s, 1);
        disp.display_cache(self.cache, &cc, 0);

        println!();
    }
}
