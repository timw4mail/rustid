#![cfg(arm_cpu)]
//! ARM CPU detection.

mod brand;
pub mod cpu;
pub mod features;
pub mod micro_arch;
pub mod os;
use crate::common::{CliFlags, CoreType, CpuDisplay};
use std::collections::{BTreeMap, HashSet};

pub use cpu::*;
pub use micro_arch::{CpuCore, Midr};
pub use os::*;

trait TArmCpu {
    /// Returns the CPU model name, if available
    #[allow(unused)]
    fn model(&self) -> Option<&str> {
        None
    }

    fn raw_midr(&self) -> HashSet<usize>;
    fn midr(&self) -> Option<&Midr>;
    fn vendor(&self) -> &str;
}

pub struct ArmFeatures;

#[allow(unused)]
pub trait TArmFeatures {
    // Base features
    fn has_fp(&self) -> bool {
        false
    }
    fn has_asimd(&self) -> bool {
        false
    }
    fn has_evtstrm(&self) -> bool {
        false
    }
    fn has_cpuid(&self) -> bool {
        false
    }

    // SIMD/NEON features
    fn has_neon(&self) -> bool {
        self.has_asimd()
    }
    fn has_asimdhp(&self) -> bool {
        false
    }
    fn has_asimdfhm(&self) -> bool {
        false
    }
    fn has_asimddp(&self) -> bool {
        false
    }
    fn has_asimdrdm(&self) -> bool {
        false
    }

    // Crypto features
    fn has_aes(&self) -> bool {
        false
    }
    fn has_pmull(&self) -> bool {
        false
    }
    fn has_sha1(&self) -> bool {
        false
    }
    fn has_sha2(&self) -> bool {
        false
    }
    fn has_sha3(&self) -> bool {
        false
    }
    fn has_sha512(&self) -> bool {
        false
    }
    fn has_sm3(&self) -> bool {
        false
    }
    fn has_sm4(&self) -> bool {
        false
    }

    // Atomics
    fn has_atomics(&self) -> bool {
        false
    }
    fn has_lse(&self) -> bool {
        self.has_atomics()
    }
    fn has_lse2(&self) -> bool {
        false
    }

    // Floating-point features
    fn has_fphp(&self) -> bool {
        false
    }
    fn has_fp16(&self) -> bool {
        false
    }
    fn has_fcma(&self) -> bool {
        false
    }
    fn has_jscvt(&self) -> bool {
        false
    }

    // Misc features
    fn has_crc32(&self) -> bool {
        false
    }
    fn has_dcpop(&self) -> bool {
        false
    }
    fn has_lrcpc(&self) -> bool {
        false
    }
    fn has_lrcpc2(&self) -> bool {
        false
    }
    fn has_flagm(&self) -> bool {
        false
    }
    fn has_flagm2(&self) -> bool {
        false
    }
    fn has_dit(&self) -> bool {
        false
    }
    fn has_ssbs(&self) -> bool {
        false
    }
    fn has_bti(&self) -> bool {
        false
    }
    fn has_pauth(&self) -> bool {
        false
    }
    fn has_pauth2(&self) -> bool {
        false
    }
    fn has_fpac(&self) -> bool {
        false
    }
    fn has_specres(&self) -> bool {
        false
    }
    fn has_specres2(&self) -> bool {
        false
    }
    fn has_csv2(&self) -> bool {
        false
    }
    fn has_csv3(&self) -> bool {
        false
    }
    fn has_ecv(&self) -> bool {
        false
    }
    fn has_sb(&self) -> bool {
        false
    }
    fn has_frintts(&self) -> bool {
        false
    }
    fn has_dpb(&self) -> bool {
        false
    }
    fn has_dpb2(&self) -> bool {
        false
    }
    fn has_dotprod(&self) -> bool {
        false
    }
    fn has_bf16(&self) -> bool {
        false
    }
    fn has_i8mm(&self) -> bool {
        false
    }
    fn has_sve(&self) -> bool {
        false
    }
    fn has_sve2(&self) -> bool {
        false
    }
    fn has_sme(&self) -> bool {
        false
    }
}

impl CpuDisplay {
    pub fn display(
        cpu_arch: &micro_arch::CpuArch,
        cores: &BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
        features: &BTreeMap<&'static str, String>,
        flags: CliFlags,
    ) {
        let cpu = CpuDisplay { flags };

        println!();

        if let Some(system) = &cpu_arch.system {
            cpu.simple_line("System", system);
        }

        if let Some(soc_model) = &cpu_arch.soc_model {
            cpu.simple_line("SoC", soc_model);
        }

        cpu.simple_line(
            "Implementer",
            <brand::Vendor as Into<&str>>::into(cpu_arch.implementer),
        );

        cpu.simple_line("Model", &cpu_arch.model);

        cpu.simple_line("Codename", cpu_arch.code_name);

        if let Some(tech) = cpu_arch.technology {
            cpu.simple_line("Process", tech);
        }

        #[allow(clippy::explicit_counter_loop)]
        if cores.len() > 1 {
            let mut i = 1;
            for ((kind, _, _), core) in cores {
                let core_num = format!("Core #{i}");
                println!("{}", cpu.label(&core_num));
                println!("{}{}", cpu.label("Count"), core.count);
                let name = Into::<&str>::into(*kind);
                println!("{}{}", cpu.label("Type"), name);

                if let Some(name) = core.name.clone() {
                    println!("{}{}", cpu.label("Codename"), name);
                }

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                cpu.display_cache(core.cache, &cc, 0);

                println!();

                i += 1;
            }
        } else {
            println!("{}", cpu.label("Cores"));
            let keys: Vec<_> = cores.keys().collect();
            let core = cores
                .get(keys[0])
                .expect("There should be a core to display");

            if let Some(name) = core.name.clone() {
                println!("{}{}", cpu.label("Name"), name);
            }

            println!("{}{}", cpu.label("Count"), core.count);

            let cc = |s| CpuDisplay::cache_count(s, core.count);
            cpu.display_cache(core.cache, &cc, 0);
        }

        // Display features
        if !features.is_empty() {
            let keys = ["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"];
            for key in keys {
                if let Some(feat_str) = features.get(key) {
                    if key == "Base" {
                        println!("{}{}", cpu.inline_sublabel("Features", "Base"), feat_str);
                    } else {
                        println!("{}{}", cpu.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
}

/// Gets the Main ID Register (MIDR).
///
/// The MIDR contains information about the CPU implementer, part number, and revision.
pub fn get_midr() -> usize {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return get_synth_midr();

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut midr: usize = 0;
        // ARMv7 and ARMv8 (AArch64) have MIDR at c0, so `mrs r0, MIDR` or `mrs x0, MIDR_EL1`
        #[cfg(all(target_arch = "arm", not(target_os = "linux")))]
        {
            // For ARMv7-A and earlier, MIDR is c0, c0, 0
            unsafe {
                core::arch::asm!("mrc p15, 0, {midr}, c0, c0, 0", midr = out(reg) midr, options(nomem, nostack));
            }
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
        {
            // For AArch64, MIDR_EL1 (EL1)
            unsafe {
                core::arch::asm!("mrs {midr}, midr_el1", midr = out(reg) midr, options(nomem, nostack));
            }
        }
        midr
    }
}
