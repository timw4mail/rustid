use crate::arm::CoreType;
use crate::arm::brand::*;
use crate::common::constants::*;
use crate::common::{Cache, UNK};

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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn to_bits(&self) -> usize {
        (self.implementer << IMPLEMENTER_OFFSET)
            | (self.variant << VARIANT_OFFSET)
            | (self.architecture << ARCHITECTURE_OFFSET)
            | (self.part << PART_OFFSET)
            | self.revision
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuCore {
    pub kind: CoreType,
    pub name: Option<String>,
    pub cache: Option<Cache>,
    pub count: u32,
}

type Implementer = Vendor;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    #[default]
    Unknown,

    AppleFirestorm,
    AppleIcestorm,
    AppleAvalanche,
    AppleBlizzard,
    AppleEverest,
    AppleSawtooth,

    Arm1176,

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

    QCScorpion,
    QCKrait,
    QCKryo,
    QCFalkor,
    QCSaphira,
    QCOryon,
}

impl MicroArch {
    pub fn core_type(&self) -> crate::common::CoreType {
        use crate::common::CoreType;
        match self {
            MicroArch::Unknown => CoreType::Performance,

            MicroArch::AppleFirestorm
            | MicroArch::AppleAvalanche
            | MicroArch::AppleEverest
            | MicroArch::AppleSawtooth => CoreType::Performance,

            MicroArch::AppleIcestorm | MicroArch::AppleBlizzard => CoreType::Efficiency,

            MicroArch::Arm1176 => CoreType::Performance,

            MicroArch::ArmCortexA7
            | MicroArch::ArmCortexA32
            | MicroArch::ArmCortexA35
            | MicroArch::ArmCortexA53
            | MicroArch::ArmCortexA55
            | MicroArch::ArmCortexA510
            | MicroArch::ArmCortexA520 => CoreType::Efficiency,

            MicroArch::ArmCortexA8
            | MicroArch::ArmCortexA9
            | MicroArch::ArmCortexA12
            | MicroArch::ArmCortexA15
            | MicroArch::ArmCortexA17
            | MicroArch::ArmCortexA65
            | MicroArch::ArmCortexA72
            | MicroArch::ArmCortexA73
            | MicroArch::ArmCortexA75
            | MicroArch::ArmCortexA76
            | MicroArch::ArmCortexA77
            | MicroArch::ArmCortexA78
            | MicroArch::ArmCortexA710
            | MicroArch::ArmCortexA715
            | MicroArch::ArmCortexA720
            | MicroArch::ArmCortexA725
            | MicroArch::ArmNeoverseE1
            | MicroArch::ArmNeoverseN1
            | MicroArch::ArmNeoverseN2
            | MicroArch::ArmNeoverseV1
            | MicroArch::ArmNeoverseV2 => CoreType::Performance,

            MicroArch::ArmCortexX1
            | MicroArch::ArmCortexX2
            | MicroArch::ArmCortexX3
            | MicroArch::ArmCortexX4 => CoreType::Super,

            MicroArch::QCScorpion
            | MicroArch::QCKrait
            | MicroArch::QCKryo
            | MicroArch::QCFalkor
            | MicroArch::QCSaphira
            | MicroArch::QCOryon => CoreType::Performance,
        }
    }
}

impl From<MicroArch> for String {
    fn from(ma: MicroArch) -> String {
        let s = match ma {
            MicroArch::Unknown => UNK,
            MicroArch::AppleFirestorm => "Firestorm",
            MicroArch::AppleIcestorm => "Icestorm",
            MicroArch::AppleAvalanche => "Avalanche",
            MicroArch::AppleBlizzard => "Blizzard",
            MicroArch::AppleEverest => "Everest",
            MicroArch::AppleSawtooth => "Sawtooth",
            MicroArch::Arm1176 => "ARM11/ARMv6",
            MicroArch::ArmCortexA7 => "Cortex-A7",
            MicroArch::ArmCortexA8 => "Cortex-A8",
            MicroArch::ArmCortexA9 => "Cortex-A9",
            MicroArch::ArmCortexA12 => "Cortex-A12",
            MicroArch::ArmCortexA15 => "Cortex-A15",
            MicroArch::ArmCortexA17 => "Cortex-A17",
            MicroArch::ArmCortexA32 => "Cortex-A32",
            MicroArch::ArmCortexA35 => "Cortex-A35",
            MicroArch::ArmCortexA53 => "Apollo", // See https://en.wikipedia.org/wiki/ARM_Cortex-A53
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

            MicroArch::QCScorpion => "Scorpion",
            MicroArch::QCKrait => "Krait",
            MicroArch::QCKryo => "Kryo",
            MicroArch::QCFalkor => "Falkor",
            MicroArch::QCSaphira => "Saphira",
            MicroArch::QCOryon => "Oryon",
        };

        String::from(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub implementer: Implementer,
    pub system: Option<String>,
    pub soc_model: Option<String>,
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
            None,
            UNK,
            None,
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
        system: Option<String>,
        model: &str,
        soc_model: Option<String>,
        micro_arch: MicroArch,
        code_name: &'static str,
        part_number: usize,
        technology: Option<&'static str>,
    ) -> Self {
        CpuArch {
            implementer,
            system,
            model: String::from(model),
            soc_model,
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
            IMPL_QUALCOMM => Self::find_qualcomm(part),
            _ => Self {
                implementer: Implementer::from(implementer),
                ..Self::default()
            },
        }
    }

    fn find_soc() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            let cpuinfo = crate::common::os::get_proc_cpuinfo_data();
            if let Some(last) = cpuinfo.last()
                && (!last.contains_key("processor"))
                && let Some(raw_soc) = last.get("Hardware")
            {
                return Some(String::from(raw_soc.trim()));
            }
            None
        }

        #[cfg(not(target_os = "linux"))]
        None
    }

    fn find_system() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            let cpuinfo = crate::common::os::get_proc_cpuinfo_data();
            if let Some(last) = cpuinfo.last()
                && (!last.contains_key("processor"))
                && let Some(raw) = last.get("Model")
            {
                return Some(String::from(raw.trim()));
            }
            None
        }

        #[cfg(not(target_os = "linux"))]
        None
    }

    fn find_arm(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str)] = &[
            (0xB76, "ARM ARM1176JZF-S", MicroArch::Arm1176, "ARM11/ARMv6"),
            (0xC07, "ARM Cortex-A7", MicroArch::ArmCortexA7, "Cortex-A7"),
            (0xC08, "ARM Cortex-A8", MicroArch::ArmCortexA8, "Cortex-A8"),
            (0xC09, "ARM Cortex-A9", MicroArch::ArmCortexA9, "Cortex-A9"),
            (
                0xC0A,
                "ARM Cortex-A12",
                MicroArch::ArmCortexA12,
                "Cortex-A12",
            ),
            (
                0xC0F,
                "ARM Cortex-A15",
                MicroArch::ArmCortexA15,
                "Cortex-A15",
            ),
            (
                0xC0E,
                "ARM Cortex-A17",
                MicroArch::ArmCortexA17,
                "Cortex-A17",
            ),
            (
                0xC20,
                "ARM Cortex-A32",
                MicroArch::ArmCortexA32,
                "Cortex-A32",
            ),
            (
                0xC23,
                "ARM Cortex-A35",
                MicroArch::ArmCortexA35,
                "Cortex-A35",
            ),
            (
                0xD03,
                "ARM Cortex-A53",
                MicroArch::ArmCortexA53,
                "Cortex-A53",
            ),
            (
                0xD05,
                "ARM Cortex-A55",
                MicroArch::ArmCortexA55,
                "Cortex-A55",
            ),
            (
                0xD08,
                "ARM Cortex-A65",
                MicroArch::ArmCortexA65,
                "Cortex-A65",
            ),
            (
                0xD0B,
                "ARM Cortex-A76",
                MicroArch::ArmCortexA76,
                "Cortex-A76",
            ),
            (
                0xD0C,
                "ARM Cortex-A73",
                MicroArch::ArmCortexA73,
                "Cortex-A73",
            ),
            (
                0xD0D,
                "ARM Cortex-A75",
                MicroArch::ArmCortexA75,
                "Cortex-A75",
            ),
            (
                0xD0E,
                "ARM Cortex-A76",
                MicroArch::ArmCortexA76,
                "Cortex-A76",
            ),
            (
                0xD10,
                "ARM Cortex-A77",
                MicroArch::ArmCortexA77,
                "Cortex-A77",
            ),
            (
                0xD11,
                "ARM Cortex-A78",
                MicroArch::ArmCortexA78,
                "Cortex-A78",
            ),
            (
                0xD46,
                "ARM Cortex-A510",
                MicroArch::ArmCortexA510,
                "Cortex-A510",
            ),
            (
                0xD80,
                "ARM Cortex-A520",
                MicroArch::ArmCortexA520,
                "Cortex-A520",
            ),
            (
                0xD47,
                "ARM Cortex-A710",
                MicroArch::ArmCortexA710,
                "Cortex-A710",
            ),
            (
                0xD4D,
                "ARM Cortex-A715",
                MicroArch::ArmCortexA715,
                "Cortex-A715",
            ),
            (
                0xD81,
                "ARM Cortex-A720",
                MicroArch::ArmCortexA720,
                "Cortex-A720",
            ),
            (
                0xD87,
                "ARM Cortex-A725",
                MicroArch::ArmCortexA725,
                "Cortex-A725",
            ),
            (0xD13, "ARM Cortex-X1", MicroArch::ArmCortexX1, "Cortex-X1"),
            (0xD48, "ARM Cortex-X2", MicroArch::ArmCortexX2, "Cortex-X2"),
            (0xD4E, "ARM Cortex-X3", MicroArch::ArmCortexX3, "Cortex-X3"),
            (0xD82, "ARM Cortex-X4", MicroArch::ArmCortexX4, "Cortex-X4"),
            (
                0xD40,
                "ARM Neoverse E1",
                MicroArch::ArmNeoverseE1,
                "Neoverse E1",
            ),
            (
                0xD41,
                "ARM Neoverse N1",
                MicroArch::ArmNeoverseN1,
                "Neoverse N1",
            ),
            (
                0xD49,
                "ARM Neoverse N2",
                MicroArch::ArmNeoverseN2,
                "Neoverse N2",
            ),
            (
                0xD44,
                "ARM Neoverse V1",
                MicroArch::ArmNeoverseV1,
                "Neoverse V1",
            ),
            (
                0xD4F,
                "ARM Neoverse V2",
                MicroArch::ArmNeoverseV2,
                "Neoverse V2",
            ),
        ];
        let soc_model = CpuArch::find_soc();
        let system = CpuArch::find_system();
        PARTS
            .iter()
            .find(|(p, _, _, _)| *p == part)
            .map(|&(_, model, ma, name)| {
                Self::new(
                    Implementer::Arm,
                    system,
                    model,
                    soc_model,
                    ma,
                    name,
                    part,
                    None,
                )
            })
            .unwrap_or_else(|| Self {
                implementer: Implementer::Arm,
                ..Self::default()
            })
    }

    fn find_apple(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, &str)] = &[
            (0x022, "Apple M1", MicroArch::AppleIcestorm, "Tonga", N5),
            (0x023, "Apple M1", MicroArch::AppleFirestorm, "Tonga", N5),
            (
                0x024,
                "Apple M1 Pro",
                MicroArch::AppleIcestorm,
                "Jade Chop",
                N5,
            ),
            (
                0x025,
                "Apple M1 Pro",
                MicroArch::AppleFirestorm,
                "Jade Chop",
                N5,
            ),
            (
                0x028,
                "Apple M1 Max",
                MicroArch::AppleIcestorm,
                "Jade 1C",
                N5,
            ),
            (
                0x029,
                "Apple M1 Max",
                MicroArch::AppleFirestorm,
                "Jade 1C",
                N5,
            ),
            (0x032, "Apple M2", MicroArch::AppleBlizzard, "Staten", N5),
            (0x033, "Apple M2", MicroArch::AppleAvalanche, "Staten", N5),
            (
                0x034,
                "Apple M2 Pro",
                MicroArch::AppleBlizzard,
                "Rhodes Chop",
                N5,
            ),
            (
                0x035,
                "Apple M2 Pro",
                MicroArch::AppleAvalanche,
                "Rhodes Chop",
                N5,
            ),
            (
                0x038,
                "Apple M2 Max",
                MicroArch::AppleBlizzard,
                "Rhodes 1C",
                N5,
            ),
            (
                0x039,
                "Apple M2 Max",
                MicroArch::AppleAvalanche,
                "Rhodes 1C",
                N5,
            ),
            (0x042, "Apple M3", MicroArch::AppleEverest, "Ibiza", N3),
            (0x043, "Apple M3", MicroArch::AppleSawtooth, "Ibiza", N3),
            (0x044, "Apple M3 Pro", MicroArch::AppleEverest, "Lobos", N3),
            (0x045, "Apple M3 Pro", MicroArch::AppleSawtooth, "Lobos", N3),
            (0x048, "Apple M3 Max", MicroArch::AppleEverest, "Palma", N3),
            (0x049, "Apple M3 Max", MicroArch::AppleSawtooth, "Palma", N3),
            (0x052, "Apple M4", MicroArch::AppleEverest, "Donan", N3),
            (0x053, "Apple M4", MicroArch::AppleSawtooth, "Donan", N3),
            (
                0x054,
                "Apple M4 Pro",
                MicroArch::AppleEverest,
                "Brava Chop",
                N3,
            ),
            (
                0x055,
                "Apple M4 Pro",
                MicroArch::AppleSawtooth,
                "Brava Chop",
                N3,
            ),
            (0x058, "Apple M4 Max", MicroArch::AppleEverest, "Brava", N3),
            (0x059, "Apple M4 Max", MicroArch::AppleSawtooth, "Brava", N3),
            (
                0x101,
                "Apple A18 Pro",
                MicroArch::AppleEverest,
                "Tahiti",
                N3,
            ),
        ];
        let soc_model = CpuArch::find_soc();
        let system = CpuArch::find_system();
        PARTS
            .iter()
            .find(|(p, _, _, _, _)| *p == part)
            .map(|&(_, model, ma, name, tech)| {
                Self::new(
                    Implementer::Apple,
                    system,
                    model,
                    soc_model,
                    ma,
                    name,
                    part,
                    Some(tech),
                )
            })
            .unwrap_or_else(|| Self {
                implementer: Implementer::Apple,
                ..Self::default()
            })
    }

    fn find_qualcomm(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x001,
                "Snapdragon X Elite",
                MicroArch::QCOryon,
                "Oryon",
                Some(N4),
            ),
            (
                0x00F,
                "Snapdragon S1/S2/S3",
                MicroArch::QCScorpion,
                "Scorpion",
                Some("65-45nm"),
            ),
            (
                0x02D,
                "Snapdragon S4",
                MicroArch::QCScorpion,
                "Scorpion",
                Some(N28),
            ),
            (
                0x04D,
                "Snapdragon S4 Plus/Pro",
                MicroArch::QCKrait,
                "Krait",
                Some(N28),
            ),
            (
                0x06F,
                "Snapdragon 800/801",
                MicroArch::QCKrait,
                "Krait 400",
                Some(N28),
            ),
            (
                0x201,
                "Snapdragon 820/821",
                MicroArch::QCKryo,
                "Kryo",
                Some(N14),
            ),
            (
                0x205,
                "Snapdragon 820/821",
                MicroArch::QCKryo,
                "Kryo",
                Some(N14),
            ),
            (
                0x211,
                "Snapdragon 820/821",
                MicroArch::QCKryo,
                "Kryo",
                Some(N14),
            ),
            (
                0x800,
                "Snapdragon 835",
                MicroArch::QCFalkor,
                "Kryo 280 Gold",
                Some(N10),
            ),
            (
                0x801,
                "Snapdragon 835",
                MicroArch::ArmCortexA53,
                "Kryo 280 Silver",
                Some(N10),
            ),
            (
                0x802,
                "Snapdragon 845",
                MicroArch::ArmCortexA75,
                "Kryo 385 Gold",
                Some(N10),
            ),
            (
                0x803,
                "Snapdragon 845",
                MicroArch::ArmCortexA55,
                "Kryo 385 Silver",
                Some(N10),
            ),
            (
                0x804,
                "Snapdragon 855",
                MicroArch::ArmCortexA76,
                "Kryo 485 Gold",
                Some(N7),
            ),
            (
                0x805,
                "Snapdragon 855",
                MicroArch::ArmCortexA55,
                "Kryo 485 Silver",
                Some(N7),
            ),
            (
                0xC00,
                "Centriq 2400",
                MicroArch::QCFalkor,
                "Falkor",
                Some(N10),
            ),
            (
                0xC01,
                "Qualcomm Saphira",
                MicroArch::QCSaphira,
                "Saphira",
                None,
            ),
        ];
        let soc_model = CpuArch::find_soc();
        let system = CpuArch::find_system();
        PARTS
            .iter()
            .find(|(p, _, _, _, _)| *p == part)
            .map(|&(_, model, ma, name, tech)| {
                Self::new(
                    Implementer::Qualcomm,
                    system,
                    model,
                    soc_model,
                    ma,
                    name,
                    part,
                    tech,
                )
            })
            .unwrap_or_else(|| Self {
                implementer: Implementer::Qualcomm,
                ..Self::default()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midr_parsing() {
        let midr = Midr::new(0x61FF0F02);
        assert_eq!(midr.implementer, IMPL_APPLE);
        assert_eq!(midr.variant, 0xF);
        assert_eq!(midr.architecture, 0xF);
        assert_eq!(midr.part, 0x0F0);
        assert_eq!(midr.revision, 0x2);
    }

    #[test]
    fn test_midr_parsing_m1() {
        let midr = Midr::new(0x611F0231);
        assert_eq!(midr.implementer, IMPL_APPLE);
        assert_eq!(midr.variant, 0x1);
        assert_eq!(midr.architecture, 0xF);
        assert_eq!(midr.part, 0x023);
        assert_eq!(midr.revision, 0x1);
    }

    #[test]
    fn test_apple_m1_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x022, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1");
        assert_eq!(cpu.micro_arch, MicroArch::AppleIcestorm);
    }

    #[test]
    fn test_apple_m1_pro_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x024, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleIcestorm);
    }

    #[test]
    fn test_apple_m2_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x032, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M2");
        assert_eq!(cpu.micro_arch, MicroArch::AppleBlizzard);
    }

    #[test]
    fn test_apple_m3_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x042, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M3");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_m4_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x052, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M4");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_a18_pro_find() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x101, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple A18 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_cpu_unknown() {
        let cpu = CpuArch::find(IMPL_APPLE, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_non_apple_implementer() {
        let cpu = CpuArch::find(IMPL_ARM, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
    }

    #[test]
    fn test_arm_cortex_a76_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD0E, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A76");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA76);
    }

    #[test]
    fn test_arm_cortex_a55_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD05, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A55");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA55);
    }

    #[test]
    fn test_arm_cortex_a53_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD03, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A53");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA53);
    }

    #[test]
    fn test_arm_cortex_x1_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD13, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-X1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexX1);
    }

    #[test]
    fn test_arm_neoverse_n1_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD41, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Neoverse N1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmNeoverseN1);
    }

    #[test]
    fn test_arm_unknown_part() {
        let cpu = CpuArch::find(IMPL_ARM, 0x999, 0x0);
        assert_eq!(cpu.model.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_micro_arch_to_string() {
        assert_eq!(String::from(MicroArch::AppleFirestorm), "Firestorm");
        assert_eq!(String::from(MicroArch::AppleAvalanche), "Avalanche");
        assert_eq!(String::from(MicroArch::ArmCortexA76), "Cortex-A76");
    }

    #[test]
    fn test_qualcomm_oryon_find() {
        let cpu = CpuArch::find(IMPL_QUALCOMM, 0x001, 0x0);
        assert_eq!(cpu.model.as_str(), "Snapdragon X Elite");
        assert_eq!(cpu.micro_arch, MicroArch::QCOryon);
    }
}
