#[cfg(any(not(nostd_os), target_os = "uefi"))]
use crate::common::CoreType;
use crate::x86::CpuSignature;
use crate::x86::constants::*;
use crate::x86::micro_arch::{CpuArch, MicroArch};
use crate::x86::vendor::TMicroArch;

/// Intel-specific microarchitecture detection and signature disambiguation.
///
/// Sources & References:
/// - Intel SDM Vol 4: Model-Specific Registers (Order Number: 335592, Table 2-1 CPUID Signatures)
/// - Intel SDM Vol 2A & Future Features Reference (Order Number: 319433)
/// - Linux Kernel: `arch/x86/include/asm/intel-family.h`
/// - Open-source library: `libcpuid/recog_intel.c`
/// - Instlatx64 raw CPUID instruction register dumps (`instlatx64.atw.hu`)
/// - Sandpile.org x86 processor architecture tables
pub struct Intel;

impl Intel {
    fn legacy_micro_arch(
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> Option<CpuArch> {
        let arch = match (
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
            _ => return None,
        };
        Some(arch)
    }

    #[cfg(not(dos))]
    fn disambiguate_hedt_server(
        model: &str,
        ma: MicroArch,
        hedt_name: &'static str,
        server_name: &'static str,
        tech: Option<&'static str>,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("Core") {
            brand_arch(ma, hedt_name, tech)
        } else {
            brand_arch(ma, server_name, tech)
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_55h(
        model: &str,
        stepping: u32,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        // CPUID 06_55H:
        // Steppings 0..=4: Skylake-SP / Skylake-X
        // Steppings 5..=7: Cascade Lake-SP / Cascade Lake-X
        // Steppings 10..=11: Cooper Lake-SP
        if stepping >= 10
            || model.contains("Cooper")
            || (model.contains("83") && model.contains('H'))
        {
            brand_arch(MicroArch::CooperLake, "Cooper Lake", Some(N14))
        } else if (5..=7).contains(&stepping)
            || model.contains("Cascade")
            || model.contains("109")
            || model.contains("82")
            || model.contains("62")
            || model.contains("52")
            || model.contains("42")
            || model.contains("32")
        {
            if model.contains("Core") {
                brand_arch(MicroArch::CascadeLake, "Cascade Lake-X", Some(N14))
            } else if model.contains("W-") {
                brand_arch(MicroArch::CascadeLake, "Cascade Lake-W", Some(N14))
            } else {
                brand_arch(MicroArch::CascadeLake, "Cascade Lake-SP", Some(N14))
            }
        } else if model.contains("Core") {
            brand_arch(MicroArch::Skylake, "Skylake-X", Some(N14))
        } else if model.contains("W-") {
            brand_arch(MicroArch::Skylake, "Skylake-W", Some(N14))
        } else if model.contains("D-") {
            brand_arch(MicroArch::Skylake, "Skylake-D", Some(N14))
        } else {
            brand_arch(MicroArch::Skylake, "Skylake-SP", Some(N14))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_8eh(
        model: &str,
        stepping: u32,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        // CPUID 06_8EH:
        // Stepping 9: Amber Lake-Y / Kaby Lake-Y / Kaby Lake-U
        // Stepping 10: Coffee Lake-U / Kaby Lake Refresh
        // Stepping 11: Whiskey Lake-U
        // Stepping 12: Whiskey Lake-U / Comet Lake-U (4-core)
        if model.contains("8100Y")
            || model.contains("8200Y")
            || model.contains("8500Y")
            || model.contains("10510Y")
            || model.contains("10210Y")
            || model.contains("10310Y")
            || model.contains("10110Y")
            || model.contains("6500Y")
            || model.contains("4425Y")
            || (stepping == 9 && (model.contains('Y') || model.contains("Amber")))
        {
            brand_arch(MicroArch::AmberLake, "Amber Lake-Y", Some(N14))
        } else if model.contains("10") && model.contains('U') && !model.contains("1000") {
            brand_arch(MicroArch::CometLake, "Comet Lake-U", Some(N14))
        } else if stepping == 11
            || (stepping == 12 && !model.contains("10"))
            || model.contains("8265U")
            || model.contains("8365U")
            || model.contains("8565U")
            || model.contains("8665U")
            || model.contains("Whiskey")
        {
            brand_arch(MicroArch::WhiskyLake, "Whiskey Lake-U", Some(N14))
        } else if model.contains("8559U")
            || model.contains("8269U")
            || model.contains("8259U")
            || model.contains("Coffee")
        {
            brand_arch(MicroArch::CoffeeLake, "Coffee Lake-U", Some(N14))
        } else if model.contains("8250U")
            || model.contains("8350U")
            || model.contains("8550U")
            || model.contains("8650U")
        {
            brand_arch(MicroArch::KabyLake, "Kaby Lake-R", Some(N14))
        } else {
            brand_arch(MicroArch::KabyLake, "Kaby Lake-U", Some(N14))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_9eh(
        model: &str,
        stepping: u32,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        // CPUID 06_9EH:
        // Stepping 9: Kaby Lake-S / Kaby Lake-H / Kaby Lake-X
        // Stepping 10..=11: Coffee Lake-S / Coffee Lake-H (8th Gen Core)
        // Stepping 12..=13: Coffee Lake Refresh (9th Gen Core)
        if stepping >= 12
            || model.contains("9900")
            || model.contains("9700")
            || model.contains("9600")
            || model.contains("9500")
            || model.contains("9400")
            || model.contains("9300")
            || model.contains("9100")
            || model.contains("CC150")
            || model.contains("2286M")
        {
            if model.contains('H') {
                brand_arch(MicroArch::CoffeeLake, "Coffee Lake-H Refresh", Some(N14))
            } else {
                brand_arch(MicroArch::CoffeeLake, "Coffee Lake-S Refresh", Some(N14))
            }
        } else if (10..=11).contains(&stepping)
            || model.contains("8700")
            || model.contains("8600")
            || model.contains("8500")
            || model.contains("8400")
            || model.contains("8300")
            || model.contains("8100")
            || model.contains("G5400")
            || model.contains("G5500")
            || model.contains("G5600")
            || model.contains("Coffee")
        {
            if model.contains('H') || model.contains('B') {
                brand_arch(MicroArch::CoffeeLake, "Coffee Lake-H", Some(N14))
            } else {
                brand_arch(MicroArch::CoffeeLake, "Coffee Lake-S", Some(N14))
            }
        } else if model.contains('X') {
            brand_arch(MicroArch::KabyLake, "Kaby Lake-X", Some(N14))
        } else if model.contains('H') {
            brand_arch(MicroArch::KabyLake, "Kaby Lake-H", Some(N14))
        } else {
            brand_arch(MicroArch::KabyLake, "Kaby Lake-S", Some(N14))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_8fh(
        model: &str,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("w3-")
            || model.contains("w5-")
            || model.contains("w7-")
            || model.contains("w9-")
            || model.contains("WS")
        {
            brand_arch(
                MicroArch::SapphireRapids,
                "Sapphire Rapids-WS",
                Some(INTEL_7),
            )
        } else if model.contains("Max") {
            brand_arch(
                MicroArch::SapphireRapids,
                "Sapphire Rapids-HBM (Xeon Max)",
                Some(INTEL_7),
            )
        } else {
            brand_arch(
                MicroArch::SapphireRapids,
                "Sapphire Rapids-SP",
                Some(INTEL_7),
            )
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_b7h(
        model: &str,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("14")
            && (model.contains("14900")
                || model.contains("14700")
                || model.contains("14600")
                || model.contains("14500")
                || model.contains("14400")
                || model.contains("14100"))
        {
            if model.contains("HX") {
                brand_arch(
                    MicroArch::RaptorLake,
                    "Raptor Lake-HX Refresh",
                    Some(INTEL_7),
                )
            } else {
                brand_arch(
                    MicroArch::RaptorLake,
                    "Raptor Lake-S Refresh",
                    Some(INTEL_7),
                )
            }
        } else if model.contains("150U")
            || model.contains("120U")
            || model.contains("100U")
            || model.contains("250U")
            || model.contains("220U")
        {
            brand_arch(MicroArch::RaptorLake, "Raptor Lake-U", Some(INTEL_7))
        } else if model.contains("270H")
            || model.contains("250H")
            || model.contains("240H")
            || model.contains("220H")
            || model.contains("210H")
        {
            brand_arch(MicroArch::RaptorLake, "Raptor Lake-H", Some(INTEL_7))
        } else if model.contains("HX") {
            brand_arch(MicroArch::RaptorLake, "Raptor Lake-HX", Some(INTEL_7))
        } else {
            brand_arch(MicroArch::RaptorLake, "Raptor Lake-S", Some(INTEL_7))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_0fh(
        model: &str,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("Xeon") {
            if model.contains("73") {
                brand_arch(MicroArch::Core, "Tigerton", Some(N65))
            } else if model.contains("53") || model.contains("32") || model.contains("Quad") {
                brand_arch(MicroArch::Core, "Clovertown", Some(N65))
            } else {
                brand_arch(MicroArch::Core, "Woodcrest", Some(N65))
            }
        } else if model.contains("Quad") || model.contains("QX") || model.contains("Q6") {
            brand_arch(MicroArch::Core, "Kentsfield", Some(N65))
        } else if model.contains("T5")
            || model.contains("T7")
            || model.contains("L7")
            || model.contains("U7")
            || model.contains('T')
            || model.contains('L')
            || model.contains('U')
        {
            brand_arch(MicroArch::Core, "Merom", Some(N65))
        } else {
            brand_arch(MicroArch::Core, "Conroe", Some(N65))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_17h(
        model: &str,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("Xeon") {
            if model.contains("54") || model.contains("33") || model.contains("Quad") {
                brand_arch(MicroArch::Core, "Harpertown", Some(N45))
            } else {
                brand_arch(MicroArch::Core, "Wolfdale-DP", Some(N45))
            }
        } else if model.contains("Quad")
            || model.contains("QX")
            || model.contains("Q8")
            || model.contains("Q9")
        {
            brand_arch(MicroArch::Core, "Yorkfield", Some(N45))
        } else if model.contains("T8")
            || model.contains("T9")
            || model.contains("P7")
            || model.contains("P8")
            || model.contains("P9")
            || model.contains("SP")
            || model.contains("SL")
            || model.contains("SU")
        {
            brand_arch(MicroArch::Core, "Penryn", Some(N45))
        } else {
            brand_arch(MicroArch::Core, "Wolfdale", Some(N45))
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_06_beh(
        model: &str,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        if model.contains("N150")
            || model.contains("N250")
            || model.contains("N350")
            || model.contains("N355")
            || model.contains("Twin")
        {
            brand_arch(MicroArch::TwinLake, "Twin Lake-N", Some(INTEL_7))
        } else {
            brand_arch(MicroArch::AlderLake, "Alder Lake-N", Some(INTEL_7))
        }
    }

    #[cfg(not(dos))]
    fn modern_micro_arch(
        model: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> Option<CpuArch> {
        let arch = match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // Pentium
            (0, 5, 0, 0, _) => brand_arch(MicroArch::P5, "P5 A-step", Some(N800)),
            (0, 5, 0, 1, _) => brand_arch(MicroArch::P5, "P5", Some(N800)),
            (0, 5, 0, 2, _) => brand_arch(MicroArch::P5, "P54C", Some(N600)),
            (0, 5, 0, 3, _) => brand_arch(MicroArch::P5, "P24T", Some(N600)),
            (0, 5, 0, 4, _) => brand_arch(MicroArch::P5, "P55C", Some(N350)), // With MMX
            (0, 5, 0, 7, _) => brand_arch(MicroArch::P5, "P54C", Some(N350)),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::P5, "Tillamook", Some(N250)),
            (0, 5, 0, 9 | 10, _) => brand_arch(MicroArch::Lakemont, "Lakemont", Some(N32)),

            // Pentium Pro
            (0, 6, 0, 1, 1) => brand_arch(MicroArch::PentiumPro, "P6", Some(N600)),
            (0, 6, 0, 1, _) => brand_arch(MicroArch::PentiumPro, "P6", Some(N350)),

            // Pentium 2
            (0, 6, 0, 3, 2) => brand_arch(MicroArch::PentiumII, "P6T (Deschutes)", Some(N250)), // Pentium II Overdrive
            (0, 6, 0, 3, _) => brand_arch(MicroArch::PentiumII, "Klamath", Some(N280)),
            (0, 6, 0, 5, _) => brand_arch(MicroArch::PentiumII, "Deschutes", Some(N250)),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::PentiumII, "Dixon / Mendocino", Some(N250)),

            // Pentium 3
            (0, 6, 0, 7, _) => brand_arch(MicroArch::PentiumIII, "Katmai", Some(N250)),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::PentiumIII, "Coppermine", Some(N180)),
            (0, 6, 0, 9, _) => brand_arch(MicroArch::PentiumIII, "Banias", Some(N130)),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::PentiumIII, "Cascades", Some(N180)),
            (0, 6, 0, 11, _) => brand_arch(MicroArch::PentiumIII, "Tualatin", Some(N130)),
            (0, 6, 0, 12, _) => brand_arch(MicroArch::PentiumIII, "Timna", Some(N180)),

            // NetBurst (P4 / Xeon)
            (0, 15, 0, 0, _) => brand_arch(MicroArch::Willamette, "Willamette", Some(N180)),
            (0, 15, 0, 1, _) => brand_arch(MicroArch::Willamette, "Willamette/Foster", Some(N180)),
            (0, 15, 0, 2, _) => brand_arch(MicroArch::Northwood, "Northwood/Gallatin", Some(N130)),
            (0, 15, 0, 3, _) => brand_arch(MicroArch::Prescott, "Prescott", Some(N90)),
            (0, 15, 0, 4, _) => brand_arch(MicroArch::Prescott, "Prescott/Potomac", Some(N90)),
            (0, 15, 0, 6, _) => brand_arch(MicroArch::CedarMill, "Cedar Mill/Tulsa", Some(N64)),

            // Pentium M / Core / Core 2
            (0, 6, 0, 13, _) => brand_arch(MicroArch::Dothan, "Dothan", Some(N90)),
            (0, 6, 0, 14, _) => brand_arch(MicroArch::Yonah, "Yonah", Some(N65)),
            (0, 6, 0, 15, _) => Self::disambiguate_06_0fh(model, brand_arch),
            (0, 6, 1, 5, _) => brand_arch(MicroArch::Core, "Tolapai", Some(N65)),
            (0, 6, 1, 6, _) => brand_arch(MicroArch::Core, "Merom-L", Some(N65)),
            (0, 6, 1, 7, _) => Self::disambiguate_06_17h(model, brand_arch),
            (0, 6, 1, 13, _) => brand_arch(MicroArch::Dunnington, "Dunnington", Some(N45)),

            // Atom Lineage (Bonnell / Saltwell / Silvermont / Airmont / Goldmont / Tremont / Gracemont / Crestmont / Darkmont)
            (0, 6, 1, 12, _) => brand_arch(MicroArch::Bonnel, "Diamondville", Some(N45)),
            (0, 6, 2, 6, _) => brand_arch(MicroArch::Bonnel, "Lincroft", Some(N45)),
            (0, 6, 2, 7, _) => brand_arch(MicroArch::Saltwell, "Penwell", Some(N32)),
            (0, 6, 3, 5, _) => brand_arch(MicroArch::Saltwell, "Cloverview", Some(N32)),
            (0, 6, 3, 6, _) => brand_arch(MicroArch::Saltwell, "Cedarview", Some(N32)),
            (0, 6, 3, 7, _) => brand_arch(MicroArch::Silvermont, "Bay Trail", Some(N22)),
            (0, 6, 4, 10, _) => brand_arch(MicroArch::Silvermont, "Merrifield", Some(N22)),
            (0, 6, 4, 13, _) => brand_arch(MicroArch::Silvermont, "Avoton", Some(N22)),
            (0, 6, 5, 10, _) => brand_arch(MicroArch::Silvermont, "Moorefield", Some(N22)),
            (0, 6, 4, 12, _) => brand_arch(MicroArch::Airmont, "Braswell", Some(N14)),
            (0, 6, 7, 5, _) => brand_arch(MicroArch::Airmont, "Lightning Mountain", Some(N14)),
            (0, 6, 5, 12, _) => brand_arch(MicroArch::Goldmont, "Apollo Lake", Some(N14)),
            (0, 6, 5, 15, _) => brand_arch(MicroArch::Goldmont, "Denverton", Some(N14)),
            (0, 6, 7, 10, _) => brand_arch(MicroArch::GoldmontPlus, "Gemini Lake", Some(N14)),
            (0, 6, 8, 6, _) => brand_arch(MicroArch::Tremont, "Jacobsville", Some(N10)),
            (0, 6, 9, 6, _) => brand_arch(MicroArch::Tremont, "Elkhart Lake", Some(N10)),
            (0, 6, 9, 12, _) => brand_arch(MicroArch::Tremont, "Jasper Lake", Some(N10)),
            (0, 6, 11, 14, _) => Self::disambiguate_06_beh(model, brand_arch),
            (0, 6, 10, 15, _) => {
                brand_arch(MicroArch::SierraForest, "Sierra Forest", Some(INTEL_3))
            }
            (0, 6, 11, 6, _) => brand_arch(MicroArch::GrandRidge, "Grand Ridge", Some(INTEL_3)),
            (0, 6, 13, 13, _) => brand_arch(
                MicroArch::ClearwaterForest,
                "Clearwater Forest",
                Some(INTEL_18A),
            ),

            // Nehalem / Westmere (1st Gen Core)
            (0, 6, 1, 10, _) => brand_arch(MicroArch::Nehalem, "Bloomfield", Some(N45)),
            (0, 6, 1, 14, _) => brand_arch(MicroArch::Nehalem, "Lynnfield", Some(N45)),
            (0, 6, 1, 15, _) => brand_arch(MicroArch::Nehalem, "Auburndale", Some(N45)),
            (0, 6, 2, 14, _) => brand_arch(MicroArch::Nehalem, "Beckton (Nehalem-EX)", Some(N45)),
            (0, 6, 2, 5, _) => brand_arch(MicroArch::Westmere, "Clarkdale / Arrandale", Some(N32)),
            (0, 6, 2, 12, _) => {
                brand_arch(MicroArch::Westmere, "Gulftown / Westmere-EP", Some(N32))
            }
            (0, 6, 2, 15, _) => brand_arch(MicroArch::Westmere, "Westmere-EX", Some(N32)),

            // Sandy Bridge / Ivy Bridge (2nd & 3rd Gen Core)
            (0, 6, 2, 10, _) => brand_arch(MicroArch::SandyBridge, "Sandy Bridge", Some(N32)),
            (0, 6, 2, 13, _) => Self::disambiguate_hedt_server(
                model,
                MicroArch::SandyBridge,
                "Sandy Bridge-E",
                "Sandy Bridge-EP",
                Some(N32),
                brand_arch,
            ),
            (0, 6, 3, 10, _) => brand_arch(MicroArch::IvyBridge, "Ivy Bridge", Some(N22)),
            (0, 6, 3, 14, _) => Self::disambiguate_hedt_server(
                model,
                MicroArch::IvyBridge,
                "Ivy Bridge-E",
                "Ivy Bridge-EP",
                Some(N22),
                brand_arch,
            ),

            // Haswell / Broadwell (4th & 5th Gen Core)
            (0, 6, 3, 12, _) => brand_arch(MicroArch::Haswell, "Haswell", Some(N22)),
            (0, 6, 3, 15, _) => Self::disambiguate_hedt_server(
                model,
                MicroArch::Haswell,
                "Haswell-E",
                "Haswell-EP",
                Some(N22),
                brand_arch,
            ),
            (0, 6, 4, 5, _) => brand_arch(MicroArch::Haswell, "Haswell-ULT", Some(N22)),
            (0, 6, 4, 6, _) => brand_arch(MicroArch::Haswell, "Haswell-H", Some(N22)),
            (0, 6, 3, 13, _) => brand_arch(MicroArch::Broadwell, "Broadwell-U", Some(N14)),
            (0, 6, 4, 7, _) => brand_arch(MicroArch::Broadwell, "Broadwell-H", Some(N14)),
            (0, 6, 4, 15, _) => Self::disambiguate_hedt_server(
                model,
                MicroArch::Broadwell,
                "Broadwell-E",
                "Broadwell-EP",
                Some(N14),
                brand_arch,
            ),
            (0, 6, 5, 6, _) => brand_arch(MicroArch::Broadwell, "Broadwell-DE", Some(N14)),

            // Skylake Family & 14nm Refreshes (6th - 10th Gen Core)
            (0, 6, 4, 14, _) => brand_arch(MicroArch::Skylake, "Skylake-U", Some(N14)),
            (0, 6, 5, 14, _) => brand_arch(MicroArch::Skylake, "Skylake-S", Some(N14)),
            (0, 6, 5, 5, s_val) => Self::disambiguate_06_55h(model, s_val, brand_arch),
            (0, 6, 8, 14, s_val) => Self::disambiguate_06_8eh(model, s_val, brand_arch),
            (0, 6, 9, 14, s_val) => Self::disambiguate_06_9eh(model, s_val, brand_arch),
            (0, 6, 10, 5, _) => brand_arch(MicroArch::CometLake, "Comet Lake-S", Some(N14)),
            (0, 6, 10, 6, _) => brand_arch(MicroArch::CometLake, "Comet Lake-U", Some(N14)),
            (0, 6, 6, 6, _) => brand_arch(MicroArch::PalmCove, "Cannon Lake-U", Some(N10)),
            (0, 6, 10, 7, _) => brand_arch(MicroArch::RocketLake, "Rocket Lake-S", Some(N14)),

            // Ice Lake / Tiger Lake / Lakefield
            (0, 6, 7, 13, _) => brand_arch(MicroArch::IcyLake, "Ice Lake-Y", Some(N10)),
            (0, 6, 7, 14, _) => brand_arch(MicroArch::IcyLake, "Ice Lake-U", Some(N10)),
            (0, 6, 6, 10, _) => brand_arch(MicroArch::IcyLake, "Ice Lake-SP", Some(N10)),
            (0, 6, 6, 12, _) => brand_arch(MicroArch::IcyLake, "Ice Lake-D", Some(N10)),
            (0, 6, 9, 13, _) => brand_arch(MicroArch::IcyLake, "Ice Lake NNPI", Some(N10)),
            (0, 6, 8, 10, _) => brand_arch(MicroArch::Lakefield, "Lakefield", Some(N10)),
            (0, 6, 8, 12, _) => brand_arch(MicroArch::TigerLake, "Tiger Lake-UP3", Some(N10SF)),
            (0, 6, 8, 13, _) => brand_arch(MicroArch::TigerLake, "Tiger Lake-H", Some(N10SF)),

            // Alder Lake / Raptor Lake / Sapphire Rapids / Emerald Rapids
            (0, 6, 9, 7, _) => brand_arch(MicroArch::AlderLake, "Alder Lake-S", Some(INTEL_7)),
            (0, 6, 9, 10, _) => brand_arch(MicroArch::AlderLake, "Alder Lake-P/H", Some(INTEL_7)),
            (0, 6, 8, 15, _) => Self::disambiguate_06_8fh(model, brand_arch),
            (0, 6, 11, 7, _) => Self::disambiguate_06_b7h(model, brand_arch),
            (0, 6, 11, 10, _) => {
                brand_arch(MicroArch::RaptorLake, "Raptor Lake-P/U", Some(INTEL_7))
            }
            (0, 6, 11, 15, _) => brand_arch(MicroArch::RaptorLake, "Raptor Lake-S", Some(INTEL_7)),
            (0, 6, 12, 15, _) => {
                brand_arch(MicroArch::EmeraldRapids, "Emerald Rapids-SP", Some(INTEL_7))
            }
            (0, 6, 13, 7, _) => {
                brand_arch(MicroArch::BartlettLake, "Bartlett Lake-S", Some(INTEL_7))
            }

            // Meteor Lake / Granite Rapids
            (0, 6, 10, 10, _) => brand_arch(MicroArch::MeteorLake, "Meteor Lake-U", Some(INTEL_4)),
            (0, 6, 10, 12, _) => brand_arch(MicroArch::MeteorLake, "Meteor Lake-H", Some(INTEL_4)),
            (0, 6, 10, 13, _) => {
                brand_arch(MicroArch::GraniteRapids, "Granite Rapids-SP", Some(INTEL_3))
            }
            (0, 6, 10, 14, _) => {
                brand_arch(MicroArch::GraniteRapids, "Granite Rapids-D", Some(INTEL_3))
            }

            // Lunar Lake / Arrow Lake
            (0, 6, 11, 13, _) => brand_arch(MicroArch::LunarLake, "Lunar Lake-M", Some(N3)),
            (0, 6, 12, 5, _) => brand_arch(MicroArch::ArrowLake, "Arrow Lake-H", Some(N3)),
            (0, 6, 12, 6, _) => brand_arch(MicroArch::ArrowLake, "Arrow Lake-S", Some(N3)),
            (0, 6, 11, 5, _) => brand_arch(MicroArch::ArrowLake, "Arrow Lake-U", Some(INTEL_3)),

            // Panther Lake / Next-Gen
            (0, 6, 12, 12, _) => {
                brand_arch(MicroArch::PantherLake, "Panther Lake-L", Some(INTEL_18A))
            }
            (0, 6, 14, 5, _) => {
                brand_arch(MicroArch::PantherLake, "Panther Lake-R", Some(INTEL_18A))
            }
            (0, 6, 13, 5, _) => brand_arch(MicroArch::WildcatLake, "Wildcat Lake", Some(INTEL_18A)),

            // Xeon Phi Lineage
            (0, 6, 5, 7, _) => brand_arch(MicroArch::KnightsLanding, "Knights Landing", Some(N14)),
            (0, 6, 8, 5, _) => brand_arch(MicroArch::KnightsMill, "Knights Mill", Some(N14)),

            // Family 18 (Nova Lake) & Family 19 (Diamond Rapids)
            (3, 15, 0, 1, _) => brand_arch(MicroArch::NovaLake, "Nova Lake", None),
            (3, 15, 0, 3, _) => brand_arch(MicroArch::NovaLake, "Nova Lake-L", None),
            (4, 15, 0, 1, _) => brand_arch(MicroArch::DiamondRapids, "Diamond Rapids-X", None),

            _ => {
                if s.display_family == 18 {
                    if s.display_model == 1 {
                        brand_arch(MicroArch::NovaLake, "Nova Lake", None)
                    } else {
                        brand_arch(MicroArch::NovaLake, "Nova Lake-L", None)
                    }
                } else if s.display_family == 19 {
                    brand_arch(MicroArch::DiamondRapids, "Diamond Rapids-X", None)
                } else {
                    return None;
                }
            }
        };
        Some(arch)
    }
}

impl TMicroArch for Intel {
    /// Detects the Intel microarchitecture based on the CPU model string and signature.
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand_arch = CpuArch::brand_arch(model, "Intel", VENDOR_INTEL);

        if let Some(arch) = Self::legacy_micro_arch(s, &brand_arch) {
            return arch;
        }

        #[cfg(not(dos))]
        if let Some(arch) = Self::modern_micro_arch(model, s, &brand_arch) {
            return arch;
        }

        brand_arch(MicroArch::Unknown, UNK, None)
    }
}

#[cfg(any(not(nostd_os), target_os = "uefi"))]
impl Intel {
    pub fn core_micro_arch(parent: MicroArch, core_type: CoreType) -> MicroArch {
        match (parent, core_type) {
            (MicroArch::Lakefield, CoreType::Performance) => MicroArch::SunnyCove,
            (MicroArch::Lakefield, CoreType::Efficiency) => MicroArch::Tremont,

            (MicroArch::AlderLake, CoreType::Performance) => MicroArch::GoldenCove,
            (MicroArch::AlderLake, CoreType::Efficiency) => MicroArch::Gracemont,

            (MicroArch::RaptorLake, CoreType::Performance) => MicroArch::RaptorCove,
            (MicroArch::RaptorLake, CoreType::Efficiency) => MicroArch::Gracemont,

            (MicroArch::MeteorLake, CoreType::Performance) => MicroArch::RedwoodCove,
            (MicroArch::MeteorLake, CoreType::Efficiency) => MicroArch::Crestmont,

            (MicroArch::ArrowLake | MicroArch::LunarLake, CoreType::Performance) => {
                MicroArch::LionCove
            }
            (MicroArch::ArrowLake | MicroArch::LunarLake, CoreType::Efficiency) => {
                MicroArch::Skymont
            }

            (MicroArch::PantherLake, CoreType::Performance) => MicroArch::CougarCove,
            (MicroArch::PantherLake, CoreType::Efficiency) => MicroArch::Darkmont,

            (MicroArch::SapphireRapids, _) => MicroArch::GoldenCove,
            (MicroArch::EmeraldRapids, _) => MicroArch::RaptorCove,
            (MicroArch::GraniteRapids, _) => MicroArch::RedwoodCove,
            (MicroArch::SierraForest | MicroArch::GrandRidge, _) => MicroArch::Crestmont,
            (MicroArch::ClearwaterForest, _) => MicroArch::Darkmont,

            _ => MicroArch::Unknown,
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
        let sig_i486 = crate::x86::micro_arch::tests::dummy_signature(4, 0, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_i486);
        assert_eq!(arch.micro_arch, MicroArch::I486);
        assert_eq!(arch.code_name, "i80486DX");

        // P5 (MicroArch::Pentium)
        let sig_p5 = crate::x86::micro_arch::tests::dummy_signature(5, 2, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_p5);
        assert_eq!(arch.micro_arch, MicroArch::P5);
        assert_eq!(arch.code_name, "P54C");

        // Nehalem Lynnfield
        let sig_nehalem = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 1, 5);
        let arch = Intel::micro_arch(model, sig_nehalem);
        assert_eq!(arch.micro_arch, MicroArch::Nehalem);
        assert_eq!(arch.code_name, "Lynnfield");

        // Core 2 Era (65nm - 06_0FH)
        let sig_06_0f = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 0, 6);
        let arch_conroe = Intel::micro_arch("Intel Core 2 Duo E6850", sig_06_0f);
        assert_eq!(arch_conroe.micro_arch, MicroArch::Core);
        assert_eq!(arch_conroe.code_name, "Conroe");

        let arch_kentsfield = Intel::micro_arch("Intel Core 2 Quad Q6600", sig_06_0f);
        assert_eq!(arch_kentsfield.micro_arch, MicroArch::Core);
        assert_eq!(arch_kentsfield.code_name, "Kentsfield");

        let arch_merom = Intel::micro_arch("Intel Core 2 Duo T7700", sig_06_0f);
        assert_eq!(arch_merom.micro_arch, MicroArch::Core);
        assert_eq!(arch_merom.code_name, "Merom");

        let arch_woodcrest = Intel::micro_arch("Intel Xeon 5160", sig_06_0f);
        assert_eq!(arch_woodcrest.micro_arch, MicroArch::Core);
        assert_eq!(arch_woodcrest.code_name, "Woodcrest");

        let arch_clovertown = Intel::micro_arch("Intel Xeon X5355", sig_06_0f);
        assert_eq!(arch_clovertown.micro_arch, MicroArch::Core);
        assert_eq!(arch_clovertown.code_name, "Clovertown");

        let arch_tigerton = Intel::micro_arch("Intel Xeon E7340", sig_06_0f);
        assert_eq!(arch_tigerton.micro_arch, MicroArch::Core);
        assert_eq!(arch_tigerton.code_name, "Tigerton");

        // Core 2 Era (45nm - 06_17H)
        let sig_06_17 = crate::x86::micro_arch::tests::dummy_signature(6, 7, 0, 1, 6);
        let arch_wolfdale = Intel::micro_arch("Intel Core 2 Duo E8400", sig_06_17);
        assert_eq!(arch_wolfdale.micro_arch, MicroArch::Core);
        assert_eq!(arch_wolfdale.code_name, "Wolfdale");

        let arch_yorkfield = Intel::micro_arch("Intel Core 2 Quad Q9550", sig_06_17);
        assert_eq!(arch_yorkfield.micro_arch, MicroArch::Core);
        assert_eq!(arch_yorkfield.code_name, "Yorkfield");

        let arch_penryn = Intel::micro_arch("Intel Core 2 Duo P8600", sig_06_17);
        assert_eq!(arch_penryn.micro_arch, MicroArch::Core);
        assert_eq!(arch_penryn.code_name, "Penryn");

        let arch_wolfdale_dp = Intel::micro_arch("Intel Xeon X5260", sig_06_17);
        assert_eq!(arch_wolfdale_dp.micro_arch, MicroArch::Core);
        assert_eq!(arch_wolfdale_dp.code_name, "Wolfdale-DP");

        let arch_harpertown = Intel::micro_arch("Intel Xeon E5450", sig_06_17);
        assert_eq!(arch_harpertown.micro_arch, MicroArch::Core);
        assert_eq!(arch_harpertown.code_name, "Harpertown");

        // Sandy Bridge
        let sig_sandy = crate::x86::micro_arch::tests::dummy_signature(6, 10, 0, 2, 7);
        let arch = Intel::micro_arch("Intel Core i7-2600K", sig_sandy);
        assert_eq!(arch.micro_arch, MicroArch::SandyBridge);
        assert_eq!(arch.code_name, "Sandy Bridge");

        // Skylake Desktop
        let sig_skylake = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 5, 3);
        let arch = Intel::micro_arch("Intel Core i7-6700K", sig_skylake);
        assert_eq!(arch.micro_arch, MicroArch::Skylake);
        assert_eq!(arch.code_name, "Skylake-S");

        // Skylake Server (06_55H stepping 4)
        let sig_skylake_sp = crate::x86::micro_arch::tests::dummy_signature(6, 5, 0, 5, 4);
        let arch = Intel::micro_arch("Intel Xeon Platinum 8180", sig_skylake_sp);
        assert_eq!(arch.micro_arch, MicroArch::Skylake);
        assert_eq!(arch.code_name, "Skylake-SP");

        // Cascade Lake Server (06_55H stepping 7)
        let sig_cascade = crate::x86::micro_arch::tests::dummy_signature(6, 5, 0, 5, 7);
        let arch = Intel::micro_arch("Intel Xeon Platinum 8280", sig_cascade);
        assert_eq!(arch.micro_arch, MicroArch::CascadeLake);
        assert_eq!(arch.code_name, "Cascade Lake-SP");

        // Cooper Lake Server (06_55H stepping 11)
        let sig_cooper = crate::x86::micro_arch::tests::dummy_signature(6, 5, 0, 5, 11);
        let arch = Intel::micro_arch("Intel Xeon Platinum 8380H", sig_cooper);
        assert_eq!(arch.micro_arch, MicroArch::CooperLake);
        assert_eq!(arch.code_name, "Cooper Lake");

        // Amber Lake-Y (06_8EH stepping 9)
        let sig_amber = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 8, 9);
        let arch = Intel::micro_arch("Intel Core m3-8100Y", sig_amber);
        assert_eq!(arch.micro_arch, MicroArch::AmberLake);
        assert_eq!(arch.code_name, "Amber Lake-Y");

        // Coffee Lake Desktop (06_9EH stepping 10)
        let sig_coffee = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 9, 10);
        let arch = Intel::micro_arch("Intel Core i7-8700K", sig_coffee);
        assert_eq!(arch.micro_arch, MicroArch::CoffeeLake);
        assert_eq!(arch.code_name, "Coffee Lake-S");

        // Coffee Lake Refresh (06_9EH stepping 12)
        let sig_coffee_r = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 9, 12);
        let arch = Intel::micro_arch("Intel Core i9-9900K", sig_coffee_r);
        assert_eq!(arch.micro_arch, MicroArch::CoffeeLake);
        assert_eq!(arch.code_name, "Coffee Lake-S Refresh");

        // Comet Lake Desktop (06_A5H)
        let sig_comet = crate::x86::micro_arch::tests::dummy_signature(6, 5, 0, 10, 1);
        let arch = Intel::micro_arch("Intel Core i9-10900K", sig_comet);
        assert_eq!(arch.micro_arch, MicroArch::CometLake);
        assert_eq!(arch.code_name, "Comet Lake-S");

        // Rocket Lake Desktop (06_A7H)
        let sig_rocket = crate::x86::micro_arch::tests::dummy_signature(6, 7, 0, 10, 1);
        let arch = Intel::micro_arch("Intel Core i9-11900K", sig_rocket);
        assert_eq!(arch.micro_arch, MicroArch::RocketLake);
        assert_eq!(arch.code_name, "Rocket Lake-S");

        // Tiger Lake Mobile (06_8CH)
        let sig_tiger = crate::x86::micro_arch::tests::dummy_signature(6, 12, 0, 8, 1);
        let arch = Intel::micro_arch("Intel Core i7-1165G7", sig_tiger);
        assert_eq!(arch.micro_arch, MicroArch::TigerLake);
        assert_eq!(arch.code_name, "Tiger Lake-UP3");

        // Alder Lake Mobile (06_9AH)
        let sig_alder = crate::x86::micro_arch::tests::dummy_signature(6, 10, 0, 9, 3);
        let arch = Intel::micro_arch("Intel Core i7-12700H", sig_alder);
        assert_eq!(arch.micro_arch, MicroArch::AlderLake);
        assert_eq!(arch.code_name, "Alder Lake-P/H");

        // Alder Lake-N (06_BEH)
        let sig_alder_n = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 11, 0);
        let arch = Intel::micro_arch("Intel Processor N100", sig_alder_n);
        assert_eq!(arch.micro_arch, MicroArch::AlderLake);
        assert_eq!(arch.code_name, "Alder Lake-N");

        // Twin Lake-N (06_BEH with N150)
        let arch_twin = Intel::micro_arch("Intel Processor N150", sig_alder_n);
        assert_eq!(arch_twin.micro_arch, MicroArch::TwinLake);
        assert_eq!(arch_twin.code_name, "Twin Lake-N");

        // Raptor Lake Desktop (06_B7H)
        let sig_raptor = crate::x86::micro_arch::tests::dummy_signature(6, 7, 0, 11, 1);
        let arch = Intel::micro_arch("Intel Core i9-13900K", sig_raptor);
        assert_eq!(arch.micro_arch, MicroArch::RaptorLake);
        assert_eq!(arch.code_name, "Raptor Lake-S");

        // Raptor Lake Refresh (06_B7H with 14900K)
        let arch_raptor_r = Intel::micro_arch("Intel Core i9-14900K", sig_raptor);
        assert_eq!(arch_raptor_r.micro_arch, MicroArch::RaptorLake);
        assert_eq!(arch_raptor_r.code_name, "Raptor Lake-S Refresh");

        // Sapphire Rapids Server (06_8FH)
        let sig_spr = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 8, 8);
        let arch = Intel::micro_arch("Intel Xeon Platinum 8480+", sig_spr);
        assert_eq!(arch.micro_arch, MicroArch::SapphireRapids);
        assert_eq!(arch.code_name, "Sapphire Rapids-SP");

        // Emerald Rapids Server (06_CFH)
        let sig_emr = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 12, 0);
        let arch = Intel::micro_arch("Intel Xeon Platinum 8592+", sig_emr);
        assert_eq!(arch.micro_arch, MicroArch::EmeraldRapids);
        assert_eq!(arch.code_name, "Emerald Rapids-SP");

        // Meteor Lake Mobile (06_ACH)
        let sig_mtl = crate::x86::micro_arch::tests::dummy_signature(6, 12, 0, 10, 4);
        let arch = Intel::micro_arch("Intel Core Ultra 7 155H", sig_mtl);
        assert_eq!(arch.micro_arch, MicroArch::MeteorLake);
        assert_eq!(arch.code_name, "Meteor Lake-H");

        // Lunar Lake Mobile (06_BDH)
        let sig_lnl = crate::x86::micro_arch::tests::dummy_signature(6, 13, 0, 11, 1);
        let arch = Intel::micro_arch("Intel Core Ultra 7 258V", sig_lnl);
        assert_eq!(arch.micro_arch, MicroArch::LunarLake);
        assert_eq!(arch.code_name, "Lunar Lake-M");

        // Arrow Lake Desktop (06_C6H)
        let sig_arl = crate::x86::micro_arch::tests::dummy_signature(6, 6, 0, 12, 2);
        let arch = Intel::micro_arch("Intel Core Ultra 9 285K", sig_arl);
        assert_eq!(arch.micro_arch, MicroArch::ArrowLake);
        assert_eq!(arch.code_name, "Arrow Lake-S");

        // Granite Rapids Server (06_ADH)
        let sig_gnr = crate::x86::micro_arch::tests::dummy_signature(6, 13, 0, 10, 0);
        let arch = Intel::micro_arch("Intel Xeon 6 6980P", sig_gnr);
        assert_eq!(arch.micro_arch, MicroArch::GraniteRapids);
        assert_eq!(arch.code_name, "Granite Rapids-SP");

        // Sierra Forest Server (06_AFH)
        let sig_srf = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 10, 0);
        let arch = Intel::micro_arch("Intel Xeon 6 6780E", sig_srf);
        assert_eq!(arch.micro_arch, MicroArch::SierraForest);
        assert_eq!(arch.code_name, "Sierra Forest");

        // HEDT vs Enterprise Server Disambiguation: Sandy Bridge (06_2DH)
        let sig_snbe = crate::x86::micro_arch::tests::dummy_signature(6, 13, 0, 2, 7);
        let arch_snbe = Intel::micro_arch("Intel Core i7-3960X", sig_snbe);
        assert_eq!(arch_snbe.micro_arch, MicroArch::SandyBridge);
        assert_eq!(arch_snbe.code_name, "Sandy Bridge-E");
        let arch_snbep = Intel::micro_arch("Intel Xeon E5-2690", sig_snbe);
        assert_eq!(arch_snbep.micro_arch, MicroArch::SandyBridge);
        assert_eq!(arch_snbep.code_name, "Sandy Bridge-EP");

        // HEDT vs Enterprise Server Disambiguation: Ivy Bridge (06_3EH)
        let sig_ivbe = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 3, 4);
        let arch_ivbe = Intel::micro_arch("Intel Core i7-4960X", sig_ivbe);
        assert_eq!(arch_ivbe.micro_arch, MicroArch::IvyBridge);
        assert_eq!(arch_ivbe.code_name, "Ivy Bridge-E");
        let arch_ivbep = Intel::micro_arch("Intel Xeon E5-2697 v2", sig_ivbe);
        assert_eq!(arch_ivbep.micro_arch, MicroArch::IvyBridge);
        assert_eq!(arch_ivbep.code_name, "Ivy Bridge-EP");

        // HEDT vs Enterprise Server Disambiguation: Haswell (06_3FH)
        let sig_hswe = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 3, 2);
        let arch_hswe = Intel::micro_arch("Intel Core i7-5960X", sig_hswe);
        assert_eq!(arch_hswe.micro_arch, MicroArch::Haswell);
        assert_eq!(arch_hswe.code_name, "Haswell-E");
        let arch_hswep = Intel::micro_arch("Intel Xeon E5-2699 v3", sig_hswe);
        assert_eq!(arch_hswep.micro_arch, MicroArch::Haswell);
        assert_eq!(arch_hswep.code_name, "Haswell-EP");

        // HEDT vs Enterprise Server Disambiguation: Broadwell (06_4FH)
        let sig_bdwe = crate::x86::micro_arch::tests::dummy_signature(6, 15, 0, 4, 1);
        let arch_bdwe = Intel::micro_arch("Intel Core i7-6950X", sig_bdwe);
        assert_eq!(arch_bdwe.micro_arch, MicroArch::Broadwell);
        assert_eq!(arch_bdwe.code_name, "Broadwell-E");
        let arch_bdwep = Intel::micro_arch("Intel Xeon E5-2699 v4", sig_bdwe);
        assert_eq!(arch_bdwep.micro_arch, MicroArch::Broadwell);
        assert_eq!(arch_bdwep.code_name, "Broadwell-EP");

        // Sapphire Rapids-WS & Xeon Max
        let arch_spr_ws = Intel::micro_arch("Intel Xeon w9-3495X", sig_spr);
        assert_eq!(arch_spr_ws.micro_arch, MicroArch::SapphireRapids);
        assert_eq!(arch_spr_ws.code_name, "Sapphire Rapids-WS");
        let arch_spr_max = Intel::micro_arch("Intel Xeon Max 9480", sig_spr);
        assert_eq!(arch_spr_max.micro_arch, MicroArch::SapphireRapids);
        assert_eq!(arch_spr_max.code_name, "Sapphire Rapids-HBM (Xeon Max)");

        // Whiskey Lake vs Comet Lake-U (06_8EH)
        let sig_whl = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 8, 11);
        let arch_whl = Intel::micro_arch("Intel Core i7-8565U", sig_whl);
        assert_eq!(arch_whl.micro_arch, MicroArch::WhiskyLake);
        assert_eq!(arch_whl.code_name, "Whiskey Lake-U");

        let sig_cml_u = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 8, 12);
        let arch_cml_u = Intel::micro_arch("Intel Core i7-10510U", sig_cml_u);
        assert_eq!(arch_cml_u.micro_arch, MicroArch::CometLake);
        assert_eq!(arch_cml_u.code_name, "Comet Lake-U");

        // Kaby Lake-X (06_9EH)
        let sig_kbl_x = crate::x86::micro_arch::tests::dummy_signature(6, 14, 0, 9, 9);
        let arch_kbl_x = Intel::micro_arch("Intel Core i7-7740X", sig_kbl_x);
        assert_eq!(arch_kbl_x.micro_arch, MicroArch::KabyLake);
        assert_eq!(arch_kbl_x.code_name, "Kaby Lake-X");

        // Xeon Phi: Knights Landing & Knights Mill
        let sig_knl = crate::x86::micro_arch::tests::dummy_signature(6, 7, 0, 5, 1);
        let arch_knl = Intel::micro_arch("Intel Xeon Phi 7250", sig_knl);
        assert_eq!(arch_knl.micro_arch, MicroArch::KnightsLanding);
        assert_eq!(arch_knl.code_name, "Knights Landing");

        let sig_knm = crate::x86::micro_arch::tests::dummy_signature(6, 5, 0, 8, 1);
        let arch_knm = Intel::micro_arch("Intel Xeon Phi 7295", sig_knm);
        assert_eq!(arch_knm.micro_arch, MicroArch::KnightsMill);
        assert_eq!(arch_knm.code_name, "Knights Mill");

        // Family 18 (Nova Lake) & Family 19 (Diamond Rapids)
        let sig_nova = crate::x86::micro_arch::tests::dummy_signature(15, 1, 3, 0, 0);
        let arch_nova = Intel::micro_arch("Intel Nova Lake", sig_nova);
        assert_eq!(arch_nova.micro_arch, MicroArch::NovaLake);
        assert_eq!(arch_nova.code_name, "Nova Lake");

        let sig_dmr = crate::x86::micro_arch::tests::dummy_signature(15, 1, 4, 0, 0);
        let arch_dmr = Intel::micro_arch("Intel Diamond Rapids", sig_dmr);
        assert_eq!(arch_dmr.micro_arch, MicroArch::DiamondRapids);
        assert_eq!(arch_dmr.code_name, "Diamond Rapids-X");

        // Unknown Intel
        let sig_unknown = crate::x86::micro_arch::tests::dummy_signature(99, 0, 0, 0, 0);
        let arch = Intel::micro_arch(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    fn test_core_micro_arch_hybrid() {
        // Alder Lake E-core bug verification: must return Gracemont, NOT Goldmont
        assert_eq!(
            Intel::core_micro_arch(MicroArch::AlderLake, CoreType::Efficiency),
            MicroArch::Gracemont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::AlderLake, CoreType::Performance),
            MicroArch::GoldenCove
        );

        // Raptor Lake
        assert_eq!(
            Intel::core_micro_arch(MicroArch::RaptorLake, CoreType::Efficiency),
            MicroArch::Gracemont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::RaptorLake, CoreType::Performance),
            MicroArch::RaptorCove
        );

        // Meteor Lake
        assert_eq!(
            Intel::core_micro_arch(MicroArch::MeteorLake, CoreType::Efficiency),
            MicroArch::Crestmont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::MeteorLake, CoreType::Performance),
            MicroArch::RedwoodCove
        );

        // Arrow Lake & Lunar Lake
        assert_eq!(
            Intel::core_micro_arch(MicroArch::ArrowLake, CoreType::Efficiency),
            MicroArch::Skymont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::ArrowLake, CoreType::Performance),
            MicroArch::LionCove
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::LunarLake, CoreType::Efficiency),
            MicroArch::Skymont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::LunarLake, CoreType::Performance),
            MicroArch::LionCove
        );

        // Panther Lake
        assert_eq!(
            Intel::core_micro_arch(MicroArch::PantherLake, CoreType::Efficiency),
            MicroArch::Darkmont
        );
        assert_eq!(
            Intel::core_micro_arch(MicroArch::PantherLake, CoreType::Performance),
            MicroArch::CougarCove
        );
    }
}
