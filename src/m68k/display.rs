use super::cpu::Cpu;
use super::micro_arch::{FpuType, MmuType};
use crate::common::{CpuDisplay, TCpuDisplay};

impl TCpuDisplay for Cpu {
    fn render_debug(&self) -> alloc::string::String {
        alloc::format!(
            "CPU Type: {}\nFPU: {:?}\nMMU: {:?}\nMach ID: {}\nSys Version: 0x{:04x}\n{:#?}",
            self.extra.cpu_type,
            self.extra.fpu_type,
            self.extra.mmu_type,
            self.extra.mach_id,
            self.extra.sys_version_bcd,
            self
        )
    }

    fn display_table_with_disp(&self, disp: &mut CpuDisplay) {
        let flags = disp.flags;

        disp.newline();

        if let Some(system) = &self.system {
            disp.display_system(system, flags);
        }

        disp.simple_line("Model", self.extra.cpu_arch.marketing_name);
        disp.simple_line("MicroArch", self.extra.cpu_arch.micro_arch.into());
        disp.simple_line("Codename", self.extra.cpu_arch.code_name);
        disp.simple_line_opt("Process", self.extra.cpu_arch.technology);

        if self.extra.fpu_type != FpuType::None || flags.verbose {
            disp.simple_line("FPU", self.extra.fpu_type.into());
        }

        if self.extra.mmu_type != MmuType::None || flags.verbose {
            disp.simple_line("MMU", self.extra.mmu_type.into());
        }

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

        disp.print_raw_newline();
    }
}
