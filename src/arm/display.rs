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
    pub fn should_show_model(cpu_info: &Cpu, verbose: bool) -> bool {
        let model = &cpu_info.cpu_arch.model;
        if model == UNK || model.is_empty() {
            return false;
        }
        if verbose {
            return true;
        }
        let code_name = cpu_info.cpu_arch.code_name;
        if code_name != UNK && !code_name.is_empty() && is_duplicate(model, code_name) {
            return false;
        }
        for core in cpu_info.cores.values() {
            if let Some(name) = &core.name {
                if name != UNK && !name.is_empty() && is_duplicate(model, name) {
                    return false;
                }
            }
        }
        true
    }

    pub fn should_show_codename(cpu_info: &Cpu, verbose: bool) -> bool {
        let code_name = cpu_info.cpu_arch.code_name;
        if code_name == UNK || code_name.is_empty() {
            return false;
        }
        if verbose {
            return true;
        }
        for core in cpu_info.cores.values() {
            if let Some(name) = &core.name {
                if name != UNK && !name.is_empty() && is_duplicate(code_name, name) {
                    return false;
                }
            }
        }
        true
    }

    pub fn should_show_core_name(core_name: Option<&str>, _verbose: bool) -> bool {
        let Some(name) = core_name else {
            return false;
        };
        name != UNK && !name.is_empty()
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

        if Self::should_show_model(cpu_info, flags.verbose) {
            disp.simple_line("Model", &cpu_info.cpu_arch.model);
        }

        if Self::should_show_codename(cpu_info, flags.verbose) {
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

                if Self::should_show_core_name(core.name.as_deref(), flags.verbose) {
                    if let Some(name) = &core.name {
                        println!("{}{}", disp.label("Name"), name);
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

            if Self::should_show_core_name(core.name.as_deref(), flags.verbose) {
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
    use crate::arm::micro_arch::CpuArch;
    use crate::common::CoreType;
    use std::collections::BTreeMap;

    fn make_test_cpu(model: &str, code_name: &'static str, core_names: &[&str]) -> Cpu {
        let mut cores = BTreeMap::new();
        for (i, &cname) in core_names.iter().enumerate() {
            let midr = Midr::new(i);
            cores.insert(
                (CoreType::Performance, midr),
                CpuCore {
                    kind: CoreType::Performance,
                    name: if cname.is_empty() {
                        None
                    } else {
                        Some(cname.to_string())
                    },
                    cache: None,
                    count: 4,
                },
            );
        }

        Cpu {
            cpu_arch: CpuArch {
                model: model.to_string(),
                code_name,
                ..Default::default()
            },
            cores,
            ..Default::default()
        }
    }

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
    fn test_should_show_model_suppressed_when_matching_core_or_codename() {
        // Model "ARM Cortex-A53" matches core "Cortex-A53" and codename "Cortex-A53"
        let cpu_a53 = make_test_cpu("ARM Cortex-A53", "Cortex-A53", &["Cortex-A53"]);
        assert!(!CpuDisplay::should_show_model(&cpu_a53, false));
        assert!(CpuDisplay::should_show_model(&cpu_a53, true));

        // Model "ARM Cortex-A72" matches core "Cortex-A72" (even with different codename "Maya")
        let cpu_a72 = make_test_cpu("ARM Cortex-A72", "Maya", &["Cortex-A72"]);
        assert!(!CpuDisplay::should_show_model(&cpu_a72, false));
        assert!(CpuDisplay::should_show_model(&cpu_a72, true));

        // Model "AmpereOne" matches codename and core "AmpereOne"
        let cpu_amp = make_test_cpu("AmpereOne", "AmpereOne", &["AmpereOne"]);
        assert!(!CpuDisplay::should_show_model(&cpu_amp, false));
        assert!(CpuDisplay::should_show_model(&cpu_amp, true));

        // Model "Apple Swift" matches core "Swift"
        let cpu_swift = make_test_cpu("Apple Swift", "Swift", &["Swift"]);
        assert!(!CpuDisplay::should_show_model(&cpu_swift, false));
        assert!(CpuDisplay::should_show_model(&cpu_swift, true));
    }

    #[test]
    fn test_should_show_model_displayed_when_distinct() {
        // Model "Apple M1" is distinct from codename "Tonga" and cores "FireStorm" / "IceStorm"
        let cpu_m1 = make_test_cpu("Apple M1", "Tonga", &["FireStorm", "IceStorm"]);
        assert!(CpuDisplay::should_show_model(&cpu_m1, false));
        assert!(CpuDisplay::should_show_model(&cpu_m1, true));

        // Model "Apple A18 Pro" is distinct from "Tahiti", "Everest", "Sawtooth"
        let cpu_a18 = make_test_cpu("Apple A18 Pro", "Tahiti", &["Everest", "Sawtooth"]);
        assert!(CpuDisplay::should_show_model(&cpu_a18, false));
        assert!(CpuDisplay::should_show_model(&cpu_a18, true));
    }

    #[test]
    fn test_should_show_codename() {
        // Codename "Cortex-A53" matches core "Cortex-A53" -> suppressed
        let cpu_a53 = make_test_cpu("ARM Cortex-A53", "Cortex-A53", &["Cortex-A53"]);
        assert!(!CpuDisplay::should_show_codename(&cpu_a53, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a53, true));

        // Codename "Maya" differs from core "Cortex-A72" -> displayed
        let cpu_a72 = make_test_cpu("ARM Cortex-A72", "Maya", &["Cortex-A72"]);
        assert!(CpuDisplay::should_show_codename(&cpu_a72, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a72, true));

        // Codename "Tahiti" differs from cores "Everest", "Sawtooth" -> displayed
        let cpu_a18 = make_test_cpu("Apple A18 Pro", "Tahiti", &["Everest", "Sawtooth"]);
        assert!(CpuDisplay::should_show_codename(&cpu_a18, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a18, true));

        // Codename UNK -> always suppressed
        let cpu_unk = make_test_cpu("ARM Cortex-A53", UNK, &["Cortex-A53"]);
        assert!(!CpuDisplay::should_show_codename(&cpu_unk, false));
        assert!(!CpuDisplay::should_show_codename(&cpu_unk, true));
    }

    #[test]
    fn test_should_show_core_name() {
        assert!(CpuDisplay::should_show_core_name(Some("Cortex-A53"), false));
        assert!(CpuDisplay::should_show_core_name(Some("Cortex-A72"), false));
        assert!(CpuDisplay::should_show_core_name(Some("FireStorm"), false));
        assert!(!CpuDisplay::should_show_core_name(None, false));
        assert!(!CpuDisplay::should_show_core_name(Some(UNK), false));
        assert!(!CpuDisplay::should_show_core_name(Some(""), false));
    }
}
