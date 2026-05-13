//! CPU detection and information for x86/x86_64 processors.

use super::brand::CpuBrand;
use super::micro_arch::{CpuArch, MicroArch};
use super::topology::Topology;
use super::*;
use super::{EXT_LEAF_1, EXT_LEAF_2, EXT_LEAF_4, LEAF_1, read_multi_leaf_str, x86_cpuid};

use crate::common::{CpuDisplay, TCpu, UNK};
use crate::println;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// CPU feature class/level enumeration.
///
/// Represents the instruction set and feature level of an x86 processor,
/// roughly based on x86-64 microarchitecture levels.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FeatureClass {
    /// 80386-class processor
    i386,
    /// 80486-class processor
    i486,
    /// Pentium-class processor (i586)
    i586,
    /// Pentium Pro/II/III-class processor (i686)
    i686,
    /// i686 with SSE instruction
    i686_SSE,
    /// i686 with SSE2 instruction
    i686_SSE2,
    /// i686 with SSE3 instruction
    i686_SSE3,
    /// x86-64 version 1 (baseline SSE/SSE2)
    x86_64_v1,
    /// x86-64 version 2 (adds CMPXCHG16B, POPCNT, SSE4.2)
    x86_64_v2,
    /// x86-64 version 3 (adds AVX, AVX2, BMI, F16C, FMA)
    x86_64_v3,
    /// x86-64 version 4 (adds AVX-512)
    x86_64_v4,
}

impl FeatureClass {
    /// Cpu Feature Detection
    ///
    /// Roughly based on <https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels>
    pub fn detect() -> FeatureClass {
        use super::*;

        if has_avx512_f() {
            return FeatureClass::x86_64_v4;
        }

        if has_avx() && has_avx2() && has_bmi1() && has_bmi2() && has_f16c() && has_fma() {
            return FeatureClass::x86_64_v3;
        }

        if has_cx16() && has_popcnt() && has_sse3() && has_sse41() && has_sse42() && has_ssse3() {
            return FeatureClass::x86_64_v2;
        }

        if has_amd64() {
            return FeatureClass::x86_64_v1;
        }

        #[cfg(target_arch = "x86")]
        if is_cyrix() {
            return vendor::Cyrix::get_feature_class();
        }

        if has_sse3() {
            return FeatureClass::i686_SSE3;
        }

        if has_sse2() {
            return FeatureClass::i686_SSE2;
        }

        if has_sse() {
            return FeatureClass::i686_SSE;
        }

        if has_cmov() {
            return FeatureClass::i686;
        }

        if has_cx8() {
            return FeatureClass::i586;
        }

        if has_cpuid() && CpuSignature::detect().family == 4 {
            return FeatureClass::i486;
        }

        if is_486() {
            return FeatureClass::i486;
        }

        FeatureClass::i386
    }

    /// Returns a string representation of the feature class.
    pub fn to_str(self) -> &'static str {
        match self {
            FeatureClass::i386 => "i386",
            FeatureClass::i486 => "i486",
            FeatureClass::i586 => "i586",
            FeatureClass::i686 => "i686",
            FeatureClass::i686_SSE => "i686-SSE",
            FeatureClass::i686_SSE2 => "i686-SSE2",
            FeatureClass::i686_SSE3 => "i686-SSE3",
            FeatureClass::x86_64_v1 => "x86_64-v1",
            FeatureClass::x86_64_v2 => "x86_64-v2",
            FeatureClass::x86_64_v3 => "x86_64-v3",
            FeatureClass::x86_64_v4 => "x86_64-v4",
        }
    }
}

/// CPU signature containing family, model, and stepping information.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CpuSignature {
    /// Extended family value from CPUID
    pub extended_family: u32,
    /// Family value from CPUID
    pub family: u32,
    /// Extended model value from CPUID
    pub extended_model: u32,
    /// Model value from CPUID
    pub model: u32,
    /// Stepping value from CPUID
    pub stepping: u32,
    /// Display family (calculated from family and extended_family)
    pub display_family: u32,
    /// Display model (calculated from model and extended_model)
    pub display_model: u32,
    /// Is this an Intel Overdrive CPU?
    pub is_overdrive: bool,
    /// Is the signature detected from CPUID?
    pub from_cpuid: bool,
}

impl CpuSignature {
    pub fn new(
        extended_family: u32,
        family: u32,
        extended_model: u32,
        model: u32,
        stepping: u32,
        from_cpuid: bool,
    ) -> Self {
        let display_family = if family == 0xF {
            family + extended_family
        } else {
            family
        };

        let display_model = if family == 0x6 || family == 0xF {
            (extended_model << 4) + model
        } else {
            model
        };

        let is_overdrive = super::is_overdrive();

        Self {
            extended_model,
            extended_family,
            family,
            model,
            stepping,
            display_family,
            display_model,
            is_overdrive,
            from_cpuid,
        }
    }

    pub fn new_synth(family: u32, model: u32, stepping: u32) -> Self {
        Self::new(0, family, 0, model, stepping, false)
    }

    /// Detects the CPU signature from CPUID leaf 1.
    pub fn detect() -> Self {
        let from_cpuid = has_cpuid();

        #[cfg(all(target_arch = "x86", target_os = "none"))]
        if !from_cpuid {
            use super::vendor::cyrix::Cyrix;

            if super::is_cyrix() {
                if Cyrix::detect().dir0 > 0x13 {
                    let sig = Cyrix::get_signature_from_device_id();
                    if sig != CpuSignature::default() {
                        return sig;
                    }
                }
            }

            if let Some(reset_sig) = super::get_reset_signature() {
                return reset_sig;
            }
        }

        let res = x86_cpuid(LEAF_1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

        Self::new(
            extended_family,
            family,
            extended_model,
            model,
            stepping,
            from_cpuid,
        )
    }
}

/// Extended CPU signature information from AMD processors.
///
/// Contains additional CPU identification data available on AMD processors
/// via the extended CPUID leaf 0x80000001.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ExtendedSignature {
    pub base_brand_id: u32,
    pub brand_id: u32,
    pub pkg_type: u32,
}

impl ExtendedSignature {
    /// Detects the CPU signature from CPUID leaf 1.
    pub fn detect() -> Self {
        let res = x86_cpuid(EXT_LEAF_1);

        let brand_id = res.ebx & 0xFFFF;
        let pkg_type = (res.ebx >> 28) & 0xF;

        Self {
            base_brand_id: super::get_brand_id(),
            brand_id,
            pkg_type,
        }
    }
}

/// Represents a complete x86/x86_64 CPU with all detected information.
#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    /// CPU architecture and microarchitecture details
    pub arch: CpuArch,
    /// Easter egg string (hidden CPU info for some AMD/Rise processors)
    pub easter_egg: Option<String>,
    /// Model brand id
    pub brand_id: u32,
    /// CPU signature (family, model, stepping)
    pub signature: CpuSignature,
    /// AMD extended cpu signature
    pub ext_signature: Option<ExtendedSignature>,
    /// Detected CPU features
    pub features: BTreeMap<&'static str, String>,
    /// Speed, threads, cores, sockets
    pub topology: Topology,
}

impl Cpu {
    /// Gets the CPU model string.
    pub fn raw_model_string() -> String {
        read_multi_leaf_str(EXT_LEAF_2, EXT_LEAF_4)
    }

    fn intel_brand_index(&self) -> Option<String> {
        let brand_id = get_brand_id();

        const CELERON: &str = "Intel(R) Celeron(R) processor";
        const XEON: &str = "Intel(R) Xeon(R) processor";
        const XEON_MP: &str = "Intel(R) Xeon(R) processor MP";

        let (family, model, stepping) = (
            self.signature.family,
            self.signature.model,
            self.signature.stepping,
        );

        // If the family and model are greater than (0xF, 0x3),
        // (Prescott, or 64-bit), this table dos not apply
        if family == 15 && model >= 3 {
            return None;
        }

        let str = match brand_id {
            0x01 | 0x0A | 0x14 => CELERON,
            0x02 | 0x04 => "Intel(R) Pentium(R) III processor",
            0x03 => match (family, model, stepping) {
                (0x6, 0xB, 0x1) => CELERON,
                _ => "Intel(R) Pentium(R) III Xeon",
            },
            0x06 => "Mobile Intel(R) Pentium(R) III processor-M",
            0x07 | 0x0F | 0x13 | 0x17 => "Mobile Intel(R) Celeron(R) processor",
            0x08 | 0x09 => "Intel(R) Pentium(R) 4 processor",
            0x0B => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON_MP,
                _ => XEON,
            },
            0x0C => XEON_MP,
            0x0E => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON,
                _ => "Mobile Intel(R) Pentium(R) 4 processor-M",
            },
            0x11 | 0x15 => "Mobile Genuine Intel(R) processor",
            0x12 => "Intel(R) Celeron(R) M processor",
            0x16 => "Intel(R) Pentium(R) M processor",
            _ => UNK,
        };

        match str {
            UNK => None,
            _ => Some(String::from(str)),
        }
    }

    #[cfg(not(target_os = "none"))]
    fn cleanup_model_string(s: &str) -> String {
        let str = s.replace("CPU", "");

        // Remove excess whitespace
        let filtered: Vec<&str> = str
            .split_ascii_whitespace()
            .filter(|p| !p.is_empty())
            .collect();
        let str: String = filtered.join(" ").as_str().into();

        // Remove speed
        if let Some(idx) = str.find('@') {
            String::from(str[..idx].trim())
        } else {
            str
        }
    }

    /// Returns a human-readable display name for the CPU model.
    ///
    /// This attempts to produce a marketing-style name based on the
    /// detected CPU, falling back to architecture class names for
    /// older or unrecognized processors.
    pub fn display_model_string(&self) -> String {
        match CpuBrand::detect() {
            CpuBrand::AMD
                // The Geode NX is special
                if CpuBrand::detect() == CpuBrand::AMD
                    && self.signature.family == 6
                    && self.signature.model == 8
                    && self.signature.stepping == 1
                => {
                    return String::from("AMD Geode NX");
                }
            CpuBrand::Cyrix => {
                // Cyrix MSR model lookup is more accurate than the 'generic' way
                return vendor::Cyrix::model_string();
            }
            CpuBrand::Intel => {
                // Check the Intel model lookup table
                if let Some(model_name) = self.intel_brand_index() {
                    return model_name;
                }
            }
            CpuBrand::SiS => return String::from("SiS 550/551/552 SoC"),
            CpuBrand::Unknown => 'nocpuid: {
                // Not a 386 or 486
                if self.arch.model != UNK || self.signature.family > 4 {
                    break 'nocpuid;
                }

                // 486s without cpuid
                let s = if is_386() {
                    "'Classic' 386"
                } else {
                    match (self.signature.family, self.signature.model) {
                        (4, 2) => "'Classic' 486 SX",
                        (4, 3) => "'Classic' 486 DX2",
                        (4, 4) => "Intel 486SL",
                        (4, 5) => "'Classic' 486 SX2",
                        _ => "'Classic' 486",
                    }
                };

                return String::from(s);
            }
            _ => (),
        }

        let s = match self.arch.micro_arch {
            // AMD
            MicroArch::Am486 => match self.arch.code_name {
                "Am486DX" => "AMD 486 DX",
                "Am486DX-40" => "AMD 486 DX-40",
                "Am486SX" => "AMD 486 SX",
                "Am486DX2" => "AMD 486 DX2",
                "Am486X2WB" => "AMD 486 DX2 with Write-Back Cache",
                "Am486DX4" => "AMD 486 DX4",
                "Am486DX4WB" => "AMD 486 DX4 with Write-Back Cache",
                _ => "'Classic' 486",
            },
            MicroArch::Am5x86 => match self.arch.code_name {
                "Am5x86WB" => "AMD 5x86 with Write-Back Cache",
                _ => "AMD 5x86",
            },
            MicroArch::SSA5 => "AMD K5",

            // Centaur
            MicroArch::Winchip => "IDT Winchip",
            MicroArch::Winchip2 => "IDT Winchip 2",
            MicroArch::Winchip2A => "IDT Winchip 2A",
            MicroArch::Winchip2B => "IDT Winchip 2B",
            MicroArch::Winchip3 => "IDT Winchip 3",
            MicroArch::Samuel
            | MicroArch::Samuel2
            | MicroArch::Ezra
            | MicroArch::EzraT
            | MicroArch::Nehemiah => "VIA C3",
            MicroArch::Esther => "VIA C7",
            MicroArch::Isaiah => {
                if self.arch.model.contains("Eden") {
                    &self.arch.model.replace("Eden", "Nano")
                } else {
                    &self.arch.model
                }
            }

            //Intel
            MicroArch::RapidCad => "Intel RapidCAD",
            MicroArch::I486 => match self.arch.code_name {
                "i80486DX" => "Intel 486 DX",
                "i80486DX-50" => "Intel 486 DX-50",
                "i80486SX" => "Intel 486 SX",
                "i80486DX2" => "Intel 486 DX2",
                "i80486SL" => "Intel 486 SL",
                "i80486SX2" => "Intel 486 SX2",
                "i80486DX2WB" => "Intel 486 DX2 with Write-Back Cache",
                "i80486DX4" => "Intel 486 DX4",
                "i80486DX4WB" => "Intel 486 DX4 with Write-Back Cache",
                _ => "'Classic' 486",
            },
            MicroArch::P5 => {
                if has_mmx() {
                    "Intel Pentium with MMX"
                } else {
                    match self.arch.code_name {
                        "P24T" => "Intel Pentium Overdrive",
                        _ => "Intel Pentium",
                    }
                }
            }
            MicroArch::PentiumPro => "Intel Pentium Pro",
            MicroArch::PentiumII => "Intel Pentium II",
            MicroArch::PentiumIII => "Intel Pentium III",

            // Rise
            MicroArch::MP6 => match self.arch.code_name {
                "Lynx" => "Rise iDragon",
                _ => "Rise mP6",
            },

            // UMC
            MicroArch::U5S => "UMC Green CPU U5S (486 SX)",
            MicroArch::U5D => "UMC Green CPU U5D (486 DX)",

            // Make sure to return the original model string if there are no overrides
            _ => {
                if self.arch.model != UNK {
                    &self.arch.model
                } else {
                    UNK
                }
            }
        };

        #[cfg(not(target_os = "none"))]
        return Self::cleanup_model_string(s);

        #[cfg(target_os = "none")]
        String::from(s)
    }

    fn easter_egg() -> Option<String> {
        let mut out: String = String::new();
        let brand = CpuBrand::detect();

        let addr = match brand {
            CpuBrand::AMD => AMD_EASTER_EGG_ADDR,
            CpuBrand::Rise | CpuBrand::SiS | CpuBrand::DMP | CpuBrand::Rdc => RISE_EASTER_EGG_ADDR,

            _ => 1,
        };

        if addr != 1 {
            let res = x86_cpuid(addr);

            let reg_list = match brand {
                // Surely there had to be a reason for this silly ordering?
                CpuBrand::Rise | CpuBrand::SiS => [res.ebx, res.edx, res.ecx, res.eax],

                _ => [res.eax, res.ebx, res.ecx, res.edx],
            };

            for &reg in &reg_list {
                let bytes = reg.to_le_bytes();
                for &b in &bytes {
                    if b != 0 {
                        out.push(b as char);
                    }
                }
            }
        }

        let trimmed = out.trim();
        if !trimmed.is_empty() {
            Some(String::from(trimmed))
        } else {
            None
        }
    }
}

impl TCpu for Cpu {
    /// Detects and returns comprehensive CPU information.
    ///
    /// Performs full CPU detection including architecture, microarchitecture,
    /// brand string, signature, features, and topology.
    fn detect() -> Self {
        let sig = CpuSignature::detect();
        Self {
            arch: CpuArch::find(&Self::raw_model_string(), sig, &vendor_str()),
            easter_egg: Self::easter_egg(),
            brand_id: get_brand_id(),
            signature: sig,
            ext_signature: match is_amd() {
                true => Some(ExtendedSignature::detect()),
                false => None,
            },
            features: get_feature_list(),
            topology: Topology::detect(),
        }
    }

    fn debug(&self) {
        #[cfg(not(target_os = "none"))]
        println!("{:#?}", self);

        #[cfg(all(target_os = "none", feature = "debug"))]
        {
            use super::is_cyrix;

            println!("{:?}", self);
            if is_cyrix() {
                println!("{:?}", super::vendor::Cyrix::detect());
            }
        }
    }

    fn display_table(&self, color: bool) {
        let disp = CpuDisplay { color };

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

        if is_hypervisor_guest() && max_hypervisor_leaf() > 0 {
            let hyp = HypervisorBrand::detect();
            println!(
                "{}{} ({})",
                disp.label("Hypervisor"),
                hypervisor_str(),
                hyp.to_str()
            );

            CpuDisplay::newline();
        }

        if self.signature.is_overdrive {
            disp.simple_line("Overdrive", "Yes");
        }

        let (raw_model, disp_model) = (Cpu::raw_model_string(), self.display_model_string());

        if disp_model != UNK {
            if raw_model.eq(UNK) {
                disp.simple_line("Model (synth)", &disp_model);
            } else if raw_model.trim().eq(&disp_model) {
                disp.simple_line("Model", &disp_model);
            } else {
                println!("{}{}", disp.label("Model"), &disp_model);
                println!("{}{}", disp.label("Model (raw)"), &raw_model);

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
        if multi_core {
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
            println!(
                "{:>16}({}, {}, {}, {}, {})",
                "",
                self.signature.extended_family,
                self.signature.family,
                self.signature.extended_model,
                self.signature.model,
                self.signature.stepping
            );
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
                        println!(
                            "{}{}",
                            disp.sublabel(key),
                            self.features
                                .get(key)
                                .expect("Somehow the key in the features BTreeMap disappeared!")
                        );
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
                #[cfg(not(target_os = "none"))]
                println!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpuid::get_feature_list;

    #[test]
    fn test_model_string() {
        let model = Cpu::raw_model_string();
        assert!(!model.is_empty());
    }

    #[test]
    fn test_cpu_features_detect() {
        let features = get_feature_list();
        // Assert that at least some features are detected (this might vary by CPU)
        assert!(!features.is_empty());
    }

    #[test]
    fn test_cpu_new() {
        let cpu = Cpu::detect();
        // Ensure that new() doesn't panic and populates some fields
        assert!(!cpu.arch.vendor_string.is_empty());
        assert!(!cpu.features.is_empty());
    }

    #[test]
    fn test_display_model_string_x32() {
        // Test case for MicroArch::Am486
        let mut arch_am486 = CpuArch {
            micro_arch: MicroArch::Am486,
            code_name: "Am486DX2",
            ..Default::default()
        };

        let cpu_am486_dx2 = Cpu {
            arch: arch_am486.clone(),
            brand_id: 0,
            easter_egg: None,
            signature: CpuSignature::detect(), // Signature doesn't affect this path
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_am486_dx2.display_model_string(), "AMD 486 DX2");

        arch_am486.code_name = "Am486X2WB";
        let cpu_am486_x2wb = Cpu {
            arch: arch_am486.clone(),
            brand_id: 0,
            easter_egg: None,
            signature: CpuSignature::detect(),
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(
            cpu_am486_x2wb.display_model_string(),
            "AMD 486 DX2 with Write-Back Cache"
        );

        // Test case for MicroArch::I486
        let cpu_i486_dx = Cpu {
            arch: CpuArch {
                micro_arch: MicroArch::I486,
                code_name: "i80486DX",
                ..Default::default()
            },
            brand_id: 0,
            easter_egg: None,
            signature: CpuSignature::detect(),
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_i486_dx.display_model_string(), "Intel 486 DX");

        // Test case for "No CPUID"
        let cpu_no_cpuid = Cpu {
            arch: CpuArch::default(),
            brand_id: 0,
            easter_egg: None,
            signature: CpuSignature {
                extended_family: 0,
                family: 0,
                extended_model: 0,
                model: 0,
                stepping: 0,
                display_family: 0,
                display_model: 0,
                is_overdrive: false,
                from_cpuid: false,
            },
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_no_cpuid.display_model_string(), UNK);
    }

    #[test]
    fn test_display_model_string() {
        // Test case for "Unknown"
        let cpu_unknown = Cpu {
            arch: CpuArch::default(),
            brand_id: 0,
            easter_egg: None,
            signature: CpuSignature {
                extended_family: 1, // Make it not all zeros
                family: 1,
                extended_model: 1,
                model: 1,
                stepping: 1,
                display_family: 1,
                display_model: 1,
                is_overdrive: false,
                from_cpuid: false,
            },
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_unknown.display_model_string(), "Unknown");
    }
}
