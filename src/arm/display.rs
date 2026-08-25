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
            && is_duplicate(model, soc)
        {
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
                && cname != UNK
                && !cname.is_empty()
                && is_duplicate(model, cname)
            {
                return false;
            }
        }
        true
    }

    /// If all core types in `cpu_info` share the exact same distinct codename,
    /// returns that shared codename. Returns `None` if core types have different codenames
    /// or if only a subset of core types have a custom codename.
    pub fn shared_core_codename(cpu_info: &Cpu) -> Option<&str> {
        if cpu_info.cores.is_empty() {
            if cpu_info.cpu_arch.code_name != UNK && !cpu_info.cpu_arch.code_name.is_empty() {
                return Some(cpu_info.cpu_arch.code_name);
            }
            return None;
        }

        // Check if every core cluster in cpu_info has the same code_name
        let mut common_cname: Option<&str> = None;
        for (i, core) in cpu_info.cores.values().enumerate() {
            let Some(cname) = &core.code_name else {
                common_cname = None;
                break;
            };
            if cname == UNK || cname.is_empty() {
                common_cname = None;
                break;
            }
            let ma_str: String = core.micro_arch.into();
            if ma_str != UNK && is_duplicate(cname, &ma_str) {
                common_cname = None;
                break;
            }
            if i == 0 {
                common_cname = Some(cname.as_str());
            } else if common_cname != Some(cname.as_str()) {
                common_cname = None;
                break;
            }
        }

        if common_cname.is_some() {
            return common_cname;
        }

        // If no core has a distinct core codename, but top-level cpu_arch.code_name is set:
        if cpu_info.cpu_arch.code_name != UNK && !cpu_info.cpu_arch.code_name.is_empty() {
            return Some(cpu_info.cpu_arch.code_name);
        }

        None
    }

    pub fn should_show_codename(cpu_info: &Cpu, verbose: bool) -> Option<&str> {
        let code_name = if let Some(shared) = Self::shared_core_codename(cpu_info) {
            shared
        } else if cpu_info.cpu_arch.code_name != UNK && !cpu_info.cpu_arch.code_name.is_empty() {
            cpu_info.cpu_arch.code_name
        } else {
            return None;
        };

        if code_name == UNK || code_name.is_empty() {
            return None;
        }

        if verbose {
            return Some(code_name);
        }

        if is_duplicate(code_name, &cpu_info.cpu_arch.model) {
            return None;
        }

        if let Some(soc) = &cpu_info.soc_model
            && is_duplicate(code_name, soc)
        {
            return None;
        }

        // If this codename matches all core micro-architectures (e.g. Cortex-A53), suppress it
        let mut all_match_ma = !cpu_info.cores.is_empty();
        for core in cpu_info.cores.values() {
            let ma_str: String = core.micro_arch.into();
            if ma_str == UNK || !is_duplicate(code_name, &ma_str) {
                all_match_ma = false;
                break;
            }
        }
        if all_match_ma {
            return None;
        }

        Some(code_name)
    }

    pub fn should_show_core_micro_arch(micro_arch: MicroArch, _verbose: bool) -> bool {
        micro_arch != MicroArch::Unknown
    }

    pub fn should_show_core_codename(core: &CpuCore, cpu_info: &Cpu, verbose: bool) -> bool {
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
        // When all core types share the same codename, display it only in the CPU/SoC section, not with the cores
        if Self::shared_core_codename(cpu_info).is_some() {
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

        if let Some(codename) = Self::should_show_codename(cpu_info, flags.verbose) {
            disp.simple_line("Codename", codename);
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
                    disp.section_line("Implementer", vendor_str);
                }

                let name = Into::<&str>::into(core.kind);
                disp.section_line("Type", name);

                let ma_str: String = core.micro_arch.into();
                if Self::should_show_core_micro_arch(core.micro_arch, flags.verbose) {
                    disp.section_line("MicroArch", &ma_str);
                }

                if Self::should_show_core_codename(core, cpu_info, flags.verbose)
                    && let Some(codename) = &core.code_name
                {
                    disp.section_line("Codename", codename);
                }

                disp.section_line("Count", &core.count.to_string());

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
                disp.section_line("Implementer", vendor_str);
            }

            let ma_str: String = core.micro_arch.into();
            if Self::should_show_core_micro_arch(core.micro_arch, flags.verbose) {
                disp.section_line("MicroArch", &ma_str);
            }

            if Self::should_show_core_codename(core, cpu_info, flags.verbose)
                && let Some(codename) = &core.code_name
            {
                disp.section_line("Codename", codename);
            }

            disp.section_line("Count", &core.count.to_string());

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
        assert_eq!(CpuDisplay::should_show_codename(&cpu_a53, false), None);
        assert_eq!(
            CpuDisplay::should_show_codename(&cpu_a53, true),
            Some("Cortex-A53")
        );

        // Codename "Maya" differs from core MicroArch "Cortex-A72" -> displayed at SoC level
        let cpu_a72 = make_test_cpu(
            "ARM Cortex-A72",
            "Maya",
            &[(Vendor::Arm, MicroArch::ArmCortexA72, Some("Maya"))],
        );
        assert_eq!(
            CpuDisplay::should_show_codename(&cpu_a72, false),
            Some("Maya")
        );
        assert_eq!(
            CpuDisplay::should_show_codename(&cpu_a72, true),
            Some("Maya")
        );

        // Codename "Tahiti" differs from cores "Everest", "Sawtooth" -> displayed
        let cpu_a18 = make_test_cpu(
            "Apple A18 Pro",
            "Tahiti",
            &[
                (Vendor::Apple, MicroArch::AppleEverest, None),
                (Vendor::Apple, MicroArch::AppleSawtooth, None),
            ],
        );
        assert_eq!(
            CpuDisplay::should_show_codename(&cpu_a18, false),
            Some("Tahiti")
        );
        assert_eq!(
            CpuDisplay::should_show_codename(&cpu_a18, true),
            Some("Tahiti")
        );

        // Codename UNK -> always suppressed
        let cpu_unk = make_test_cpu(
            "ARM Cortex-A53",
            UNK,
            &[(Vendor::Arm, MicroArch::ArmCortexA53, Some("Cortex-A53"))],
        );
        assert_eq!(CpuDisplay::should_show_codename(&cpu_unk, false), None);
        assert_eq!(CpuDisplay::should_show_codename(&cpu_unk, true), None);
    }

    #[test]
    fn test_should_show_core_codename() {
        // Multi-cluster with DIFFERENT codenames: Kryo 485 Gold and Kryo 485 Silver
        let cpu_snapdragon = make_test_cpu(
            "Snapdragon 855",
            UNK,
            &[
                (
                    Vendor::Qualcomm,
                    MicroArch::ArmCortexA76,
                    Some("Kryo 485 Gold"),
                ),
                (
                    Vendor::Qualcomm,
                    MicroArch::ArmCortexA55,
                    Some("Kryo 485 Silver"),
                ),
            ],
        );
        let core_gold = cpu_snapdragon
            .cores
            .get(&(CoreType::Performance, Midr::new(0)))
            .expect("gold core missing");
        let core_silver = cpu_snapdragon
            .cores
            .get(&(CoreType::Efficiency, Midr::new(1)))
            .expect("silver core missing");

        // Since core types have different codenames, each core should show its own codename
        assert!(CpuDisplay::should_show_core_codename(
            core_gold,
            &cpu_snapdragon,
            false
        ));
        assert!(CpuDisplay::should_show_core_codename(
            core_silver,
            &cpu_snapdragon,
            false
        ));

        // Single cluster or all clusters sharing the same codename (e.g. Cortex-A72 "Maya")
        let cpu_a72 = make_test_cpu(
            "ARM Cortex-A72",
            "Maya",
            &[(Vendor::Arm, MicroArch::ArmCortexA72, Some("Maya"))],
        );
        let core_a72 = cpu_a72
            .cores
            .get(&(CoreType::Performance, Midr::new(0)))
            .expect("a72 core missing");

        // When all core types share the same codename, it's displayed only in the CPU/SoC section, NOT with the cores
        assert!(!CpuDisplay::should_show_core_codename(
            core_a72, &cpu_a72, false
        ));
        assert!(CpuDisplay::should_show_core_codename(
            core_a72, &cpu_a72, true
        ));

        // When codename is duplicate of micro_arch (e.g. Cortex-A53)
        let cpu_a53 = make_test_cpu(
            "ARM Cortex-A53",
            "Cortex-A53",
            &[(Vendor::Arm, MicroArch::ArmCortexA53, Some("Cortex-A53"))],
        );
        let core_a53 = cpu_a53
            .cores
            .get(&(CoreType::Efficiency, Midr::new(0)))
            .expect("a53 core missing");
        assert!(!CpuDisplay::should_show_core_codename(
            core_a53, &cpu_a53, false
        ));

        // When codename is None -> suppressed
        let cpu_apple = make_test_cpu(
            "Apple M1",
            "Tonga",
            &[(Vendor::Apple, MicroArch::AppleFirestorm, None)],
        );
        let core_apple = cpu_apple
            .cores
            .get(&(CoreType::Performance, Midr::new(0)))
            .expect("apple core missing");
        assert!(!CpuDisplay::should_show_core_codename(
            core_apple, &cpu_apple, false
        ));
        assert!(!CpuDisplay::should_show_core_codename(
            core_apple, &cpu_apple, true
        ));
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
        let cpu_snapdragon = make_test_cpu(
            "Snapdragon 855",
            UNK,
            &[
                (
                    Vendor::Qualcomm,
                    MicroArch::ArmCortexA76,
                    Some("Kryo 485 Gold"),
                ),
                (Vendor::Arm, MicroArch::ArmCortexA55, Some("Cortex-A55")),
            ],
        );
        let gold = cpu_snapdragon
            .cores
            .get(&(CoreType::Performance, Midr::new(0)))
            .expect("gold core missing");
        let silver = cpu_snapdragon
            .cores
            .get(&(CoreType::Efficiency, Midr::new(1)))
            .expect("silver core missing");

        assert_eq!(gold.implementer, Vendor::Qualcomm);
        assert_eq!(silver.implementer, Vendor::Arm);
        assert!(CpuDisplay::should_show_core_codename(
            gold,
            &cpu_snapdragon,
            false
        ));
        assert!(!CpuDisplay::should_show_core_codename(
            silver,
            &cpu_snapdragon,
            false
        ));
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
