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
/// Nuclei System Technology
pub const VENDOR_NUCLEI: usize = 0xc23;
/// XiangShan (ICT, CAS)
pub const VENDOR_XIANGSHAN: usize = 0x557;

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
    Nuclei,
    XiangShan,
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
            Nuclei => "Nuclei",
            XiangShan => "XiangShan",
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
            VENDOR_NUCLEI => Self::Nuclei,
            VENDOR_XIANGSHAN => Self::XiangShan,
            _ => Self::Unknown,
        }
    }
}

pub fn format_uarch(raw: &str) -> String {
    let parts: Vec<&str> = raw.splitn(2, ',').collect();
    let vendor_str = match parts[0] {
        "sifive" => "SiFive",
        "thead" => "T-Head",
        "starfive" => "StarFive",
        "kendryte" => "Canaan",
        "wch" => "WCH",
        "nuclei" => "Nuclei",
        "xiangshan" => "XiangShan",
        other => {
            let mut chars = other.chars();
            match chars.next() {
                None => return String::new(),
                Some(first) => {
                    let rest: String = chars.collect();
                    return format!("{}{}", first.to_uppercase(), rest.to_lowercase());
                }
            }
        }
    };
    if parts.len() > 1 {
        format!("{} {}", vendor_str, parts[1].to_uppercase())
    } else {
        String::from(vendor_str)
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

    #[test]
    fn test_format_uarch_sifive_with_suffix() {
        assert_eq!(format_uarch("sifive,u74-mc"), "SiFive U74-MC");
    }

    #[test]
    fn test_format_uarch_sifive_simple() {
        assert_eq!(format_uarch("sifive,u74"), "SiFive U74");
    }

    #[test]
    fn test_format_uarch_thead() {
        assert_eq!(format_uarch("thead,c906"), "T-Head C906");
    }

    #[test]
    fn test_format_uarch_kendryte() {
        assert_eq!(format_uarch("kendryte,k210"), "Canaan K210");
    }

    #[test]
    fn test_format_uarch_no_vendor() {
        assert_eq!(format_uarch("u74"), "U74");
    }

    #[test]
    fn test_format_uarch_empty() {
        assert_eq!(format_uarch(""), "");
    }
}
