pub const IMPL_ARM: usize = 0x41;
pub const IMPL_QUALCOMM: usize = 0x51;
pub const IMPL_APPLE: usize = 0x61;

#[allow(unused)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Vendor {
    #[default]
    Arm,
    AMC,
    Ampere,
    Apple,
    Broadcom,
    Cavium,
    DEC,
    Freescale,
    Fujitsu,
    Infineon,
    Intel,
    Marvell,
    Mediatek,
    Nvidia,
    Rockchip,
    Samsung,
    Qualcomm,
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
            Arm => "ARM",
            AMC => "Applied Micro Circuits Corporation",
            Ampere => "Ampere Computing",
            Apple => "Apple",
            Broadcom => "Broadcom",
            Cavium => "Cavium",
            DEC => "DEC",
            Freescale => "Motorolla or Freescale Semiconductor",
            Fujitsu => "Fujitsu",
            Infineon => "Infineon",
            Intel => "Intel",
            Marvell => "Marvell",
            Mediatek => "Mediatek",
            Nvidia => "Nvidia",
            Rockchip => "Rockchip",
            Samsung => "Samsung",
            Qualcomm => "Qualcomm",
            Unknown => "Unknown",
        }
    }
}

impl From<usize> for Vendor {
    fn from(v: usize) -> Self {
        // Several mappings taken from https://github.com/bp0/armids/blob/master/arm.ids
        // See also: https://developer.arm.com/documentation/111107/2026-03/AArch32-Registers/MIDR--Main-ID-Register
        match v {
            0x41 => Self::Arm,
            0x42 => Self::Broadcom,
            0x43 => Self::Cavium,
            0x44 => Self::DEC,
            0x46 => Self::Fujitsu,
            0x49 => Self::Infineon,
            0x4d => Self::Freescale,
            0x4e => Self::Nvidia,
            0x50 => Self::AMC,
            0x51 => Self::Qualcomm,
            0x53 => Self::Samsung,
            0x56 => Self::Marvell,
            0x61 => Self::Apple,
            0x69 => Self::Intel,
            0xC0 => Self::Ampere,
            _ => Self::Unknown,
        }
    }
}
