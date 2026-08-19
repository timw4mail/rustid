use super::cpu::Cpu;
use super::micro_arch::MicroArch;
use super::*;

#[cfg(not(dos))]
use super::cache::is_asymmetric_dual_ccd_x3d;
use crate::common::{CliFlags, CpuDisplay, DataSource, TCpuDisplay, UNK};
use crate::println;
use alloc::format;
#[cfg(not(dos))]
use alloc::string::String;

fn yes_no(b: bool) -> &'static str {
    if b { "Yes" } else { "No" }
}

#[cfg(not(dos))]
impl CpuDisplay {
    /// Computes the number of cache instances on x86 taking SMT / APIC ID allocation into account.
    pub fn x86_cache_instances(
        share_count: u32,
        core_count: u32,
        thread_count: u32,
        socket_count: u32,
    ) -> u32 {
        if share_count == 0 {
            socket_count.max(1)
        } else {
            let smt_width = cpuid_threads_per_core()
                .max(thread_count / core_count.max(1))
                .max(1);
            let cores_sharing = (share_count / smt_width).max(1);
            let count = core_count / cores_sharing;
            count.max(socket_count.max(1))
        }
    }

    /// Computes the cache count display prefix for x86 CPUs (e.g. "4x " or "").
    pub fn x86_cache_count(
        share_count: u32,
        core_count: u32,
        thread_count: u32,
        socket_count: u32,
    ) -> String {
        let count = Self::x86_cache_instances(share_count, core_count, thread_count, socket_count);
        if count < 2 {
            String::new()
        } else {
            format!("{}x ", count)
        }
    }
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
                println!("{}{}", disp.label("Model"), disp_model);

                if flags.verbose {
                    println!("{}{}", disp.label("Model (raw)"), raw_model);
                }

                disp.newline();
            }
        }
    }

    fn print_topology(&self, flags: CliFlags, disp: &CpuDisplay) {
        if !self.cores.is_empty() {
            println!(
                "{}{} cores ({} threads) across {} core types",
                disp.label("Cpu Topology"),
                self.topology.cores.count,
                self.topology.threads.count,
                self.cores.len()
            );
            disp.newline();

            for (i, core) in self.cores.iter().enumerate() {
                let core_label = format!("Core #{}", i + 1);
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

                let smt = cpuid_threads_per_core()
                    .max(core.threads / core.count.max(1))
                    .max(1);
                let cc = |s: u32| {
                    let cores_sharing = if s == 0 { 1 } else { (s / smt).max(1) };
                    CpuDisplay::cache_count(cores_sharing, core.count)
                };
                disp.display_cache(core.cache, &cc, self.topology.sockets.count);
            }

            return;
        }

        let multi_core = self.topology.cores.count > 1 || self.topology.sockets.count > 1;

        if multi_core || flags.verbose {
            let lbl = disp.label("Topology");
            let socket_str = if self.topology.sockets.count == 1 {
                "socket"
            } else {
                "sockets"
            };
            let core_str = if self.topology.cores.count == 1 {
                "core"
            } else {
                "cores"
            };
            let thread_str = if self.topology.threads.count == 1 {
                "thread"
            } else {
                "threads"
            };

            println!(
                "{}{} {}, {} {}, {} {}",
                lbl,
                self.topology.sockets.count,
                socket_str,
                self.topology.cores.count,
                core_str,
                self.topology.threads.count,
                thread_str
            );

            disp.newline();
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

            disp.newline();
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

            disp.newline();
        }
    }
}

// Cpu features display
impl Cpu {
    fn print_simple_features_list(&self, disp: &CpuDisplay) {
        disp.simple_line(
            "Features",
            self.features
                .get("Base")
                .expect("There should be at least one key in the features BTreeMap."),
        );
    }

    fn print_full_features_list(&self, disp: &CpuDisplay) {
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
                        self.features
                            .get(key)
                            .expect("Somehow the key in the features BTreeMap disappeared!")
                    );
                }
            }
        }
    }

    #[cfg(not(dos))]
    fn print_centaur_features(&self, flags: CliFlags, disp: &CpuDisplay) {
        use alloc::vec::Vec;

        let centaur_map = vendor::Centaur::get_feature_list();
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

    #[allow(unused_variables)]
    fn print_features(&self, flags: CliFlags, disp: &CpuDisplay) {
        if !self.features.is_empty() {
            // Simple features list
            if self.features.len() == 1 {
                self.print_simple_features_list(disp);
            } else {
                self.print_full_features_list(disp);
            }

            // Centaur features list
            #[cfg(not(dos))]
            if is_centaur() {
                self.print_centaur_features(flags, disp);
            }

            disp.newline();
        }
    }
}

impl TCpuDisplay for Cpu {
    fn debug(&self) {
        #[cfg(not(any(dos, dos32a)))]
        println!("{:#?}", self);

        #[cfg(dos32a)]
        {
            use super::is_cyrix;

            println!("{:?}", self);
            if is_cyrix() {
                println!("{:?}", super::vendor::Cyrix::detect());
            }
        }

        #[cfg(dos)]
        {
            use super::is_cyrix;

            println!("Cpu {{");
            println!("  has_cpuid: {}", self.has_cpuid);
            println!(
                "  arch: CpuArch {{ model: \"{}\", micro_arch: \"{}\", code_name: \"{}\", brand: \"{}\" }}",
                self.arch.model,
                self.arch.micro_arch.as_str(),
                self.arch.code_name,
                self.arch.brand_name
            );
            println!(
                "  signature: CpuSignature {{ family: {}, model: {}, stepping: {}, source: \"{}\" }}",
                self.signature.display_family,
                self.signature.display_model,
                self.signature.stepping,
                match self.signature.source {
                    DataSource::CpuReset => "CpuReset",
                    DataSource::CpuMsr => "CpuMsr",
                    DataSource::Cpuid => "Cpuid",
                    _ => "Other",
                }
            );
            println!(
                "  topology: Sockets={}, Cores={}, Threads={}, Speed={}MHz (measured={})",
                self.topology.sockets.count,
                self.topology.cores.count,
                self.topology.threads.count,
                self.topology.speed.base,
                self.topology.speed.measured
            );
            println!("}}");

            if is_cyrix() {
                let cyrix = super::vendor::Cyrix::detect();
                println!(
                    "Cyrix {{ dir0: {:02X}h, revision: {:02X}h, stepping: {:X}h, multiplier: \"{}\", model: \"{}\" }}",
                    cyrix.dir0,
                    cyrix.revision,
                    cyrix.stepping,
                    cyrix.multiplier,
                    cyrix.emodel.to_str()
                );
            }
        }
    }

    fn display_table(&self, flags: CliFlags) {
        let disp = CpuDisplay { flags };

        #[cfg(target_os = "uefi")]
        {
            let fw = crate::x86::efi::os::detect_firmware();
            let mut vendor = alloc::string::String::new();
            for &ch in &fw.vendor[..fw.vendor_len] {
                vendor.push(ch);
            }
            let major = (fw.revision >> 16) & 0xFFFF;
            let minor = fw.revision & 0xFFFF;
            let val = if vendor.is_empty() {
                format!("EFI {}.{:02}", major, minor)
            } else {
                format!("{} (EFI {}.{:02})", vendor, major, minor)
            };
            disp.simple_line("Firmware", &val);
        }

        #[cfg(any(not(nostd_os), target_os = "uefi"))]
        if let Some(system) = &self.system {
            disp.simple_line("System", &disp.format_system_name(system));
        }

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

            disp.newline();
        }

        // Hypervisor vendor_string (brand_name)
        #[cfg(not(dos))]
        if let Some(hyp_str) = &self.hyp_vendor_str {
            let hyp = HypervisorBrand::from(hyp_str.as_str());
            println!("{}{} ({})", disp.label("Hypervisor"), hyp_str, hyp.to_str());

            disp.newline();
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
        #[cfg(not(dos))]
        if self.cores.is_empty() {
            let cache_count = |share_count: u32| -> String {
                CpuDisplay::x86_cache_count(
                    share_count,
                    self.topology.cores.count,
                    self.topology.threads.count,
                    self.topology.sockets.count,
                )
            };

            if is_asymmetric_dual_ccd_x3d(&self.display_model_string(), self.topology.dies.count)
                && let Some(cache) = self.topology.cache
                && let Some(l3) = cache.l3
            {
                let x3d_mb = l3.size / (1024 * 1024);
                const NON_X3D_MB: u32 = 32;

                let override_str = if l3.assoc > 0 {
                    format!(
                        "{}MB {}-way (X3D) + {}MB {}-way",
                        x3d_mb, l3.assoc, NON_X3D_MB, l3.assoc
                    )
                } else {
                    format!("{}MB (X3D) + {}MB", x3d_mb, NON_X3D_MB)
                };

                disp.display_cache_ext(
                    self.topology.cache,
                    &cache_count,
                    self.topology.sockets.count,
                    Some(&override_str),
                );
            } else {
                disp.display_cache(
                    self.topology.cache,
                    &cache_count,
                    self.topology.sockets.count,
                );
            }
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
                #[cfg(not(any(dos, dos32a)))]
                println!();
            }
        }
    }
}
