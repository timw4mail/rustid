use crate::common::UNK;
use crate::cpuid::CpuSignature;
use crate::cpuid::brand::VENDOR_INTEL;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};

/// Intel-specific microarchitecture detection.
pub struct Intel;

impl Intel {
    /// Detects the Intel microarchitecture based on the CPU model string and signature.
    pub fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> CpuArch {
            CpuArch::new(model, ma, code_name, "Intel", VENDOR_INTEL, tech)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            (0, 3, 0, 4, _) => brand_arch(MicroArch::RapidCad, "RapidCad", None),

            // 486
            (0, 4, 0, 0, _) => brand_arch(MicroArch::I486, "i80486DX", None),
            (0, 4, 0, 1, _) => brand_arch(MicroArch::I486, "i80486DX-50", None),
            (0, 4, 0, 2, _) => brand_arch(MicroArch::I486, "i80486SX", None),
            (0, 4, 0, 3, _) => brand_arch(MicroArch::I486, "i80486DX2", None),
            (0, 4, 0, 4, _) => brand_arch(MicroArch::I486, "i80486SL", None),
            (0, 4, 0, 5, _) => brand_arch(MicroArch::I486, "i80486SX2", None),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::I486, "i80486DX2WB", None),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::I486, "i80486DX4", None),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::I486, "i80486DX4WB", None),

            // Pentium
            (0, 5, 0, 0, _) => brand_arch(MicroArch::P5, "P5 A-step", Some("800nm")),
            (0, 5, 0, 1, _) => brand_arch(MicroArch::P5, "P5", Some("800nm")),
            (0, 5, 0, 2, _) => brand_arch(MicroArch::P5, "P54C", None),
            (0, 5, 0, 3, _) => brand_arch(MicroArch::P5, "P24T", Some("600nm")),
            (0, 5, 0, 4, _) => brand_arch(MicroArch::P5, "P55C", Some("350nm")), // With MMX
            (0, 5, 0, 7, _) => brand_arch(MicroArch::P5, "P54C", Some("350nm")),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::P5, "P55C", Some("250nm")),
            (0, 5, 0, 9 | 10, _) => brand_arch(MicroArch::Lakemont, "Lakemont", Some("32nm")),

            // Pentium Pro
            (0, 6, 0, 1, 1) => brand_arch(MicroArch::PentiumPro, "P6", None),
            (0, 6, 0, 1, 2) => brand_arch(MicroArch::PentiumPro, "P6", Some("600nm")),
            (0, 6, 0, 1, 6..10) => brand_arch(MicroArch::PentiumPro, "P6", Some("350nm")),
            (0, 6, 0, 3, 2) => brand_arch(MicroArch::PentiumII, "P6T (Deschutes)", Some("250nm")), // Pentium II Overdrive

            // Pentium 2
            (0, 6, 0, 0..=2, _) => brand_arch(MicroArch::PentiumII, UNK, None),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::PentiumII, "Klamath", Some("350nm")),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::PentiumIII, UNK, None),
            (0, 6, 0, 5, 1) => brand_arch(MicroArch::PentiumII, "Deschutes", Some("250nm")),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::PentiumII, "Dixon / Mendocino", None),

            // Pentium 3
            (0, 6, 0, 7, _) => brand_arch(MicroArch::PentiumIII, "Katmai", Some("250nm")),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::PentiumIII, "Coppermine", Some("180nm")),
            (0, 6, 0, 9, 5) => brand_arch(MicroArch::PentiumIII, "Banias", Some("130nm")),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::PentiumIII, "Cascades", Some("180nm")),
            (0, 6, 0, 11, _) => brand_arch(MicroArch::PentiumIII, "Tualatin", Some("130nm")),

            // The dark ages
            // from sandpile.org
            (0, 15, 0, 0, _) => brand_arch(MicroArch::Willamette, "Willamette", Some("180nm")),
            (0, 15, 0, 1, _) => {
                brand_arch(MicroArch::Willamette, "Willamette/Foster", Some("180nm"))
            }
            (0, 15, 0, 2, _) => {
                brand_arch(MicroArch::Northwood, "Northwood/Gallatin", Some("130nm"))
            }
            (0, 15, 0, 3, _) => brand_arch(MicroArch::Prescott, "Prescott", Some("90nm")),
            (0, 15, 0, 4, _) => brand_arch(MicroArch::Prescott, "Prescott/Potomac", Some("90nm")),
            (0, 15, 0, 6, _) => brand_arch(MicroArch::CedarMill, "Cedar Mill/Tulsa", Some("64nm")),

            // Pentium M/Core/Core 2
            (0, 6, 0, 12, _) => brand_arch(MicroArch::PentiumIII, "Timna", Some("180nm")),
            (0, 6, 0, 13, 8) => brand_arch(MicroArch::Dothan, "Dothan", Some("90nm")),
            (0, 6, 0, 14, _) => brand_arch(MicroArch::Yonah, "Yonah", Some("65nm")),
            (0, 6, 0, 15, 6) => brand_arch(MicroArch::Core, "Merom", Some("65nm")),
            (0, 6, 1, 6, _) => brand_arch(MicroArch::Core, "Merom-L", Some("65nm")),
            (0, 6, 1, 7, 0) => brand_arch(MicroArch::Core, "Yorkfield", Some("45nm")),
            (0, 6, 1, 7, 10) => brand_arch(MicroArch::Core, "Penryn", Some("45nm")),

            // Core i-series
            (0, 6, 1, 13, _) => brand_arch(MicroArch::Dunnington, "Dunnington", Some("45nm")),
            (0, 6, 1, 14, 5) => brand_arch(MicroArch::Nehalem, "Lynnfield", Some("45nm")),
            (0, 6, 2, 10, 7) => brand_arch(MicroArch::SandyBridge, "Sandy Bridge", Some("32nm")),
            (0, 6, 2, 12, 0) => brand_arch(MicroArch::Westmere, "Arrandale", Some("32nm")),
            (0, 6, 3, 12, _) => brand_arch(MicroArch::Haswell, "Haswell", Some("22nm")),
            (0, 6, 3, 15, _) => brand_arch(MicroArch::Haswell, "Haswell-EP 4S", Some("22nm")),
            (0, 6, 4, 12, 4) => brand_arch(MicroArch::Airmont, "Braswell", Some("14nm")),
            (0, 6, 7, 10, 8) => brand_arch(MicroArch::GoldmontPlus, "Gemini Lake", Some("14nm")),
            (0, 6, 8, 14, 9) => brand_arch(MicroArch::AmberLake, "Amber Lake-Y", Some("14nm")),
            (0, 6, 11, 14, _) => brand_arch(MicroArch::AlderLake, "Alder Lake-N", Some("10nm")),
            _ => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cpu_arch_find_intel() {
        let model = "Intel Processor";

        // I486
        let sig_i486 = crate::cpuid::micro_arch::tests::dummy_signature(4, 0, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_i486);
        assert_eq!(arch.micro_arch, MicroArch::I486);
        assert_eq!(arch.code_name, "i80486DX");

        // P5 (MicroArch::Pentium)
        let sig_p5 = crate::cpuid::micro_arch::tests::dummy_signature(5, 2, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_p5);
        assert_eq!(arch.micro_arch, MicroArch::P5);
        assert_eq!(arch.code_name, "P54C");

        // Nehalem
        let sig_nehalem = crate::cpuid::micro_arch::tests::dummy_signature(6, 14, 0, 1, 5);
        let arch = Intel::micro_arch(model, sig_nehalem);
        assert_eq!(arch.micro_arch, MicroArch::Nehalem);
        assert_eq!(arch.code_name, "Lynnfield");

        // Unknown Intel
        let sig_unknown = crate::cpuid::micro_arch::tests::dummy_signature(99, 0, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }
}
