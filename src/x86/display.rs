use super::cpu::Cpu;
use super::micro_arch::MicroArch;
use super::*;

#[cfg(not(dos_real))]
use super::cache::is_asymmetric_dual_ccd_x3d;
use crate::common::{CliFlags, CpuDisplay, DataSource, TCpuDisplay, UNK};
use crate::format;
#[cfg(not(dos_real))]
use alloc::string::String;

#[cfg(not(dos_real))]
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
            alloc::format!("{}x ", count)
        }
    }
}

// Formatting/display helpers
impl Cpu {
    fn print_misc_flags(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        let overdrive = self.signature.is_overdrive;
        let cpuid = self.has_cpuid;

        if flags.verbose {
            disp.simple_line("CPUID", CpuDisplay::yes_no(cpuid));
            disp.simple_line("Overdrive", CpuDisplay::yes_no(overdrive));
        } else {
            if !cpuid {
                disp.simple_line("CPUID", "No");
            }
            if overdrive {
                disp.simple_line("Overdrive", "Yes");
            }
        }
    }

    fn print_model(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        let (raw_model, disp_model) = (Cpu::raw_model_string(), self.display_model_string());

        if disp_model != UNK {
            if raw_model.eq(UNK) {
                disp.simple_line("Model (synth)", &disp_model);
            } else {
                disp.display_with_raw("Model", &disp_model, Some(&raw_model), flags.verbose);
            }
        }
    }

    fn print_topology(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        if self.is_hybrid() {
            disp.display_topology_line(
                self.topology.sockets.count,
                self.topology.cores.count,
                self.topology.threads.count,
                true,
                self.cores.len(),
            );

            for (i, core) in self.cores.iter().enumerate() {
                disp.core_heading(i);

                let type_str: &str = core.kind.into();
                disp.section_line("Type", type_str);

                disp.section_line_opt("Codename", core.name.as_deref());

                disp.section_line(
                    "Topology",
                    &CpuDisplay::format_core_threads(core.count, core.threads),
                );

                disp.display_frequency(
                    core.speed,
                    CliFlags {
                        compact: true,
                        ..flags
                    },
                );

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

        let multi_core = self.topology.cores.count > 1
            || self.topology.threads.count > 1
            || self.topology.sockets.count > 1;

        if multi_core || flags.verbose {
            disp.display_topology_line(
                self.topology.sockets.count,
                self.topology.cores.count,
                self.topology.threads.count,
                false,
                1,
            );
        }
    }

    fn print_speed(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        let speed = self
            .cores
            .first()
            .and_then(|c| c.speed)
            .unwrap_or(self.topology.speed);

        disp.display_frequency(Some(speed), flags);
    }

    fn print_signature(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        if self.signature != CpuSignature::default() {
            let key = if self.signature.source == DataSource::Cpuid
                || self.signature.source == DataSource::CpuidDump
            {
                "Signature"
            } else {
                "Synthetic Sig"
            };

            let l1 = format!(
                "{}Family {:X}h, Model {:X}h, Stepping {:X}h",
                disp.label(key),
                self.signature.display_family,
                self.signature.display_model,
                self.signature.stepping
            );
            disp.print_line(&l1);
            if flags.verbose {
                let l2 = format!(
                    "{:>16}({:X}, {:X}, {:X}, {:X}, {:X})",
                    disp.sublabel("hex"),
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
                disp.print_line(&l2);
                let l3 = format!(
                    "{:>16}({}, {}, {}, {}, {})",
                    disp.sublabel("dec"),
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
                disp.print_line(&l3);
            } else {
                let l2 = format!(
                    "{:>16}({}, {}, {}, {}, {})",
                    "",
                    self.signature.extended_family,
                    self.signature.family,
                    self.signature.extended_model,
                    self.signature.model,
                    self.signature.stepping
                );
                disp.print_line(&l2);
            }

            disp.newline();
        }
    }

    #[cfg(not(dos_real))]
    fn format_centaur_features(&self, flags: CliFlags) -> Option<String> {
        use alloc::vec::Vec;

        let centaur_map = vendor::Centaur::get_feature_list();
        if !centaur_map.is_empty() {
            let mut list: Vec<String> = Vec::new();
            for (name, enabled) in &centaur_map {
                if *enabled {
                    list.push(String::from(*name));
                } else {
                    if flags.color {
                        list.push(CpuDisplay::ansi_color(ANSI_BRIGHT_BLACK, name));
                    } else {
                        list.push(alloc::format!("{name}(disabled)"));
                    }
                }
            }

            if !list.is_empty() {
                return Some(list.join(", "));
            }
        }
        None
    }

    #[allow(unused_variables)]
    fn print_features(&self, flags: CliFlags, disp: &mut CpuDisplay) {
        #[allow(unused_mut)]
        let mut features = self.features.clone();

        #[cfg(not(dos_real))]
        if is_centaur()
            && let Some(centaur_str) = self.format_centaur_features(flags)
        {
            features.insert("Centaur", centaur_str);
        }

        if !features.is_empty() {
            let keys = [
                "Base", "SSE", "AVX", "AVX512", "Security", "Math", "Other", "Centaur", "Cyrix",
            ];
            disp.display_features(&features, &keys);
        }
    }
}

impl TCpuDisplay for Cpu {
    fn render_debug(&self) -> String {
        #[cfg(not(dos_os))]
        {
            #[allow(unused_mut)]
            let mut out = alloc::format!("{:#?}", self);
            #[cfg(target_arch = "x86")]
            if is_cyrix() {
                out.push_str(&alloc::format!("\n\n{:#?}", super::vendor::Cyrix::detect()));
            }
            out
        }

        #[cfg(dos_ext)]
        {
            use super::is_cyrix;

            let mut out = alloc::format!("{:?}", self);
            if is_cyrix() {
                out.push_str(&alloc::format!("\n{:?}", super::vendor::Cyrix::detect()));
            }
            out
        }

        #[cfg(dos_real)]
        {
            use super::is_cyrix;

            let mut out = String::new();
            out.push_str("Cpu {\n");
            out.push_str(&alloc::format!("  has_cpuid: {}\n", self.has_cpuid));
            out.push_str(&alloc::format!(
                "  arch: CpuArch {{ model: \"{}\", micro_arch: \"{}\", code_name: \"{}\", brand: \"{}\" }}\n",
                self.arch.model,
                self.arch.micro_arch.as_str(),
                self.arch.code_name,
                self.arch.brand_name
            ));
            out.push_str(&alloc::format!(
                "  signature: CpuSignature {{ family: {}, model: {}, stepping: {}, source: \"{}\" }}\n",
                self.signature.display_family,
                self.signature.display_model,
                self.signature.stepping,
                match self.signature.source {
                    DataSource::CpuReset => "CpuReset",
                    DataSource::CpuMsr => "CpuMsr",
                    DataSource::Cpuid => "Cpuid",
                    _ => "Other",
                }
            ));
            out.push_str(&alloc::format!(
                "  topology: Sockets={}, Cores={}, Threads={}, Speed={}MHz (measured={})\n",
                self.topology.sockets.count,
                self.topology.cores.count,
                self.topology.threads.count,
                self.topology.speed.base,
                self.topology.speed.measured
            ));
            out.push_str("}\n");

            if is_cyrix() {
                let cyrix = super::vendor::Cyrix::detect();
                out.push_str(&alloc::format!(
                    "Cyrix {{ dir0: {:02X}h, revision: {:02X}h, stepping: {:X}h, multiplier: \"{}\", model: \"{}\" }}\n",
                    cyrix.dir0,
                    cyrix.revision,
                    cyrix.stepping,
                    cyrix.multiplier,
                    cyrix.emodel.to_str()
                ));
            }
            out
        }
    }

    fn display_table_with_disp(&self, disp: &mut CpuDisplay) {
        let flags = disp.flags;

        #[cfg(uefi)]
        {
            let fw = crate::x86::efi::os::detect_firmware();
            let mut vendor = alloc::string::String::new();
            for &ch in &fw.vendor[..fw.vendor_len] {
                vendor.push(ch);
            }
            let major = (fw.revision >> 16) & 0xFFFF;
            let minor = fw.revision & 0xFFFF;
            let val = if vendor.is_empty() {
                alloc::format!("EFI {}.{:02}", major, minor)
            } else {
                alloc::format!("{} (EFI {}.{:02})", vendor, major, minor)
            };
            disp.simple_line("Firmware", &val);
        }

        #[cfg(not(dos_os))]
        if let Some(system) = &self.system {
            disp.display_system(system, flags);
        }

        let ma = self.arch.micro_arch.as_str();

        disp.simple_line("Architecture", FeatureClass::detect().to_str());

        // Vendor_string (brand_name)
        if self.arch.brand_name != UNK {
            disp.simple_line_with_detail("Vendor", &self.arch.vendor_string, self.arch.brand_name);
        }

        // Hypervisor vendor_string (brand_name)
        #[cfg(not(dos_real))]
        if let Some(hyp_str) = &self.hyp_vendor_str {
            let hyp = HypervisorBrand::from(hyp_str.as_str());
            disp.simple_line_with_detail("Hypervisor", hyp_str, hyp.to_str());
        }

        // Cpu model string
        self.print_model(flags, disp);

        disp.simple_line_if_known("MicroArch", ma);

        if !(self.arch.code_name == "Unknown"
            || self.arch.code_name == ma
            || self.arch.micro_arch == MicroArch::I486)
        {
            disp.simple_line("Codename", self.arch.code_name);
        }

        // Process node
        disp.simple_line_opt("Process Node", self.arch.technology);

        // Easter Egg (AMD K6, K8, Jaguar or Rise mp6)
        disp.simple_line_opt("Easter Egg", self.easter_egg.as_deref());

        // Overdrive, CPUID support, etc
        self.print_misc_flags(flags, disp);

        // Sockets / Cores / Threads
        self.print_topology(flags, disp);

        // Cache
        #[cfg(not(dos_real))]
        if !self.is_hybrid() {
            let cache_count = |share_count: u32| -> String {
                CpuDisplay::x86_cache_count(
                    share_count,
                    self.topology.cores.count,
                    self.topology.threads.count,
                    self.topology.sockets.count,
                )
            };

            let cache_opt = self
                .cores
                .first()
                .and_then(|c| c.cache)
                .or(self.topology.cache);

            if is_asymmetric_dual_ccd_x3d(&self.display_model_string(), self.topology.dies.count)
                && let Some(cache) = cache_opt
                && let Some(l3) = cache.l3
            {
                let x3d_mb = l3.size / (1024 * 1024);
                const NON_X3D_MB: u32 = 32;

                let override_str = if l3.assoc > 0 {
                    alloc::format!(
                        "{}MB {}-way (X3D) + {}MB {}-way",
                        x3d_mb,
                        l3.assoc,
                        NON_X3D_MB,
                        l3.assoc
                    )
                } else {
                    alloc::format!("{}MB (X3D) + {}MB", x3d_mb, NON_X3D_MB)
                };

                disp.display_cache_ext(
                    cache_opt,
                    &cache_count,
                    self.topology.sockets.count,
                    Some(&override_str),
                );
            } else {
                disp.display_cache(cache_opt, &cache_count, self.topology.sockets.count);
            }
        }

        // Clock Speed (Base/Boost)
        if !self.is_hybrid() {
            self.print_speed(flags, disp);
        }

        // CPU Signature
        self.print_signature(flags, disp);

        // CPU Features
        self.print_features(flags, disp);

        #[cfg(target_arch = "x86")]
        if is_cyrix() {
            let cyrix = vendor::Cyrix::detect();

            if cyrix.dir0 != 0xFF {
                let l1 = format!(
                    "{}{:X}h",
                    disp.inline_sublabel("Cyrix", "Model number"),
                    cyrix.dir0
                );
                disp.print_line(&l1);
                let l2 = format!("{}{:X}h", disp.sublabel("Revision"), cyrix.revision);
                disp.print_line(&l2);
                let l3 = format!("{}{:X}h", disp.sublabel("Stepping"), cyrix.stepping);
                disp.print_line(&l3);
                if !cyrix.multiplier.is_empty() && cyrix.multiplier != "0" {
                    let l4 = format!("{}{}x", disp.sublabel("Bus Multiplier"), &cyrix.multiplier);
                    disp.print_line(&l4);
                }
                disp.newline();
            }
        }
    }
}
