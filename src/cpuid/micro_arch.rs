use crate::cpuid::CpuSignature;
use crate::cpuid::brand::{CpuBrand, VENDOR_AMD, VENDOR_CENTAUR, VENDOR_INTEL};
use heapless::String;

const UNK: &str = "Unknown";

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

impl Into<String<64>> for MicroArch {
    fn into(self) -> String<64> {
        let s = match self {
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
            MicroArch::ZenPlus => "ZenPlus",
            MicroArch::Zen2 => "Zen2",
            MicroArch::Zen3 => "Zen3",
            MicroArch::Zen3Plus => "Zen3Plus",
            MicroArch::Zen4 => "Zen4",
            MicroArch::Zen4C => "Zen4C",
            MicroArch::Zen5 => "Zen5",
            MicroArch::Zen5C => "Zen5C",

            // Centaur (IDT)
            MicroArch::Winchip => "Winchip",
            MicroArch::Winchip2 => "Winchip2",
            MicroArch::Winchip2A => "Winchip2A",
            MicroArch::Winchip2B => "Winchip2B",
            MicroArch::Winchip3 => "Winchip3",

            // Centaur (VIA)
            MicroArch::Samuel => "Samuel",
            MicroArch::Samuel2 => "Samuel2",
            MicroArch::Ezra => "Ezra",
            MicroArch::EzraT => "EzraT",
            MicroArch::Nehemiah => "Nehemiah",
            MicroArch::NehemiahP => "NehemiahP",
            MicroArch::Esther => "Esther",
            MicroArch::Isaiah => "Isaiah",

            // Centaur (Zhaoxin)
            MicroArch::Wudaokou => "Wudaokou",
            MicroArch::Lujiazui => "Lujiazui",

            // Cyrix
            MicroArch::FiveX86 => "FiveX86",
            MicroArch::M1 => "M1",
            MicroArch::M2 => "M2",
            MicroArch::MediaGx => "MediaGx",
            MicroArch::Geode => "Geode",

            // DM& P
            MicroArch::VortexDX3 => "VortexDX3",

            // Intel
            MicroArch::I486 => "I486",
            MicroArch::P5 => "P5",
            MicroArch::P5MMX => "P5 MMX",
            MicroArch::Lakemont => "Lakemont",
            MicroArch::P6Pro => "P6 (Pentium Pro)",
            MicroArch::P6PentiumII => "P6 (PentiumII)",
            MicroArch::P6PentiumIII => "P6 (PentiumIII)",
            MicroArch::Dothan => "Dothan",
            MicroArch::Yonah => "Yonah",
            MicroArch::Merom => "Merom",
            MicroArch::Penryn => "Penryn",
            MicroArch::Nehalem => "Nehalem",
            MicroArch::Westmere => "Westmere",
            MicroArch::Bonnel => "Bonnel",
            MicroArch::Saltwell => "Saltwell",
            MicroArch::Silvermont => "Silvermont",
            MicroArch::SandyBridge => "SandyBridge",
            MicroArch::IvyBridge => "IvyBridge",
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

impl ufmt::uDebug for MicroArch {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        let s = match self {
            MicroArch::Zen => "Zen",
            MicroArch::Unknown => UNK,
            MicroArch::Am486 => "Am486",
            MicroArch::Zen4 => "Zen4",
            MicroArch::I486 => "I486",
            MicroArch::P5 => "P5",
            _ => "OtherArch",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub struct CpuArch {
    pub model: String<64>,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub brand_name: String<64>,
    pub vendor_string: String<64>,
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

impl CpuArch {
    pub fn new(
        model: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        brand_name: &str,
        vendor_string: &str,
    ) -> Self {
        let mut model_s: String<64> = String::new();
        let _ = model_s.push_str(model);

        let mut brand_s: String<64> = String::new();
        let _ = brand_s.push_str(brand_name);

        let mut vendor_s: String<64> = String::new();
        let _ = vendor_s.push_str(vendor_string);

        CpuArch {
            model: model_s,
            micro_arch,
            code_name,
            brand_name: brand_s,
            vendor_string: vendor_s,
        }
    }

    pub fn find(model: &str, s: CpuSignature, vendor_string: &str) -> Self {
        let arch = |ma: MicroArch, code_name: &'static str, brand_name: &str| -> Self {
            CpuArch::new(model, ma, code_name, brand_name, vendor_string)
        };

        // Brand for Centaur CPUs is by signature, not vendor string
        if vendor_string == VENDOR_CENTAUR {
            return Self::find_centaur(model, s, vendor_string);
        }

        let brand = CpuBrand::from(vendor_string);
        let brand_arch = |ma: MicroArch, code_name: &'static str| -> Self {
            arch(ma, code_name, brand.as_str())
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
                (0, 6, 0, 1, 1) => brand_arch(MicroArch::VortexDX3, "Vortex86DX3"),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
            },
            CpuBrand::Cyrix => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 4, 0, 9, _) => brand_arch(MicroArch::FiveX86, "5x86"),
                (0, 5, 0, 2 | 3, _) => brand_arch(MicroArch::M1, "M1/6x86"),
                (0, 5, 0, 4, _) => brand_arch(MicroArch::MediaGx, "MediaGX GXm"),
                (0, 6, 0, 0, _) => brand_arch(MicroArch::M2, "M2/6x86MX"),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
            },
            CpuBrand::Rise => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 5, 0, 0, _) => brand_arch(MicroArch::MP6, "Kirin"),
                (0, 5, 0, 2, _) => brand_arch(MicroArch::MP6Shrink, "Lynx"),
                (0, 5, 0, 8, _) => brand_arch(MicroArch::MP6Shrink, UNK),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
            },
            CpuBrand::Umc | CpuBrand::Transmeta => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                // UMC
                (0, 4, 0, 1, _) => brand_arch(MicroArch::U5D, "U5D"),
                (0, 4, 0, 2, _) => brand_arch(MicroArch::U5S, "U5S"),

                // Transmeta
                (0, 5, 0, 4, _) => brand_arch(MicroArch::Crusoe, "Crusoe"),
                (0, 15, 0, 2 | 3, _) => brand_arch(MicroArch::Efficeon, "Efficeon"),

                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
            },
            CpuBrand::Hygon
            | CpuBrand::IDT
            | CpuBrand::NationalSemiconductor
            | CpuBrand::NexGen
            | CpuBrand::SiS
            | CpuBrand::Unknown
            | CpuBrand::Via
            | CpuBrand::Zhaoxin => brand_arch(MicroArch::Unknown, UNK),
        }
    }

    fn find_amd(model: &str, s: CpuSignature) -> Self {
        let brand_arch = |ma: MicroArch, code_name: &'static str| -> Self {
            Self::new(model, ma, code_name, "AMD", VENDOR_AMD)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // 486
            (0, 4, 0, 3, _) => brand_arch(MicroArch::Am486, "Am486DX2"),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::Am486, "Am486X2WB"),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::Am486, "Am486DX4"),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::Am486, "Am486DX4WB"),

            // K5
            (0, 4, 0, 14, _) => brand_arch(MicroArch::Am5x86, "Am5x86"),
            (0, 4, 0, 15, _) => brand_arch(MicroArch::Am5x86, "Am5x86WB"),
            (0, 5, 0, 0, _) => brand_arch(MicroArch::SSA5, "SSA5 (K5)"),
            (0, 5, 0, 1..=3, _) => brand_arch(MicroArch::K5, "K5"),

            // K6
            (0, 5, 0, 6 | 7, _) => brand_arch(MicroArch::K6, "K6"),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::K6, "Chompers/CXT (K6-2)"),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-III)"),
            (0, 5, 0, 10, _) => brand_arch(MicroArch::K7, "Thoroughbred (Geode NX)"),
            (0, 5, 0, 13, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-2+/K6-III+)"),

            // K7
            (0, 6, 0, 1, _) => brand_arch(MicroArch::K7, "Argon"),
            (0, 6, 0, 2, _) => brand_arch(MicroArch::K7, "Pluto"),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::K7, "Spitfire"),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::K7, "Thunderbird"),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::K7, "Palomino"),
            (0, 6, 0, 7, _) => brand_arch(MicroArch::K7, "Morgan"),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::K7, "Thoroughbred"),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::K7, "Thorton/Barton"),

            // K8

            // K10
            (5, 15, _, _, _) => brand_arch(MicroArch::Bobcat, "Zacate"),

            // Bulldozer/Piledriver/Steamroller
            (6, 15, 0, 0 | 1, _) => brand_arch(MicroArch::Bulldozer, "Zambezi"),
            (6, 15, 0 | 1, 2, _) => brand_arch(MicroArch::Piledriver, "Vishera"),
            (6, 15, 3, 0 | 8, _) => brand_arch(MicroArch::Steamroller, "Godavari"),
            (6, 15, 6 | 7, 0 | 5, _) => brand_arch(MicroArch::Excavator, "Bristol Ridge/Carrizo"),

            // Zen
            (8, 15, 1, 1, 0) => brand_arch(MicroArch::Zen, "RavenRidge"),
            (10, 15, 2, 1, _) => brand_arch(MicroArch::Zen3, "Vermeer"),
            (10, 15, 6, 1, 2) => brand_arch(MicroArch::Zen4, "Raphael"),
            (10, 15, 7, 4, 1) => brand_arch(MicroArch::Zen4, "Phoenix"),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
        }
    }

    fn find_centaur(model: &str, s: CpuSignature, vendor_string: &str) -> Self {
        let brand = match s.family {
            5 => CpuBrand::IDT,
            6 => CpuBrand::Via,
            _ => CpuBrand::Zhaoxin,
        };

        let brand_arch = |ma: MicroArch, code_name: &'static str| -> Self {
            Self::new(model, ma, code_name, brand.as_str(), vendor_string)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // IDT
            (0, 5, 0, 4, _) => brand_arch(MicroArch::Winchip, "C6"),
            (0, 5, 0, 8, 5) => brand_arch(MicroArch::Winchip2, "Winchip 2 (C2)"),
            (0, 5, 0, 8, 7) => brand_arch(MicroArch::Winchip2A, "Winchip 2A (W2A)"),
            (0, 5, 0, 8, 10) => brand_arch(MicroArch::Winchip2B, "Winchip 2B (W2B)"),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::Winchip3, "Winchip 3 (C3)"),

            // VIA
            (0, 6, 0, 6, _) => brand_arch(MicroArch::Samuel, "Samuel (C5A)"),
            (0, 6, 0, 7, 0..=7) => brand_arch(MicroArch::Samuel2, "Samuel 2 (C5B)"),
            (0, 6, 0, 7, 8..=15) => brand_arch(MicroArch::Ezra, "Ezra (C5C)"),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::EzraT, "Ezra-T (C5N)"),
            (0, 6, 0, 9, 0..=7) => brand_arch(MicroArch::Nehemiah, "Nehemiah (C5XL)"),
            (0, 6, 0, 9, 8..=15) => brand_arch(MicroArch::NehemiahP, "Nehemiah+ (C5P)"),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::Esther, "Esther (C5J)"),
            (0, 6, 1, 9..=12, 8) => brand_arch(MicroArch::Isaiah, "Isaiah (CNS)"),
            (0, 6, 1, 15, _) => brand_arch(MicroArch::Isaiah, "Isaiah (CN)"),

            // Zhaoxin
            (0, 7, 1, 11, 0) => brand_arch(MicroArch::Wudaokou, "WuDaoKou"),
            (0, 7, 3, 11, 0) => brand_arch(MicroArch::Lujiazui, "LuJiaZui"),

            // Anything else
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, ""),
        }
    }

    fn find_intel(model: &str, s: CpuSignature) -> Self {
        let brand_arch = |ma: MicroArch, code_name: &'static str| -> Self {
            Self::new(model, ma, code_name, "Intel", VENDOR_INTEL)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // 486
            (0, 4, 0, 0, _) => brand_arch(MicroArch::I486, "i80486DX"),
            (0, 4, 0, 1, _) => brand_arch(MicroArch::I486, "i80486DX-50"),
            (0, 4, 0, 2, _) => brand_arch(MicroArch::I486, "i80486SX"),
            (0, 4, 0, 3, _) => brand_arch(MicroArch::I486, "i80486DX2"),
            (0, 4, 0, 4, _) => brand_arch(MicroArch::I486, "i80486SL"),
            (0, 4, 0, 5, _) => brand_arch(MicroArch::I486, "i80486SX2"),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::I486, "i80486DX2WB"),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::I486, "i80486DX4"),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::I486, "i80486DX4WB"),

            // Pentium
            (0, 5, 0, 0 | 1, _) => brand_arch(MicroArch::P5, "P5"),
            (0, 5, 0, 2, _) => brand_arch(MicroArch::P5, "P54C"),
            (0, 5, 0, 3, _) => brand_arch(MicroArch::P5, "P24T"),
            (0, 5, 0, 4, _) => brand_arch(MicroArch::P5MMX, "P55C"),
            (0, 5, 0, 7, _) => brand_arch(MicroArch::P5MMX, "P54C"),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::P5MMX, "P55C (250nm)"),
            (0, 5, 0, 9 | 10, _) => brand_arch(MicroArch::Lakemont, "Lakemont"),

            // Pentium Pro
            (0, 6, 0, 1, 1 | 2 | 6..10) => brand_arch(MicroArch::P6Pro, "P6"),

            // Pentium 2
            (0, 6, 0, 0..=2, _) => brand_arch(MicroArch::P6PentiumII, UNK),
            (0, 6, 0, 3, 2) => brand_arch(MicroArch::P6PentiumII, "Deschutes"), // Pentium II Overdrive
            (0, 6, 0, 3, _) => brand_arch(MicroArch::P6PentiumII, "Klamath"),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::P6PentiumII, UNK),
            (0, 6, 0, 5, 1) => brand_arch(MicroArch::P6PentiumII, "Deschutes"),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::P6PentiumII, "Dixon / Mendocino"),

            // Pentium 3
            (0, 6, 0, 7, _) => brand_arch(MicroArch::P6PentiumIII, "Katmai"),
            (0, 6, 0, 8, _) => brand_arch(MicroArch::P6PentiumIII, "Coppermine"),
            (0, 6, 0, 9, 5) => brand_arch(MicroArch::P6PentiumIII, "Banias"),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::P6PentiumIII, "Coppermine T"),
            (0, 6, 0, 11, _) => brand_arch(MicroArch::P6PentiumIII, "Tualatin"),

            // Core i-series
            (0, 6, 1, 14, 5) => brand_arch(MicroArch::Nehalem, "Lynnfield"),
            (0, 6, 2, 10, 7) => brand_arch(MicroArch::SandyBridge, "SandyBridge"),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK),
        }
    }
}
