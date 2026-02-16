use crate::cpuid;
use crate::cpuid::fns;

const VENDOR_AMD: &str = "AuthenticAMD";
const VENDOR_CENTAUR: &str = "CentaurHauls";
const VENDOR_CYRIX: &str = "CyrixInstead";
const VENDOR_DMP: &str = "Vortex86 SoC";
const VENDOR_HYGON: &str = "HygonGenuine";
const VENDOR_INTEL: &str = "GenuineIntel";
const VENDOR_NEXGEN: &str = "NexGenDriven";
const VENDOR_NSC: &str = "Geode by NSC";
const VENDOR_RISE: &str = "RiseRiseRise";
const VENDOR_SIS: &str = "SiS SiS SiS ";
const VENDOR_TRANSMETA: &str = "GenuineTMx86";
const VENDOR_UMC: &str = "UMC UMC UMC ";

#[derive(Debug)]
pub struct CpuArch {
    model: String,
    micro_arch: MicroArch,
    code_name: String,
    brand_name: String,
    vendor_string: String,
}

impl CpuArch {
    pub fn new(
        model: String,
        micro_arch: MicroArch,
        code_name: &'static str,
        brand_name: &str,
        vendor_string: &str,
    ) -> Self {
        CpuArch {
            model: model.to_string(),
            micro_arch,
            code_name: code_name.to_string(),
            brand_name: brand_name.to_string(),
            vendor_string: vendor_string.to_string(),
        }
    }

    pub fn find(model: String, s: CpuSignature, vendor_string: &str) -> Self {
        let arch = |s: MicroArch, code_name: &'static str, brand_name: String| -> Self {
            CpuArch::new(model.clone(), s, code_name, &brand_name, vendor_string)
        };

        // Brand for Centaur CPUs is by signature, not vendor string
        if vendor_string == VENDOR_CENTAUR {
            return Self::find_centaur(&model, s, vendor_string);
        }

        let brand = CpuBrand::from(vendor_string.to_string());
        let brand_arch =
            |s: MicroArch, code_name: &'static str| -> Self { arch(s, code_name, brand.into()) };

        match brand {
            CpuBrand::AMD => Self::find_amd(&model, s),
            CpuBrand::Intel => Self::find_intel(&model, s),
            CpuBrand::DMP => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 6, 0, 1, 1) => brand_arch(MicroArch::VortexDX3, "Vortex86DX3"),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
            },
            CpuBrand::Cyrix => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 4, 0, 9, _) => brand_arch(MicroArch::FiveX86, "5x86"),
                (0, 5, 0, 2, _) => brand_arch(MicroArch::M1, "M1/6x86"),
                (0, 5, 0, 4, _) => brand_arch(MicroArch::MediaGx, "MediaGX GXm"),
                (0, 6, 0, 0, _) => brand_arch(MicroArch::M2, "M2/6x86MX"),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
            },
            CpuBrand::Rise => match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (0, 5, 0, 0, _) => brand_arch(MicroArch::MP6, "mP6"),
                (0, 5, 0, 2, _) => brand_arch(MicroArch::MP6Shrink, "mP6"),
                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
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

                (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
            },
            CpuBrand::Hygon
            | CpuBrand::IDT
            | CpuBrand::NationalSemiconductor
            | CpuBrand::NexGen
            | CpuBrand::SiS
            | CpuBrand::Unknown
            | CpuBrand::Via
            | CpuBrand::Zhaoxin => brand_arch(MicroArch::Unknown, "Unknown"),
        }
    }

    fn find_amd(model: impl Into<String>, s: CpuSignature) -> Self {
        let brand_arch = |s: MicroArch, code_name: &'static str| -> Self {
            CpuArch::new(model.into(), s, code_name, "AMD", VENDOR_AMD)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // AMD
            (0, 4, 0, 3, _) => brand_arch(MicroArch::Am486, "Am486DX2"),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::Am486, "Am486X2WB"),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::Am486, "Am486DX4"),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::Am486, "Am486DX4WB"),
            (0, 4, 0, 14, _) => brand_arch(MicroArch::Am5x86, "Am5x86"),
            (0, 4, 0, 15, _) => brand_arch(MicroArch::Am5x86, "Am5x86WB"),
            (0, 5, 0, 0, _) => brand_arch(MicroArch::SSA5, "SSA5 (K5)"),
            (0, 5, 0, 1..=3, _) => brand_arch(MicroArch::K5, "K5"),
            (0, 5, 0, 6 | 7, _) => brand_arch(MicroArch::K6, "K6"),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::K6, "Chompers/CXT (K6-2)"),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-III)"),
            (0, 5, 0, 13, _) => brand_arch(MicroArch::K6, "Sharptooth (K6-2+/K6-III+)"),
            (8, 15, 1, 1, 0) => brand_arch(MicroArch::Zen, "RavenRidge"),
            (10, 15, 2, 1, _) => brand_arch(MicroArch::Zen3, "Vermeer"),
            (10, 15, 6, 1, 2) => brand_arch(MicroArch::Zen4, "Raphael"),
            (10, 15, 7, 4, 1) => brand_arch(MicroArch::Zen4, "Phoenix"),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
        }
    }

    fn find_centaur(model: impl Into<String>, s: CpuSignature, vendor_string: &str) -> Self {
        let arch = |s: MicroArch, code_name: &'static str, brand_name: String| -> Self {
            CpuArch::new(model.into(), s, code_name, &brand_name, vendor_string)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // IDT
            (0, 5, 0, 4, _) => arch(MicroArch::Winchip, "C6", CpuBrand::IDT.into()),
            (0, 5, 0, 8, 5) => arch(MicroArch::Winchip2, "C2", CpuBrand::IDT.into()),
            (0, 5, 0, 8, 7) => arch(MicroArch::Winchip2A, "W2A", CpuBrand::IDT.into()),
            (0, 5, 0, 8, 10) => arch(MicroArch::Winchip2B, "W2B", CpuBrand::IDT.into()),
            (0, 5, 0, 9, _) => arch(MicroArch::Winchip3, "C3", CpuBrand::IDT.into()),

            // VIA
            (0, 6, 0, 6, _) => arch(MicroArch::Samuel, "Samuel (C5A)", CpuBrand::Via.into()),
            (0, 6, 0, 7, 0..=7) => arch(MicroArch::Samuel2, "Samuel 2 (C5B)", CpuBrand::Via.into()),
            (0, 6, 0, 7, 8..=15) => arch(MicroArch::Ezra, "Ezra (C5C)", CpuBrand::Via.into()),
            (0, 6, 0, 8, _) => arch(MicroArch::EzraT, "Ezra-T (C5N)", CpuBrand::Via.into()),
            (0, 6, 0, 9, 0..=7) => {
                arch(MicroArch::Nehemiah, "Nehemiah (C5XL)", CpuBrand::Via.into())
            }
            (0, 6, 0, 9, 8..=15) => arch(
                MicroArch::NehemiahP,
                "Nehemiah+ (C5P)",
                CpuBrand::Via.into(),
            ),
            (0, 6, 0, 10, _) => arch(MicroArch::Esther, "Esther (C5J)", CpuBrand::Via.into()),
            (0, 6, 1, 9 | 10 | 11 | 12, 8) => {
                arch(MicroArch::Isaiah, "Isaiah (CNS)", CpuBrand::Via.into())
            }
            (0, 6, 1, 15, _) => arch(MicroArch::Isaiah, "Isaiah (CN)", CpuBrand::Via.into()),

            // Zhaoxin
            (0, 7, 1, 11, 0) => arch(MicroArch::Wudaokou, "WuDaoKou", CpuBrand::Zhaoxin.into()),
            (0, 7, 3, 11, 0) => arch(MicroArch::Lujiazui, "LuJiaZui", CpuBrand::Zhaoxin.into()),
            (_, _, _, _, _) => arch(MicroArch::Unknown, "", CpuBrand::Unknown.into()),
        }
    }

    fn find_intel(model: impl Into<String>, s: CpuSignature) -> Self {
        let brand_arch = |s: MicroArch, code_name: &'static str| -> Self {
            CpuArch::new(model.into(), s, code_name, "Intel", VENDOR_INTEL)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // Pentium Pro
            (0, 6, 0, 1, 1 | 2 | 6..10) => brand_arch(MicroArch::P6Pro, "P6"),
            (0, 6, 1, 14, 5) => brand_arch(MicroArch::Nehalem, "Lynnfield"),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, "Unknown"),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBrand {
    AMD,
    Cyrix,
    DMP,
    Hygon,
    IDT,
    Intel,
    NationalSemiconductor,
    NexGen,
    Rise,
    SiS,
    Transmeta,
    Umc,
    Unknown,
    Via,
    Zhaoxin,
}

impl From<String> for CpuBrand {
    fn from(brand: String) -> Self {
        match brand.as_str() {
            VENDOR_AMD => CpuBrand::AMD,
            // Well, this one is more complicated...
            // "CentaurHauls" => CpuBrand::Via,
            VENDOR_CYRIX => CpuBrand::Cyrix,
            VENDOR_DMP => CpuBrand::Via,
            VENDOR_HYGON => CpuBrand::Hygon,
            VENDOR_INTEL => CpuBrand::Intel,
            VENDOR_NEXGEN => CpuBrand::NexGen,
            VENDOR_NSC => CpuBrand::NationalSemiconductor,
            VENDOR_RISE => CpuBrand::Rise,
            VENDOR_SIS => CpuBrand::SiS,
            VENDOR_TRANSMETA => CpuBrand::Transmeta,
            VENDOR_UMC => CpuBrand::Umc,
            _ => CpuBrand::Unknown,
        }
    }
}

impl Into<String> for CpuBrand {
    fn into(self) -> String {
        let s = match self {
            CpuBrand::AMD => "AMD",
            CpuBrand::Cyrix => "Cyrix",
            CpuBrand::DMP => "DM&P",
            CpuBrand::Hygon => "Hygon",
            CpuBrand::IDT => "IDT",
            CpuBrand::Intel => "Intel",
            CpuBrand::NationalSemiconductor => "National Semiconductor",
            CpuBrand::NexGen => "NexGen",
            CpuBrand::Rise => "Rise",
            CpuBrand::SiS => "SiS",
            CpuBrand::Transmeta => "Transmeta",
            CpuBrand::Umc => "UMC",
            CpuBrand::Via => "Via",
            CpuBrand::Zhaoxin => "Zhaoxin",
            _ => "Unknown",
        };

        s.to_string()
    }
}

#[derive(Debug)]
pub struct CpuFeatures {
    three_d_now: bool,
    mmx: bool,
    sse: bool,
    sse2: bool,
    sse3: bool,
    sse41: bool,
    sse42: bool,
    ssse3: bool,
    avx: bool,
    avx2: bool,
    avx512f: bool,
    fma: bool,
    bmi1: bool,
    bmi2: bool,
}

impl CpuFeatures {
    pub fn detect() -> Self {
        Self {
            three_d_now: fns::has_3dnow(),
            mmx: fns::has_mmx(),
            sse: fns::has_sse(),
            sse2: fns::has_sse2(),
            sse3: fns::has_sse3(),
            sse41: fns::has_sse41(),
            sse42: fns::has_sse42(),
            ssse3: fns::has_ssse3(),
            avx: fns::has_avx(),
            avx2: fns::has_avx2(),
            avx512f: fns::has_avx512f(),
            fma: fns::has_fma(),
            bmi1: fns::has_bmi1(),
            bmi2: fns::has_bmi2(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CpuSignature {
    extended_family: u32,
    family: u32,
    extended_model: u32,
    model: u32,
    stepping: u32,
    display_family: u32,
    display_model: u32,
}

impl CpuSignature {
    pub fn detect() -> Self {
        let res = cpuid::native_cpuid(1);
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

        Self {
            extended_model,
            extended_family,
            family,
            model,
            stepping,
            display_family,
            display_model,
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    cpu_arch: CpuArch,
    threads: u32,
    signature: CpuSignature,
    features: CpuFeatures,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            cpu_arch: CpuArch::find(
                fns::model_string(),
                CpuSignature::detect(),
                fns::vendor_id().as_str(),
            ),
            threads: fns::logical_cores(),
            signature: CpuSignature::detect(),
            features: CpuFeatures::detect(),
        }
    }

    pub fn display(&self) {
        println!("{:#?}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id() {
        let vendor = fns::vendor_id();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_model_string() {
        let model = fns::model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }
}
