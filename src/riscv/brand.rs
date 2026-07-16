//! RISC-V vendor ID mapping from the `mvendorid` CSR.

/// SiFive, Inc.
pub const VENDOR_SIFIVE: usize = 0x489;
/// T-Head (Alibaba)
pub const VENDOR_THEAD: usize = 0x5b7;
/// StarFive Technology
pub const VENDOR_STARFIVE: usize = 0x1bc;
/// Kendryte (Canaan)
pub const VENDOR_KENDRYTE: usize = 0x31e;
/// WCH (Nanjing Qinheng Microelectronics)
pub const VENDOR_WCH: usize = 0x1fc;

#[allow(unused)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Vendor {
    #[default]
    Unknown,
    SiFive,
    THead,
    StarFive,
    Kendryte,
    WCH,
}

impl From<Vendor> for String {
    fn from(val: Vendor) -> Self {
        let s: &'static str = val.into();
        String::from(s)
    }
}

impl From<Vendor> for &'static str {
    fn from(val: Vendor) -> &'static str {
        use Vendor::*;
        match val {
            Unknown => "Unknown",
            SiFive => "SiFive",
            THead => "T-Head",
            StarFive => "StarFive",
            Kendryte => "Canaan",
            WCH => "WCH",
        }
    }
}

impl From<usize> for Vendor {
    fn from(v: usize) -> Self {
        match v {
            VENDOR_SIFIVE => Self::SiFive,
            VENDOR_THEAD => Self::THead,
            VENDOR_STARFIVE => Self::StarFive,
            VENDOR_KENDRYTE => Self::Kendryte,
            VENDOR_WCH => Self::WCH,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_from_sifive() {
        assert_eq!(Vendor::from(0x489), Vendor::SiFive);
    }

    #[test]
    fn test_vendor_from_thead() {
        assert_eq!(Vendor::from(0x5b7), Vendor::THead);
    }

    #[test]
    fn test_vendor_unknown() {
        assert_eq!(Vendor::from(0x999), Vendor::Unknown);
    }

    #[test]
    fn test_vendor_to_str() {
        let v: &str = Vendor::SiFive.into();
        assert_eq!(v, "SiFive");
    }
}
