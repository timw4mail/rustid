use super::cpu::Cpu;
use super::micro_arch::MicroArch;
use super::*;

use crate::common::{CliFlags, CpuDisplay, DataSource, TCpuDisplay, UNK};
use crate::println;
use alloc::string::String;

fn yes_no(b: bool) -> &'static str {
    if b { "Yes" } else { "No" }
}

// Formatting/display helpers
impl Cpu {
    fn print_misc_flags(&self, flags: CliFlags, disp: &CpuDisplay) {
        let overdrive = self.signature.is_overdrive;
        let cpuid = self.has_cpuid;

        if flags.verbose {
            disp.simple_line("CPUID", yes_no(cpuid));
            disp.simple_line("Overdrive", yes_no(overdrive));
        } else {
            if !cpuid {
                disp.simple_line("CPUID", "No");
            }
            if overdrive {
                disp.simple_line("Overdrive", "Yes");
            }
        }
    }

    fn print_model(&self, flags: CliFlags, disp: &CpuDisplay) {
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
    }

    fn print_topology(&self, flags: CliFlags, disp: &CpuDisplay) {
        if !self.cores.is_empty() {
            println!(
                "{}{} cores ({} threads) across {} core types",
                disp.label("Cpu Topology"),
                self.topology.cores,
                self.topology.threads,
                self.cores.len()
            );
            CpuDisplay::newline();

            for (i, core) in self.cores.iter().enumerate() {
                let core_label = alloc::format!("Core #{}", i + 1);
                println!("{}", disp.label(&core_label));

                let type_str: &str = core.kind.into();
                println!("{}{}", disp.label("Type"), type_str);

                if let Some(name) = &core.name {
                    println!("{}{}", disp.label("Codename"), name);
                }

                if core.count != core.threads {
                    println!(
                        "{}{} cores ({} threads)",
                        disp.label("Topology"),
                        core.count,
                        core.threads
                    );
                } else {
                    println!("{}{} cores", disp.label("Topology"), core.count);
                }

                let cc = |s: u32| CpuDisplay::cache_count(s, core.count);
                disp.display_cache(core.cache, &cc, self.topology.sockets);
            }

            return;
        }

        let multi_core = self.topology.cores > 1 || self.topology.sockets > 1;

        if multi_core || flags.verbose {
            let lbl = disp.label("Topology");
            if self.topology.sockets > 1 || flags.verbose {
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
    }

    fn print_speed(&self, disp: &CpuDisplay) {
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
    }
    fn print_signature(&self, flags: CliFlags, disp: &CpuDisplay) {
        if self.signature != CpuSignature::default() {
            let key = if self.signature.source == DataSource::Cpuid
                || self.signature.source == DataSource::CpuidDump
            {
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
    }

    fn print_features(&self, flags: CliFlags, disp: &CpuDisplay) {
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
    }
}

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

        let ma = self.arch.micro_arch.as_str();

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

        // Cpu model string
        self.print_model(flags, &disp);

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

        // Overdrive, CPUID support, etc
        self.print_misc_flags(flags, &disp);

        // Sockets / Cores / Threads
        self.print_topology(flags, &disp);

        // Cache
        if self.cores.is_empty() {
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
        self.print_speed(&disp);

        // CPU Signature
        self.print_signature(flags, &disp);

        // CPU Features
        self.print_features(flags, &disp);

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
