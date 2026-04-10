use crate::common::constants::*;
use crate::cpuid::brand::VENDOR_AMD;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::vendor::TMicroArch;
use crate::cpuid::{CpuSignature, logical_cores};

/// AMD-specific microarchitecture detection.
pub struct Amd;

impl TMicroArch for Amd {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> CpuArch {
            CpuArch::new(model, ma, code_name, "AMD", VENDOR_AMD, tech)
        };

        match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // 486
            (0, 4, 0, 0, _) => brand_arch(MicroArch::Am486, "Am486DX", None),
            (0, 4, 0, 1, _) => brand_arch(MicroArch::Am486, "Am486DX-40", None),
            (0, 4, 0, 2, _) => brand_arch(MicroArch::Am486, "Am486SX", None),
            (0, 4, 0, 3, _) => brand_arch(MicroArch::Am486, "Am486DX2", None),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::Am486, "Am486X2WB", None),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::Am486, "Am486DX4", None),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::Am486, "Am486DX4WB", None),
            (0, 4, 0, 14, _) => brand_arch(MicroArch::Am5x86, "Am5x86", None),
            (0, 4, 0, 15, _) => brand_arch(MicroArch::Am5x86, "Am5x86WB", None),

            // K5
            (0, 5, 0, 0, _) => brand_arch(MicroArch::SSA5, "SSA/5", Some(N350)),
            (0, 5, 0, 1..=3, _) => brand_arch(MicroArch::K5, "5k86", Some(N350)),

            // K6 (K6, K6-2, K6-III, K6-2+/K6-III+)
            // NexGenerationAMD
            (0, 5, 0, 6, _) => brand_arch(MicroArch::K6, "Model 6", Some(N300)), // Per Cpu-world
            (0, 5, 0, 7, _) => brand_arch(MicroArch::K6, "Little Foot", Some(N250)),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::K6, "Chompers/CXT", Some(N250)), // K6-2
            (0, 5, 0, 9, _) => brand_arch(MicroArch::K6, "Sharptooth", Some(N250)),   // K6-III
            (0, 5, 0, 10, _) => brand_arch(MicroArch::K7, "Thoroughbred (Geode NX)", Some(N130)), // Per instlatx64
            (0, 5, 0, 12 | 13, _) => {
                // K6-2+/K6-III+
                // per sandpile.org
                brand_arch(MicroArch::K6, "Sharptooth", Some(N180))
            }

            // K7 (Athlon/Duron/Sempron/Geode NX)
            (0, 6, 0, 1, _) => brand_arch(MicroArch::K7, "Argon", Some(N250)),
            (0, 6, 0, 2, _) => brand_arch(MicroArch::K7, "Pluto", Some(N180)),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::K7, "Spitfire", Some(N180)), // Duron, per sandpile.org
            (0, 6, 0, 4, _) => brand_arch(MicroArch::K7, "Thunderbird", Some(N180)),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::K7, "Palomino", Some(N180)),
            (0, 6, 0, 7, _) => brand_arch(MicroArch::K7, "Morgan", Some(N180)), // Duron, per sandpile.org

            // Geode NX, Per AMD's documentation (https://www.amd.com/content/dam/amd/en/documents/archived-tech-docs/datasheets/31177H_nx_databook.pdf)
            (0, 6, 0, 8, 1) => brand_arch(MicroArch::K7, "Thoroughbred", Some(N130)),

            (0, 6, 0, 8, _) => brand_arch(MicroArch::K7, "Thoroughbred", Some(N130)),
            (0, 6, 0, 10, _) => brand_arch(MicroArch::K7, "Thorton/Barton", Some(N130)),

            // Family 08h (K8)
            // IT'S HAMMER TIME
            (0, 15, 0, 13, 0) => brand_arch(MicroArch::K8, "NewCastle", Some(N130)),
            (0, 15, 2, 3, 2) => brand_arch(MicroArch::K8, "Toledo", Some(N90)),
            (0, 15, 2, 15, 2) => brand_arch(MicroArch::K8, "Venice", Some(N90)),
            (0, 15, 3, 7, 2) => brand_arch(MicroArch::K8, "San Diego", Some(N90)),
            (0, 15, 4, 15, 2) => brand_arch(MicroArch::K8, "Manilla", Some(N90)),
            (0, 15, 4, 11, 2) => brand_arch(MicroArch::K8, "Windsor", Some(N90)),
            (0, 15, 5, 15, 2) => brand_arch(MicroArch::K8, "Orleans", Some(N90)),
            (0, 15, 6, 11, 2) => brand_arch(MicroArch::K8, "Brisbane", Some(N65)),
            (0, 15, 7, 15, 2) => brand_arch(MicroArch::K8, "Sparta", Some(N65)),

            // Family 10h (K10)
            // Phenom X2 (Athlon X2), Phenom X3, Phenom X4
            (1, 15, 0, 2, 3) => match logical_cores() {
                2 => brand_arch(MicroArch::K10, "Kuma", Some(N65)),
                3 => brand_arch(MicroArch::K10, "Toliman", Some(N65)),
                _ => brand_arch(MicroArch::K10, "Agena", Some(N65)),
            },
            // Phenom II X2, Phenom II X4
            (1, 15, 0, 4, _) => match logical_cores() {
                2 => brand_arch(MicroArch::K10, "Callisto", Some(N45)),
                _ => brand_arch(MicroArch::K10, "Deneb", Some(N45)),
            },
            (1, 15, 0, 5, 3) => brand_arch(MicroArch::K10, "Propus", Some(N45)),
            (1, 15, 0, 6, 2) => brand_arch(MicroArch::K10, "Sargas", Some(N45)),
            (1, 15, 0, 6, 3) => brand_arch(MicroArch::K10, "Regor", Some(N45)),
            (1, 15, 0, 10, 0) => brand_arch(MicroArch::K10, "Thuban", Some(N45)),

            // Family 14h
            (5, 15, 0, 2, 0) => brand_arch(MicroArch::Bobcat, "Zacate", Some(N40)),

            // Family 15h (Bulldozer/Piledriver/Steamroller/Excavator)
            (6, 15, 0, 0 | 1, _) => brand_arch(MicroArch::Bulldozer, "Zambezi", Some(N32)),
            (6, 15, 0 | 1, 2, _) => brand_arch(MicroArch::Piledriver, "Vishera", Some(N32)),
            (6, 15, 1, 0, 1) => brand_arch(MicroArch::Piledriver, "Trinity", Some(N32)),
            (6, 15, 3, 0 | 8, _) => brand_arch(MicroArch::Steamroller, "Godavari", Some(N28)),
            (6, 15, 6 | 7, 0 | 5, _) => {
                brand_arch(MicroArch::Excavator, "Bristol Ridge/Carrizo", Some(N28))
            }

            // Family 16h
            // HELLO KITTY! ^-^
            (7, 15, 0, 0, 1) => brand_arch(MicroArch::Jaguar, "Kabini", Some(N28)),

            // Zen

            // Family 17h
            (8, 15, 0, 1, 1) => brand_arch(MicroArch::Zen, "Summit Ridge", Some(N14)),
            (8, 15, 1, 1, 0) => brand_arch(MicroArch::Zen, "Raven Ridge", Some(N14)),
            (8, 15, 7, 1, 0) => brand_arch(MicroArch::Zen2, "Matisse", Some(N7)),

            // Family 19h
            (10, 15, 2, 1, _) => brand_arch(MicroArch::Zen3, "Vermeer", Some(N7)),
            (10, 15, 5, 0, 0) => brand_arch(MicroArch::Zen3, "Cezanne", Some(N7)),
            (10, 15, 6, 1, 2) => brand_arch(MicroArch::Zen4, "Raphael", Some(N5)),
            (10, 15, 7, 4, 1) => brand_arch(MicroArch::Zen4, "Phoenix", Some(N4)),
            _ => brand_arch(MicroArch::Unknown, UNK, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpuid::UNK;
    use crate::cpuid::micro_arch::tests::dummy_signature;

    #[test]
    fn test_cpu_arch_find_amd() {
        let model = "AMD Processor";

        // Am486
        let sig_am486 = dummy_signature(4, 3, 0, 0, 0);
        let arch = Amd::micro_arch(model, sig_am486);
        assert_eq!(arch.micro_arch, MicroArch::Am486);
        assert_eq!(arch.code_name, "Am486DX2");

        // K5
        let sig_k5 = dummy_signature(5, 1, 0, 0, 0);
        let arch = Amd::micro_arch(model, sig_k5);
        assert_eq!(arch.micro_arch, MicroArch::K5);
        assert_eq!(arch.code_name, "5k86");

        // Zen4
        let sig_zen4 = dummy_signature(15, 1, 10, 6, 2);
        let arch = Amd::micro_arch(model, sig_zen4);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Raphael");

        // Unknown AMD
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = Amd::micro_arch(model, sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }
}
