use super::cpu::Cpu;
use super::micro_arch::MicroArch;
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
        if let Some(soc) = &cpu_info.soc_model
            && is_duplicate(model, soc) {
                return false;
            }
        let code_name = cpu_info.cpu_arch.code_name;
        if code_name != UNK && !code_name.is_empty() && is_duplicate(model, code_name) {
            return false;
        }
        for core in cpu_info.cores.values() {
            let ma_str: String = core.micro_arch.into();
            if ma_str != UNK && is_duplicate(model, &ma_str) {
                return false;
            }
            if let Some(cname) = &core.code_name
                && cname != UNK && !cname.is_empty() && is_duplicate(model, cname) {
                    return false;
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
            let ma_str: String = core.micro_arch.into();
            if ma_str != UNK && is_duplicate(code_name, &ma_str) {
                return false;
            }
            if let Some(cname) = &core.code_name
                && cname != UNK && !cname.is_empty() && is_duplicate(code_name, cname) {
                    return false;
                }
        }
        true
    }

    pub fn should_show_core_micro_arch(micro_arch: MicroArch, _verbose: bool) -> bool {
        micro_arch != MicroArch::Unknown
    }

    pub fn should_show_core_codename(core: &CpuCore, verbose: bool) -> bool {
        let Some(code_name) = &core.code_name else {
            return false;
        };
        if code_name == UNK || code_name.is_empty() {
            return false;
        }
        if verbose {
            return true;
        }
        let ma_str: String = core.micro_arch.into();
        if ma_str != UNK && is_duplicate(code_name, &ma_str) {
            return false;
        }
        true
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

                let vendor_str: &str = core.implementer.into();
                if vendor_str != UNK {
                    println!("{}{}", disp.label("Implementer"), vendor_str);
                }

                let name = Into::<&str>::into(core.kind);
                println!("{}{}", disp.label("Type"), name);

                let ma_str: String = core.micro_arch.into();
                if Self::should_show_core_micro_arch(core.micro_arch, flags.verbose) {
                    println!("{}{}", disp.label("MicroArch"), ma_str);
                }

                if Self::should_show_core_codename(core, flags.verbose)
                    && let Some(codename) = &core.code_name {
                        println!("{}{}", disp.label("Codename"), codename);
                    }

                println!("{}{}", disp.label("Count"), core.count);

                let cc = |s| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, 0);

                if core.cache.is_none() {
                    disp.newline();
                }

                i += 1;
            }
        } else if let Some(core) = cpu_info.cores.values().next() {
            println!("{}", disp.label("Cores"));

            let vendor_str: &str = core.implementer.into();
            if vendor_str != UNK {
                println!("{}{}", disp.label("Implementer"), vendor_str);
            }

            let ma_str: String = core.micro_arch.into();
            if Self::should_show_core_micro_arch(core.micro_arch, flags.verbose) {
                println!("{}{}", disp.label("MicroArch"), ma_str);
            }

            if Self::should_show_core_codename(core, flags.verbose)
                && let Some(codename) = &core.code_name {
                    println!("{}{}", disp.label("Codename"), codename);
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
                    <brand::Vendor as Into<&str>>::into(brand::Vendor::from(midr.implementer))
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
    use crate::arm::brand::Vendor;
    use crate::arm::micro_arch::CpuArch;
    use crate::common::CoreType;
    use std::collections::{BTreeMap, HashSet};

    fn make_test_cpu(
        model: &str,
        code_name: &'static str,
        core_info: &[(Vendor, MicroArch, Option<&str>)],
    ) -> Cpu {
        let mut cores = BTreeMap::new();
        for (i, &(implementer, ma, cname)) in core_info.iter().enumerate() {
            let midr = Midr::new(i);
            let kind = ma.core_type();
            cores.insert(
                (kind, midr),
                CpuCore {
                    implementer,
                    kind,
                    micro_arch: ma,
                    code_name: cname.map(String::from),
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
        // Model "ARM Cortex-A53" matches core MicroArch "Cortex-A53" and codename "Cortex-A53"
        let cpu_a53 = make_test_cpu(
            "ARM Cortex-A53",
            "Cortex-A53",
            &[(Vendor::Arm, MicroArch::ArmCortexA53, Some("Cortex-A53"))],
        );
        assert!(!CpuDisplay::should_show_model(&cpu_a53, false));
        assert!(CpuDisplay::should_show_model(&cpu_a53, true));

        // Model "ARM Cortex-A72" matches core MicroArch "Cortex-A72" (even with different codename "Maya")
        let cpu_a72 = make_test_cpu(
            "ARM Cortex-A72",
            "Maya",
            &[(Vendor::Arm, MicroArch::ArmCortexA72, Some("Maya"))],
        );
        assert!(!CpuDisplay::should_show_model(&cpu_a72, false));
        assert!(CpuDisplay::should_show_model(&cpu_a72, true));

        // Model "AmpereOne" matches codename and core "AmpereOne"
        let cpu_amp = make_test_cpu(
            "AmpereOne",
            "AmpereOne",
            &[(Vendor::Ampere, MicroArch::AmpereOne, Some("AmpereOne"))],
        );
        assert!(!CpuDisplay::should_show_model(&cpu_amp, false));
        assert!(CpuDisplay::should_show_model(&cpu_amp, true));

        // Model "Apple Swift" matches core "Swift"
        let cpu_swift = make_test_cpu(
            "Apple Swift",
            "Swift",
            &[(Vendor::Apple, MicroArch::AppleSwift, Some("Swift"))],
        );
        assert!(!CpuDisplay::should_show_model(&cpu_swift, false));
        assert!(CpuDisplay::should_show_model(&cpu_swift, true));
    }

    #[test]
    fn test_should_show_model_displayed_when_distinct() {
        // Model "Apple M1" is distinct from SoC codename "Tonga" and cores "FireStorm" / "IceStorm"
        let cpu_m1 = make_test_cpu(
            "Apple M1",
            "Tonga",
            &[
                (Vendor::Apple, MicroArch::AppleFirestorm, None),
                (Vendor::Apple, MicroArch::AppleIcestorm, None),
            ],
        );
        assert!(CpuDisplay::should_show_model(&cpu_m1, false));
        assert!(CpuDisplay::should_show_model(&cpu_m1, true));

        // Model "Apple A18 Pro" is distinct from "Tahiti", "Everest", "Sawtooth"
        let cpu_a18 = make_test_cpu(
            "Apple A18 Pro",
            "Tahiti",
            &[
                (Vendor::Apple, MicroArch::AppleEverest, None),
                (Vendor::Apple, MicroArch::AppleSawtooth, None),
            ],
        );
        assert!(CpuDisplay::should_show_model(&cpu_a18, false));
        assert!(CpuDisplay::should_show_model(&cpu_a18, true));
    }

    #[test]
    fn test_should_show_codename() {
        // Codename "Cortex-A53" matches core MicroArch "Cortex-A53" -> suppressed
        let cpu_a53 = make_test_cpu(
            "ARM Cortex-A53",
            "Cortex-A53",
            &[(Vendor::Arm, MicroArch::ArmCortexA53, Some("Cortex-A53"))],
        );
        assert!(!CpuDisplay::should_show_codename(&cpu_a53, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a53, true));

        // Codename "Maya" differs from core MicroArch "Cortex-A72" -> displayed at SoC level if distinct
        let cpu_a72 = make_test_cpu(
            "ARM Cortex-A72",
            "Maya",
            &[(Vendor::Arm, MicroArch::ArmCortexA72, None)],
        );
        assert!(CpuDisplay::should_show_codename(&cpu_a72, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a72, true));

        // Codename "Tahiti" differs from cores "Everest", "Sawtooth" -> displayed
        let cpu_a18 = make_test_cpu(
            "Apple A18 Pro",
            "Tahiti",
            &[
                (Vendor::Apple, MicroArch::AppleEverest, None),
                (Vendor::Apple, MicroArch::AppleSawtooth, None),
            ],
        );
        assert!(CpuDisplay::should_show_codename(&cpu_a18, false));
        assert!(CpuDisplay::should_show_codename(&cpu_a18, true));

        // Codename UNK -> always suppressed
        let cpu_unk = make_test_cpu(
            "ARM Cortex-A53",
            UNK,
            &[(Vendor::Arm, MicroArch::ArmCortexA53, Some("Cortex-A53"))],
        );
        assert!(!CpuDisplay::should_show_codename(&cpu_unk, false));
        assert!(!CpuDisplay::should_show_codename(&cpu_unk, true));
    }

    #[test]
    fn test_should_show_core_codename() {
        // When codename is "Enyo" and micro_arch is Cortex-A76 -> distinct, should show
        let core_a76 = CpuCore {
            implementer: Vendor::Arm,
            kind: CoreType::Performance,
            micro_arch: MicroArch::ArmCortexA76,
            code_name: Some("Enyo".to_string()),
            cache: None,
            count: 4,
        };
        assert!(CpuDisplay::should_show_core_codename(&core_a76, false));
        assert!(CpuDisplay::should_show_core_codename(&core_a76, true));

        // When codename is "Cortex-A53" and micro_arch is Cortex-A53 -> duplicate, should suppress unless verbose
        let core_a53 = CpuCore {
            implementer: Vendor::Arm,
            kind: CoreType::Efficiency,
            micro_arch: MicroArch::ArmCortexA53,
            code_name: Some("Cortex-A53".to_string()),
            cache: None,
            count: 4,
        };
        assert!(!CpuDisplay::should_show_core_codename(&core_a53, false));
        assert!(CpuDisplay::should_show_core_codename(&core_a53, true));

        // When codename is None -> suppressed
        let core_none = CpuCore {
            implementer: Vendor::Apple,
            kind: CoreType::Performance,
            micro_arch: MicroArch::AppleFirestorm,
            code_name: None,
            cache: None,
            count: 4,
        };
        assert!(!CpuDisplay::should_show_core_codename(&core_none, false));
        assert!(!CpuDisplay::should_show_core_codename(&core_none, true));

        // Semi-custom Qualcomm core: Kryo 485 Gold on Cortex-A76 -> distinct, should show
        let core_kryo = CpuCore {
            implementer: Vendor::Qualcomm,
            kind: CoreType::Performance,
            micro_arch: MicroArch::ArmCortexA76,
            code_name: Some("Kryo 485 Gold".to_string()),
            cache: None,
            count: 4,
        };
        assert!(CpuDisplay::should_show_core_codename(&core_kryo, false));
        assert!(CpuDisplay::should_show_core_codename(&core_kryo, true));
    }

    #[test]
    fn test_multi_implementer_cores() {
        // Tegra X2: Nvidia Denver 2 + ARM Cortex-A57
        let denver = CpuCore {
            implementer: Vendor::Nvidia,
            kind: CoreType::Performance,
            micro_arch: MicroArch::NvidiaDenver2,
            code_name: Some("Denver 2".to_string()),
            cache: None,
            count: 2,
        };
        let a57 = CpuCore {
            implementer: Vendor::Arm,
            kind: CoreType::Performance,
            micro_arch: MicroArch::ArmCortexA57,
            code_name: Some("Cortex-A57".to_string()),
            cache: None,
            count: 4,
        };

        assert_eq!(denver.implementer, Vendor::Nvidia);
        assert_eq!(a57.implementer, Vendor::Arm);
        assert_ne!(denver.implementer, a57.implementer);

        // Snapdragon 855: Qualcomm Kryo 485 Gold (Cortex-A76) + ARM Cortex-A55
        let gold = CpuCore {
            implementer: Vendor::Qualcomm,
            kind: CoreType::Performance,
            micro_arch: MicroArch::ArmCortexA76,
            code_name: Some("Kryo 485 Gold".to_string()),
            cache: None,
            count: 4,
        };
        let silver = CpuCore {
            implementer: Vendor::Arm,
            kind: CoreType::Efficiency,
            micro_arch: MicroArch::ArmCortexA55,
            code_name: Some("Cortex-A55".to_string()),
            cache: None,
            count: 4,
        };

        assert_eq!(gold.implementer, Vendor::Qualcomm);
        assert_eq!(silver.implementer, Vendor::Arm);
        assert!(CpuDisplay::should_show_core_codename(&gold, false));
        assert!(!CpuDisplay::should_show_core_codename(&silver, false));
    }

    #[test]
    fn test_display_single_cluster() {
        let cpu = make_test_cpu(
            "ARM Cortex-A72",
            "Maya",
            &[(Vendor::Arm, MicroArch::ArmCortexA72, Some("Maya"))],
        );
        let flags = CliFlags {
            color: false,
            compact: false,
            verbose: false,
        };
        CpuDisplay::display(&cpu, flags);
    }

    #[test]
    fn test_display_multi_implementer() {
        let cpu = make_test_cpu(
            "Tegra X2",
            "Parker",
            &[
                (Vendor::Nvidia, MicroArch::NvidiaDenver2, None),
                (Vendor::Arm, MicroArch::ArmCortexA57, Some("Cortex-A57")),
            ],
        );
        let flags = CliFlags {
            color: false,
            compact: false,
            verbose: false,
        };
        CpuDisplay::display(&cpu, flags);
    }

    #[test]
    fn test_cpu_debug_with_multi_implementer() {
        let mut midrs = HashSet::new();
        midrs.insert(Midr::new(0x4E000030)); // Nvidia Denver 2
        midrs.insert(Midr::new(0x410FD070)); // ARM Cortex-A57

        let cpu = Cpu {
            midrs,
            ..Default::default()
        };
        cpu.debug();
    }
}
