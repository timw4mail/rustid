#![allow(unused)]

pub const IMPL_AMC: usize = 0x50;
pub const IMPL_AMPERE: usize = 0xC0;
pub const IMPL_APPLE: usize = 0x61;
pub const IMPL_ARM: usize = 0x41;

pub const IMPL_BROADCOM: usize = 0x42;
pub const IMPL_CAVIUM: usize = 0x43;
pub const IMPL_DEC: usize = 0x44;
pub const IMPL_FARADAY: usize = 0x66;
pub const IMPL_FREESCALE: usize = 0x4D;
pub const IMPL_FUJITSU: usize = 0x46;
pub const IMPL_HISILICON: usize = 0x48;
pub const IMPL_INFINEON: usize = 0x49;
pub const IMPL_INTEL: usize = 0x69;
pub const IMPL_MARVELL: usize = 0x56;
pub const IMPL_MICROSOFT: usize = 0x6D;
pub const IMPL_NVIDIA: usize = 0x4E;
pub const IMPL_PHYTIUM: usize = 0x70;
pub const IMPL_QUALCOMM: usize = 0x51;
pub const IMPL_SAMSUNG: usize = 0x53;

#[allow(unused)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Vendor {
    Amc,
    Ampere,
    Apple,
    #[default]
    Arm,
    Broadcom,
    Cavium,
    Dec,
    Faraday,
    Freescale,
    Fujitsu,
    HiSilicon,
    Infineon,
    Intel,
    Marvell,
    Mediatek,
    Microsoft,
    Nvidia,
    Phytium,
    Qualcomm,
    Rockchip,
    Samsung,
    Unknown,
}

impl From<Vendor> for String {
    fn from(val: Vendor) -> Self {
        let str: &'static str = val.into();

        String::from(str)
    }
}

impl From<Vendor> for &'static str {
    fn from(val: Vendor) -> &'static str {
        use Vendor::*;

        match val {
            Amc => "Applied Micro Circuits Corporation",
            Ampere => "Ampere Computing",
            Apple => "Apple",
            Arm => "ARM",
            Broadcom => "Broadcom",
            Cavium => "Cavium",
            Dec => "DEC",
            Faraday => "Faraday",
            Freescale => "Motorola or Freescale Semiconductor",
            Fujitsu => "Fujitsu",
            HiSilicon => "HiSilicon",
            Infineon => "Infineon",
            Intel => "Intel",
            Marvell => "Marvell",
            Mediatek => "Mediatek",
            Microsoft => "Microsoft",
            Nvidia => "Nvidia",
            Phytium => "Phytium",
            Qualcomm => "Qualcomm",
            Rockchip => "Rockchip",
            Samsung => "Samsung",
            Unknown => "Unknown",
        }
    }
}

impl From<usize> for Vendor {
    /// Maps MIDR implementer byte (bits [31:24]) to CPU vendor.
    ///
    /// Reference Data Sources:
    /// - util-linux: https://github.com/util-linux/util-linux/blob/master/sys-utils/lscpu-arm.c
    /// - pytorch/cpuinfo: https://github.com/pytorch/cpuinfo/blob/main/src/arm/uarch.c
    /// - Linux kernel: arch/arm64/include/asm/cputype.h
    /// - bp0/armids: https://github.com/bp0/armids/blob/master/arm.ids
    fn from(v: usize) -> Self {
        match v {
            0x41 => Self::Arm,
            0x42 => Self::Broadcom,
            0x43 => Self::Cavium,
            0x44 => Self::Dec,
            0x46 => Self::Fujitsu,
            0x48 => Self::HiSilicon,
            0x49 => Self::Infineon,
            0x4D => Self::Freescale,
            0x4E => Self::Nvidia,
            0x50 => Self::Amc,
            0x51 => Self::Qualcomm,
            0x53 => Self::Samsung,
            0x56 => Self::Marvell,
            0x61 => Self::Apple,
            0x66 => Self::Faraday,
            0x69 => Self::Intel,
            0x6D => Self::Microsoft,
            0x70 => Self::Phytium,
            0xC0 => Self::Ampere,
            _ => Self::Unknown,
        }
    }
}
