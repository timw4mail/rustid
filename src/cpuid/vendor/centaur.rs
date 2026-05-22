use crate::cpuid::brand::CpuBrand;
use crate::cpuid::constants::*;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::vendor::TMicroArch;
use crate::cpuid::{CpuSignature, is_valid_leaf, is_zhaoxin, x86_cpuid};

#[cfg(not(target_os = "none"))]
use alloc::collections::BTreeMap;

pub struct Centaur;

fn centaur_cpu_brand() -> CpuBrand {
    if is_zhaoxin() {
        CpuBrand::Zhaoxin
    } else {
        let s = CpuSignature::detect();
        match s.family {
            5 => CpuBrand::IDT,
            6 => CpuBrand::Via,
            _ => CpuBrand::Zhaoxin,
        }
    }
}

impl TMicroArch for Centaur {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand = centaur_cpu_brand();

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
            (0, 5, 0, 4, _) => brand_arch(MicroArch::Winchip, "C6", Some(N350)),
            (0, 5, 0, 8, 5) => brand_arch(MicroArch::Winchip2, "C2", Some(N350)),
            (0, 5, 0, 8, 7) => brand_arch(MicroArch::Winchip2A, "W2A", Some(N250)),
            (0, 5, 0, 8, 10) => brand_arch(MicroArch::Winchip2B, "W2B", Some(N250)),
            (0, 5, 0, 9, _) => brand_arch(MicroArch::Winchip3, "C3", Some(N250)),

            // VIA
            (0, 6, 0, 6, _) => brand_arch(MicroArch::Samuel, "C5A", Some(N180)),
            (0, 6, 0, 7, 0..=7) => brand_arch(MicroArch::Samuel2, "C5B", Some(N150)),
            (0, 6, 0, 7, 8..=15) => brand_arch(MicroArch::Ezra, "C5C", Some(N130)),
            (0, 6, 0, 8, 0..=7) => brand_arch(MicroArch::EzraT, "C5N", Some(N130)), // per sandpile.org
            (0, 6, 0, 8, 8..=15) => brand_arch(MicroArch::Nehemiah, "C5X", Some(N130)), // per sandpile.org
            (0, 6, 0, 9, 0..=7) => brand_arch(MicroArch::Nehemiah, "C5XL", Some(N130)),
            (0, 6, 0, 9, 8..=15) => brand_arch(MicroArch::NehemiahP, "C5P", Some(N130)),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::Esther, "C5J Model A", Some(N90)),
            (0, 6, 0, 13, _) => brand_arch(MicroArch::Esther, "C5J Model D", Some(N90)), // OLPC XO 1.5

            // From instlatx64
            (0, 6, 0, 15, 0..8) => brand_arch(MicroArch::Isaiah, "CNA", Some(N65)),
            (0, 6, 0, 15, 8) => brand_arch(MicroArch::Isaiah, "CNB A1", None),
            (0, 6, 0, 15, 10) => brand_arch(MicroArch::Isaiah, "CNB A2", None),
            (0, 6, 0, 15, 12) => brand_arch(MicroArch::Isaiah, "CNC/CNQ A1", None),
            (0, 6, 0, 15, 13) => brand_arch(MicroArch::Isaiah, "CNC/CNQ A2", Some(N40)), // My hardware
            (0, 6, 0, 15, 14) => brand_arch(MicroArch::Isaiah, "CNR", None),
            (0, 6, 1, 15, _) => brand_arch(MicroArch::Isaiah, "CN", Some(N65)),

            // Zhaoxin
            (0, 6, 1, 9, _) => brand_arch(MicroArch::ZhangJiang, "ZhangJiang", Some(N28)),
            (0, 7, 1, 11, 0) => brand_arch(MicroArch::Wudaokou, "WuDaoKou", Some(N28)),
            (0, 7, 3, 11, _) => brand_arch(MicroArch::Lujiazui, "LuJiaZui", Some(N16)),

            // Anything else
            _ => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }
}

// ----------------------------------------------------------------------------
// ! Centaur Feature Detection (CPUID leaf 0xC0000001, EDX register)
//
// These are VIA PadLock security features and Centaur-specific extensions.
// For ACE and ACE2, both the present bit AND the enable bit must be set.
// ----------------------------------------------------------------------------

use crate::cpuid::{CENTAUR_LEAF_1, EXT_LEAF_1, Reg, has_feature};

fn has_centaur_feature(bit: u32) -> bool {
    if !is_valid_leaf(CENTAUR_LEAF_1) {
        return false;
    }

    // If CENTAUR_LEAF_1 == EXT_LEAF_1, these leaves do not contain Centaur-specific data
    // See: <https://www.ardent-tool.com/CPU/docs/IDT_Centaur/WinChip2/wc_2_datasheet_a2.pdf>
    let ext_leaf = x86_cpuid(EXT_LEAF_1);
    let centaur_leaf = x86_cpuid(CENTAUR_LEAF_1);
    if ext_leaf == centaur_leaf {
        return false;
    }

    // If we know that the Centaur leaf is different, we can check for centaur-specific values
    has_feature(CENTAUR_LEAF_1, Reg::Edx, bit)
}

/// Alternate Instruction Set Support
///
/// This flag is different for Zhaoxin
#[must_use]
pub fn has_ais() -> bool {
    centaur_cpu_brand() != CpuBrand::Zhaoxin && has_centaur_feature(0)
}
pub fn ais_enabled() -> bool {
    has_ais() && has_centaur_feature(1)
}

/// Chinese Cipher Security SM2
pub fn has_sm2() -> bool {
    centaur_cpu_brand() == CpuBrand::Zhaoxin && has_centaur_feature(0)
}
pub fn sm2_enabled() -> bool {
    has_sm2() && has_centaur_feature(0)
}

/// Random Number Generator (`xstore` instruction)
#[must_use]
pub fn has_rng() -> bool {
    has_centaur_feature(2)
}
pub fn rng_enabled() -> bool {
    has_rng() && has_centaur_feature(3)
}

/// Chinese Cipher Security SM3 & SM4
pub fn has_sm3_4() -> bool {
    centaur_cpu_brand() == CpuBrand::Zhaoxin && has_centaur_feature(4)
}
pub fn sm3_4_enabled() -> bool {
    has_sm3_4() && has_centaur_feature(5)
}

/// Advanced Cryptography Engine (AES encryption/decryption)
#[must_use]
pub fn has_ace() -> bool {
    has_centaur_feature(6)
}
#[must_use]
pub fn ace_enabled() -> bool {
    has_ace() && has_centaur_feature(7)
}

/// Advanced Cryptography Engine 2 (AES 192/256-bit keys)
///
/// Requires both presence (bit 3) and enable (bit 5).
#[must_use]
pub fn has_ace2() -> bool {
    has_centaur_feature(8)
}
pub fn ace2_enabled() -> bool {
    has_ace2() && has_centaur_feature(9)
}

/// `PadLock` Hash Engine (SHA-1/SHA-256)
#[must_use]
pub fn has_phe() -> bool {
    has_centaur_feature(10)
}
pub fn phe_enabled() -> bool {
    has_phe() && has_centaur_feature(11)
}

/// `PadLock` Montgomery Multiplier (big-integer modular exponentiation)
#[must_use]
pub fn has_pmm() -> bool {
    has_centaur_feature(12)
}
pub fn pmm_enabled() -> bool {
    has_pmm() && has_centaur_feature(13)
}

/// Enhanced RNG (`xstore2` instruction)
#[must_use]
pub fn has_rng2() -> bool {
    has_centaur_feature(22)
}
pub fn rng2_enabled() -> bool {
    has_rng2() && has_centaur_feature(23)
}

/// `PadLock` Hash Engine 2 (SHA-512)
#[must_use]
pub fn has_phe2() -> bool {
    has_centaur_feature(25)
}
pub fn phe2_enabled() -> bool {
    has_phe2() && has_centaur_feature(26)
}

/// `Padlock` Montgomery Multiplier 2/ RSA
#[must_use]
pub fn has_rsa() -> bool {
    has_centaur_feature(27)
}
pub fn rsa_enabled() -> bool {
    has_rsa() && has_centaur_feature(28)
}

pub type CentaurFeatureMap<'a> = &'a [(
    &'static str,
    crate::cpuid::features::FeatureFn,
    crate::cpuid::features::FeatureFn,
)];

#[cfg(not(target_os = "none"))]
impl Centaur {
    pub fn get_feature_list() -> BTreeMap<&'static str, bool> {
        const CENTAUR_FEATURES: CentaurFeatureMap = &[
            ("AIS", has_ais, ais_enabled),
            ("CCS_SM2", has_sm2, sm2_enabled),
            ("CCS_SM3_SM4", has_sm3_4, sm3_4_enabled),
            ("RNG", has_rng, rng_enabled),
            ("RNG2", has_rng2, rng2_enabled),
            ("ACE", has_ace, ace_enabled),
            ("ACE2", has_ace2, ace2_enabled),
            ("PHE", has_phe, phe_enabled),
            ("PHE2", has_phe2, phe2_enabled),
            ("PMM", has_pmm, pmm_enabled),
            ("RSA", has_rsa, rsa_enabled),
        ];

        let mut map: BTreeMap<&'static str, bool> = BTreeMap::new();

        for (name, exists, enabled) in CENTAUR_FEATURES {
            if exists() {
                let active = enabled();
                map.insert(*name, active);
            }
        }

        map
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
