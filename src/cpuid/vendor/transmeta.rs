use crate::cpuid::brand::CpuBrand;
use crate::cpuid::constants::*;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::vendor::TMicroArch;
use crate::cpuid::{CpuSignature, read_multi_leaf_str};
use alloc::string::String;

/// Transmeta-specific microarchitecture detection.
#[derive(Debug, Default, PartialEq)]
pub struct Transmeta {
    pub version_str: String,
}

impl Transmeta {
    pub fn detect() -> Self {
        Self {
            version_str: Self::version_str(),
        }
    }

    fn version_str() -> String {
        read_multi_leaf_str(TRANSMETA_LEAF_3, TRANSMETA_LEAF_6)
    }
}

impl TMicroArch for Transmeta {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand = CpuBrand::from(VENDOR_TRANSMETA);
        let brand = brand.to_brand_name();

        match (s.family, s.model, s.stepping) {
            (5, 4, _) => CpuArch::new(
                model,
                MicroArch::Crusoe,
                "Crusoe",
                brand,
                VENDOR_TRANSMETA,
                Some(N130),
            ),
            (15, 2 | 3, _) => CpuArch::new(
                model,
                MicroArch::Efficeon,
                "Efficeon",
                brand,
                VENDOR_TRANSMETA,
                Some(N130),
            ),
            _ => CpuArch::new(
                model,
                MicroArch::Unknown,
                UNK,
                brand,
                VENDOR_TRANSMETA,
                None,
            ),
        }
    }
}
