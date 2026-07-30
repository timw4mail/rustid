use super::cpu::Cpu;
use crate::common::{CliFlags, CpuDisplay, TCpuDisplay};

impl TCpuDisplay for Cpu {
    fn debug(&self) {
        println!("PVR: {:x}", self.pvr);
        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        println!();

        let cpu = CpuDisplay { flags };

        if let Some(system) = &self.system {
            cpu.simple_line("System", &cpu.format_system_name(&system));
        }

        cpu.simple_line("Model", self.cpu_arch.marketing_name);
        cpu.simple_line("MicroArch", self.cpu_arch.micro_arch.into());
        cpu.simple_line("Code Name", self.cpu_arch.code_name);
        if let Some(tech) = self.cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        if let Some(clock_mhz) = self.clock_speed {
            println!(
                "{}{}",
                cpu.label("Frequency"),
                CpuDisplay::format_frequency(clock_mhz)
            );
            CpuDisplay::newline();
        }

        // TODO handle multiple cores/sockets
        let cc = |s| CpuDisplay::cache_count(s, 1);
        cpu.display_cache(self.cache, &cc, 0);

        println!();
    }
}
