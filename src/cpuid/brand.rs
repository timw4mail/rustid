use crate::cpuid::x86_cpuid;
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
        Self::from(Self::vendor_id())
    }

    /// Gets the CPU vendor ID string (e.g., "GenuineIntel", "AuthenticAMD").
    pub fn vendor_id() -> String<12> {
        let res = x86_cpuid(0);
        let mut bytes = [0u8; 12];

        bytes[0..4].copy_from_slice(&res.ebx.to_le_bytes());
        bytes[4..8].copy_from_slice(&res.edx.to_le_bytes());
        bytes[8..12].copy_from_slice(&res.ecx.to_le_bytes());

        let mut s = String::new();
        for &b in &bytes {
            if b != 0 {
                let _ = s.push(b as char);
            }
        }

        s
    }

    pub fn as_str(&self) -> &str {
        match self {
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
