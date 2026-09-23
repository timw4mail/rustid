//! Motorola 680x0 CPU detection for Classic Mac OS.

pub mod cpu;
pub mod display;
pub mod gestalt;
pub mod micro_arch;

pub use cpu::Cpu;

#[cfg(target_arch = "m68k")]
pub fn query_gestalt_m68k() -> (u32, u32, u32, u32, u32, u32) {
    unsafe {
        let mut cpu_val: i32 = 0;
        let mut fpu_val: i32 = 0;
        let mut mmu_val: i32 = 0;
        let mut mach_val: i32 = 0;
        let mut clk_val: i32 = 0;
        let mut sysv_val: i32 = 0;

        gestalt_raw(gestalt::GESTALT_CPU_TYPE, &mut cpu_val);
        gestalt_raw(gestalt::GESTALT_FPU_TYPE, &mut fpu_val);
        gestalt_raw(gestalt::GESTALT_MMU_TYPE, &mut mmu_val);
        gestalt_raw(gestalt::GESTALT_MACH_TYPE, &mut mach_val);
        gestalt_raw(gestalt::GESTALT_PROC_CLK, &mut clk_val);
        gestalt_raw(gestalt::GESTALT_SYS_VERSION, &mut sysv_val);

        (
            cpu_val as u32,
            fpu_val as u32,
            mmu_val as u32,
            mach_val as u32,
            clk_val as u32,
            sysv_val as u32,
        )
    }
}

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn gestalt_raw(selector: u32, response: *mut i32) -> i16;
}

#[cfg(test)]
mod tests {
    use super::gestalt::*;
    use super::micro_arch::*;
    use crate::common::TCpuDisplay;
    use crate::common::TDetect;

    #[test]
    fn test_m68k_micro_arch_find() {
        let arch_68000 = CpuArch::find_by_gestalt(1, 0, 0);
        assert_eq!(arch_68000.micro_arch, MicroArch::M68000);
        assert_eq!(arch_68000.default_fpu, FpuType::None);

        let arch_68030 = CpuArch::find_by_gestalt(4, 2, 3);
        assert_eq!(arch_68030.micro_arch, MicroArch::M68030);
        assert_eq!(arch_68030.default_fpu, FpuType::M68882);
        assert_eq!(arch_68030.default_mmu, MmuType::Integrated030);

        let arch_68lc040 = CpuArch::find_by_gestalt(5, 0, 4);
        assert_eq!(arch_68lc040.micro_arch, MicroArch::M68LC040);
        assert_eq!(arch_68lc040.default_fpu, FpuType::None);

        let arch_68040 = CpuArch::find_by_gestalt(5, 3, 4);
        assert_eq!(arch_68040.micro_arch, MicroArch::M68040);
        assert_eq!(arch_68040.default_fpu, FpuType::Integrated040);
    }

    #[test]
    fn test_m68k_gestalt_model_names() {
        assert_eq!(get_mac_model_name(4), "Macintosh Plus");
        assert_eq!(get_mac_model_name(9), "Macintosh SE/30");
        assert_eq!(get_mac_model_name(20), "Macintosh Quadra 700");
        assert_eq!(get_mac_model_name(45), "Macintosh Quadra 605 / LC 475");
    }

    #[test]
    fn test_m68k_os_version_formatting() {
        assert_eq!(format_mac_os_version(0x0608), "System 6.0.8");
        assert_eq!(format_mac_os_version(0x0710), "System 7.1");
        assert_eq!(format_mac_os_version(0x0755), "System 7.5.5");
        assert_eq!(format_mac_os_version(0x0810), "Mac OS 8.1");
        assert_eq!(format_mac_os_version(0x0922), "Mac OS 9.2.2");
    }

    #[test]
    fn test_m68k_cpu_display_and_detect() {
        let cpu = super::Cpu::detect();
        let flags = crate::common::CliFlags::default();
        let table = cpu.render_table(flags);
        assert!(table.contains("MC68030") || table.contains("Motorola"));
    }
}
