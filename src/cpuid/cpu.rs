//! CPU detection and information for x86/x86_64 processors.

use super::brand::CpuBrand;
use super::micro_arch::{CpuArch, MicroArch};
use super::topology::Topology;
use super::{EXT_LEAF_1, EXT_LEAF_2, EXT_LEAF_4, FeatureList, read_multi_leaf_str, x86_cpuid};
use crate::common::cache::Level1Cache;
use crate::common::{TCpu, UNK};
use crate::println;

use core::str::FromStr;
use heapless::String;

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

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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

            #[cfg(target_arch = "x86")]
            if has_amd64() {
                return FeatureClass::x86_64_v1;
            }

            #[cfg(target_arch = "x86_64")]
            FeatureClass::x86_64_v1
        }

        #[cfg(target_arch = "x86")]
        {
            if is_cyrix() {
                return vendor::Cyrix::get_feature_class();
            }

            if has_cmov() {
                return FeatureClass::i686;
            }

            if CpuSignature::detect().family >= 5 {
                return FeatureClass::i586;
            }

            if is_486() || (has_cpuid() && CpuSignature::detect().family == 4) {
                return FeatureClass::i486;
            }

            FeatureClass::i386
        }
    }

    /// Returns a string representation of the feature class.
    pub fn to_str(self) -> &'static str {
        match self {
            FeatureClass::i386 => "i386",
            FeatureClass::i486 => "i486",
            FeatureClass::i586 => "i586",
            FeatureClass::i686 => "i686",
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
    /// Detects the CPU signature from CPUID leaf 1.
    pub fn detect() -> Self {
        let from_cpuid = super::has_cpuid();

        #[cfg(target_arch = "x86")]
        if !from_cpuid {
            #[cfg(target_os = "none")]
            {
                if let Some(reset_sig) = super::get_reset_signature() {
                    return reset_sig;
                }
            }
        }

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
            from_cpuid,
        }
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
    pub easter_egg: Option<String<64>>,
    /// Model brand id
    pub brand_id: u32,
    /// CPU signature (family, model, stepping)
    pub signature: CpuSignature,
    /// AMD extended cpu signature
    pub ext_signature: Option<ExtendedSignature>,
    /// Detected CPU features
    pub features: FeatureList,
    /// Speed, threads, cores, sockets
    pub topology: Topology,
}

impl Cpu {
    /// Gets the CPU model string.
    pub fn raw_model_string() -> String<64> {
        read_multi_leaf_str(EXT_LEAF_2, EXT_LEAF_4)
    }

    fn intel_brand_index(&self) -> Option<String<64>> {
        if !super::is_intel() {
            return None;
        }

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

    /// Returns a human-readable display name for the CPU model.
    ///
    /// This attempts to produce a marketing-style name based on the
    /// detected CPU, falling back to architecture class names for
    /// older or unrecognized processors.
    pub fn display_model_string(&self) -> String<64> {
        #[cfg(target_arch = "x86")]
        if super::is_cyrix() {
            return super::vendor::Cyrix::model_string();
        }

        // Check the Intel model lookup table
        if let Some(model_name) = self.intel_brand_index() {
            return model_name;
        }

        if self.arch.model != UNK {
            return self.arch.model.clone();
        }

        #[cfg(target_arch = "x86")]
        if CpuBrand::detect() == CpuBrand::Unknown {
            let s = if super::is_386() {
                "'Classic' 386"
            } else {
                "'Classic' 486"
            };

            return String::from_str(s).unwrap();
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
                _ => "'Classic' 486",
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
        const AMD_EASTER_EGG_ADDR: u32 = 0x8FFF_FFFF;
        #[cfg(target_arch = "x86")]
        const RISE_EASTER_EGG_ADDR: u32 = 0x0000_5A4E;

        let mut out: String<64> = String::new();
        let brand = CpuBrand::detect();

        let addr = match brand {
            CpuBrand::AMD => AMD_EASTER_EGG_ADDR,

            #[cfg(target_arch = "x86")]
            CpuBrand::Rise => RISE_EASTER_EGG_ADDR,

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
}

impl TCpu for Cpu {
    /// Detects and returns comprehensive CPU information.
    ///
    /// Performs full CPU detection including architecture, microarchitecture,
    /// brand string, signature, features, and topology.
    fn detect() -> Self {
        let sig = CpuSignature::detect();
        Self {
            arch: CpuArch::find(Self::raw_model_string().as_str(), sig, &super::vendor_str()),
            easter_egg: Self::easter_egg(),
            brand_id: super::get_brand_id(),
            signature: sig,
            ext_signature: match super::is_amd() {
                true => Some(ExtendedSignature::detect()),
                false => None,
            },
            features: super::get_feature_list(),
            topology: Topology::detect(),
        }
    }

    fn debug(&self) {
        #[cfg(not(target_os = "none"))]
        println!("{:#?}", self);

        #[cfg(target_os = "none")]
        {
            use super::is_cyrix;

            println!("{:?}", self.arch);
            if is_cyrix() {
                println!("{:?}", super::vendor::Cyrix::detect());
            }
        }
    }

    fn display_table(&self) {
        use heapless::format;

        let ma: String<64> = self.arch.micro_arch.into();
        let ma = ma.as_str();

        let multi_core = self.topology.cores > 1;

        let cache_count = |share_count| {
            if (!multi_core) || share_count == 0 || (self.topology.threads / share_count) <= 1 {
                format!("")
            } else {
                format!("{}x ", self.topology.threads / share_count)
            }
            .unwrap()
        };

        let label: fn(&str) -> String<32> = |label| format!("{:>14}:{:1}", label, "").unwrap();
        let sublabel: fn(&str) -> String<32> =
            |label| format!("{:>16}{}:{:1}", "", label, "").unwrap();

        let simple_line = |l, v: &str| {
            let l = label(l);
            println!("{}{}", l, v);
            println!();
        };

        println!();

        simple_line("Architecture", FeatureClass::detect().to_str());

        // Vendor_string (brand_name)
        if self.arch.brand_name != UNK {
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
            simple_line("Process Node", tech.as_str());
        }

        // Easter Egg (AMD K6, K8, Jaguar or Rise mp6)
        if let Some(easter_egg) = &self.easter_egg {
            simple_line("Easter Egg", easter_egg.as_str());
        }

        // Sockets
        if self.topology.sockets > 1 {
            println!("{}{}", label("Sockets"), self.topology.sockets);
            println!();
        }

        // Cores / Threads
        if multi_core {
            if self.topology.cores != self.topology.threads {
                println!(
                    "{}{} cores ({} threads)",
                    label("Cores"),
                    self.topology.cores,
                    self.topology.threads
                );
            } else {
                println!("{}{} cores", label("Cores"), self.topology.cores);
            }

            println!();
        }

        if let Some(cache) = self.topology.cache {
            match cache.l1 {
                Level1Cache::Unified(cache) => {
                    println!("{}L1: Unified {:>4} KB", label("Cache"), cache.size / 1024);
                }
                Level1Cache::Split { data, instruction } => {
                    let data_count: String<4> = cache_count(data.share_count);
                    let instruction_count = cache_count(instruction.share_count);

                    println!(
                        "{}L1d: {}{} KB, {}-way",
                        label("Cache"),
                        &data_count,
                        data.size / 1024,
                        data.assoc
                    );
                    println!(
                        "{}{}{} KB, {}-way",
                        sublabel("L1i"),
                        &instruction_count,
                        instruction.size / 1024,
                        instruction.assoc
                    );
                }
            }

            if let Some(cache) = cache.l2 {
                let count = cache_count(cache.share_count);

                let mut num = cache.size / 1024;
                let unit = if num >= 1024 { "MB" } else { "KB" };

                if num >= 1024 {
                    num /= 1024;
                }

                println!(
                    "{} {}{} {}, {}-way",
                    sublabel("L2"),
                    &count,
                    num,
                    unit,
                    cache.assoc
                );
            }

            if let Some(cache) = cache.l3 {
                let mut num = cache.size / 1024;
                let unit = if num >= 1024 { "MB" } else { "KB" };

                if num >= 1024 {
                    num /= 1024
                }

                println!("{} {} {}, {}-way", sublabel("L3"), num, unit, cache.assoc);
            }

            println!();
        }

        // Clock Speed (Base/Boost)
        if self.topology.speed.base > 0 {
            let base = self.topology.speed.base;
            let boost = self.topology.speed.boost;

            let print_speed = |l: &str, mhz: u32| {
                let is_ghz = mhz >= 1000;
                let unit = if is_ghz { "GHz" } else { "MHz" };
                let whole = if is_ghz { mhz / 1000 } else { mhz };
                let fract = if is_ghz { (mhz % 1000) / 10 } else { 0 };

                if is_ghz {
                    println!("{}{}.{:02} {}", l, whole, fract, unit);
                } else {
                    println!("{}{}.00 {}", l, whole, unit);
                }
            };

            print_speed(label("Frequency").as_str(), base);
            if boost > base {
                print_speed(sublabel("Boost").as_str(), boost);
            }

            println!();
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
                label(key),
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
            #[cfg(target_os = "none")]
            let mut features: String<128> = String::new();

            #[cfg(not(target_os = "none"))]
            let mut features: String<512> = String::new();

            self.features.iter().for_each(|feature| {
                let _ = features.push_str(feature);
                let _ = features.push_str(" ");
            });

            simple_line("Features", features.as_str());
        }

        #[cfg(target_arch = "x86")]
        if super::is_cyrix() {
            let cyrix = super::vendor::Cyrix::detect();

            if cyrix.dir0 != 0xFF {
                println!("{}Model number: {:X}h", label("Cyrix"), cyrix.dir0);
                println!("{}{:X}h", sublabel("Revision"), cyrix.revision);
                println!("{}{:X}h", sublabel("Stepping"), cyrix.stepping);
                if !cyrix.multiplier.is_empty() && cyrix.multiplier != "0" {
                    println!(
                        "{}{}x",
                        sublabel("Bus Multiplier"),
                        cyrix.multiplier.as_str()
                    );
                }
                println!();
            }
        }

        println!();
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
    #[cfg(target_arch = "x86")]
    fn test_display_model_string_x32() {
        // Test case for MicroArch::Am486
        let mut arch_am486 = CpuArch::default();
        arch_am486.micro_arch = MicroArch::Am486;

        arch_am486.code_name = "Am486DX2";
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
        let mut arch_i486 = CpuArch::default();
        arch_i486.micro_arch = MicroArch::I486;

        arch_i486.code_name = "i80486DX";
        let cpu_i486_dx = Cpu {
            arch: arch_i486.clone(),
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
