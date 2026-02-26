use crate::cpuid::UNK;
use crate::cpuid::{fns, x86_cpuid};
use heapless::String;

pub const VENDOR_AMD: &str = "AuthenticAMD";
pub const VENDOR_CENTAUR: &str = "CentaurHauls";
pub const VENDOR_CYRIX: &str = "CyrixInstead";
pub const VENDOR_DMP: &str = "Vortex86 SoC";
pub const VENDOR_HYGON: &str = "HygonGenuine";
pub const VENDOR_INTEL: &str = "GenuineIntel";
pub const VENDOR_NEXGEN: &str = "NexGenDriven";
pub const VENDOR_NSC: &str = "Geode by NSC";
pub const VENDOR_RISE: &str = "RiseRiseRise";
pub const VENDOR_SIS: &str = "SiS SiS SiS ";
pub const VENDOR_TRANSMETA: &str = "GenuineTMx86";
pub const VENDOR_UMC: &str = "UMC UMC UMC ";
pub const VENDOR_ZHAOXIN: &str = "  Shanghai  ";

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

impl CpuBrand {
    pub fn detect() -> Self {
        let vendor_str = Self::vendor_str();
        let vendor_str = if vendor_str.is_empty() && fns::is_cyrix() {
            VENDOR_CYRIX
        } else {
            vendor_str.as_str()
        };

        Self::from(vendor_str)
    }

    /// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
    pub fn vendor_str() -> String<12> {
        let mut s = String::new();
        if !fns::has_cpuid() && fns::is_cyrix() {
            let _ = s.push_str(VENDOR_CYRIX);
            return s;
        }

        let res = x86_cpuid(0);
        let mut bytes = [0u8; 12];

        bytes[0..4].copy_from_slice(&res.ebx.to_le_bytes());
        bytes[4..8].copy_from_slice(&res.edx.to_le_bytes());
        bytes[8..12].copy_from_slice(&res.ecx.to_le_bytes());

        for &b in &bytes {
            if b != 0 {
                let _ = s.push(b as char);
            }
        }

        s
    }

    pub fn to_vendor_str(&self) -> &str {
        match self {
            CpuBrand::AMD => VENDOR_AMD,
            CpuBrand::Cyrix => VENDOR_CYRIX,
            CpuBrand::DMP => VENDOR_DMP,
            CpuBrand::Hygon => VENDOR_HYGON,
            CpuBrand::IDT | CpuBrand::Via | CpuBrand::Zhaoxin => VENDOR_CENTAUR,
            CpuBrand::Intel => VENDOR_INTEL,
            CpuBrand::NationalSemiconductor => VENDOR_NSC,
            CpuBrand::NexGen => VENDOR_NEXGEN,
            CpuBrand::Rise => VENDOR_RISE,
            CpuBrand::SiS => VENDOR_SIS,
            CpuBrand::Transmeta => VENDOR_TRANSMETA,
            CpuBrand::Umc => VENDOR_UMC,
            CpuBrand::Unknown => UNK,
        }
    }

    pub fn to_brand_name(&self) -> &str {
        match self {
            CpuBrand::AMD => "AMD",
            CpuBrand::Cyrix => "Cyrix/IBM/ST/TI",
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
            CpuBrand::Unknown => UNK,
        }
    }
}

impl From<&str> for CpuBrand {
    fn from(brand: &str) -> Self {
        match brand {
            VENDOR_AMD => CpuBrand::AMD,
            VENDOR_CYRIX => CpuBrand::Cyrix,
            VENDOR_DMP => CpuBrand::DMP,
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

impl From<String<12>> for CpuBrand {
    fn from(brand: String<12>) -> Self {
        Self::from(brand.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use std::println;

    #[test]
    fn test_vendor_id() {
        let vendor = CpuBrand::vendor_str();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_detect_cpu_brand() {
        let brand = CpuBrand::detect();
        // We cannot assert a specific brand as it depends on the CPU running the test.
        // Just ensure it doesn't panic.
        println!("Detected CPU Brand: {:?}", brand);
    }

    #[test]
    fn test_to_vendor_str() {
        assert_eq!(CpuBrand::AMD.to_vendor_str(), VENDOR_AMD);
        assert_eq!(CpuBrand::Cyrix.to_vendor_str(), VENDOR_CYRIX);
        assert_eq!(CpuBrand::DMP.to_vendor_str(), VENDOR_DMP);
        assert_eq!(CpuBrand::Hygon.to_vendor_str(), VENDOR_HYGON);
        assert_eq!(CpuBrand::IDT.to_vendor_str(), VENDOR_CENTAUR);
        assert_eq!(CpuBrand::Intel.to_vendor_str(), VENDOR_INTEL);
        assert_eq!(CpuBrand::NationalSemiconductor.to_vendor_str(), VENDOR_NSC);
        assert_eq!(CpuBrand::NexGen.to_vendor_str(), VENDOR_NEXGEN);
        assert_eq!(CpuBrand::Rise.to_vendor_str(), VENDOR_RISE);
        assert_eq!(CpuBrand::SiS.to_vendor_str(), VENDOR_SIS);
        assert_eq!(CpuBrand::Transmeta.to_vendor_str(), VENDOR_TRANSMETA);
        assert_eq!(CpuBrand::Umc.to_vendor_str(), VENDOR_UMC);
        assert_eq!(CpuBrand::Via.to_vendor_str(), VENDOR_CENTAUR);
        assert_eq!(CpuBrand::Zhaoxin.to_vendor_str(), VENDOR_CENTAUR);
        assert_eq!(CpuBrand::Unknown.to_vendor_str(), UNK);
    }

    #[test]
    fn test_to_brand_name() {
        assert_eq!(CpuBrand::AMD.to_brand_name(), "AMD");
        assert_eq!(CpuBrand::Cyrix.to_brand_name(), "Cyrix/IBM/ST/TI");
        assert_eq!(CpuBrand::DMP.to_brand_name(), "DM&P");
        assert_eq!(CpuBrand::Hygon.to_brand_name(), "Hygon");
        assert_eq!(CpuBrand::IDT.to_brand_name(), "IDT");
        assert_eq!(CpuBrand::Intel.to_brand_name(), "Intel");
        assert_eq!(
            CpuBrand::NationalSemiconductor.to_brand_name(),
            "National Semiconductor"
        );
        assert_eq!(CpuBrand::NexGen.to_brand_name(), "NexGen");
        assert_eq!(CpuBrand::Rise.to_brand_name(), "Rise");
        assert_eq!(CpuBrand::SiS.to_brand_name(), "SiS");
        assert_eq!(CpuBrand::Transmeta.to_brand_name(), "Transmeta");
        assert_eq!(CpuBrand::Umc.to_brand_name(), "UMC");
        assert_eq!(CpuBrand::Via.to_brand_name(), "Via");
        assert_eq!(CpuBrand::Zhaoxin.to_brand_name(), "Zhaoxin");
        assert_eq!(CpuBrand::Unknown.to_brand_name(), UNK);
    }

    #[test]
    fn test_from_str_for_cpu_brand() {
        assert_eq!(CpuBrand::from(VENDOR_AMD), CpuBrand::AMD);
        assert_eq!(CpuBrand::from(VENDOR_CYRIX), CpuBrand::Cyrix);
        assert_eq!(CpuBrand::from(VENDOR_DMP), CpuBrand::DMP);
        assert_eq!(CpuBrand::from(VENDOR_HYGON), CpuBrand::Hygon);
        assert_eq!(CpuBrand::from(VENDOR_INTEL), CpuBrand::Intel);
        assert_eq!(CpuBrand::from(VENDOR_NEXGEN), CpuBrand::NexGen);
        assert_eq!(CpuBrand::from(VENDOR_NSC), CpuBrand::NationalSemiconductor);
        assert_eq!(CpuBrand::from(VENDOR_RISE), CpuBrand::Rise);
        assert_eq!(CpuBrand::from(VENDOR_SIS), CpuBrand::SiS);
        assert_eq!(CpuBrand::from(VENDOR_TRANSMETA), CpuBrand::Transmeta);
        assert_eq!(CpuBrand::from(VENDOR_UMC), CpuBrand::Umc);
        assert_eq!(CpuBrand::from("SomeOtherVendor"), CpuBrand::Unknown);
    }

    #[test]
    fn test_from_heapless_string_for_cpu_brand() {
        let amd_string: String<12> = String::from_str(VENDOR_AMD).unwrap();
        assert_eq!(CpuBrand::from(amd_string), CpuBrand::AMD);

        let unknown_string: String<12> = String::from_str("UNKNOWN_VEN").unwrap();
        assert_eq!(CpuBrand::from(unknown_string), CpuBrand::Unknown);
    }
}
