#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub enum Vendor {
    ARM,
    Apple,
    Broadcom,
    Mediatek,
    Rockchip,
    Qualcomm,
    Unknown,
}

impl Into<String> for Vendor {
    fn into(self) -> String {
        use Vendor::*;

        let s = match self {
            ARM => "ARM",
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
            0x41 => Self::ARM,
            0xB3 => Self::Apple,
            _ => Self::Unknown,
        }
    }
}
