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
            CpuArch::new(model, s, code_name, &brand_name, vendor_string)
        };

        // Brand for Centaur CPUs...is complicated
        if vendor_string == VENDOR_CENTAUR {
            return match (
                s.extended_family,
                s.family,
                s.extended_model,
                s.model,
                s.stepping,
            ) {
                (_, _, _, _, _) => arch(MicroArch::Unknown, "", CpuBrand::Unknown.into()),
            };
        }

        let brand = CpuBrand::from(vendor_string.to_string());
        let brand_arch = |s: MicroArch, code_name: &'static str | -> Self {
            arch(s, code_name, brand.into())
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            (8, 15, 1, 1, 0) => brand_arch(MicroArch::Zen, "RavenRidge"),
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, ""),
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
    Isiah,

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
    Infineon,

    // UMC
    U5S,
    U5D
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
            _ => "Unknown"
        };

        s.to_string()
    }
}

#[derive(Debug)]
pub struct CpuFeatures {
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
