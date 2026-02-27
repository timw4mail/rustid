//! CPU detection and information for x86/x86_64 processors.

use crate::cpuid::brand::{CpuBrand, VENDOR_INTEL};
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::{FeatureList, UNK, fns, x86_cpuid};
use crate::println;
use heapless::String;

use core::str::FromStr;

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

        let is_overdrive = fns::is_overdrive();

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

/// Represents a complete x86/x86_64 CPU with all detected information.
#[derive(Debug)]
pub struct Cpu {
    /// CPU architecture and microarchitecture details
    pub arch: CpuArch,
    /// Easter egg string (hidden CPU info for some AMD/Rise processors)
    pub easter_egg: Option<String<64>>,
    pub brand_id: u32,
    /// Number of logical processors/threads
    pub threads: u32,
    /// CPU signature (family, model, stepping)
    pub signature: CpuSignature,
    /// Detected CPU features
    pub features: FeatureList,
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
                Self::model_string().as_str(),
                CpuSignature::detect(),
                &fns::vendor_str(),
            ),
            easter_egg: Self::easter_egg(),
            brand_id: fns::get_brand_id(),
            threads: fns::logical_cores(),
            signature: CpuSignature::detect(),
            features: fns::get_feature_list(),
        }
    }

    /// Gets the CPU model string.
    fn model_string() -> String<64> {
        let mut model: String<64> = String::new();
        if fns::max_extended_leaf() < 0x8000_0004 {
            let _ = model.push_str("Unknown");
            return model;
        }

        for leaf in 0x8000_0002..=0x8000_0004 {
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

    fn intel_brand_index(&self) -> Option<&'static str> {
        let brand_id = fns::get_brand_id();

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

        match brand_id {
            0x01 | 0x0A | 0x14 => Some(CELERON),
            0x02 | 0x04 => Some("Intel® Pentium® III processor"),
            0x03 => match (family, model, stepping) {
                (0x6, 0xB, 0x1) => Some(CELERON),
                _ => Some("Intel® Pentium® III Xeon"),
            },
            0x06 => Some("Mobile Intel® Pentium® III processor-M"),
            0x07 | 0x0F | 0x13 | 0x17 => Some("Mobile Intel® Celeron® processor"),
            0x08 | 0x09 => Some("Intel® Pentium® 4 processor"),
            0x0B => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => Some(XEON_MP),
                _ => Some(XEON),
            },
            0x0C => Some(XEON_MP),
            0x0E => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => Some(XEON),
                _ => Some("Mobile Intel® Pentium® 4 processor-M"),
            },
            0x11 | 0x15 => Some("Mobile Genuine Intel® processor"),
            0x12 => Some("Intel® Celeron® M processor"),
            0x16 => Some("Intel® Pentium® M processor"),
            _ => None,
        }
    }

    fn display_model_string(&self) -> &str {
        if &self.arch.model != "Unknown" {
            return &self.arch.model;
        }

        // First, let's see if there's something in Intel's brand mapping table
        if self.arch.vendor_string == VENDOR_INTEL
            && let Some(model_name) = self.intel_brand_index()
        {
            return model_name;
        }

        match self.arch.micro_arch {
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
                if fns::has_mmx() {
                    "Intel Pentium with MMX"
                } else {
                    "Intel Pentium"
                }
            }
            MicroArch::PentiumPro => "Intel Pentium Pro",
            MicroArch::PentiumII => "Intel Pentium II",
            MicroArch::PentiumIII => "Intel Pentium III",

            // Cyrix
            MicroArch::Cy5x86 => "5x86",
            MicroArch::M1 => {
                if fns::has_cx8() {
                    "6x86L"
                } else {
                    "6x86"
                }
            }
            MicroArch::M2 => "6x86MX (MII)",

            // UMCs
            MicroArch::U5S => "UMC Green CPU 486 U5-SX",
            MicroArch::U5D => "UMC Green CPU 486 U5-DX",

            _ => {
                if self.signature == CpuSignature::default() || !fns::has_cpuid() {
                    if fns::is_cyrix() && fns::is_486() {
                        "Cyrix/IBM 486"
                    } else if fns::is_386() {
                        "386 Class CPU"
                    } else {
                        "486 Class CPU"
                    }
                } else {
                    "Unknown"
                }
            }
        }
    }

    fn easter_egg() -> Option<String<64>> {
        let mut out: String<64> = String::new();

        let addr = match CpuBrand::detect() {
            CpuBrand::AMD => 0x8FFF_FFFF,
            CpuBrand::Rise => 0x0000_5A4E,
            _ => 1,
        };

        if addr != 1 {
            let res = x86_cpuid(addr);

            for &reg in &[res.eax, res.ebx, res.ecx, res.edx] {
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

    // TODO: Show cpu cache size(s)
    // TODO: Show cpu speed
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

        simple_line("Model", self.display_model_string());

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

        // TODO: Cache Size(s)

        // TODO: Clock Speed (Base/Boost)

        // TODO: Cores/Threads

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
    use crate::println;

    #[test]
    fn test_model_string() {
        let model = Cpu::model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }

    #[test]
    fn test_cpu_features_detect() {
        let features = fns::get_feature_list();
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
            features: fns::get_feature_list(),
        };
        assert_eq!(cpu_am486_dx2.display_model_string(), "AMD 486 DX2");

        arch_am486.code_name = "Am486X2WB";
        let cpu_am486_x2wb = Cpu {
            arch: arch_am486.clone(),
            brand_id: 0,
            easter_egg: None,
            threads: 1,
            signature: CpuSignature::detect(),
            features: fns::get_feature_list(),
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
            features: fns::get_feature_list(),
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
            features: fns::get_feature_list(),
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
            features: fns::get_feature_list(),
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
