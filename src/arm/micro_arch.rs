use crate::arm::brand::{IMPL_APPLE, IMPL_ARM, Vendor};

pub const IMPLEMENTER_MASK: usize = 0xFF000000;
pub const VARIANT_MASK: usize = 0x00F00000;
pub const ARCHITECTURE_MASK: usize = 0x000F0000;
pub const PART_MASK: usize = 0x0000FFF0;
pub const REVISION_MASK: usize = 0x0000000F;

pub const IMPLEMENTER_OFFSET: usize = 24;
pub const VARIANT_OFFSET: usize = 20;
pub const ARCHITECTURE_OFFSET: usize = 16;
pub const PART_OFFSET: usize = 4;
pub const REVISION_OFFSET: usize = 0;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Midr {
    pub implementer: usize,
    pub variant: usize,
    pub architecture: usize,
    pub part: usize,
    pub revision: usize,
}

impl Midr {
    pub fn new(midr: usize) -> Midr {
        Midr {
            implementer: (midr & IMPLEMENTER_MASK) >> IMPLEMENTER_OFFSET,
            variant: (midr & VARIANT_MASK) >> VARIANT_OFFSET,
            architecture: (midr & ARCHITECTURE_MASK) >> ARCHITECTURE_OFFSET,
            part: (midr & PART_MASK) >> PART_OFFSET,
            revision: midr & REVISION_MASK,
        }
    }
}

pub const UNK: &str = "Unknown";
type Implementer = Vendor;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CoreMicroArch {
    #[default]
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    #[default]
    Unknown,

    AppleCyclone,
    AppleTyphoon,
    AppleSwift,
    AppleTwister,
    AppleHurricane,
    AppleZephyr,
    AppleMonsoon,
    AppleMistral,
    AppleTempest,
    AppleVortex,
    AppleLightning,
    AppleThunder,
    AppleFirestorm,
    AppleIcestorm,
    AppleAvalanche,
    AppleBlizzard,
    AppleEverest,
    AppleSawmill,
    AppleGibraltar,
    AppleHull,
    AppleIce,
    AppleDawn,
    AppleTahiti,
    AppleTonga,
    AppleJadeChop,
    AppleJade1C,
    AppleJade2C,

    ArmCortexA7,
    ArmCortexA8,
    ArmCortexA9,
    ArmCortexA12,
    ArmCortexA15,
    ArmCortexA17,
    ArmCortexA32,
    ArmCortexA35,
    ArmCortexA53,
    ArmCortexA55,
    ArmCortexA65,
    ArmCortexA72,
    ArmCortexA73,
    ArmCortexA75,
    ArmCortexA76,
    ArmCortexA77,
    ArmCortexA78,
    ArmCortexA510,
    ArmCortexA520,
    ArmCortexA710,
    ArmCortexA715,
    ArmCortexA720,
    ArmCortexA725,
    ArmCortexX1,
    ArmCortexX2,
    ArmCortexX3,
    ArmCortexX4,
    ArmNeoverseE1,
    ArmNeoverseN1,
    ArmNeoverseN2,
    ArmNeoverseV1,
    ArmNeoverseV2,
}

impl From<MicroArch> for String {
    fn from(ma: MicroArch) -> String {
        let s = match ma {
            MicroArch::Unknown => UNK,
            MicroArch::AppleCyclone => "Cyclone",
            MicroArch::AppleTyphoon => "Typhoon",
            MicroArch::AppleSwift => "Swift",
            MicroArch::AppleTwister => "Twister",
            MicroArch::AppleHurricane => "Hurricane",
            MicroArch::AppleZephyr => "Zephyr",
            MicroArch::AppleMonsoon => "Monsoon",
            MicroArch::AppleMistral => "Mistral",
            MicroArch::AppleTempest => "Tempest",
            MicroArch::AppleVortex => "Vortex",
            MicroArch::AppleLightning => "Lightning",
            MicroArch::AppleThunder => "Thunder",
            MicroArch::AppleFirestorm => "Firestorm",
            MicroArch::AppleIcestorm => "Icestorm",
            MicroArch::AppleAvalanche => "Avalanche",
            MicroArch::AppleBlizzard => "Blizzard",
            MicroArch::AppleEverest => "Everest",
            MicroArch::AppleSawmill => "Sawmill",
            MicroArch::AppleGibraltar => "Gibraltar",
            MicroArch::AppleHull => "Hull",
            MicroArch::AppleIce => "Ice",
            MicroArch::AppleDawn => "Dawn",
            MicroArch::AppleTahiti => "Tahiti",
            MicroArch::AppleTonga => "Tonga",
            MicroArch::AppleJadeChop => "Jade Chop",
            MicroArch::AppleJade1C => "Jade 1C",
            MicroArch::AppleJade2C => "Jade 2C",
            MicroArch::ArmCortexA7 => "Cortex-A7",
            MicroArch::ArmCortexA8 => "Cortex-A8",
            MicroArch::ArmCortexA9 => "Cortex-A9",
            MicroArch::ArmCortexA12 => "Cortex-A12",
            MicroArch::ArmCortexA15 => "Cortex-A15",
            MicroArch::ArmCortexA17 => "Cortex-A17",
            MicroArch::ArmCortexA32 => "Cortex-A32",
            MicroArch::ArmCortexA35 => "Cortex-A35",
            MicroArch::ArmCortexA53 => "Cortex-A53",
            MicroArch::ArmCortexA55 => "Cortex-A55",
            MicroArch::ArmCortexA65 => "Cortex-A65",
            MicroArch::ArmCortexA72 => "Cortex-A72",
            MicroArch::ArmCortexA73 => "Cortex-A73",
            MicroArch::ArmCortexA75 => "Cortex-A75",
            MicroArch::ArmCortexA76 => "Cortex-A76",
            MicroArch::ArmCortexA77 => "Cortex-A77",
            MicroArch::ArmCortexA78 => "Cortex-A78",
            MicroArch::ArmCortexA510 => "Cortex-A510",
            MicroArch::ArmCortexA520 => "Cortex-A520",
            MicroArch::ArmCortexA710 => "Cortex-A710",
            MicroArch::ArmCortexA715 => "Cortex-A715",
            MicroArch::ArmCortexA720 => "Cortex-A720",
            MicroArch::ArmCortexA725 => "Cortex-A725",
            MicroArch::ArmCortexX1 => "Cortex-X1",
            MicroArch::ArmCortexX2 => "Cortex-X2",
            MicroArch::ArmCortexX3 => "Cortex-X3",
            MicroArch::ArmCortexX4 => "Cortex-X4",
            MicroArch::ArmNeoverseE1 => "Neoverse E1",
            MicroArch::ArmNeoverseN1 => "Neoverse N1",
            MicroArch::ArmNeoverseN2 => "Neoverse N2",
            MicroArch::ArmNeoverseV1 => "Neoverse V1",
            MicroArch::ArmNeoverseV2 => "Neoverse V2",
        };

        String::from(s)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum CoreType {
    Super,
    #[default]
    Performance,
    Efficiency,
}

impl From<String> for CoreType {
    fn from(val: String) -> Self {
        match val.as_str() {
            "Super" => CoreType::Super,
            "Performance" => CoreType::Performance,
            "Efficiency" => CoreType::Efficiency,
            _ => CoreType::Performance,
        }
    }
}

impl From<CoreType> for String {
    fn from(val: CoreType) -> Self {
        let s = match val {
            CoreType::Super => "Super",
            CoreType::Performance => "Performance",
            CoreType::Efficiency => "Efficiency",
        };

        String::from(s)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Core {
    kind: CoreType,
    count: usize,
    micro_arch: CoreMicroArch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub implementer: Implementer,
    pub model: String,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub part_number: usize,
    pub technology: Option<&'static str>,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(
            Implementer::default(),
            UNK,
            MicroArch::default(),
            UNK,
            0,
            None,
        )
    }
}

impl CpuArch {
    pub fn new(
        implementer: Implementer,
        model: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        part_number: usize,
        technology: Option<&'static str>,
    ) -> Self {
        CpuArch {
            implementer,
            model: String::from(model),
            micro_arch,
            code_name,
            part_number,
            technology,
        }
    }

    pub fn find(implementer: usize, part: usize, _variant: usize) -> Self {
        match implementer {
            IMPL_ARM => Self::find_arm(part),
            IMPL_APPLE => Self::find_apple(part),
            _ => Self::default(),
        }
    }

    fn find_arm(part: usize) -> Self {
        match part {
            // Cortex-A7 series
            0xC07 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A7",
                MicroArch::ArmCortexA7,
                "Cortex-A7",
                0xC07,
                None,
            ),

            // Cortex-A8 series
            0xC08 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A8",
                MicroArch::ArmCortexA8,
                "Cortex-A8",
                0xC08,
                None,
            ),

            // Cortex-A9 series
            0xC09 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A9",
                MicroArch::ArmCortexA9,
                "Cortex-A9",
                0xC09,
                None,
            ),

            // Cortex-A12 series
            0xC0A => Self::new(
                Implementer::Arm,
                "ARM Cortex-A12",
                MicroArch::ArmCortexA12,
                "Cortex-A12",
                0xC0A,
                None,
            ),

            // Cortex-A15 series
            0xC0F => Self::new(
                Implementer::Arm,
                "ARM Cortex-A15",
                MicroArch::ArmCortexA15,
                "Cortex-A15",
                0xC0F,
                None,
            ),

            // Cortex-A17 series
            0xC0E => Self::new(
                Implementer::Arm,
                "ARM Cortex-A17",
                MicroArch::ArmCortexA17,
                "Cortex-A17",
                0xC0E,
                None,
            ),

            // Cortex-A32 series
            0xC20 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A32",
                MicroArch::ArmCortexA32,
                "Cortex-A32",
                0xC20,
                None,
            ),

            // Cortex-A35 series
            0xC23 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A35",
                MicroArch::ArmCortexA35,
                "Cortex-A35",
                0xC23,
                None,
            ),

            // Cortex-A53 series
            0xD03 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A53",
                MicroArch::ArmCortexA53,
                "Cortex-A53",
                0xD03,
                None,
            ),

            // Cortex-A55 series
            0xD05 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A55",
                MicroArch::ArmCortexA55,
                "Cortex-A55",
                0xD05,
                None,
            ),

            // Cortex-A65 series
            0xD08 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A65",
                MicroArch::ArmCortexA65,
                "Cortex-A65",
                0xD08,
                None,
            ),

            // Cortex-A72 series
            0xD0B => Self::new(
                Implementer::Arm,
                "ARM Cortex-A72",
                MicroArch::ArmCortexA72,
                "Cortex-A72",
                0xD0B,
                None,
            ),

            // Cortex-A73 series
            0xD0C => Self::new(
                Implementer::Arm,
                "ARM Cortex-A73",
                MicroArch::ArmCortexA73,
                "Cortex-A73",
                0xD0C,
                None,
            ),

            // Cortex-A75 series
            0xD0D => Self::new(
                Implementer::Arm,
                "ARM Cortex-A75",
                MicroArch::ArmCortexA75,
                "Cortex-A75",
                0xD0D,
                None,
            ),

            // Cortex-A76 series
            0xD0E => Self::new(
                Implementer::Arm,
                "ARM Cortex-A76",
                MicroArch::ArmCortexA76,
                "Cortex-A76",
                0xD0E,
                None,
            ),

            // Cortex-A77 series
            0xD10 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A77",
                MicroArch::ArmCortexA77,
                "Cortex-A77",
                0xD10,
                None,
            ),

            // Cortex-A78 series
            0xD11 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A78",
                MicroArch::ArmCortexA78,
                "Cortex-A78",
                0xD11,
                None,
            ),

            // Cortex-X1 series
            0xD13 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X1",
                MicroArch::ArmCortexX1,
                "Cortex-X1",
                0xD13,
                None,
            ),

            // Cortex-X2 series
            0xD20 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X2",
                MicroArch::ArmCortexX2,
                "Cortex-X2",
                0xD20,
                None,
            ),

            // Cortex-X3 series
            0xD21 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X3",
                MicroArch::ArmCortexX3,
                "Cortex-X3",
                0xD21,
                None,
            ),

            // Cortex-X4 series
            0xD22 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X4",
                MicroArch::ArmCortexX4,
                "Cortex-X4",
                0xD22,
                None,
            ),

            // Neoverse E1
            0xD40 => Self::new(
                Implementer::Arm,
                "ARM Neoverse E1",
                MicroArch::ArmNeoverseE1,
                "Neoverse E1",
                0xD40,
                None,
            ),

            // Neoverse N1
            0xD41 => Self::new(
                Implementer::Arm,
                "ARM Neoverse N1",
                MicroArch::ArmNeoverseN1,
                "Neoverse N1",
                0xD41,
                None,
            ),

            // Neoverse N2
            0xD42 => Self::new(
                Implementer::Arm,
                "ARM Neoverse N2",
                MicroArch::ArmNeoverseN2,
                "Neoverse N2",
                0xD42,
                None,
            ),

            // Neoverse V1
            0xD60 => Self::new(
                Implementer::Arm,
                "ARM Neoverse V1",
                MicroArch::ArmNeoverseV1,
                "Neoverse V1",
                0xD60,
                None,
            ),

            // Neoverse V2
            0xD61 => Self::new(
                Implementer::Arm,
                "ARM Neoverse V2",
                MicroArch::ArmNeoverseV2,
                "Neoverse V2",
                0xD61,
                None,
            ),

            _ => Self::default(),
        }
    }

    fn find_apple(part: usize) -> Self {
        match part {
            // A-series chips
            0x101 => Self::new(
                Implementer::Apple,
                "Apple A18 Pro",
                MicroArch::AppleTahiti,
                "Tahiti",
                0x101,
                Some("3nm"),
            ),

            // M1 series - Icestorm (E) / Firestorm (P)
            0x008 => Self::new(
                Implementer::Apple,
                "Apple M1",
                MicroArch::AppleTonga,
                "Tonga",
                0x008,
                Some("5nm"),
            ),
            0x009 => Self::new(
                Implementer::Apple,
                "Apple M1 Pro",
                MicroArch::AppleJadeChop,
                "Jade Chop",
                0x009,
                Some("5nm"),
            ),
            0x00A => Self::new(
                Implementer::Apple,
                "Apple M1 Pro",
                MicroArch::AppleJadeChop,
                "Jade Chop",
                0x00A,
                Some("5nm"),
            ),
            0x00B => Self::new(
                Implementer::Apple,
                "Apple M1 Max",
                MicroArch::AppleJade1C,
                "Jade 1C",
                0x00B,
                Some("5nm"),
            ),

            // M2 series - Blizzard (E) / Avalanche (P)
            0x00C => Self::new(
                Implementer::Apple,
                "Apple M2",
                MicroArch::AppleAvalanche,
                "Staten",
                0x00C,
                Some("5nm"),
            ),
            0x00E => Self::new(
                Implementer::Apple,
                "Apple M2 Pro",
                MicroArch::AppleAvalanche,
                "Rhodes Chop",
                0x00E,
                Some("5nm"),
            ),
            0x010 => Self::new(
                Implementer::Apple,
                "Apple M2 Max",
                MicroArch::AppleAvalanche,
                "Rhodes 1C",
                0x010,
                Some("5nm"),
            ),

            // M3 series - Cream (E) / Everest (P)
            0x011 => Self::new(
                Implementer::Apple,
                "Apple M3",
                MicroArch::AppleHull,
                "Ibiza",
                0x011,
                Some("3nm"),
            ),
            0x012 => Self::new(
                Implementer::Apple,
                "Apple M3 Pro",
                MicroArch::AppleHull,
                "Lobos",
                0x012,
                Some("3nm"),
            ),
            0x013 => Self::new(
                Implementer::Apple,
                "Apple M3 Max",
                MicroArch::AppleHull,
                "Palma",
                0x013,
                Some("3nm"),
            ),

            // M4 series - S4 (E) / S3 (P) - Names TBD
            0x014 => Self::new(
                Implementer::Apple,
                "Apple M4",
                MicroArch::AppleDawn,
                "Donan",
                0x014,
                Some("3nm"),
            ),
            0x015 => Self::new(
                Implementer::Apple,
                "Apple M4 Pro",
                MicroArch::AppleDawn,
                "Brava Chop",
                0x015,
                Some("3nm"),
            ),
            0x016 => Self::new(
                Implementer::Apple,
                "Apple M4 Max",
                MicroArch::AppleDawn,
                "Brava",
                0x016,
                Some("3nm"),
            ),

            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midr_parsing() {
        let midr = Midr::new(0x61FF0F02);
        assert_eq!(midr.implementer, 0x61);
        assert_eq!(midr.variant, 0xF);
        assert_eq!(midr.architecture, 0xF);
        assert_eq!(midr.part, 0x0F0);
        assert_eq!(midr.revision, 0x2);
    }

    #[test]
    fn test_apple_m1_find() {
        let cpu = CpuArch::find(0x61, 0x009, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleJadeChop);
    }

    #[test]
    fn test_apple_m1_pro_find() {
        let cpu = CpuArch::find(0x61, 0x00A, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleJadeChop);
    }

    #[test]
    fn test_apple_m2_find() {
        let cpu = CpuArch::find(0x61, 0x00C, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M2");
        assert_eq!(cpu.micro_arch, MicroArch::AppleAvalanche);
    }

    #[test]
    fn test_apple_m3_find() {
        let cpu = CpuArch::find(0x61, 0x011, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M3");
        assert_eq!(cpu.micro_arch, MicroArch::AppleHull);
    }

    #[test]
    fn test_apple_m4_find() {
        let cpu = CpuArch::find(0x61, 0x014, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M4");
        assert_eq!(cpu.micro_arch, MicroArch::AppleDawn);
    }

    #[test]
    fn test_apple_a18_pro_find() {
        let cpu = CpuArch::find(0x61, 0x101, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple A18 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleTahiti);
    }

    #[test]
    fn test_apple_cpu_unknown() {
        let cpu = CpuArch::find(0x61, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_non_apple_implementer() {
        let cpu = CpuArch::find(0x41, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
    }

    #[test]
    fn test_arm_cortex_a76_find() {
        let cpu = CpuArch::find(0x41, 0xD0E, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A76");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA76);
    }

    #[test]
    fn test_arm_cortex_a55_find() {
        let cpu = CpuArch::find(0x41, 0xD05, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A55");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA55);
    }

    #[test]
    fn test_arm_cortex_a53_find() {
        let cpu = CpuArch::find(0x41, 0xD03, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A53");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA53);
    }

    #[test]
    fn test_arm_cortex_x1_find() {
        let cpu = CpuArch::find(0x41, 0xD13, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-X1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexX1);
    }

    #[test]
    fn test_arm_neoverse_n1_find() {
        let cpu = CpuArch::find(0x41, 0xD41, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Neoverse N1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmNeoverseN1);
    }

    #[test]
    fn test_arm_unknown_part() {
        let cpu = CpuArch::find(0x41, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_micro_arch_to_string() {
        assert_eq!(String::from(MicroArch::AppleFirestorm), "Firestorm");
        assert_eq!(String::from(MicroArch::AppleAvalanche), "Avalanche");
        assert_eq!(String::from(MicroArch::ArmCortexA76), "Cortex-A76");
    }
}
