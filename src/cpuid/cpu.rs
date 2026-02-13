use crate::cpuid;
use crate::cpuid::fns;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBrand {
    AMD,
    Cyrix,
    Hygon,
    IDT,
    Intel,
    NationalSemiconductor,
    Rise,
    Umc,
    Unknown,
    Via,
    Zhaoxin,
}

impl From<&String> for CpuBrand {
    fn from(brand: &String) -> Self {
        match brand.as_str() {
            "AuthenticAMD" => CpuBrand::AMD,
            // Well, this one is more complicated...
            "CentaurHauls" => CpuBrand::Via,
            "CyrixInstead" => CpuBrand::Cyrix,
            "GenuineIntel" => CpuBrand::Intel,
            "Geode by NSC" => CpuBrand::NationalSemiconductor,
            "RiseRiseRise" => CpuBrand::Rise,
            "UMC UMC UMC " => CpuBrand::Umc,
            _ => CpuBrand::Unknown,
        }
    }
}

impl Into<String> for CpuBrand {
    fn into(self) -> String {
        match self {
            CpuBrand::AMD => "AMD".to_string(),
            CpuBrand::Cyrix => "Cyrix".to_string(),
            CpuBrand::Hygon => "Hygon".to_string(),
            CpuBrand::IDT => "IDT".to_string(),
            CpuBrand::Intel => "Intel".to_string(),
            CpuBrand::NationalSemiconductor => "National Semiconductor".to_string(),
            CpuBrand::Rise => "Rise".to_string(),
            CpuBrand::Umc => "UMC".to_string(),
            CpuBrand::Unknown => "Unknown".to_string(),
            CpuBrand::Via => "Via".to_string(),
            CpuBrand::Zhaoxin => "Zhaoxin".to_string(),
        }
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
    brand: String,
    model: String,
    threads: u32,
    vendor_string: String,
    signature: CpuSignature,
    features: CpuFeatures,
}

impl Cpu {
    pub fn new() -> Self {
        let vendor_string = fns::vendor_id();

        Self {
            brand: CpuBrand::from(&vendor_string).into(),
            model: fns::model_string(),
            threads: fns::logical_cores(),
            vendor_string,
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
