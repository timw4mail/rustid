pub const IMPL_ARM: usize = 0x41;
pub const IMPL_QUALCOMM: usize = 0x51;
pub const IMPL_APPLE: usize = 0x61;

#[allow(unused)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Vendor {
    #[default]
    Arm,
    Apple,
    Broadcom,
    Intel,
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
            Apple => "Apple",
            Broadcom => "Broadcom",
            Intel => "Intel",
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
        match v {
            0x41 => Self::Arm,
            0x42 => Self::Broadcom,
            0x4e => Self::Nvidia,
            0x51 => Self::Qualcomm,
            0x53 => Self::Samsung,
            0x61 => Self::Apple,
            0x69 => Self::Intel,
            _ => Self::Unknown,
        }
    }
}
