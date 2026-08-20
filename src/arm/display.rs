use super::cpu::Cpu;
use super::*;
use crate::common::{CliFlags, CpuDisplay, TCpuDisplay, UNK};

fn is_duplicate(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let a_lower = a.to_ascii_lowercase();
    let b_lower = b.to_ascii_lowercase();
    a_lower == b_lower || a_lower.contains(&b_lower) || b_lower.contains(&a_lower)
}

impl CpuDisplay {
    pub fn should_show_codename(model: &str, code_name: &str, verbose: bool) -> bool {
        if code_name == UNK || code_name.is_empty() {
            return false;
        }
        if verbose {
            return true;
        }
        !is_duplicate(model, code_name)
    }

    pub fn should_show_core_name(
        _model: &str,
        code_name: &str,
        core_name: Option<&str>,
        verbose: bool,
    ) -> bool {
        let Some(name) = core_name else {
            return false;
        };
        if name == UNK || name.is_empty() {
            return false;
        }
        if verbose {
            return true;
        }
        !is_duplicate(name, code_name)
    }

    pub fn display(cpu_info: &Cpu, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        disp.newline();

        if let Some(system) = &cpu_info.system {
            disp.display_system(system, flags);
        }

        if let Some(soc_model) = &cpu_info.soc_model {
            disp.simple_line("SoC", soc_model);
        }

        disp.simple_line(
            "Implementer",
            <brand::Vendor as Into<&str>>::into(cpu_info.cpu_arch.implementer),
        );

        disp.simple_line("Model", &cpu_info.cpu_arch.model);

        if Self::should_show_codename(
            &cpu_info.cpu_arch.model,
            cpu_info.cpu_arch.code_name,
            flags.verbose,
        ) {
            disp.simple_line("Codename", cpu_info.cpu_arch.code_name);
        }

        if let Some(tech) = cpu_info.cpu_arch.technology {
            disp.simple_line("Process", tech);
        }

        #[allow(clippy::explicit_counter_loop)]
        if cpu_info.cores.len() > 1 {
            let mut i = 1;
            for core in cpu_info.cores.values() {
                let core_num = format!("Core #{i}");
                println!("{}", disp.label(&core_num));
                println!("{}{}", disp.label("Count"), core.count);
                let name = Into::<&str>::into(core.kind);
                println!("{}{}", disp.label("Type"), name);

                if Self::should_show_core_name(
                    &cpu_info.cpu_arch.model,
                    cpu_info.cpu_arch.code_name,
                    core.name.as_deref(),
                    flags.verbose,
                ) {
                    if let Some(name) = &core.name {
                        println!("{}{}", disp.label("Codename"), name);
                    }
                }

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, 0);

                if core.cache.is_none() {
                    disp.newline();
                }

                i += 1;
            }
        } else if let Some(core) = cpu_info.cores.values().next() {
            println!("{}", disp.label("Cores"));

            if Self::should_show_core_name(
                &cpu_info.cpu_arch.model,
                cpu_info.cpu_arch.code_name,
                core.name.as_deref(),
                flags.verbose,
            ) {
                if let Some(name) = &core.name {
                    println!("{}{}", disp.label("Name"), name);
                }
            }

            println!("{}{}", disp.label("Count"), core.count);

            let cc = |s| CpuDisplay::cache_count(s, core.count);
            disp.display_cache(core.cache, &cc, 0);
        }

        // Display features
        if !cpu_info.features.is_empty() {
            let keys = ["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"];
            for key in keys {
                if let Some(feat_str) = cpu_info.features.get(key) {
                    if key == "Base" {
                        println!("{}{}", disp.inline_sublabel("Features", "Base"), feat_str);
                    } else {
                        println!("{}{}", disp.sublabel(key), feat_str);
                    }
                }
            }
            println!();
        }
    }
}

impl TCpuDisplay for Cpu {
    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        if !self.midrs.is_empty() {
            println!("Main ID Register (MIDR) values:");
            for (i, midr) in self.midrs.iter().enumerate() {
                println!("Midr {i}:");
                println!("    Raw: 0x{:X}", midr.raw);
                println!(
                    "    Implementer: 0x{:X} ({})",
                    midr.implementer,
                    self.vendor()
                );
                println!("    Variant: 0x{:X}", midr.variant);
                println!("    Part Number: 0x{:X}", midr.part);
                println!("    Revision: 0x{:X}", midr.revision);
            }

            println!();
        }

        println!("{:#?}", self);
    }

    fn display_table(&self, flags: CliFlags) {
        CpuDisplay::display(self, flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_duplicate() {
        assert!(is_duplicate("ARM Cortex-A53", "Cortex-A53"));
        assert!(is_duplicate("Cortex-A53", "ARM Cortex-A53"));
        assert!(is_duplicate("Apple Swift", "Swift"));
        assert!(is_duplicate("AmpereOne", "AmpereOne"));
        assert!(is_duplicate("cortex-a53", "CORTEX-A53"));

        assert!(!is_duplicate("ARM Cortex-A72", "Maya"));
        assert!(!is_duplicate("Maya", "Cortex-A72"));
        assert!(!is_duplicate("Apple A18 Pro", "Tahiti"));
        assert!(!is_duplicate("Everest", "Tahiti"));
        assert!(!is_duplicate("Sawtooth", "Tahiti"));
        assert!(!is_duplicate("Apple M1", "Tonga"));
        assert!(!is_duplicate("FireStorm", "Tonga"));
        assert!(!is_duplicate("", "Maya"));
        assert!(!is_duplicate("Maya", ""));
    }

    #[test]
    fn test_should_show_codename_duplicate() {
        assert!(!CpuDisplay::should_show_codename(
            "ARM Cortex-A53",
            "Cortex-A53",
            false
        ));
        assert!(CpuDisplay::should_show_codename(
            "ARM Cortex-A53",
            "Cortex-A53",
            true
        ));
        assert!(!CpuDisplay::should_show_codename(
            "Apple Swift",
            "Swift",
            false
        ));
        assert!(CpuDisplay::should_show_codename(
            "Apple Swift",
            "Swift",
            true
        ));
        assert!(!CpuDisplay::should_show_codename(
            "AmpereOne",
            "AmpereOne",
            false
        ));
        assert!(CpuDisplay::should_show_codename(
            "AmpereOne",
            "AmpereOne",
            true
        ));
    }

    #[test]
    fn test_should_show_codename_different() {
        assert!(CpuDisplay::should_show_codename(
            "ARM Cortex-A72",
            "Maya",
            false
        ));
        assert!(CpuDisplay::should_show_codename(
            "ARM Cortex-A72",
            "Maya",
            true
        ));
        assert!(CpuDisplay::should_show_codename(
            "Apple A18 Pro",
            "Tahiti",
            false
        ));
        assert!(CpuDisplay::should_show_codename("Apple M1", "Tonga", false));
    }

    #[test]
    fn test_should_show_codename_unknown() {
        assert!(!CpuDisplay::should_show_codename(
            "ARM Cortex-A53",
            UNK,
            false
        ));
        assert!(!CpuDisplay::should_show_codename(
            "ARM Cortex-A53",
            UNK,
            true
        ));
        assert!(!CpuDisplay::should_show_codename(
            "ARM Cortex-A53",
            "",
            false
        ));
    }

    #[test]
    fn test_should_show_core_name_duplicate() {
        assert!(!CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            Some("Cortex-A53"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            Some("Cortex-A53"),
            true
        ));
        assert!(!CpuDisplay::should_show_core_name(
            "Apple Swift",
            "Swift",
            Some("Swift"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "Apple Swift",
            "Swift",
            Some("Swift"),
            true
        ));
    }

    #[test]
    fn test_should_show_core_name_different() {
        // When codename is different from core name, both values should be shown
        assert!(CpuDisplay::should_show_core_name(
            "ARM Cortex-A72",
            "Maya",
            Some("Cortex-A72"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "ARM Cortex-A72",
            "Maya",
            Some("Cortex-A72"),
            true
        ));
        assert!(CpuDisplay::should_show_core_name(
            "Apple M1",
            "Tonga",
            Some("FireStorm"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "Apple M1",
            "Tonga",
            Some("IceStorm"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "Apple A18 Pro",
            "Tahiti",
            Some("Everest"),
            false
        ));
        assert!(CpuDisplay::should_show_core_name(
            "Apple A18 Pro",
            "Tahiti",
            Some("Sawtooth"),
            false
        ));
    }

    #[test]
    fn test_should_show_core_name_none_or_unknown() {
        assert!(!CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            None,
            false
        ));
        assert!(!CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            None,
            true
        ));
        assert!(!CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            Some(UNK),
            false
        ));
        assert!(!CpuDisplay::should_show_core_name(
            "ARM Cortex-A53",
            "Cortex-A53",
            Some(UNK),
            true
        ));
    }
}
