use super::cpu::Cpu;
use super::micro_arch::MicroArch;
use super::*;

use crate::common::{CliFlags, CpuDisplay, TCpuDisplay, UNK};
use crate::println;
use alloc::string::String;

impl TCpuDisplay for Cpu {
    fn debug(&self) {
        #[cfg(not(dos))]
        println!("{:#?}", self);

        #[cfg(all(dos, feature = "debug"))]
        {
            use super::is_cyrix;

            println!("{:?}", self);
            if is_cyrix() {
                println!("{:?}", super::vendor::Cyrix::detect());
            }
        }
    }

    fn display_table(&self, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        let ma: String = self.arch.micro_arch.into();
        let ma: &str = &ma;

        let multi_core = self.topology.cores > 1 || self.topology.sockets > 1;

        disp.simple_line("Architecture", FeatureClass::detect().to_str());

        // Vendor_string (brand_name)
        if self.arch.brand_name != UNK {
            println!(
                "{}{} ({})",
                disp.label("Vendor"),
                self.arch.vendor_string,
                self.arch.brand_name
            );

            CpuDisplay::newline();
        }

        // Hypervisor vendor_string (brand_name)
        #[cfg(not(dos))]
        if let Some(hyp_str) = &self.hyp_vendor_str {
            let hyp = HypervisorBrand::from(hyp_str.as_str());
            println!("{}{} ({})", disp.label("Hypervisor"), hyp_str, hyp.to_str());

            CpuDisplay::newline();
        }

        if self.signature.is_overdrive {
            disp.simple_line("Overdrive", "Yes");
        }

        if !self.has_cpuid {
            disp.simple_line("CPUID", "No");
        }

        let (raw_model, disp_model) = (Cpu::raw_model_string(), self.display_model_string());

        if disp_model != UNK {
            if raw_model.eq(UNK) {
                disp.simple_line("Model (synth)", &disp_model);
            } else if raw_model.trim().eq(&disp_model) {
                disp.simple_line("Model", &disp_model);
            } else {
                println!("{}{}", disp.label("Model"), &disp_model);

                if flags.verbose {
                    println!("{}{}", disp.label("Model (raw)"), &raw_model);
                }

                CpuDisplay::newline();
            }
        }

        if ma != UNK {
            disp.simple_line("MicroArch", ma);
        }

        if !(self.arch.code_name == "Unknown"
            || self.arch.code_name == ma
            || self.arch.micro_arch == MicroArch::I486)
        {
            disp.simple_line("Codename", self.arch.code_name);
        }

        // Process node
        if let Some(tech) = &self.arch.technology {
            disp.simple_line("Process Node", tech);
        }

        // Easter Egg (AMD K6, K8, Jaguar or Rise mp6)
        if let Some(easter_egg) = &self.easter_egg {
            disp.simple_line("Easter Egg", easter_egg);
        }

        // Sockets / Cores / Threads
        if self.cores.len() > 1 {
            println!(
                "{} {} cores ({} threads) across {} core types",
                disp.label("Topology"),
                self.topology.cores,
                self.topology.threads,
                self.cores.len()
            );

            for (i, ((_kind, _name), core)) in self.cores.iter().enumerate() {
                let core_label = alloc::format!("Core #{}", i + 1);
                println!("{}", disp.label(&core_label));
                println!("{}{}", disp.label("Count"), core.count);
                let type_str: String = core.kind.into();
                println!("{}{}", disp.label("Type"), type_str);
                if let Some(name) = &core.name {
                    println!("{}{}", disp.label("Codename"), name);
                }
                let cc = |s: u32| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, self.topology.sockets);
            }
        } else if multi_core {
            let lbl = disp.label("Topology");
            if self.topology.sockets > 1 {
                println!(
                    "{}{} sockets, {} cores, {} threads",
                    lbl, self.topology.sockets, self.topology.cores, self.topology.threads
                );
            } else if self.topology.cores != self.topology.threads {
                println!(
                    "{}{} cores ({} threads)",
                    lbl, self.topology.cores, self.topology.threads
                );
            } else {
                println!("{}{} cores", lbl, self.topology.cores);
            }

            CpuDisplay::newline();
        }

        // Cache
        if self.cores.len() <= 1 {
            let cache_count = |share_count: u32| -> String {
                #[allow(clippy::manual_checked_ops)]
                let count = if share_count == 0 {
                    self.topology.sockets
                } else {
                    self.topology.threads / share_count
                };

                if count < 2 {
                    String::new()
                } else {
                    alloc::format!("{}x ", count)
                }
            };

            disp.display_cache(self.topology.cache, &cache_count, self.topology.sockets);
        }

        // Clock Speed (Base/Boost)
        if self.topology.speed.base > 0 {
            let base = self.topology.speed.base;
            let boost = self.topology.speed.boost;

            if boost > base {
                println!(
                    "{}{}",
                    disp.inline_sublabel("Frequency", "Base"),
                    CpuDisplay::format_frequency(base)
                );
                println!(
                    "{}{}",
                    disp.sublabel("Boost"),
                    CpuDisplay::format_frequency(boost)
                );
            } else {
                println!(
                    "{}{}",
                    disp.label("Frequency"),
                    CpuDisplay::format_frequency(base)
                );
            }

            CpuDisplay::newline();
        }

        // CPU Signature
        if self.signature != CpuSignature::default() {
            let key = if self.signature.from_cpuid {
                "Signature"
            } else {
                "Synthetic Sig"
            };

            println!(
                "{}Family {:X}h, Model {:X}h, Stepping {:X}h",
                disp.label(key),
                self.signature.display_family,
                self.signature.display_model,
                self.signature.stepping
            );
            if flags.verbose {
                println!(
                    "{:>16}({:X}, {:X}, {:X}, {:X}, {:X})",
                    disp.sublabel("hex"),
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
                println!(
                    "{:>16}({}, {}, {}, {}, {})",
                    disp.sublabel("dec"),
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
            } else {
                println!(
                    "{:>16}({}, {}, {}, {}, {})",
                    "",
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
            }

            CpuDisplay::newline();
        }

        // CPU Features
        if !self.features.is_empty() {
            if self.features.len() == 1 {
                disp.simple_line(
                    "Features",
                    self.features
                        .get("Base")
                        .expect("There should be at least one key in the features BTreeMap."),
                );
            } else {
                let keys = [
                    "Base", "SSE", "AVX", "AVX512", "Security", "Math", "Other", "Centaur",
                ];
                for key in keys {
                    if self.features.contains_key(key) {
                        if key == "Base" {
                            println!(
                                "{}{}",
                                disp.inline_sublabel("Features", "Base"),
                                self.features.get(key).expect("Missing Base key?")
                            )
                        } else {
                            println!(
                                "{}{}",
                                disp.sublabel(key),
                                self.features.get(key).expect(
                                    "Somehow the key in the features BTreeMap disappeared!"
                                )
                            );
                        }
                    }
                }

                #[cfg(not(dos))]
                if is_centaur() {
                    use alloc::format;
                    use alloc::vec::Vec;

                    let centaur_map = super::vendor::Centaur::get_feature_list();
                    if !centaur_map.is_empty() {
                        let mut list: Vec<String> = Vec::new();
                        for (name, enabled) in &centaur_map {
                            if *enabled {
                                list.push(String::from(*name));
                            } else {
                                if flags.color {
                                    list.push(CpuDisplay::ansi_color(ANSI_BRIGHT_BLACK, name))
                                } else {
                                    list.push(format!("{name}(disabled)"));
                                }
                            }
                        }

                        if !list.is_empty() {
                            println!("{}{}", disp.sublabel("Centaur"), list.join(", "));
                        }
                    }
                }

                CpuDisplay::newline();
            }
        }

        #[cfg(target_arch = "x86")]
        if is_cyrix() {
            let cyrix = vendor::Cyrix::detect();

            if cyrix.dir0 != 0xFF {
                println!("{}Model number: {:X}h", disp.label("Cyrix"), cyrix.dir0);
                println!("{}{:X}h", disp.sublabel("Revision"), cyrix.revision);
                println!("{}{:X}h", disp.sublabel("Stepping"), cyrix.stepping);
                if !cyrix.multiplier.is_empty() && cyrix.multiplier != "0" {
                    println!("{}{}x", disp.sublabel("Bus Multiplier"), &cyrix.multiplier);
                }
                #[cfg(not(dos))]
                println!();
            }
        }
    }
}
