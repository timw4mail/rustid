//! CPU detection and information for x86/x86_64 processors.

#[allow(unused_imports)]
use super::brand::{CpuBrand, VENDOR_AMD, VENDOR_CYRIX, VENDOR_INTEL};
use super::micro_arch::{CpuArch, MicroArch};
use super::topology::{Level1Cache, Topology};
use super::{EXT_LEAF_1, EXT_LEAF_2, EXT_LEAF_4, FeatureList, UNK, x86_cpuid};

use crate::println;

use core::str::FromStr;
use heapless::String;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FeatureClass {
    i386,
    i486,
    i586,
    i686,
    x86_64_v1,
    x86_64_v2,
    x86_64_v3,
    x86_64_v4,
}

impl FeatureClass {
    /// Rough detection of cpu feature class
    ///
    /// Roughly based on https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels
    pub fn detect() -> FeatureClass {
        use super::*;

        #[cfg(target_arch = "x86_64")]
        {
            if has_avx512f() {
                return FeatureClass::x86_64_v4;
            }

            if has_avx() && has_avx2() && has_bmi1() && has_bmi2() && has_f16c() && has_fma() {
                return FeatureClass::x86_64_v3;
            }

            if has_cx16() && has_popcnt() && has_sse3() && has_sse41() && has_sse42() && has_ssse3()
            {
                return FeatureClass::x86_64_v2;
            }

            FeatureClass::x86_64_v1
        }

        #[cfg(target_arch = "x86")]
        {
            if has_cmov() {
                return FeatureClass::i686;
            }

            if CpuSignature::detect().family >= 5 {
                return FeatureClass::i586;
            }

            if is_486() || (is_cpuid_486() && CpuSignature::detect().family == 4) {
                return FeatureClass::i486;
            }

            FeatureClass::i386
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            FeatureClass::i386 => "i386",
            FeatureClass::i486 => "i486",
            FeatureClass::i586 => "i586",
            FeatureClass::i686 => "i686",
            FeatureClass::x86_64_v1 => "x86_64_v1",
            FeatureClass::x86_64_v2 => "x86_64_v2",
            FeatureClass::x86_64_v3 => "x86_64_v3",
            FeatureClass::x86_64_v4 => "x86_64_v4",
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
}

impl CpuSignature {
    /// Detects the CPU signature from CPUID leaf 1.
    pub fn detect() -> Self {
        let res = x86_cpuid(1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

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
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug)]
pub struct Cpu {
    /// CPU architecture and microarchitecture details
    pub arch: CpuArch,
    /// Easter egg string (hidden CPU info for some AMD/Rise processors)
    pub easter_egg: Option<String<64>>,
    /// Model brand id
    pub brand_id: u32,
    /// Number of logical processors/threads
    pub threads: u32,
    /// CPU signature (family, model, stepping)
    pub signature: CpuSignature,
    /// AMD extended cpu signature
    #[cfg(target_arch = "x86_64")]
    pub ext_signature: Option<ExtendedSignature>,
    /// Detected CPU features
    pub features: FeatureList,
    pub topology: Topology,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            arch: CpuArch::find(
                Self::raw_model_string().as_str(),
                CpuSignature::detect(),
                &super::vendor_str(),
            ),
            easter_egg: Self::easter_egg(),
            brand_id: super::get_brand_id(),
            threads: super::logical_cores(),
            signature: CpuSignature::detect(),
            #[cfg(target_arch = "x86_64")]
            ext_signature: if super::vendor_str() == VENDOR_AMD {
                Some(ExtendedSignature::detect())
            } else {
                None
            },
            features: super::get_feature_list(),
            topology: Topology::detect(),
        }
    }

    /// Gets the CPU model string.
    fn raw_model_string() -> String<64> {
        let mut model: String<64> = String::new();
        if super::max_extended_leaf() < EXT_LEAF_4 {
            let _ = model.push_str("Unknown");
            return model;
        }

        for leaf in EXT_LEAF_2..=EXT_LEAF_4 {
            let res = x86_cpuid(leaf);
            for reg in &[res.eax, res.ebx, res.ecx, res.edx] {
                for &b in &reg.to_le_bytes() {
                    if b != 0 {
                        let _ = model.push(b as char);
                    }
                }
            }
        }

        let trimmed = model.trim();
        let mut out: String<64> = String::new();
        let _ = out.push_str(trimmed);
        out
    }

    fn intel_brand_index(&self) -> Option<String<64>> {
        let brand_id = super::get_brand_id();

        const CELERON: &str = "Intel® Celeron® processor";
        const XEON: &str = "Intel® Xeon® processor";
        const XEON_MP: &str = "Intel® Xeon® processor MP";

        let (family, model, stepping) = (
            self.signature.family,
            self.signature.model,
            self.signature.stepping,
        );

        // If the family and model are greater than (0xF, 0x3),
        // this table does not apply
        if family > 0xF || (family == 0xF && model >= 0x3) {
            return None;
        }

        let str = match brand_id {
            0x01 | 0x0A | 0x14 => CELERON,
            0x02 | 0x04 => "Intel® Pentium® III processor",
            0x03 => match (family, model, stepping) {
                (0x6, 0xB, 0x1) => CELERON,
                _ => "Intel® Pentium® III Xeon",
            },
            0x06 => "Mobile Intel® Pentium® III processor-M",
            0x07 | 0x0F | 0x13 | 0x17 => "Mobile Intel® Celeron® processor",
            0x08 | 0x09 => "Intel® Pentium® 4 processor",
            0x0B => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON_MP,
                _ => XEON,
            },
            0x0C => XEON_MP,
            0x0E => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON,
                _ => "Mobile Intel® Pentium® 4 processor-M",
            },
            0x11 | 0x15 => "Mobile Genuine Intel® processor",
            0x12 => "Intel® Celeron® M processor",
            0x16 => "Intel® Pentium® M processor",
            _ => UNK,
        };

        match str {
            UNK => None,
            _ => Some(String::from_str(str).unwrap()),
        }
    }

    fn display_model_string(&self) -> String<64> {
        if &self.arch.model != "Unknown" {
            return self.arch.model.clone();
        }

        #[cfg(target_arch = "x86")]
        if self.arch.vendor_string == VENDOR_CYRIX {
            return super::cyrix::Cyrix::model_string();
        }

        if self.arch.vendor_string == VENDOR_INTEL
            && let Some(model_name) = self.intel_brand_index()
        {
            return model_name;
        }

        if self.signature == CpuSignature::default() || !super::has_cpuid() {
            let s = if super::is_386() {
                "386 Class CPU"
            } else {
                "486 Class CPU"
            };

            return String::from_str(s).unwrap();
        }

        let s = match self.arch.micro_arch {
            // AMD
            MicroArch::Am486 => match self.arch.code_name {
                "Am486DX2" => "AMD 486 DX2",
                "Am486X2WB" => "AMD 486 DX2 with Write-Back Cache",
                "Am486DX4" => "AMD 486 DX4",
                "Am486DX4WB" => "AMD 486 DX4 with Write-Back Cache",
                _ => "486 Class CPU",
            },
            MicroArch::SSA5 | MicroArch::K5 => "AMD K5",

            //Intel
            MicroArch::I486 => match self.arch.code_name {
                "i80486DX" => "Intel 486 DX",
                "RapidCAD" => "Intel RapidCAD",
                "i80486DX-50" => "Intel 486 DX-50",
                "i80486SX" => "Intel 486 SX",
                "i80486DX2" => "Intel 486 DX2",
                "i80486SL" => "Intel 486 SL",
                "i80486SX2" => "Intel 486 SX2",
                "i80486DX2WB" => "Intel 486 DX2 with Write-Back Cache",
                "i80486DX4" => "Intel 486 DX4",
                "i80486DX4WB" => "Intel 486 DX4 with Write-Back Cache",
                _ => "486 Class CPU",
            },
            MicroArch::P5 => {
                if super::has_mmx() {
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

            // IDT
            MicroArch::Winchip => "IDT Winchip",

            // Rise
            MicroArch::MP6 => match self.arch.code_name {
                "Lynx" => "Rise iDragon",
                _ => "Rise mP6",
            },

            // UMCs
            MicroArch::U5S => "UMC Green CPU U5S (486 SX)",
            MicroArch::U5D => "UMC Green CPU U5D (486 DX)",

            _ => UNK,
        };

        String::from_str(s).unwrap()
    }

    fn easter_egg() -> Option<String<64>> {
        let mut out: String<64> = String::new();
        let brand = CpuBrand::detect();

        let addr = match brand {
            CpuBrand::AMD => 0x8FFF_FFFF,

            #[cfg(target_arch = "x86")]
            CpuBrand::Rise => 0x0000_5A4E,

            _ => 1,
        };

        if addr != 1 {
            let res = x86_cpuid(addr);

            let reg_list = match brand {
                // Surely there had to be a reason for this silly ordering?
                #[cfg(target_arch = "x86")]
                CpuBrand::Rise => [res.ebx, res.edx, res.ecx, res.eax],

                _ => [res.eax, res.ebx, res.ecx, res.edx],
            };

            for &reg in &reg_list {
                let bytes = reg.to_le_bytes();
                for &b in &bytes {
                    if b != 0 {
                        let _ = out.push(b as char);
                    }
                }
            }
        }

        let trimmed = out.trim();
        if !trimmed.is_empty() {
            let final_out: String<64> = String::from_str(trimmed).unwrap();
            Some(final_out)
        } else {
            None
        }
    }

    pub fn debug(&self) {
        #[cfg(not(target_os = "none"))]
        println!("{:#?}", self);

        #[cfg(target_os = "none")]
        println!("{:?}", self);
    }

    pub fn display_table(&self) {
        use heapless::format;

        let ma: String<64> = self.arch.micro_arch.into();
        let ma = ma.as_str();

        let label: fn(&str) -> String<32> = |label| format!("{:>14}:{:1}", label, "").unwrap();

        let simple_line = |l, v: &str| {
            let l = label(l);
            println!("{}{}", l, v);
            println!();
        };

        println!();

        // Vendor_string (brand_name)
        if self.arch.vendor_string.is_empty() && self.arch.brand_name == UNK {
            simple_line("Vendor", "Unknown");
        } else {
            println!(
                "{}{} ({})",
                label("Vendor"),
                self.arch.vendor_string.as_str(),
                self.arch.brand_name.as_str()
            );
            println!();
        }

        if self.signature.is_overdrive {
            simple_line("Overdrive", "Yes");
        }

        simple_line("Model", self.display_model_string().as_str());

        simple_line("Architecture", FeatureClass::detect().to_str());

        // TODO: Cores/Threads

        if ma != UNK {
            simple_line("MicroArch", ma);
        }

        if !(self.arch.code_name == "Unknown"
            || self.arch.code_name == ma
            || self.arch.micro_arch == MicroArch::I486)
        {
            simple_line("Codename", self.arch.code_name);
        }

        // Process node
        if let Some(tech) = &self.arch.technology {
            simple_line("Node", tech.as_str());
        }

        // Easter Egg (AMD K6, K8, Jaguar or Rise mp6)
        if let Some(easter_egg) = &self.easter_egg {
            simple_line("Easter Egg", easter_egg.as_str());
        }

        #[cfg(target_arch = "x86")]
        if self.arch.vendor_string == VENDOR_CYRIX {
            let cyrix = super::cyrix::Cyrix::detect();

            println!("{}{}{}", label("Cyrix"), "Model number: ", cyrix.dir0);
            println!("{:>16} Multiplier: {}x", "", cyrix.multiplier);
            println!("{:>16} Revision: {:X}h", "", cyrix.revision);
            println!("{:>16} Stepping: {:X}h", "", cyrix.stepping);
        }

        // TODO: Clock Speed (Base/Boost)
        #[cfg(not(target_os = "none"))]
        if self.topology.speed.base > 10 {
            let mhz = self.topology.speed.base as f32;
            let ghz = mhz / 1000f32;
            if mhz > 1000f32 {
                println!("{}{:.2} GHz", label("Speed"), ghz);
            } else {
                println!("{}{:.2} MHz", label("Speed"), mhz);
            }
            println!();
        }

        if let Some(cache) = self.topology.cache {
            match cache.l1 {
                Level1Cache::Unified(cache) => {
                    let size = cache.size;
                    println!("{}L1: Unified {} KB", label("Cache"), size / 1024);
                }
                Level1Cache::Split { data, instruction } => {
                    println!("{}L1d: {} KB", label("Cache"), data.size / 1024);
                    println!("{:>16}L1i: {} KB", "", instruction.size / 1024);
                }
            }

            if let Some(cache) = cache.l2 {
                let unit = if cache.size >= 1024 { "MB" } else { "GB" };
                let mut num = cache.size / 1024;

                if num >= 1024 {
                    num /= 1024;
                }

                println!("{:>16}L2: {} {}", "", num, unit);
            }

            if let Some(cache) = cache.l3 {
                let unit = if cache.size >= 1024 { "MB" } else { "KB" };
                let mut num = cache.size / 1024;
                if num >= 1024 {
                    num /= 1024
                }

                println!("{:>16}L3: {} {}", "", num, unit);
            }

            println!();
        }

        // CPU Signature
        if self.signature != CpuSignature::default() {
            println!(
                "{}Family {:X}h, Model {:X}h, Stepping {:X}h",
                label("Signature"),
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
            println!();
        }

        // CPU Features
        if !self.features.is_empty() {
            #[cfg(target_arch = "x86")]
            let mut features: String<32> = String::new();

            #[cfg(target_arch = "x86_64")]
            let mut features: String<512> = String::new();

            self.features.iter().for_each(|feature| {
                let _ = features.push_str(feature);
                let _ = features.push_str(" ");
            });

            simple_line("Features", features.as_str());
        }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpuid::get_feature_list;
    use crate::println;

    #[test]
    fn test_model_string() {
        let model = Cpu::raw_model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }

    #[test]
    fn test_cpu_features_detect() {
        let features = get_feature_list();
        println!("Detected CPU Features: {:?}", features);
        // Assert that at least some features are detected (this might vary by CPU)
        assert!(!features.is_empty());
    }

    #[test]
    fn test_cpu_signature_detect() {
        let signature = CpuSignature::detect();
        println!("Detected CPU Signature: {:?}", signature);
        // Basic assertions to ensure fields are populated, not necessarily specific values
        assert!(signature.family > 0 || signature.extended_family > 0 || signature.model > 0);
    }

    #[test]
    fn test_cpu_new() {
        let cpu = Cpu::new();
        println!("New CPU instance: {:?}", cpu);
        // Ensure that new() doesn't panic and populates some fields
        assert!(!cpu.arch.vendor_string.is_empty());
        assert!(!cpu.features.is_empty());
    }

    #[test]
    fn test_display_model_string() {
        // Test case for MicroArch::Am486
        let mut arch_am486 = CpuArch::default();
        arch_am486.micro_arch = MicroArch::Am486;

        arch_am486.code_name = "Am486DX2";
        let cpu_am486_dx2 = Cpu {
            arch: arch_am486.clone(),
            brand_id: 0,
            easter_egg: None,
            threads: 1,
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
            threads: 1,
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
        let mut arch_i486 = CpuArch::default();
        arch_i486.micro_arch = MicroArch::I486;

        arch_i486.code_name = "i80486DX";
        let cpu_i486_dx = Cpu {
            arch: arch_i486.clone(),
            brand_id: 0,
            easter_egg: None,
            threads: 1,
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
            threads: 1,
            signature: CpuSignature {
                extended_family: 0,
                family: 0,
                extended_model: 0,
                model: 0,
                stepping: 0,
                display_family: 0,
                display_model: 0,
                is_overdrive: false,
            },
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_no_cpuid.display_model_string(), "486 Class CPU");

        // Test case for "Unknown"
        let cpu_unknown = Cpu {
            arch: CpuArch::default(),
            brand_id: 0,
            easter_egg: None,
            threads: 1,
            signature: CpuSignature {
                extended_family: 1, // Make it not all zeros
                family: 1,
                extended_model: 1,
                model: 1,
                stepping: 1,
                display_family: 1,
                display_model: 1,
                is_overdrive: false,
            },
            ext_signature: None,
            features: get_feature_list(),
            topology: Topology::default(),
        };
        assert_eq!(cpu_unknown.display_model_string(), "Unknown");
    }

    #[test]
    fn test_easter_egg() {
        let easter_egg = Cpu::easter_egg();
        println!("Easter Egg: {:?}", easter_egg);
        // We cannot assert a specific value, just ensure it runs
    }

    #[test]
    fn test_cpu_debug() {
        let cpu = Cpu::new();
        cpu.debug();
        // This primarily prints, so we just ensure it doesn't panic
    }

    #[test]
    fn test_cpu_display_table() {
        let cpu = Cpu::new();
        cpu.display_table();
        // This primarily prints, so we just ensure it doesn't panic
    }
}
