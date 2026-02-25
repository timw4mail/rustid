use crate::cpuid::CpuSignature;
use crate::cpuid::brand::{CpuBrand, VENDOR_AMD, VENDOR_CENTAUR, VENDOR_INTEL};
use core::str::FromStr;
use heapless::String;
use ufmt::derive::uDebug;

const UNK: &str = "Unknown";

#[derive(Debug, Clone, Copy, PartialEq, Eq, uDebug)]
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
    Wudaokou,
    Lujiazui,

    // Cyrix
    FiveX86,
    M1,
    M2,
    MediaGx,
    Geode, //Cyrix/NatSemi

    // DM&P
    VortexDX3,

    // Intel
    I486,
    P5,
    P5MMX,
    Lakemont,
    P6Pro,
    P6PentiumII,
    P6PentiumIII,
    Dothan,
    Yonah,
    Merom,
    Penryn,
    Nehalem,
    Westmere,
    Bonnel,
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

    // Rise
    MP6,
    MP6Shrink,

    // Transmeta
    Crusoe,
    Efficeon,

    // UMC
    U5S,
    U5D,
}

impl From<MicroArch> for String<64> {
    fn from(ma: MicroArch) -> String<64> {
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
            MicroArch::Zen3Plus => "Zen 3P+",
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
            MicroArch::Wudaokou => "WuDaoKou",
            MicroArch::Lujiazui => "LuJiaZui",

            // Cyrix
            MicroArch::FiveX86 => "5x86",
            MicroArch::M1 => "M1",
            MicroArch::M2 => "M2",
            MicroArch::MediaGx => "MediaGX",
            MicroArch::Geode => "Geode",

            // DM& P
            MicroArch::VortexDX3 => "VortexDX3",

            // Intel
            MicroArch::I486 => "I486",
            MicroArch::P5 => "P5",
            MicroArch::P5MMX => "P5 MMX",
            MicroArch::Lakemont => "Lakemont",
            MicroArch::P6Pro => "P6",
            MicroArch::P6PentiumII => "P6",
            MicroArch::P6PentiumIII => "P6",
            MicroArch::Dothan => "Dothan",
            MicroArch::Yonah => "Yonah",
            MicroArch::Merom => "Merom",
            MicroArch::Penryn => "Penryn",
            MicroArch::Nehalem => "Nehalem",
            MicroArch::Westmere => "Westmere",
            MicroArch::Bonnel => "Bonnel",
            MicroArch::Saltwell => "Saltwell",
            MicroArch::Silvermont => "Silvermont",
            MicroArch::SandyBridge => "Sandy Bridge",
            MicroArch::IvyBridge => "Ivy Bridge",
            MicroArch::Haswell => "Haswell",
            MicroArch::Broadwell => "Broadwell",
            MicroArch::Airmont => "Airmont",
            MicroArch::KabyLake => "KabyLake",
            MicroArch::Skylake => "Skylake",
            MicroArch::CascadeLake => "CascadeLake",
            MicroArch::KnightsLanding => "KnightsLanding",
            MicroArch::Goldmont => "Goldmont",
            MicroArch::PalmCove => "PalmCove",
            MicroArch::SunnyCove => "SunnyCove",
            MicroArch::GoldmontPlus => "GoldmontPlus",
            MicroArch::IcyLake => "IcyLake",
            MicroArch::Tremont => "Tremont",
            MicroArch::TigerLake => "TigerLake",
            MicroArch::WhiskyLake => "WhiskyLake",
            MicroArch::SapphireRapids => "SapphireRapids",
            MicroArch::AlderLake => "AlderLake",
            MicroArch::CoffeeLake => "CoffeeLake",
            MicroArch::CometLake => "CometLake",
            MicroArch::RaptorLake => "RaptorLake",
            MicroArch::KnightsFerry => "KnightsFerry",
            MicroArch::KnightsCorner => "KnightsCorner",
            MicroArch::Willamette => "Willamette",
            MicroArch::Northwood => "Northwood",
            MicroArch::Prescott => "Prescott",
            MicroArch::CedarMill => "CedarMill",

            // Rise
            MicroArch::MP6 => "MP6",
            MicroArch::MP6Shrink => "MP6 (die shrink)",

            // Transmeta
            MicroArch::Crusoe => "Crusoe",
            MicroArch::Efficeon => "Efficeon",

            // UMC
            MicroArch::U5S => "U5S",
            MicroArch::U5D => "U5D",
        };

        let mut out: String<64> = String::new();
        let _ = out.push_str(s);

        out
    }
}

#[derive(Debug, Clone)]
pub struct CpuArch {
    pub model: String<64>,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub brand_name: String<64>,
    pub vendor_string: String<64>,
    pub technology: Option<String<32>>,
}

impl ufmt::uDebug for CpuArch {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.write_str("CpuArch { model: \"")?;
        f.write_str(self.model.as_str())?;
        f.write_str("\", micro_arch: ")?;
        ufmt::uDebug::fmt(&self.micro_arch, f)?;
        f.write_str(", code_name: \"")?;
        f.write_str(self.code_name)?;
        f.write_str("\", brand_name: \"")?;
        f.write_str(self.brand_name.as_str())?;
        f.write_str("\", vendor_string: \"")?;
        f.write_str(self.vendor_string.as_str())?;
        f.write_str("\" }")
    }
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(
            "Unknown",
            MicroArch::Unknown,
            UNK,
            "Unknown",
            "Unknown",
            None,
        )
    }
}

impl CpuArch {
    pub fn new(
        model: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        brand_name: &str,
        vendor_string: &str,
        technology: Option<&str>,
    ) -> Self {
        let mut model_s: String<64> = String::new();
        let _ = model_s.push_str(model);

        let mut brand_s: String<64> = String::new();
        let _ = brand_s.push_str(brand_name);

        let mut vendor_s: String<64> = String::new();
        let _ = vendor_s.push_str(vendor_string);

        let technology = technology.map(|s| String::from_str(s).unwrap());

        CpuArch {
            model: model_s,
            micro_arch,
            code_name,
            brand_name: brand_s,
            vendor_string: vendor_s,
            technology,
        }
    }

    pub fn find(model: &str, s: CpuSignature, vendor_string: &str) -> Self {
        let arch = |ma: MicroArch,
                    code_name: &'static str,
                    brand_name: &str,
                    tech: Option<&str>|
         -> Self {
            CpuArch::new(model, ma, code_name, brand_name, vendor_string, tech)
        };

        // Brand for Centaur CPUs is by signature, not vendor string
        if vendor_string == VENDOR_CENTAUR {
            return Self::find_centaur(model, s, vendor_string);
        }

        let brand = CpuBrand::from(vendor_string);
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> Self {
            arch(ma, code_name, brand.to_brand_name(), tech)
        };

        match brand {
            CpuBrand::AMD => Self::find_amd(model, s),
            CpuBrand::Intel => Self::find_intel(model, s),
            CpuBrand::DMP => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 6, 0, 1, 1) => brand_arch(MicroArch::VortexDX3, "Vortex86DX3", None),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
            },
            CpuBrand::Cyrix => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 4, 0, 9, _) => brand_arch(MicroArch::FiveX86, "5x86", None),
                (0, 5, 0, 2 | 3, _) => brand_arch(MicroArch::M1, "M1", None),
                (0, 5, 0, 4, _) => brand_arch(MicroArch::MediaGx, "MediaGX GXm", Some("350nm")),
                (0, 6, 0, 0, _) => brand_arch(MicroArch::M2, "M2", None),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
            },
            CpuBrand::NationalSemiconductor => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 5, 0, 4, _) => brand_arch(MicroArch::Geode, "GX1", Some("180nm")),
                (0, 5, 0, 9, _) => brand_arch(MicroArch::Geode, "GX2", Some("180nm")),
                (0, 5, 0, 10, _) => brand_arch(MicroArch::Geode, "GX3", None),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
            },
            CpuBrand::Rise => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 5, 0, 0, _) => brand_arch(MicroArch::MP6, "Kirin", Some("250nm")),
                (0, 5, 0, 2, _) => brand_arch(MicroArch::MP6Shrink, "Lynx", Some("180nm")),
                (0, 5, 0, 8, _) => brand_arch(MicroArch::MP6Shrink, UNK, None),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
            },
            CpuBrand::Umc | CpuBrand::Transmeta => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                // UMC
                (0, 4, 0, 1, _) => brand_arch(MicroArch::U5D, "U5D", Some("600nm")),
                (0, 4, 0, 2, _) => brand_arch(MicroArch::U5S, "U5S", Some("600nm")),

                // Transmeta
                (0, 5, 0, 4, _) => brand_arch(MicroArch::Crusoe, "Crusoe", Some("130nm")),
                (0, 15, 0, 2 | 3, _) => brand_arch(MicroArch::Efficeon, "Efficeon", None),

                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
            },
            CpuBrand::Hygon
            | CpuBrand::IDT
            | CpuBrand::NexGen
            | CpuBrand::SiS
            | CpuBrand::Unknown
            | CpuBrand::Via
            | CpuBrand::Zhaoxin => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }

    fn find_amd(model: &str, s: CpuSignature) -> Self {
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> Self {
            Self::new(model, ma, code_name, "AMD", VENDOR_AMD, tech)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // 486
            (0, 4, 0, 3, _) => brand_arch(MicroArch::Am486, "Am486DX2", None),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::Am486, "Am486X2WB", None),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::Am486, "Am486DX4", None),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::Am486, "Am486DX4WB", None),
            (0, 4, 0, 14, _) => brand_arch(MicroArch::Am5x86, "Am5x86", None),
            (0, 4, 0, 15, _) => brand_arch(MicroArch::Am5x86, "Am5x86WB", None),

            // K5
            (0, 5, 0, 0, _) => brand_arch(MicroArch::SSA5, "SSA5", Some("350nm")),
            (0, 5, 0, 1..=3, _) => brand_arch(MicroArch::K5, "5k86", Some("350nm")),

            // K6
            (0, 5, 0, 6, _) => brand_arch(MicroArch::K6, "K6", Some("300nm")),
            (0, 5, 0, 7, _) => brand_arch(MicroArch::K6, "Little Foot", Some("250nm")),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::K6, "Chompers/CXT (K6-2)", None),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-III)", None),
            (0, 5, 0, 10, _) => brand_arch(MicroArch::K7, "Thoroughbred (Geode NX)", Some("130nm")),
            (0, 5, 0, 13, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-2+/K6-III+)", None),

            // K7
            (0, 6, 0, 1, _) => brand_arch(MicroArch::K7, "Argon", Some("250nm")),
            (0, 6, 0, 2, _) => brand_arch(MicroArch::K7, "Pluto", Some("180nm")),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::K7, "Spitfire", None),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::K7, "Thunderbird", None),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::K7, "Palomino", None),
            (0, 6, 0, 7, _) => brand_arch(MicroArch::K7, "Morgan", None),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::K7, "Thoroughbred", None),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::K7, "Thorton/Barton", None),

            // K8
            (0, 15, 0, 13, 0) => brand_arch(MicroArch::K8, "NewCastle", Some("130nm")),
            (0, 15, 2, 3, 2) => brand_arch(MicroArch::K8, "Toledo", Some("90nm")),
            (0, 15, 2, 15, 2) => brand_arch(MicroArch::K8, "Venice", Some("90nm")),
            (0, 15, 3, 7, 2) => brand_arch(MicroArch::K8, "San Diego", Some("90nm")),
            (0, 15, 4, 15, 2) => brand_arch(MicroArch::K8, "Manilla", Some("90nm")),
            (0, 15, 4, 12, 2) => brand_arch(MicroArch::K8, "Windsor", Some("90nm")),
            (0, 15, 5, 15, 2) => brand_arch(MicroArch::K8, "Orleans", Some("90nm")),
            (0, 15, 6, 12, 2) => brand_arch(MicroArch::K8, "Brisbane", Some("65nm")),
            (0, 15, 7, 15, 2) => brand_arch(MicroArch::K8, "Sparta", Some("65nm")),

            // K10
            (5, 15, _, _, _) => brand_arch(MicroArch::Bobcat, "Zacate", Some("40nm")),

            // Bulldozer/Piledriver/Steamroller
            (6, 15, 0, 0 | 1, _) => brand_arch(MicroArch::Bulldozer, "Zambezi", Some("32nm")),
            (6, 15, 0 | 1, 2, _) => brand_arch(MicroArch::Piledriver, "Vishera", Some("32nm")),
            (6, 15, 3, 0 | 8, _) => brand_arch(MicroArch::Steamroller, "Godavari", Some("28nm")),
            (6, 15, 6 | 7, 0 | 5, _) => {
                brand_arch(MicroArch::Excavator, "Bristol Ridge/Carrizo", Some("28nm"))
            }

            // Zen
            (8, 15, 1, 1, 0) => brand_arch(MicroArch::Zen, "RavenRidge", Some("14nm")),
            (10, 15, 2, 1, _) => brand_arch(MicroArch::Zen3, "Vermeer", Some("7nm")),
            (10, 15, 6, 1, 2) => brand_arch(MicroArch::Zen4, "Raphael", Some("5nm")),
            (10, 15, 7, 4, 1) => brand_arch(MicroArch::Zen4, "Phoenix", Some("4nm")),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }

    fn find_centaur(model: &str, s: CpuSignature, vendor_string: &str) -> Self {
        let brand = match s.family {
            5 => CpuBrand::IDT,
            6 => CpuBrand::Via,
            _ => CpuBrand::Zhaoxin,
        };

        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> Self {
            Self::new(
                model,
                ma,
                code_name,
                brand.to_brand_name(),
                vendor_string,
                tech,
            )
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // IDT
            (0, 5, 0, 4, _) => brand_arch(MicroArch::Winchip, "C6", Some("350nm")),
            (0, 5, 0, 8, 5) => brand_arch(MicroArch::Winchip2, "Winchip 2 (C2)", Some("350nm")),
            (0, 5, 0, 8, 7) => brand_arch(MicroArch::Winchip2A, "Winchip 2A (W2A)", Some("250nm")),
            (0, 5, 0, 8, 10) => brand_arch(MicroArch::Winchip2B, "Winchip 2B (W2B)", Some("250nm")),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::Winchip3, "Winchip 3 (C3)", Some("250nm")),

            // VIA
            (0, 6, 0, 6, _) => brand_arch(MicroArch::Samuel, "Samuel (C5A)", Some("180nm")),
            (0, 6, 0, 7, 0..=7) => brand_arch(MicroArch::Samuel2, "Samuel 2 (C5B)", Some("150nm")),
            (0, 6, 0, 7, 8..=15) => brand_arch(MicroArch::Ezra, "Ezra (C5C)", Some("130nm")),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::EzraT, "Ezra-T (C5N)", Some("130nm")),
            (0, 6, 0, 9, 0..=7) => {
                brand_arch(MicroArch::Nehemiah, "Nehemiah (C5XL)", Some("130nm"))
            }
            (0, 6, 0, 9, 8..=15) => brand_arch(MicroArch::NehemiahP, "Nehemiah+ (C5P)", None),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::Esther, "Esther (C5J)", Some("90nm")),
            (0, 6, 1, 9..=12, 8) => brand_arch(MicroArch::Isaiah, "Isaiah (CNS)", None),
            (0, 6, 1, 15, _) => brand_arch(MicroArch::Isaiah, "Isaiah (CN)", Some("65nm")),

            // Zhaoxin
            (0, 7, 1, 11, 0) => brand_arch(MicroArch::Wudaokou, "WuDaoKou", Some("28nm")),
            (0, 7, 3, 11, 0) => brand_arch(MicroArch::Lujiazui, "LuJiaZui", Some("16nm")),

            // Anything else
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }

    fn find_intel(model: &str, s: CpuSignature) -> Self {
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> Self {
            Self::new(model, ma, code_name, "Intel", VENDOR_INTEL, tech)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
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
            (0, 5, 0, 0 | 1, _) => brand_arch(MicroArch::P5, "P5", Some("800nm")),
            (0, 5, 0, 2, _) => brand_arch(MicroArch::P5, "P54C", None),
            (0, 5, 0, 3, _) => brand_arch(MicroArch::P5, "P24T", Some("600nm")),
            (0, 5, 0, 4, _) => brand_arch(MicroArch::P5MMX, "P55C", Some("350nm")),
            (0, 5, 0, 7, _) => brand_arch(MicroArch::P5MMX, "P54C", Some("350nm")),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::P5MMX, "P55C", Some("250nm")),
            (0, 5, 0, 9 | 10, _) => brand_arch(MicroArch::Lakemont, "Lakemont", Some("32nm")),

            // Pentium Pro
            (0, 6, 0, 1, 1) => brand_arch(MicroArch::P6Pro, "P6", None),
            (0, 6, 0, 1, 2) => brand_arch(MicroArch::P6Pro, "P6", Some("600nm")),
            (0, 6, 0, 1, 6..10) => brand_arch(MicroArch::P6Pro, "P6", Some("350nm")),
            (0, 6, 0, 3, 2) => brand_arch(MicroArch::P6PentiumII, "P6T (Deschutes)", Some("250nm")), // Pentium II Overdrive

            // Pentium 2
            (0, 6, 0, 0..=2, _) => brand_arch(MicroArch::P6PentiumII, UNK, None),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::P6PentiumII, "Klamath", Some("350nm")),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::P6PentiumII, UNK, None),
            (0, 6, 0, 5, 1) => brand_arch(MicroArch::P6PentiumII, "Deschutes", Some("250nm")),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::P6PentiumII, "Dixon / Mendocino", None),

            // Pentium 3
            (0, 6, 0, 7, _) => brand_arch(MicroArch::P6PentiumIII, "Katmai", Some("250nm")),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::P6PentiumIII, "Coppermine", Some("180nm")),
            (0, 6, 0, 9, 5) => brand_arch(MicroArch::P6PentiumIII, "Banias", Some("130nm")),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::P6PentiumIII, "Coppermine T", Some("180nm")),
            (0, 6, 0, 11, _) => brand_arch(MicroArch::P6PentiumIII, "Tualatin", Some("130nm")),

            // Core i-series
            (0, 6, 1, 14, 5) => brand_arch(MicroArch::Nehalem, "Lynnfield", Some("45nm")),
            (0, 6, 2, 10, 7) => brand_arch(MicroArch::SandyBridge, "Sandy Bridge", Some("32nm")),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpuid::brand::{
        VENDOR_CYRIX, VENDOR_DMP, VENDOR_RISE, VENDOR_TRANSMETA, VENDOR_UMC,
    };

    #[test]
    fn test_micro_arch_from_string() {
        assert_eq!(String::<64>::from(MicroArch::Am486).as_str(), "Am486");
        assert_eq!(String::<64>::from(MicroArch::ZenPlus).as_str(), "Zen+");
        assert_eq!(String::<64>::from(MicroArch::Winchip).as_str(), "Winchip");
        assert_eq!(String::<64>::from(MicroArch::Lujiazui).as_str(), "LuJiaZui");
        assert_eq!(String::<64>::from(MicroArch::FiveX86).as_str(), "5x86");
        assert_eq!(
            String::<64>::from(MicroArch::VortexDX3).as_str(),
            "VortexDX3"
        );
        assert_eq!(String::<64>::from(MicroArch::I486).as_str(), "I486");
        assert_eq!(String::<64>::from(MicroArch::Crusoe).as_str(), "Crusoe");
        assert_eq!(String::<64>::from(MicroArch::U5S).as_str(), "U5S");
        assert_eq!(String::<64>::from(MicroArch::Unknown).as_str(), "Unknown");
    }

    #[test]
    fn test_micro_arch_udebug() {
        use ufmt::uwrite;
        let mut s = heapless::String::<64>::new();

        uwrite!(&mut s, "{:?}", MicroArch::Zen).unwrap();
        assert_eq!(s.as_str(), "Zen");
        s.clear();

        uwrite!(&mut s, "{:?}", MicroArch::Unknown).unwrap();
        assert_eq!(s.as_str(), "Unknown");
        s.clear();

        uwrite!(&mut s, "{:?}", MicroArch::Am486).unwrap();
        assert_eq!(s.as_str(), "Am486");
        s.clear();

        // Test a non-specific case
        uwrite!(&mut s, "{:?}", MicroArch::K5).unwrap();
        assert_eq!(s.as_str(), "K5");
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
        assert_eq!(arch.model.as_str(), "Test Model");
        assert_eq!(arch.micro_arch, MicroArch::Zen);
        assert_eq!(arch.code_name, "Test Codename");
        assert_eq!(arch.brand_name.as_str(), "Test Brand");
        assert_eq!(arch.vendor_string.as_str(), "Test Vendor");
        assert!(arch.technology.is_none());
    }

    // Helper to create a dummy CpuSignature
    fn dummy_signature(
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
        }
    }

    #[test]
    fn test_cpu_arch_find_amd() {
        let model = "AMD Processor";

        // Am486
        let sig_am486 = dummy_signature(4, 3, 0, 0, 0);
        let arch = CpuArch::find_amd(model, sig_am486);
        assert_eq!(arch.micro_arch, MicroArch::Am486);
        assert_eq!(arch.code_name, "Am486DX2");

        // K5
        let sig_k5 = dummy_signature(5, 1, 0, 0, 0);
        let arch = CpuArch::find_amd(model, sig_k5);
        assert_eq!(arch.micro_arch, MicroArch::K5);
        assert_eq!(arch.code_name, "5k86");

        // Zen4
        let sig_zen4 = dummy_signature(15, 1, 10, 6, 2);
        let arch = CpuArch::find_amd(model, sig_zen4);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Raphael");

        // Unknown AMD
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find_amd(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    fn test_cpu_arch_find_intel() {
        let model = "Intel Processor";

        // I486
        let sig_i486 = dummy_signature(4, 0, 0, 0, 0);
        let arch = CpuArch::find_intel(model, sig_i486);
        assert_eq!(arch.micro_arch, MicroArch::I486);
        assert_eq!(arch.code_name, "i80486DX");

        // P5 (Pentium)
        let sig_p5 = dummy_signature(5, 2, 0, 0, 0);
        let arch = CpuArch::find_intel(model, sig_p5);
        assert_eq!(arch.micro_arch, MicroArch::P5);
        assert_eq!(arch.code_name, "P54C");

        // Nehalem
        let sig_nehalem = dummy_signature(6, 14, 0, 1, 5);
        let arch = CpuArch::find_intel(model, sig_nehalem);
        assert_eq!(arch.micro_arch, MicroArch::Nehalem);
        assert_eq!(arch.code_name, "Lynnfield");

        // Unknown Intel
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find_intel(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
    fn test_cpu_arch_find_centaur() {
        let model = "Centaur Processor";
        let vendor_str = VENDOR_CENTAUR;

        // IDT Winchip
        let sig_winchip = dummy_signature(5, 4, 0, 0, 0);
        let arch = CpuArch::find_centaur(model, sig_winchip, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Winchip);
        assert_eq!(arch.code_name, "C6");

        // VIA Ezra
        let sig_ezra = dummy_signature(6, 7, 0, 0, 8);
        let arch = CpuArch::find_centaur(model, sig_ezra, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Ezra);
        assert_eq!(arch.code_name, "Ezra (C5C)");

        // Zhaoxin Lujiazui
        let sig_lujiazui = dummy_signature(7, 11, 0, 3, 0);
        let arch = CpuArch::find_centaur(model, sig_lujiazui, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Lujiazui);
        assert_eq!(arch.code_name, "LuJiaZui");

        // Unknown Centaur
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find_centaur(model, sig_unknown, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK); // Centaur unknown code_name is empty
    }

    #[test]
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
    fn test_cpu_arch_find_cyrix() {
        let model = "Cyrix Processor";
        let vendor_str = VENDOR_CYRIX;

        // FiveX86
        let sig_fivex86 = dummy_signature(4, 9, 0, 0, 0);
        let arch = CpuArch::find(model, sig_fivex86, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::FiveX86);
        assert_eq!(arch.code_name, "5x86");

        // M2
        let sig_m2 = dummy_signature(6, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_m2, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::M2);
        assert_eq!(arch.code_name, "M2");

        // Unknown Cyrix
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = CpuArch::find(model, sig_unknown, vendor_str);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }

    #[test]
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
        assert_eq!(arch.brand_name.as_str(), "Unknown");
    }
}
