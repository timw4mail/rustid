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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub marketing_name: String,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub part_number: usize,
    pub technology: Option<&'static str>,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(UNK, MicroArch::Unknown, UNK, 0, None)
    }
}

impl CpuArch {
    pub fn new(
        marketing_name: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        part_number: usize,
        technology: Option<&'static str>,
    ) -> Self {
        CpuArch {
            marketing_name: String::from(marketing_name),
            micro_arch,
            code_name,
            part_number,
            technology,
        }
    }

    pub fn find(implementer: usize, part: usize, _variant: usize) -> Self {
        match implementer {
            0x61 | 0xB3 | 0xB9 | 0x63 => Self::find_apple(part),
            _ => Self::default(),
        }
    }

    fn find_apple(part: usize) -> Self {
        match part {
            0x001 => Self::new(
                "Apple A7",
                MicroArch::AppleCyclone,
                "Cyclone (E) / Typhoon (P)",
                0x001,
                Some("28nm"),
            ),
            0x002 => Self::new(
                "Apple A8",
                MicroArch::AppleTyphoon,
                "Typhoon (E) / Swift (P)",
                0x002,
                Some("20nm"),
            ),
            0x003 => Self::new(
                "Apple A9",
                MicroArch::AppleTwister,
                "Twister (E) / Hurricane (P)",
                0x003,
                Some("16nm"),
            ),
            0x004 => Self::new(
                "Apple A10",
                MicroArch::AppleZephyr,
                "Hurricane (E) / Zephyr (P)",
                0x004,
                Some("16nm"),
            ),
            0x005 => Self::new(
                "Apple A11",
                MicroArch::AppleMonsoon,
                "Mistral (E) / Monsoon (P)",
                0x005,
                Some("10nm"),
            ),
            0x006 => Self::new(
                "Apple A12",
                MicroArch::AppleVortex,
                "Vortex (E) / Tempest (P)",
                0x006,
                Some("7nm"),
            ),
            0x007 => Self::new(
                "Apple A13",
                MicroArch::AppleLightning,
                "Lightning (E) / Thunder (P)",
                0x007,
                Some("7nm"),
            ),
            0x008 => Self::new(
                "Apple A14 / M1",
                MicroArch::AppleFirestorm,
                "Firestorm (E) / Icestorm (P)",
                0x008,
                Some("5nm"),
            ),
            0x009 => Self::new(
                "Apple M1 Pro / Max / Ultra",
                MicroArch::AppleFirestorm,
                "Firestorm (E) / Icestorm (P)",
                0x009,
                Some("5nm"),
            ),
            0x00A => Self::new(
                "Apple M2",
                MicroArch::AppleAvalanche,
                "Avalanche (E) / Blizzard (P)",
                0x00A,
                Some("5nm"),
            ),
            0x00B => Self::new(
                "Apple M2 Pro / Max / M2 Ultra",
                MicroArch::AppleAvalanche,
                "Avalanche (E) / Blizzard (P)",
                0x00B,
                Some("5nm"),
            ),
            0x00C => Self::new(
                "Apple A15 / M1",
                MicroArch::AppleAvalanche,
                "Avalanche (E) / Blizzard (P)",
                0x00C,
                Some("5nm"),
            ),
            0x00D => Self::new(
                "Apple M3",
                MicroArch::AppleHull,
                "Gibraltar (E) / Hull (P)",
                0x00D,
                Some("3nm"),
            ),
            0x00E => Self::new(
                "Apple M3 Pro / Max / M3 Ultra",
                MicroArch::AppleHull,
                "Gibraltar (E) / Hull (P)",
                0x00E,
                Some("3nm"),
            ),
            0x00F => Self::new(
                "Apple A16 / M1",
                MicroArch::AppleEverest,
                "Everest (E) / Sawmill (P)",
                0x00F,
                Some("4nm"),
            ),
            0x010 => Self::new(
                "Apple M4",
                MicroArch::AppleDawn,
                "Ice (E) / Dawn (P)",
                0x010,
                Some("3nm"),
            ),
            0x011 => Self::new(
                "Apple M4 Pro / Max / M4 Ultra",
                MicroArch::AppleDawn,
                "Ice (E) / Dawn (P)",
                0x011,
                Some("3nm"),
            ),
            0x012 => Self::new(
                "Apple A17 / M1",
                MicroArch::AppleDawn,
                "Ice (E) / Dawn (P)",
                0x012,
                Some("3nm"),
            ),
            0x100 => Self::new(
                "Apple A18",
                MicroArch::AppleIce,
                "Ice (E) / Dawn (P)",
                0x100,
                Some("3nm"),
            ),
            0x101 => Self::new(
                "Apple A18 Pro",
                MicroArch::AppleIce,
                "Ice (E) / Dawn (P)",
                0x101,
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
    fn test_apple_cpu_find() {
        let cpu = CpuArch::find(0x61, 0x008, 0x0);
        assert_eq!(cpu.marketing_name.as_str(), "Apple A14 / M1");
        assert_eq!(cpu.micro_arch, MicroArch::AppleFirestorm);
        assert_eq!(cpu.part_number, 0x008);
        assert_eq!(cpu.technology, Some("5nm"));
    }

    #[test]
    fn test_apple_cpu_unknown() {
        let cpu = CpuArch::find(0x61, 0x999, 0x0);
        assert_eq!(cpu.marketing_name.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_non_apple_implementer() {
        let cpu = CpuArch::find(0x41, 0xD0B, 0x0);
        assert_eq!(cpu.marketing_name.as_str(), UNK);
    }

    #[test]
    fn test_micro_arch_to_string() {
        assert_eq!(String::from(MicroArch::AppleFirestorm), "Firestorm");
        assert_eq!(String::from(MicroArch::AppleAvalanche), "Avalanche");
        assert_eq!(String::from(MicroArch::ArmCortexA76), "Cortex-A76");
    }
}
