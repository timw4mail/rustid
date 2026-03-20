#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub enum Vendor {
    Arm,
    Apple,
    Broadcom,
    Mediatek,
    Rockchip,
    Qualcomm,
    Unknown,
}

impl From<Vendor> for String {
    fn from(val: Vendor) -> Self {
        use Vendor::*;

        let s = match val {
            Arm => "ARM",
            Apple => "Apple",
            Broadcom => "Broadcom",
            Mediatek => "Mediatek",
            Rockchip => "Rockchip",
            Qualcomm => "Qualcomm",
            Unknown => "Unknown",
        };

        String::from(s)
    }
}

impl From<usize> for Vendor {
    fn from(v: usize) -> Self {
        match v {
            0x41 => Self::Arm,
            0x61 | 0xB3 | 0xB9 | 0x63 => Self::Apple,
            _ => Self::Unknown,
        }
    }
}
