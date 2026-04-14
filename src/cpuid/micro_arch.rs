//! CPU Microarchitecture Detection
//!
//! This module provides microarchitecture detection and identification
//! for x86/x86_64 processors based on CPU signature and vendor information.

use super::constants::*;
use super::vendor::TMicroArch;
use super::vendor::*;
use super::{CpuBrand, CpuSignature, Str, is_centaur, is_zhaoxin};

/// CPU Microarchitecture enumeration.
///
/// Lists all known x86/x86_64 microarchitectures from various vendors
/// including Intel, AMD, VIA/Centaur, Cyrix, and others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    Unknown,

    // AMD
    Am486,
    Am5x86,
    SSA5,
    K5,
    K6,
    K7,
    K8,
    K10,
    Bobcat,
    Puma2008,
    Bulldozer,
    Piledriver,
    Steamroller,
    Excavator,
    Jaguar,
    Puma2014,
    Zen,
    ZenPlus,
    Zen2,
    Zen3,
    Zen3Plus,
    Zen4,
    Zen4C,
    Zen5,
    Zen5C,

    // Centaur (IDT)
    Winchip,
    Winchip2,
    Winchip2A,
    Winchip2B,
    Winchip3,

    // Centaur (Via)
    Samuel,
    Samuel2,
    Ezra,
    EzraT,
    Nehemiah,
    NehemiahP,
    Esther,
    Isaiah,

    // Centaur (Zhaoxin)
    ZhangJiang,
    Wudaokou,
    Lujiazui,

    // Cyrix
    Cx486DX,
    Cx486S,
    Cx486DLC,
    Cy5x86,
    M1,
    M2,
    MediaGx,
    Geode, // Cyrix/NatSemi

    // DM&P
    VortexDX,
    VortexMX,
    VortexDX3,

    // Intel
    I486,
    RapidCad,
    P5,
    Lakemont,
    // ! P6 start
    // These are the same MicroArch, but the distinction is handy for generating
    // separate model strings, since they don't have one set
    PentiumPro,
    PentiumII,
    PentiumIII,
    // ! P6 end
    Dothan,
    Yonah,
    Merom,
    Penryn,
    Nehalem,
    Westmere,
    Bonnel,
    Dunnington,
    Core,
    Saltwell,
    Silvermont,
    SandyBridge,
    IvyBridge,
    Haswell,
    Broadwell,
    Airmont,
    KabyLake,
    Skylake,
    CascadeLake,
    KnightsLanding,
    Goldmont,
    PalmCove,
    SunnyCove,
    GoldmontPlus,
    IcyLake,
    Tremont,
    TigerLake,
    WhiskyLake,
    SapphireRapids,
    AlderLake,
    CoffeeLake,
    CometLake,
    RaptorLake,
    KnightsFerry,
    KnightsCorner,
    Willamette,
    Northwood,
    Prescott,
    CedarMill,
    AmberLake,

    // RDC
    Iad,

    // Rise
    MP6,
    MP62,

    // SiS
    SiS55x,

    // Transmeta
    Crusoe,
    Efficeon,

    // UMC
    U5S,
    U5D,
}

impl From<MicroArch> for Str<32> {
    fn from(ma: MicroArch) -> Str<32> {
        let s = match ma {
            MicroArch::Unknown => UNK,

            // AMD
            MicroArch::Am486 => "Am486",
            MicroArch::Am5x86 => "Am5x86",
            MicroArch::SSA5 => "SSA5",
            MicroArch::K5 => "K5",
            MicroArch::K6 => "K6",
            MicroArch::K7 => "K7",
            MicroArch::K8 => "K8",
            MicroArch::K10 => "K10",
            MicroArch::Bobcat => "Bobcat",
            MicroArch::Puma2008 => "Puma2008",
            MicroArch::Bulldozer => "Bulldozer",
            MicroArch::Piledriver => "Piledriver",
            MicroArch::Steamroller => "Steamroller",
            MicroArch::Excavator => "Excavator",
            MicroArch::Jaguar => "Jaguar",
            MicroArch::Puma2014 => "Puma2014",
            MicroArch::Zen => "Zen",
            MicroArch::ZenPlus => "Zen+",
            MicroArch::Zen2 => "Zen 2",
            MicroArch::Zen3 => "Zen 3",
            MicroArch::Zen3Plus => "Zen 3+",
            MicroArch::Zen4 => "Zen 4",
            MicroArch::Zen4C => "Zen 4C",
            MicroArch::Zen5 => "Zen 5",
            MicroArch::Zen5C => "Zen 5C",

            // Centaur (IDT)
            MicroArch::Winchip => "Winchip",
            MicroArch::Winchip2 => "Winchip 2",
            MicroArch::Winchip2A => "Winchip 2A",
            MicroArch::Winchip2B => "Winchip 2B",
            MicroArch::Winchip3 => "Winchip 3",

            // Centaur (VIA)
            MicroArch::Samuel => "Samuel",
            MicroArch::Samuel2 => "Samuel 2",
            MicroArch::Ezra => "Ezra",
            MicroArch::EzraT => "EzraT",
            MicroArch::Nehemiah => "Nehemiah",
            MicroArch::NehemiahP => "Nehemiah P",
            MicroArch::Esther => "Esther",
            MicroArch::Isaiah => "Isaiah",

            // Centaur (Zhaoxin)
            MicroArch::ZhangJiang => "ZhangJiang",
            MicroArch::Wudaokou => "WuDaoKou",
            MicroArch::Lujiazui => "LuJiaZui",

            // Cyrix
            MicroArch::Cx486DLC => "Cx486DLC",
            MicroArch::Cx486DX => "Cx486DX",
            MicroArch::Cx486S => "Cx486S",
            MicroArch::Cy5x86 => "Cx5x86",
            MicroArch::M1 => "M1",
            MicroArch::M2 => "M2",
            MicroArch::MediaGx => "MediaGX",
            MicroArch::Geode => "Geode",

            // DM& P
            MicroArch::VortexDX => "Vortex86DX",
            MicroArch::VortexMX => "Vortex86MX",
            MicroArch::VortexDX3 => "Vortex86DX3",

            // Intel
            MicroArch::I486 => "i486",
            MicroArch::RapidCad => "RapidCad",
            MicroArch::P5 => "P5",
            MicroArch::Lakemont => "Lakemont",
            MicroArch::PentiumPro | MicroArch::PentiumII | MicroArch::PentiumIII => "P6",
            MicroArch::Dothan => "Dothan",
            MicroArch::Yonah => "Yonah",
            MicroArch::Merom => "Merom",
            MicroArch::Penryn => "Penryn",
            MicroArch::Nehalem => "Nehalem",
            MicroArch::Westmere => "Westmere",
            MicroArch::Bonnel => "Bonnel",
            MicroArch::Dunnington => "Dunnington",
            MicroArch::Core => "Core",
            MicroArch::Saltwell => "Saltwell",
            MicroArch::Silvermont => "Silvermont",
            MicroArch::SandyBridge => "Sandy Bridge",
            MicroArch::IvyBridge => "Ivy Bridge",
            MicroArch::Haswell => "Haswell",
            MicroArch::Broadwell => "Broadwell",
            MicroArch::Airmont => "Airmont",
            MicroArch::KabyLake => "Kaby Lake",
            MicroArch::Skylake => "Skylake",
            MicroArch::CascadeLake => "Cascade Lake",
            MicroArch::KnightsLanding => "Knights Landing",
            MicroArch::Goldmont => "Goldmont",
            MicroArch::PalmCove => "Palm Cove",
            MicroArch::SunnyCove => "Sunny Cove",
            MicroArch::GoldmontPlus => "Goldmont Plus",
            MicroArch::IcyLake => "Icy Lake",
            MicroArch::Tremont => "Tremont",
            MicroArch::TigerLake => "Tiger Lake",
            MicroArch::WhiskyLake => "Whisky Lake",
            MicroArch::SapphireRapids => "Sapphire Rapids",
            MicroArch::AlderLake => "Alder Lake",
            MicroArch::CoffeeLake => "Coffee Lake",
            MicroArch::CometLake => "Comet Lake",
            MicroArch::RaptorLake => "Raptor Lake",
            MicroArch::KnightsFerry => "Knights Ferry",
            MicroArch::KnightsCorner => "Knights Corner",
            MicroArch::Willamette => "Willamette",
            MicroArch::Northwood => "Northwood",
            MicroArch::Prescott => "Prescott",
            MicroArch::CedarMill => "Cedar Mill",
            MicroArch::AmberLake => "Amber Lake",

            // RDC
            MicroArch::Iad => "Iad",

            // Rise
            MicroArch::MP6 => "mP6",
            MicroArch::MP62 => "mP6-II",

            // SiS
            MicroArch::SiS55x => "SiS55x",

            // Transmeta
            MicroArch::Crusoe => "Crusoe",
            MicroArch::Efficeon => "Efficeon",

            // UMC
            MicroArch::U5S => "U5S",
            MicroArch::U5D => "U5D",
        };

        Str::from(s)
    }
}

/// Complete CPU architecture information.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    /// CPU model string
    pub model: Str<64>,
    /// Microarchitecture family
    pub micro_arch: MicroArch,
    /// Specific code name (e.g., "Skylake", "Zen 3")
    pub code_name: &'static str,
    /// Brand name (e.g., "Intel", "AMD")
    pub brand_name: Str<32>,
    /// Raw vendor string from CPUID
    pub vendor_string: Str<12>,
    /// Process technology node (e.g., "14nm", "7nm")
    pub technology: Option<Str<8>>,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(UNK, MicroArch::Unknown, UNK, UNK, UNK, None)
    }
}

impl CpuArch {
    /// Creates a new CpuArch with the specified parameters.
    pub fn new(
        model: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        brand_name: &str,
        vendor_string: &str,
        technology: Option<&str>,
    ) -> Self {
        let model_s: Str<64> = Str::from(model);
        let brand_s: Str<32> = Str::from(brand_name);
        let vendor_s: Str<12> = Str::from(vendor_string);

        let technology = technology.map(Str::from);

        CpuArch {
            model: model_s,
            micro_arch,
            code_name,
            brand_name: brand_s,
            vendor_string: vendor_s,
            technology,
        }
    }

    /// Finds and returns the CPU architecture based on model string, signature, and vendor.
    ///
    /// Uses CPUID information to determine the microarchitecture and code name.
    pub fn find(model: &str, s: CpuSignature, vendor_string: &str) -> Self {
        let arch = |ma: MicroArch,
                    code_name: &'static str,
                    brand_name: &str,
                    tech: Option<&str>|
         -> Self {
            CpuArch::new(model, ma, code_name, brand_name, vendor_string, tech)
        };

        let brand = CpuBrand::from(vendor_string);
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> Self {
            arch(ma, code_name, brand.to_brand_name(), tech)
        };

        let unknown_model = brand_arch(MicroArch::Unknown, UNK, None);

        // Brand for Centaur CPUs is by signature, not vendor string
        if is_centaur() || is_zhaoxin() {
            return Centaur::micro_arch(model, s);
        }

        #[cfg(target_arch = "x86")]
        match brand {
            CpuBrand::AMD => Amd::micro_arch(model, s),

            CpuBrand::Intel => Intel::micro_arch(model, s),

            CpuBrand::Cyrix => Cyrix::micro_arch(model, s),

            CpuBrand::NationalSemiconductor => match (s.family, s.model, s.stepping) {
                (5, 4, _) => brand_arch(MicroArch::Geode, "GX1", Some(N180)),
                (5, 9, _) => brand_arch(MicroArch::Geode, "GX2", Some(N180)),
                (5, 10, _) => brand_arch(MicroArch::Geode, "GX3", None),
                _ => unknown_model,
            },

            // From sandpile.org
            CpuBrand::Rdc => match (s.family, s.model, s.stepping) {
                (5, 8, _) => brand_arch(MicroArch::Iad, "Iad", None),
                _ => unknown_model,
            },

            CpuBrand::Rise => match (s.family, s.model, s.stepping) {
                (5, 0, _) => brand_arch(MicroArch::MP6, "Kirin", Some(N250)),
                (5, 2, _) => brand_arch(MicroArch::MP6, "Lynx", Some(N180)),

                // These two come from instlatx64
                (5, 8, _) => brand_arch(MicroArch::MP62, UNK, Some(N250)),
                (5, 9, _) => brand_arch(MicroArch::MP62, UNK, Some(N180)),
                _ => unknown_model,
            },

            CpuBrand::Transmeta => Transmeta::micro_arch(model, s),

            // As long as the signature doesn't overlap, might as well match for multiple brands
            CpuBrand::DMP | CpuBrand::SiS | CpuBrand::Umc => {
                match (s.family, s.model, s.stepping) {
                    // UMC
                    (4, 1, _) => brand_arch(MicroArch::U5D, "U5D", Some(N600)),
                    (4, 2, _) => brand_arch(MicroArch::U5S, "U5S", Some(N600)),

                    // SiS
                    (5, 0, _) => brand_arch(MicroArch::SiS55x, UNK, None),

                    // DM&P
                    (5, 2, _) => brand_arch(MicroArch::VortexDX, "Vortex86DX", None),
                    (5, 8, _) => brand_arch(MicroArch::VortexMX, "Vortex86MX", None),
                    (6, 1, 1) => brand_arch(MicroArch::VortexDX3, "Vortex86DX3", None),

                    _ => unknown_model,
                }
            }

            CpuBrand::NexGen => unknown_model,

            _ => unknown_model,
        }

        #[cfg(target_arch = "x86_64")]
        match brand {
            CpuBrand::AMD => Amd::micro_arch(model, s),

            CpuBrand::Intel => Intel::micro_arch(model, s),

            CpuBrand::Hygon => unknown_model,

            _ => unknown_model,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_micro_arch_from_string() {
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(Str::<_>::from(MicroArch::Winchip), "Winchip");
            assert_eq!(Str::<_>::from(MicroArch::Cy5x86), "Cx5x86");
            assert_eq!(Str::<_>::from(MicroArch::VortexDX3), "Vortex86DX3");
            assert_eq!(Str::<_>::from(MicroArch::Crusoe), "Crusoe");
            assert_eq!(Str::<_>::from(MicroArch::U5S), "U5S");
        }

        assert_eq!(Str::<_>::from(MicroArch::Am486), "Am486");
        assert_eq!(Str::<_>::from(MicroArch::ZenPlus), "Zen+");
        assert_eq!(Str::<_>::from(MicroArch::Lujiazui), "LuJiaZui");
        assert_eq!(Str::<_>::from(MicroArch::I486), "i486");
        assert_eq!(Str::<_>::from(MicroArch::Unknown), UNK);
    }

    #[test]
    fn test_cpu_arch_new() {
        let arch = CpuArch::new(
            "Test Model",
            MicroArch::Zen,
            "Test Codename",
            "Test Brand",
            "Test Vendor",
            None,
        );
        assert_eq!(arch.model, "Test Model");
        assert_eq!(arch.micro_arch, MicroArch::Zen);
        assert_eq!(arch.code_name, "Test Codename");
        assert_eq!(arch.brand_name, "Test Brand");
        assert_eq!(arch.vendor_string, "Test Vendor");
        assert!(arch.technology.is_none());
    }

    // Helper to create a dummy CpuSignature
    pub fn dummy_signature(
        family: u32,
        model: u32,
        ext_family: u32,
        ext_model: u32,
        stepping: u32,
    ) -> CpuSignature {
        CpuSignature {
            extended_family: ext_family,
            family,
            extended_model: ext_model,
            model,
            stepping,
            display_family: family, // Simplified for tests
            display_model: model,   // Simplified for tests
            is_overdrive: false,
            from_cpuid: false,
        }
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn test_cpu_arch_find_dmp() {
        let model = "DMP Processor";
        let vendor_str = VENDOR_DMP;

        // VortexDX3
        let sig_vortex = dummy_signature(6, 1, 0, 0, 1);
        let arch = CpuArch::find(model, sig_vortex, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::VortexDX3);
        assert_eq!(arch.code_name, "Vortex86DX3");

        // Unknown DMP
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_unknown, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn test_cpu_arch_find_rise() {
        let model = "Rise Processor";
        let vendor_str = VENDOR_RISE;

        // MP6
        let sig_mp6 = dummy_signature(5, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_mp6, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::MP6);
        assert_eq!(arch.code_name, "Kirin");

        // Unknown Rise
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_unknown, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn test_cpu_arch_find_umc_transmeta() {
        let model = "Processor";

        // UMC U5D
        let umc_vendor_str = VENDOR_UMC;
        let sig_u5d = dummy_signature(4, 1, 0, 0, 0);
        let arch = CpuArch::find(model, sig_u5d, umc_vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::U5D);
        assert_eq!(arch.code_name, "U5D");

        // Transmeta Crusoe
        let transmeta_vendor_str = VENDOR_TRANSMETA;
        let sig_crusoe = dummy_signature(5, 4, 0, 0, 0);
        let arch = CpuArch::find(model, sig_crusoe, transmeta_vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Crusoe);
        assert_eq!(arch.code_name, "Crusoe");

        // Unknown Umc/Transmeta
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_unknown, umc_vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    fn test_cpu_arch_find_unknown_brand() {
        let model = "Unknown Processor";
        let vendor_str = "UnknownVendor"; // Not in our defined VENDOR_*
        let sig = dummy_signature(1, 1, 1, 1, 1);

        let arch = CpuArch::find(model, sig, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
        assert_eq!(arch.brand_name, UNK);
    }
}
