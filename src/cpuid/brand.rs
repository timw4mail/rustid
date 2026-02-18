use crate::cpuid::x86_cpuid;

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
    pub fn vendor_id() -> String {
        let res = x86_cpuid(0);
        let mut bytes = Vec::with_capacity(12);
        for &reg in &[res.ebx, res.edx, res.ecx] {
            bytes.extend_from_slice(&reg.to_le_bytes());
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }
}
impl From<String> for CpuBrand {
    fn from(brand: String) -> Self {
        match brand.as_str() {
            VENDOR_AMD => CpuBrand::AMD,
            // Well, this one is more complicated...
            // "CentaurHauls" => CpuBrand::Via,
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
