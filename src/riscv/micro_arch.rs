//! RISC-V microarchitecture identification.
//!
//! Parses the `misa` CSR to determine ISA extensions, and maps
//! vendor/architecture IDs to known CPU cores.

use crate::common::CoreType;
use crate::common::constants::*;
use crate::common::{Cache, UNK};
use crate::riscv::brand::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CpuCore {
    pub kind: CoreType,
    pub name: Option<String>,
    pub cache: Option<Cache>,
    pub count: u32,
}

/// RISC-V `misa` register layout.
///
/// Bits 63:62 — XLEN (00=32, 01=64, 10=128)
/// Bits 61:0  — Extension bitmap (bit 0 = A, bit 1 = B, ..., bit 25 = Z)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Misa {
    pub xlen: usize,
    pub extensions: u64,
}

impl Misa {
    pub fn from_raw(val: u64) -> Self {
        let xlen_raw = (val >> 62) & 0x3;
        let xlen = match xlen_raw {
            1 => 64,
            2 => 128,
            _ => 32,
        };
        Misa {
            xlen,
            extensions: val & 0x3FFF_FFFF_FFFF_FFFF,
        }
    }

    /// Returns true if the given single-letter extension is present.
    pub fn has_ext(&self, ch: char) -> bool {
        let c = ch.to_ascii_uppercase();
        if c < 'A' || c > 'Z' {
            return false;
        }
        let bit = (c as u64) - ('A' as u64);
        (self.extensions >> bit) & 1 == 1
    }

    /// Returns the ISA string representation (e.g. "rv64gc").
    pub fn to_isa_string(&self) -> String {
        let prefix = match self.xlen {
            32 => "rv32",
            64 => "rv64",
            128 => "rv128",
            _ => "rv??",
        };
        let mut exts = String::new();
        // Standard order: I, M, A, F, D, C, V, plus others alphabetically
        let order = [
            'I', 'M', 'A', 'F', 'D', 'Q', 'C', 'V', 'H', 'S', 'U', 'B', 'K', 'J', 'T', 'P',
        ];
        for ch in order {
            if self.has_ext(ch) {
                exts.push(ch.to_ascii_lowercase());
            }
        }
        // Append any remaining extensions alphabetically
        for i in 0..26 {
            let ch = (b'A' + i) as char;
            if self.has_ext(ch) && !order.contains(&ch) {
                exts.push(ch.to_ascii_lowercase());
            }
        }
        format!("{prefix}{exts}")
    }
}

/// Known RISC-V microarchitectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    Unknown,

    // SiFive
    SiFiveU74,
    SiFiveU76,
    SiFiveS76,
    SiFiveP450,
    SiFiveP470,
    SiFiveP550,
    SiFiveP650,
    SiFiveP670,
    SiFiveP870,

    // T-Head
    THeadC906,
    THeadC910,
    THeadC920,
    THeadC930,

    // Kendryte
    KendryteK210,

    // WCH
    WCHCH32V,
}

impl MicroArch {
    pub fn core_type(&self) -> CoreType {
        match self {
            MicroArch::Unknown => CoreType::Performance,

            MicroArch::SiFiveU74
            | MicroArch::SiFiveU76
            | MicroArch::SiFiveS76
            | MicroArch::SiFiveP450
            | MicroArch::SiFiveP470
            | MicroArch::SiFiveP550
            | MicroArch::SiFiveP650
            | MicroArch::SiFiveP670
            | MicroArch::SiFiveP870
            | MicroArch::THeadC906
            | MicroArch::THeadC910
            | MicroArch::THeadC920
            | MicroArch::THeadC930
            | MicroArch::KendryteK210
            | MicroArch::WCHCH32V => CoreType::Performance,
        }
    }
}

impl MicroArch {
    pub fn as_str(&self) -> &str {
        match self {
            MicroArch::Unknown => UNK,
            MicroArch::SiFiveU74 => "SiFive U74",
            MicroArch::SiFiveU76 => "SiFive U76",
            MicroArch::SiFiveS76 => "SiFive S76",
            MicroArch::SiFiveP450 => "SiFive P450",
            MicroArch::SiFiveP470 => "SiFive P470",
            MicroArch::SiFiveP550 => "SiFive P550",
            MicroArch::SiFiveP650 => "SiFive P650",
            MicroArch::SiFiveP670 => "SiFive P670",
            MicroArch::SiFiveP870 => "SiFive P870",
            MicroArch::THeadC906 => "T-Head C906",
            MicroArch::THeadC910 => "T-Head C910",
            MicroArch::THeadC920 => "T-Head C920",
            MicroArch::THeadC930 => "T-Head C930",
            MicroArch::KendryteK210 => "Kendryte K210",
            MicroArch::WCHCH32V => "WCH CH32V",
        }
    }
}

impl From<MicroArch> for String {
    fn from(ma: MicroArch) -> String {
        String::from(ma.as_str())
    }
}

/// CPU architecture identification from vendor + architecture IDs.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub vendor: Vendor,
    pub model: String,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub marchid: usize,
    pub technology: Option<&'static str>,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self {
            vendor: Vendor::default(),
            model: String::from(UNK),
            micro_arch: MicroArch::Unknown,
            code_name: UNK,
            marchid: 0,
            technology: None,
        }
    }
}

impl CpuArch {
    pub fn find(vendor: usize, marchid: usize) -> Self {
        match vendor {
            VENDOR_SIFIVE => Self::find_sifive(marchid),
            VENDOR_THEAD => Self::find_thead(marchid),
            VENDOR_STARFIVE => Self::find_starfive(marchid),
            VENDOR_KENDRYTE => Self::find_kendryte(marchid),
            VENDOR_WCH => Self::find_wch(marchid),
            _ => Self {
                vendor: Vendor::from(vendor),
                ..Self::default()
            },
        }
    }

    /// Try to identify the CPU from a device tree `compatible` string.
    ///
    /// This is used as a fallback when CSR-based identification fails.
    pub fn find_by_compatible(compat: &str) -> Option<Self> {
        let lower = compat.to_lowercase();
        // Check for comma-separated compatible strings (DT can have multiple)
        for entry in lower.split(',') {
            let entry = entry.trim();
            if let Some(arch) = Self::match_compatible_entry(entry) {
                return Some(arch);
            }
        }
        None
    }

    fn match_compatible_entry(entry: &str) -> Option<Self> {
        match entry {
            // SiFive cores
            "sifive,u74" => Some(CpuArch {
                vendor: Vendor::SiFive,
                model: String::from("SiFive U74"),
                micro_arch: MicroArch::SiFiveU74,
                code_name: "U74",
                marchid: 0,
                technology: Some(N28),
            }),
            "sifive,u76" => Some(CpuArch {
                vendor: Vendor::SiFive,
                model: String::from("SiFive U76"),
                micro_arch: MicroArch::SiFiveU76,
                code_name: "U76",
                marchid: 0,
                technology: None,
            }),
            "sifive,s7" | "sifive,s76" => Some(CpuArch {
                vendor: Vendor::SiFive,
                model: String::from("SiFive S76"),
                micro_arch: MicroArch::SiFiveS76,
                code_name: "S76",
                marchid: 0,
                technology: None,
            }),
            // T-Head cores
            "thead,c906" => Some(CpuArch {
                vendor: Vendor::THead,
                model: String::from("T-Head C906"),
                micro_arch: MicroArch::THeadC906,
                code_name: "C906",
                marchid: 0,
                technology: Some(N28),
            }),
            "thead,c910" => Some(CpuArch {
                vendor: Vendor::THead,
                model: String::from("T-Head C910"),
                micro_arch: MicroArch::THeadC910,
                code_name: "C910",
                marchid: 0,
                technology: Some(N16),
            }),
            "thead,c920" => Some(CpuArch {
                vendor: Vendor::THead,
                model: String::from("T-Head C920"),
                micro_arch: MicroArch::THeadC920,
                code_name: "C920",
                marchid: 0,
                technology: None,
            }),
            "thead,c930" => Some(CpuArch {
                vendor: Vendor::THead,
                model: String::from("T-Head C930"),
                micro_arch: MicroArch::THeadC930,
                code_name: "C930",
                marchid: 0,
                technology: None,
            }),
            // StarFive SoCs
            "starfive,jh7100" => Some(CpuArch {
                vendor: Vendor::SiFive,
                model: String::from("StarFive JH7100"),
                micro_arch: MicroArch::SiFiveU74,
                code_name: "JH7100",
                marchid: 0,
                technology: None,
            }),
            "starfive,jh7110" => Some(CpuArch {
                vendor: Vendor::SiFive,
                model: String::from("StarFive JH7110"),
                micro_arch: MicroArch::SiFiveU74,
                code_name: "JH7110",
                marchid: 0,
                technology: Some(N28),
            }),
            // Kendryte
            "canaan,k210" => Some(CpuArch {
                vendor: Vendor::Kendryte,
                model: String::from("Kendryte K210"),
                micro_arch: MicroArch::KendryteK210,
                code_name: "K210",
                marchid: 0,
                technology: Some(N28),
            }),
            _ => None,
        }
    }

    fn find_impl(
        marchid: usize,
        vendor: Vendor,
        parts: &[(
            usize,
            &'static str,
            MicroArch,
            &'static str,
            Option<&'static str>,
        )],
    ) -> Self {
        parts
            .iter()
            .find(|(p, _, _, _, _)| *p == marchid)
            .map(|&(_, model, ma, name, tech)| CpuArch {
                vendor,
                model: String::from(model),
                micro_arch: ma,
                code_name: name,
                marchid,
                technology: tech,
            })
            .unwrap_or_else(move || Self {
                vendor,
                ..Self::default()
            })
    }

    fn find_sifive(marchid: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x0000_0007,
                "SiFive U74",
                MicroArch::SiFiveU74,
                "U74",
                Some(N28),
            ),
            (0x0000_0002, "SiFive U76", MicroArch::SiFiveU76, "U76", None),
            (0x0000_0003, "SiFive S76", MicroArch::SiFiveS76, "S76", None),
            (
                0x0000_0010,
                "SiFive P470",
                MicroArch::SiFiveP470,
                "P470",
                None,
            ),
            (
                0x0000_0020,
                "SiFive P450",
                MicroArch::SiFiveP450,
                "P450",
                None,
            ),
            (
                0x0000_0040,
                "SiFive P550",
                MicroArch::SiFiveP550,
                "P550",
                Some(N7),
            ),
            (
                0x0000_0050,
                "SiFive P650",
                MicroArch::SiFiveP650,
                "P650",
                None,
            ),
            (
                0x0000_0080,
                "SiFive P670",
                MicroArch::SiFiveP670,
                "P670",
                None,
            ),
            (
                0x0000_0090,
                "SiFive P870",
                MicroArch::SiFiveP870,
                "P870",
                None,
            ),
        ];
        Self::find_impl(marchid, Vendor::SiFive, PARTS)
    }

    fn find_thead(marchid: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x0000_0000,
                "T-Head C906",
                MicroArch::THeadC906,
                "C906",
                Some(N28),
            ),
            (
                0x0000_0001,
                "T-Head C910",
                MicroArch::THeadC910,
                "C910",
                Some(N16),
            ),
            (
                0x0000_0002,
                "T-Head C920",
                MicroArch::THeadC920,
                "C920",
                None,
            ),
            (
                0x0000_0003,
                "T-Head C930",
                MicroArch::THeadC930,
                "C930",
                None,
            ),
        ];
        Self::find_impl(marchid, Vendor::THead, PARTS)
    }

    fn find_starfive(marchid: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[(
            0x0000_0007,
            "StarFive JH7110",
            MicroArch::SiFiveU74,
            "JH7110",
            None,
        )];
        Self::find_impl(marchid, Vendor::SiFive, PARTS)
    }

    fn find_kendryte(marchid: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[(
            0x0000_0000,
            "Kendryte K210",
            MicroArch::KendryteK210,
            "K210",
            Some(N28),
        )];
        Self::find_impl(marchid, Vendor::Kendryte, PARTS)
    }

    fn find_wch(marchid: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] =
            &[(0x0000_0000, "WCH CH32V", MicroArch::WCHCH32V, "CH32V", None)];
        Self::find_impl(marchid, Vendor::WCH, PARTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misa_64g() {
        // rv64gc: XLEN=64, I=1, M=1, A=1, F=1, D=1, C=1
        let mut ext: u64 = 0;
        ext |= 1 << ('I' as u64 - 'A' as u64);
        ext |= 1 << ('M' as u64 - 'A' as u64);
        ext |= 1 << ('A' as u64 - 'A' as u64);
        ext |= 1 << ('F' as u64 - 'A' as u64);
        ext |= 1 << ('D' as u64 - 'A' as u64);
        ext |= 1 << ('C' as u64 - 'A' as u64);
        let raw = (1u64 << 62) | ext;
        let misa = Misa::from_raw(raw);
        assert_eq!(misa.xlen, 64);
        assert!(misa.has_ext('I'));
        assert!(misa.has_ext('M'));
        assert!(misa.has_ext('A'));
        assert!(misa.has_ext('F'));
        assert!(misa.has_ext('D'));
        assert!(misa.has_ext('C'));
        assert!(!misa.has_ext('V'));
    }

    #[test]
    fn test_misa_to_isa_string() {
        let mut ext: u64 = 0;
        ext |= 1 << ('I' as u64 - 'A' as u64);
        ext |= 1 << ('M' as u64 - 'A' as u64);
        ext |= 1 << ('A' as u64 - 'A' as u64);
        ext |= 1 << ('F' as u64 - 'A' as u64);
        ext |= 1 << ('D' as u64 - 'A' as u64);
        ext |= 1 << ('C' as u64 - 'A' as u64);
        let raw = (1u64 << 62) | ext;
        let misa = Misa::from_raw(raw);
        assert_eq!(misa.to_isa_string(), "rv64imafdc");
    }

    #[test]
    fn test_misa_32i() {
        let ext = 1u64 << ('I' as u64 - 'A' as u64);
        let misa = Misa::from_raw(ext);
        assert_eq!(misa.xlen, 32);
        assert_eq!(misa.to_isa_string(), "rv32i");
    }

    #[test]
    fn test_micro_arch_to_string() {
        assert_eq!(String::from(MicroArch::SiFiveU74), "SiFive U74");
        assert_eq!(String::from(MicroArch::Unknown), UNK);
    }

    #[test]
    fn test_sifive_u74_find() {
        let cpu = CpuArch::find(VENDOR_SIFIVE, 0x0000_0001);
        assert_eq!(cpu.model.as_str(), "SiFive U74");
        assert_eq!(cpu.micro_arch, MicroArch::SiFiveU74);
    }

    #[test]
    fn test_unknown_vendor() {
        let cpu = CpuArch::find(0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
    }
}
