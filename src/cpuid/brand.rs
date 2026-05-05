//! CPU Vendor Identification
//!
//! This module provides CPU vendor detection for x86/x86_64 processors
//! using the CPUID instruction.

use super::constants::*;
use alloc::string::String;

/// CPU brand/vendor enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBrand {
    AMD,

    #[cfg(target_arch = "x86")]
    Cyrix,

    #[cfg(target_arch = "x86")]
    DMP,

    Hygon,

    #[cfg(target_arch = "x86")]
    IDT,

    Intel,

    #[cfg(target_arch = "x86")]
    NationalSemiconductor,

    #[cfg(target_arch = "x86")]
    NexGen,

    #[cfg(target_arch = "x86")]
    Rdc,

    #[cfg(target_arch = "x86")]
    Rise,

    #[cfg(target_arch = "x86")]
    SiS,

    #[cfg(target_arch = "x86")]
    Transmeta,

    #[cfg(target_arch = "x86")]
    Umc,

    Unknown,

    Via,

    Zhaoxin,
}

impl CpuBrand {
    /// Detects the CPU brand/vendor from CPUID information.
    pub fn detect() -> Self {
        let vendor_str = super::vendor_str();

        Self::from(vendor_str)
    }

    /// Converts the CPU brand to its vendor ID string (e.g., "AuthenticAMD").
    pub fn to_vendor_str(&self) -> &str {
        match self {
            CpuBrand::AMD => VENDOR_AMD,

            #[cfg(target_arch = "x86")]
            CpuBrand::Cyrix => VENDOR_CYRIX,

            #[cfg(target_arch = "x86")]
            CpuBrand::DMP => VENDOR_DMP,

            CpuBrand::Hygon => VENDOR_HYGON,

            #[cfg(target_arch = "x86")]
            CpuBrand::IDT => VENDOR_CENTAUR,

            CpuBrand::Via | CpuBrand::Zhaoxin => VENDOR_CENTAUR,

            CpuBrand::Intel => VENDOR_INTEL,

            #[cfg(target_arch = "x86")]
            CpuBrand::NationalSemiconductor => VENDOR_NSC,

            #[cfg(target_arch = "x86")]
            CpuBrand::NexGen => VENDOR_NEXGEN,

            #[cfg(target_arch = "x86")]
            CpuBrand::Rdc => VENDOR_RDC,

            #[cfg(target_arch = "x86")]
            CpuBrand::Rise => VENDOR_RISE,

            #[cfg(target_arch = "x86")]
            CpuBrand::SiS => VENDOR_SIS,

            #[cfg(target_arch = "x86")]
            CpuBrand::Transmeta => VENDOR_TRANSMETA,

            #[cfg(target_arch = "x86")]
            CpuBrand::Umc => VENDOR_UMC,

            CpuBrand::Unknown => UNK,
        }
    }

    /// Converts the CPU brand to a human-readable name (e.g., "AMD", "Intel").
    pub fn to_brand_name(&self) -> &str {
        match self {
            CpuBrand::AMD => "AMD",

            #[cfg(target_arch = "x86")]
            CpuBrand::Cyrix => super::vendor::cyrix::Cyrix::brand_string(),

            #[cfg(target_arch = "x86")]
            CpuBrand::DMP => "DM&P",

            CpuBrand::Hygon => "Hygon",

            #[cfg(target_arch = "x86")]
            CpuBrand::IDT => "IDT",

            CpuBrand::Intel => "Intel",

            #[cfg(target_arch = "x86")]
            CpuBrand::NationalSemiconductor => "National Semiconductor",

            #[cfg(target_arch = "x86")]
            CpuBrand::NexGen => "NexGen",

            #[cfg(target_arch = "x86")]
            CpuBrand::Rdc => "RDC",

            #[cfg(target_arch = "x86")]
            CpuBrand::Rise => "Rise",

            #[cfg(target_arch = "x86")]
            CpuBrand::SiS => "SiS",

            #[cfg(target_arch = "x86")]
            CpuBrand::Transmeta => "Transmeta",

            #[cfg(target_arch = "x86")]
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

            #[cfg(target_arch = "x86")]
            VENDOR_CYRIX => CpuBrand::Cyrix,

            #[cfg(target_arch = "x86")]
            VENDOR_DMP => CpuBrand::DMP,

            VENDOR_HYGON => CpuBrand::Hygon,

            VENDOR_INTEL => CpuBrand::Intel,

            #[cfg(target_arch = "x86")]
            VENDOR_NEXGEN => CpuBrand::NexGen,

            #[cfg(target_arch = "x86")]
            VENDOR_NSC => CpuBrand::NationalSemiconductor,

            #[cfg(target_arch = "x86")]
            VENDOR_RDC => CpuBrand::Rdc,

            #[cfg(target_arch = "x86")]
            VENDOR_RISE => CpuBrand::Rise,

            #[cfg(target_arch = "x86")]
            VENDOR_SIS => CpuBrand::SiS,

            #[cfg(target_arch = "x86")]
            VENDOR_TRANSMETA => CpuBrand::Transmeta,

            #[cfg(target_arch = "x86")]
            VENDOR_UMC => CpuBrand::Umc,

            VENDOR_ZHAOXIN => CpuBrand::Zhaoxin,

            _ => CpuBrand::Unknown,
        }
    }
}

impl From<String> for CpuBrand {
    fn from(brand: String) -> Self {
        Self::from(&*brand)
    }
}

pub enum HypervisorBrand {
    Bhyve,
    MicrosoftHyperV,
    LinuxKVM,
    Parallels,
    Qemu,
    Qnx,
    VirtualBox,
    VmWare,
    Xen,
    Unknown,
}

impl HypervisorBrand {
    pub fn detect() -> Self {
        let str = super::hypervisor_str();
        Self::from(str.as_str())
    }

    pub fn to_str(&self) -> &'static str {
        match &self {
            HypervisorBrand::Bhyve => "Bhyve",
            HypervisorBrand::MicrosoftHyperV => "Microsoft HyperV",
            HypervisorBrand::LinuxKVM => "Linux KVM",
            HypervisorBrand::Parallels => "Parallels",
            HypervisorBrand::Qemu => "QEMU",
            HypervisorBrand::Qnx => "QNX",
            HypervisorBrand::VirtualBox => "VirtualBox",
            HypervisorBrand::VmWare => "VMWare",
            HypervisorBrand::Xen => "Xen",
            HypervisorBrand::Unknown => UNK,
        }
    }
}

impl From<&str> for HypervisorBrand {
    fn from(s: &str) -> Self {
        match s {
            HYP_VENDOR_BHYVE => HypervisorBrand::Bhyve,
            HYP_VENDOR_HYPERV => HypervisorBrand::MicrosoftHyperV,
            HYP_VENDOR_KVM => HypervisorBrand::LinuxKVM,
            HYP_VENDOR_PARALLELS | HYP_VENDOR_PARALLELS_ALT => HypervisorBrand::Parallels,
            HYP_VENDOR_QEMU => HypervisorBrand::Qemu,
            HYP_VENDOR_QNX => HypervisorBrand::Qnx,
            HYP_VENDOR_VBOX => HypervisorBrand::VirtualBox,
            HYP_VENDOR_VMWARE => HypervisorBrand::VmWare,
            HYP_VENDOR_XEN => HypervisorBrand::Xen,
            _ => HypervisorBrand::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_vendor_str() {
        assert_eq!(CpuBrand::AMD.to_vendor_str(), VENDOR_AMD);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Cyrix.to_vendor_str(), VENDOR_CYRIX);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::DMP.to_vendor_str(), VENDOR_DMP);

        assert_eq!(CpuBrand::Hygon.to_vendor_str(), VENDOR_HYGON);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::IDT.to_vendor_str(), VENDOR_CENTAUR);

        assert_eq!(CpuBrand::Intel.to_vendor_str(), VENDOR_INTEL);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::NationalSemiconductor.to_vendor_str(), VENDOR_NSC);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::NexGen.to_vendor_str(), VENDOR_NEXGEN);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Rdc.to_vendor_str(), VENDOR_RDC);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Rise.to_vendor_str(), VENDOR_RISE);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::SiS.to_vendor_str(), VENDOR_SIS);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Transmeta.to_vendor_str(), VENDOR_TRANSMETA);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Umc.to_vendor_str(), VENDOR_UMC);

        assert_eq!(CpuBrand::Via.to_vendor_str(), VENDOR_CENTAUR);

        assert_eq!(CpuBrand::Zhaoxin.to_vendor_str(), VENDOR_CENTAUR);

        assert_eq!(CpuBrand::Unknown.to_vendor_str(), UNK);
    }

    #[test]
    fn test_to_brand_name() {
        assert_eq!(CpuBrand::AMD.to_brand_name(), "AMD");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::DMP.to_brand_name(), "DM&P");

        assert_eq!(CpuBrand::Hygon.to_brand_name(), "Hygon");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::IDT.to_brand_name(), "IDT");

        assert_eq!(CpuBrand::Intel.to_brand_name(), "Intel");

        #[cfg(target_arch = "x86")]
        assert_eq!(
            CpuBrand::NationalSemiconductor.to_brand_name(),
            "National Semiconductor"
        );

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::NexGen.to_brand_name(), "NexGen");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Rise.to_brand_name(), "Rise");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::SiS.to_brand_name(), "SiS");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Transmeta.to_brand_name(), "Transmeta");

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::Umc.to_brand_name(), "UMC");

        assert_eq!(CpuBrand::Via.to_brand_name(), "Via");

        assert_eq!(CpuBrand::Zhaoxin.to_brand_name(), "Zhaoxin");

        assert_eq!(CpuBrand::Unknown.to_brand_name(), UNK);
    }

    #[test]
    fn test_from_str_for_cpu_brand() {
        assert_eq!(CpuBrand::from(VENDOR_AMD), CpuBrand::AMD);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_CYRIX), CpuBrand::Cyrix);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_DMP), CpuBrand::DMP);

        assert_eq!(CpuBrand::from(VENDOR_HYGON), CpuBrand::Hygon);

        assert_eq!(CpuBrand::from(VENDOR_INTEL), CpuBrand::Intel);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_NEXGEN), CpuBrand::NexGen);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_NSC), CpuBrand::NationalSemiconductor);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_RISE), CpuBrand::Rise);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_SIS), CpuBrand::SiS);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_TRANSMETA), CpuBrand::Transmeta);

        #[cfg(target_arch = "x86")]
        assert_eq!(CpuBrand::from(VENDOR_UMC), CpuBrand::Umc);

        assert_eq!(CpuBrand::from("SomeOtherVendor"), CpuBrand::Unknown);
    }

    #[test]
    fn test_from_cpuid_str_for_cpu_brand() {
        let amd_string = String::from(VENDOR_AMD);
        assert_eq!(CpuBrand::from(amd_string), CpuBrand::AMD);

        let unknown_string = String::from("UNKNOWN_VEN");
        assert_eq!(CpuBrand::from(unknown_string), CpuBrand::Unknown);
    }
}
