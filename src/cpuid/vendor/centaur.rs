use crate::common::UNK;
use crate::cpuid::brand::{CpuBrand, VENDOR_CENTAUR};
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::vendor::TMicroArch;
use crate::cpuid::{CpuSignature, is_zhaoxin};

pub struct Centaur;

impl TMicroArch for Centaur {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand = if is_zhaoxin() {
            CpuBrand::Zhaoxin
        } else {
            match s.family {
                #[cfg(target_arch = "x86")]
                5 => CpuBrand::IDT,
                6 => CpuBrand::Via,
                _ => CpuBrand::Zhaoxin,
            }
        };

        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> CpuArch {
            CpuArch::new(
                model,
                ma,
                code_name,
                brand.to_brand_name(),
                VENDOR_CENTAUR,
                tech,
            )
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // IDT
            (0, 5, 0, 4, _) => brand_arch(MicroArch::Winchip, "C6", Some("350nm")),
            (0, 5, 0, 8, 5) => brand_arch(MicroArch::Winchip2, "C2", Some("350nm")),
            (0, 5, 0, 8, 7) => brand_arch(MicroArch::Winchip2A, "W2A", Some("250nm")),
            (0, 5, 0, 8, 10) => brand_arch(MicroArch::Winchip2B, "W2B", Some("250nm")),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::Winchip3, "C3", Some("250nm")),

            // VIA
            (0, 6, 0, 6, _) => brand_arch(MicroArch::Samuel, "C5A", Some("180nm")),
            (0, 6, 0, 7, 0..=7) => brand_arch(MicroArch::Samuel2, "C5B", Some("150nm")),
            (0, 6, 0, 7, 8..=15) => brand_arch(MicroArch::Ezra, "C5C", Some("130nm")),
            (0, 6, 0, 8, 0..=7) => brand_arch(MicroArch::EzraT, "C5N", Some("130nm")), // per sandpile.org
            (0, 6, 0, 8, 8..=15) => brand_arch(MicroArch::Nehemiah, "C5X", Some("130nm")), // per sandpile.org
            (0, 6, 0, 9, 0..=7) => brand_arch(MicroArch::Nehemiah, "C5XL", Some("130nm")),
            (0, 6, 0, 9, 8..=15) => brand_arch(MicroArch::NehemiahP, "C5P", Some("130nm")),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::Esther, "C5J", Some("90nm")),
            (0, 5, 0, 13, _) => brand_arch(MicroArch::Esther, "C5J Model D", Some("90nm")), // OLPC XO 1.5

            // From instlatx64
            (0, 6, 0, 15, 1 | 2) => brand_arch(MicroArch::Isaiah, "CN", None),
            (0, 6, 0, 15, 0..8) => brand_arch(MicroArch::Isaiah, "CNA", Some("65nm")),
            (0, 6, 0, 15, 8) => brand_arch(MicroArch::Isaiah, "CNB A1", None),
            (0, 6, 0, 15, 10) => brand_arch(MicroArch::Isaiah, "CNB A2", None),
            (0, 6, 0, 15, 12) => brand_arch(MicroArch::Isaiah, "CNC/CNQ", None),
            (0, 6, 0, 15, 13) => brand_arch(MicroArch::Isaiah, "CNQ A2", Some("40nm")), // My hardware
            (0, 6, 0, 15, 14) => brand_arch(MicroArch::Isaiah, "CNR", None),
            (0, 6, 1, 15, _) => brand_arch(MicroArch::Isaiah, "CN", Some("65nm")),

            // Zhaoxin
            (0, 6, 1, 9, _) => brand_arch(MicroArch::ZhangJiang, "ZhangJiang", Some("28nm")),
            (0, 7, 1, 11, 0) => brand_arch(MicroArch::Wudaokou, "WuDaoKou", Some("28nm")),
            (0, 7, 3, 11, _) => brand_arch(MicroArch::Lujiazui, "LuJiaZui", Some("16nm")),

            // Anything else
            (_, _, _, _, _) => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpuid::micro_arch::tests::dummy_signature;

    #[test]
    fn test_cpu_arch_find_centaur() {
        let model = "Centaur Processor";

        // IDT Winchip
        let sig_winchip = dummy_signature(5, 4, 0, 0, 0);
        let arch = Centaur::micro_arch(model, sig_winchip);
        assert_eq!(arch.micro_arch, MicroArch::Winchip);
        assert_eq!(arch.code_name, "C6");

        // VIA Ezra
        let sig_ezra = dummy_signature(6, 7, 0, 0, 8);
        let arch = Centaur::micro_arch(model, sig_ezra);
        assert_eq!(arch.micro_arch, MicroArch::Ezra);
        assert_eq!(arch.code_name, "C5C");

        // Zhaoxin Lujiazui
        let sig_lujiazui = dummy_signature(7, 11, 0, 3, 0);
        let arch = Centaur::micro_arch(model, sig_lujiazui);
        assert_eq!(arch.micro_arch, MicroArch::Lujiazui);
        assert_eq!(arch.code_name, "LuJiaZui");

        // Unknown Centaur
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = Centaur::micro_arch(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK); // Centaur unknown code_name is empty
    }
}
