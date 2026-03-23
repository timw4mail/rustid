use crate::cpuid::brand::{CpuBrand, VENDOR_TRANSMETA};
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::vendor::TMicroArch;
use crate::cpuid::{CpuSignature, UNK};

/// Transmeta-specific microarchitecture detection.
#[derive(Debug, Default, PartialEq)]
pub struct Transmeta {}

impl TMicroArch for Transmeta {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand = CpuBrand::from(VENDOR_TRANSMETA);

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            (0, 5, 0, 4, _) => CpuArch::new(
                model,
                MicroArch::Crusoe,
                "Crusoe",
                brand.to_brand_name(),
                VENDOR_TRANSMETA,
                Some("130nm"),
            ),
            (0, 15, 0, 2 | 3, _) => CpuArch::new(
                model,
                MicroArch::Efficeon,
                "Efficeon",
                brand.to_brand_name(),
                VENDOR_TRANSMETA,
                Some("130nm"),
            ),

            (_, _, _, _, _) => CpuArch::new(
                model,
                MicroArch::Unknown,
                UNK,
                brand.to_brand_name(),
                VENDOR_TRANSMETA,
                None,
            ),
        }
    }
}
