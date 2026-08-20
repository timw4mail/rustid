use crate::arm::brand::*;
use crate::common::CoreType;
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
    pub raw: usize,
}

impl Midr {
    pub fn new(midr: usize) -> Midr {
        Midr {
            implementer: (midr & IMPLEMENTER_MASK) >> IMPLEMENTER_OFFSET,
            variant: (midr & VARIANT_MASK) >> VARIANT_OFFSET,
            architecture: (midr & ARCHITECTURE_MASK) >> ARCHITECTURE_OFFSET,
            part: (midr & PART_MASK) >> PART_OFFSET,
            revision: midr & REVISION_MASK,
            raw: midr,
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

/// ARM Microarchitectures across vendors.
///
/// Reference Data Sources:
/// - util-linux lscpu-arm.c: https://github.com/util-linux/util-linux/blob/master/sys-utils/lscpu-arm.c
/// - pytorch/cpuinfo uarch.c: https://github.com/pytorch/cpuinfo/blob/main/src/arm/uarch.c
/// - Linux kernel cputype.h: arch/arm64/include/asm/cputype.h
/// - bp0/armids: https://github.com/bp0/armids/blob/master/arm.ids
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    #[default]
    Unknown,

    // Apple Silicon
    AppleSwift,
    AppleCyclone,
    AppleTyphoon,
    AppleTwister,
    AppleHurricane,
    AppleMonsoon,
    AppleMistral,
    AppleVortex,
    AppleTempest,
    AppleLightning,
    AppleThunder,
    AppleIcestorm,
    AppleFirestorm,
    AppleBlizzard,
    AppleAvalanche,
    AppleSawtooth,
    AppleEverest,

    // ARM Cortex & Classic
    Arm1176,
    ArmCortexA5,
    ArmCortexA7,
    ArmCortexA8,
    ArmCortexA9,
    ArmCortexA12,
    ArmCortexA15,
    ArmCortexA17,
    ArmCortexA32,
    ArmCortexA34,
    ArmCortexA35,
    ArmCortexA53,
    ArmCortexA55,
    ArmCortexA57,
    ArmCortexA65,
    ArmCortexA72,
    ArmCortexA73,
    ArmCortexA75,
    ArmCortexA76,
    ArmCortexA76AE,
    ArmCortexA77,
    ArmCortexA78,
    ArmCortexA78AE,
    ArmCortexA78C,
    ArmCortexA510,
    ArmCortexA520,
    ArmCortexA710,
    ArmCortexA715,
    ArmCortexA720,
    ArmCortexA725,
    ArmCortexA320,
    ArmCortexX1,
    ArmCortexX1C,
    ArmCortexX2,
    ArmCortexX3,
    ArmCortexX4,
    ArmCortexX925,

    // ARM Neoverse
    ArmNeoverseE1,
    ArmNeoverseN1,
    ArmNeoverseN2,
    ArmNeoverseN3,
    ArmNeoverseV1,
    ArmNeoverseV2,
    ArmNeoverseV3,

    // Ampere
    AmpereOne,

    // Broadcom
    BrahmaB15,
    BrahmaB53,

    // Cavium / Marvell
    ThunderX,
    ThunderX2,
    OcteonTX2,

    // Fujitsu
    FujitsuA64FX,

    // HiSilicon
    Kunpeng920,
    Kunpeng950,

    // Nvidia
    NvidiaDenver,
    NvidiaDenver2,
    NvidiaCarmel,

    // Phytium
    PhytiumFTC,

    // Qualcomm
    QCScorpion,
    QCKrait,
    QCKryo,
    QCFalkor,
    QCSaphira,
    QCOryon,

    // Samsung
    ExynosM1,
    ExynosM2,
    ExynosM3,
    ExynosM4,
    ExynosM5,
}

impl MicroArch {
    pub fn core_type(&self) -> crate::common::CoreType {
        use crate::common::CoreType;
        match self {
            MicroArch::Unknown => CoreType::Performance,

            // Apple Efficiency Cores
            MicroArch::AppleIcestorm
            | MicroArch::AppleBlizzard
            | MicroArch::AppleSawtooth
            | MicroArch::AppleMistral
            | MicroArch::AppleTempest
            | MicroArch::AppleThunder => CoreType::Efficiency,

            // Apple Performance Cores
            MicroArch::AppleFirestorm
            | MicroArch::AppleAvalanche
            | MicroArch::AppleEverest
            | MicroArch::AppleSwift
            | MicroArch::AppleCyclone
            | MicroArch::AppleTyphoon
            | MicroArch::AppleTwister
            | MicroArch::AppleHurricane
            | MicroArch::AppleMonsoon
            | MicroArch::AppleVortex
            | MicroArch::AppleLightning => CoreType::Performance,

            // ARM Efficiency Cores
            MicroArch::ArmCortexA5
            | MicroArch::ArmCortexA7
            | MicroArch::ArmCortexA32
            | MicroArch::ArmCortexA34
            | MicroArch::ArmCortexA35
            | MicroArch::ArmCortexA53
            | MicroArch::ArmCortexA55
            | MicroArch::ArmCortexA510
            | MicroArch::ArmCortexA520 => CoreType::Efficiency,

            // ARM Super / Ultra Cores
            MicroArch::ArmCortexX1
            | MicroArch::ArmCortexX1C
            | MicroArch::ArmCortexX2
            | MicroArch::ArmCortexX3
            | MicroArch::ArmCortexX4
            | MicroArch::ArmCortexX925 => CoreType::Super,

            // ARM Performance Cores (and all other server/performance microarchitectures)
            _ => CoreType::Performance,
        }
    }
}

impl From<MicroArch> for String {
    fn from(ma: MicroArch) -> String {
        let s = match ma {
            MicroArch::Unknown => UNK,

            MicroArch::AppleSwift => "Swift",
            MicroArch::AppleCyclone => "Cyclone",
            MicroArch::AppleTyphoon => "Typhoon",
            MicroArch::AppleTwister => "Twister",
            MicroArch::AppleHurricane => "Hurricane",
            MicroArch::AppleMonsoon => "Monsoon",
            MicroArch::AppleMistral => "Mistral",
            MicroArch::AppleVortex => "Vortex",
            MicroArch::AppleTempest => "Tempest",
            MicroArch::AppleLightning => "Lightning",
            MicroArch::AppleThunder => "Thunder",
            MicroArch::AppleFirestorm => "Firestorm",
            MicroArch::AppleIcestorm => "Icestorm",
            MicroArch::AppleAvalanche => "Avalanche",
            MicroArch::AppleBlizzard => "Blizzard",
            MicroArch::AppleEverest => "Everest",
            MicroArch::AppleSawtooth => "Sawtooth",

            MicroArch::Arm1176 => "ARM11/ARMv6",
            MicroArch::ArmCortexA5 => "Cortex-A5",
            MicroArch::ArmCortexA7 => "Cortex-A7",
            MicroArch::ArmCortexA8 => "Cortex-A8",
            MicroArch::ArmCortexA9 => "Cortex-A9",
            MicroArch::ArmCortexA12 => "Cortex-A12",
            MicroArch::ArmCortexA15 => "Cortex-A15",
            MicroArch::ArmCortexA17 => "Cortex-A17",
            MicroArch::ArmCortexA32 => "Cortex-A32",
            MicroArch::ArmCortexA34 => "Cortex-A34",
            MicroArch::ArmCortexA35 => "Cortex-A35",
            MicroArch::ArmCortexA53 => "Cortex-A53",
            MicroArch::ArmCortexA55 => "Cortex-A55",
            MicroArch::ArmCortexA57 => "Cortex-A57",
            MicroArch::ArmCortexA65 => "Cortex-A65",
            MicroArch::ArmCortexA72 => "Cortex-A72",
            MicroArch::ArmCortexA73 => "Cortex-A73",
            MicroArch::ArmCortexA75 => "Cortex-A75",
            MicroArch::ArmCortexA76 => "Cortex-A76",
            MicroArch::ArmCortexA76AE => "Cortex-A76AE",
            MicroArch::ArmCortexA77 => "Cortex-A77",
            MicroArch::ArmCortexA78 => "Cortex-A78",
            MicroArch::ArmCortexA78AE => "Cortex-A78AE",
            MicroArch::ArmCortexA78C => "Cortex-A78C",
            MicroArch::ArmCortexA510 => "Cortex-A510",
            MicroArch::ArmCortexA520 => "Cortex-A520",
            MicroArch::ArmCortexA710 => "Cortex-A710",
            MicroArch::ArmCortexA715 => "Cortex-A715",
            MicroArch::ArmCortexA720 => "Cortex-A720",
            MicroArch::ArmCortexA725 => "Cortex-A725",
            MicroArch::ArmCortexA320 => "Cortex-A320",
            MicroArch::ArmCortexX1 => "Cortex-X1",
            MicroArch::ArmCortexX1C => "Cortex-X1C",
            MicroArch::ArmCortexX2 => "Cortex-X2",
            MicroArch::ArmCortexX3 => "Cortex-X3",
            MicroArch::ArmCortexX4 => "Cortex-X4",
            MicroArch::ArmCortexX925 => "Cortex-X925",

            MicroArch::ArmNeoverseE1 => "Neoverse E1",
            MicroArch::ArmNeoverseN1 => "Neoverse N1",
            MicroArch::ArmNeoverseN2 => "Neoverse N2",
            MicroArch::ArmNeoverseN3 => "Neoverse N3",
            MicroArch::ArmNeoverseV1 => "Neoverse V1",
            MicroArch::ArmNeoverseV2 => "Neoverse V2",
            MicroArch::ArmNeoverseV3 => "Neoverse V3",

            MicroArch::AmpereOne => "AmpereOne",

            MicroArch::BrahmaB15 => "Brahma B15",
            MicroArch::BrahmaB53 => "Brahma B53",

            MicroArch::ThunderX => "ThunderX",
            MicroArch::ThunderX2 => "ThunderX2",
            MicroArch::OcteonTX2 => "OcteonTX2",

            MicroArch::FujitsuA64FX => "A64FX",

            MicroArch::Kunpeng920 => "Kunpeng 920",
            MicroArch::Kunpeng950 => "Kunpeng 950",

            MicroArch::NvidiaDenver => "Denver",
            MicroArch::NvidiaDenver2 => "Denver 2",
            MicroArch::NvidiaCarmel => "Carmel",

            MicroArch::PhytiumFTC => "FTC",

            MicroArch::QCScorpion => "Scorpion",
            MicroArch::QCKrait => "Krait",
            MicroArch::QCKryo => "Kryo",
            MicroArch::QCFalkor => "Falkor",
            MicroArch::QCSaphira => "Saphira",
            MicroArch::QCOryon => "Oryon",

            MicroArch::ExynosM1 => "Exynos M1",
            MicroArch::ExynosM2 => "Exynos M2",
            MicroArch::ExynosM3 => "Exynos M3",
            MicroArch::ExynosM4 => "Exynos M4",
            MicroArch::ExynosM5 => "Exynos M5",
        };

        String::from(s)
    }
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
        Self {
            implementer: Implementer::default(),
            model: String::from(UNK),
            micro_arch: MicroArch::default(),
            code_name: UNK,
            part_number: 0,
            technology: None,
        }
    }
}

impl CpuArch {
    pub fn find(implementer: usize, part: usize, _variant: usize) -> Self {
        match implementer {
            IMPL_ARM => Self::find_arm(part),
            IMPL_APPLE => Self::find_apple(part),
            IMPL_QUALCOMM => Self::find_qualcomm(part),
            IMPL_SAMSUNG => Self::find_samsung(part),
            IMPL_NVIDIA => Self::find_nvidia(part),
            IMPL_AMPERE => Self::find_ampere(part),
            IMPL_HISILICON => Self::find_hisilicon(part),
            IMPL_FUJITSU => Self::find_fujitsu(part),
            IMPL_BROADCOM => Self::find_broadcom(part),
            IMPL_CAVIUM => Self::find_cavium(part),
            IMPL_PHYTIUM => Self::find_phytium(part),
            _ => Self {
                implementer: Implementer::from(implementer),
                ..Self::default()
            },
        }
    }

    fn find_impl(
        part: usize,
        implementer: Implementer,
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
            .find(|(p, _, _, _, _)| *p == part)
            .map(|&(_, model, ma, name, tech)| CpuArch {
                implementer,
                model: String::from(model),
                micro_arch: ma,
                code_name: name,
                part_number: part,
                technology: tech,
            })
            .unwrap_or_else(move || Self {
                implementer,
                ..Self::default()
            })
    }

    /// ARM Ltd implementer (0x41) part lookups.
    ///
    /// References:
    /// - util-linux: https://github.com/util-linux/util-linux/blob/master/sys-utils/lscpu-arm.c
    /// - pytorch/cpuinfo: https://github.com/pytorch/cpuinfo/blob/main/src/arm/uarch.c
    /// - Linux kernel: arch/arm64/include/asm/cputype.h
    fn find_arm(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            // Raspberry Pi 1B
            (
                0xB76,
                "ARM ARM1176JZF-S",
                MicroArch::Arm1176,
                "ARM11/ARMv6",
                None,
            ),
            (
                0xC05,
                "ARM Cortex-A5",
                MicroArch::ArmCortexA5,
                "Cortex-A5",
                None,
            ),
            // Raspberry Pi 2B
            (
                0xC07,
                "ARM Cortex-A7",
                MicroArch::ArmCortexA7,
                "Cortex-A7",
                None,
            ),
            (
                0xC08,
                "ARM Cortex-A8",
                MicroArch::ArmCortexA8,
                "Cortex-A8",
                None,
            ),
            (
                0xC09,
                "ARM Cortex-A9",
                MicroArch::ArmCortexA9,
                "Cortex-A9",
                None,
            ),
            (
                0xC0C,
                "ARM Cortex-A12",
                MicroArch::ArmCortexA12,
                "Cortex-A12",
                None,
            ),
            (
                0xC0D,
                "ARM Cortex-A17",
                MicroArch::ArmCortexA17,
                "Cortex-A17",
                None,
            ),
            (
                0xC0E,
                "ARM Cortex-A17",
                MicroArch::ArmCortexA17,
                "Cortex-A17",
                None,
            ),
            (
                0xC0F,
                "ARM Cortex-A15",
                MicroArch::ArmCortexA15,
                "Cortex-A15",
                None,
            ),
            (
                0xC20,
                "ARM Cortex-A32",
                MicroArch::ArmCortexA32,
                "Cortex-A32",
                None,
            ),
            (
                0xC23,
                "ARM Cortex-A35",
                MicroArch::ArmCortexA35,
                "Cortex-A35",
                None,
            ),
            (
                0xD01,
                "ARM Cortex-A32",
                MicroArch::ArmCortexA32,
                "Cortex-A32",
                None,
            ),
            (
                0xD02,
                "ARM Cortex-A34",
                MicroArch::ArmCortexA34,
                "Cortex-A34",
                None,
            ),
            // Raspberry Pi 3,
            // Raspberry Pi Zero 2
            (
                0xD03,
                "ARM Cortex-A53",
                MicroArch::ArmCortexA53,
                "Cortex-A53",
                None,
            ),
            (
                0xD04,
                "ARM Cortex-A35",
                MicroArch::ArmCortexA35,
                "Cortex-A35",
                None,
            ),
            (
                0xD05,
                "ARM Cortex-A55",
                MicroArch::ArmCortexA55,
                "Cortex-A55",
                None,
            ),
            (
                0xD06,
                "ARM Cortex-A65",
                MicroArch::ArmCortexA65,
                "Cortex-A65",
                None,
            ),
            (
                0xD07,
                "ARM Cortex-A57",
                MicroArch::ArmCortexA57,
                "Cortex-A57",
                None,
            ),
            // Raspberry Pi 4
            (
                0xD08,
                "ARM Cortex-A72",
                MicroArch::ArmCortexA72,
                "Maya",
                None,
            ),
            (
                0xD09,
                "ARM Cortex-A73",
                MicroArch::ArmCortexA73,
                "Cortex-A73",
                None,
            ),
            (
                0xD0A,
                "ARM Cortex-A75",
                MicroArch::ArmCortexA75,
                "Cortex-A75",
                None,
            ),
            // Raspberry Pi 5
            (
                0xD0B,
                "ARM Cortex-A76",
                MicroArch::ArmCortexA76,
                "Enyo",
                None,
            ),

            (
                0xD0C,
                "ARM Neoverse N1",
                MicroArch::ArmNeoverseN1,
                "Neoverse N1",
                None,
            ),
            (
                0xD0D,
                "ARM Cortex-A77",
                MicroArch::ArmCortexA77,
                "Cortex-A77",
                None,
            ),
            (
                0xD0E,
                "ARM Cortex-A76AE",
                MicroArch::ArmCortexA76AE,
                "Cortex-A76AE",
                None,
            ),
            (
                0xD40,
                "ARM Neoverse V1",
                MicroArch::ArmNeoverseV1,
                "Neoverse V1",
                None,
            ),
            (
                0xD41,
                "ARM Cortex-A78",
                MicroArch::ArmCortexA78,
                "Cortex-A78",
                None,
            ),
            (
                0xD42,
                "ARM Cortex-A78AE",
                MicroArch::ArmCortexA78AE,
                "Cortex-A78AE",
                None,
            ),
            (
                0xD44,
                "ARM Cortex-X1",
                MicroArch::ArmCortexX1,
                "Cortex-X1",
                None,
            ),
            (
                0xD46,
                "ARM Cortex-A510",
                MicroArch::ArmCortexA510,
                "Cortex-A510",
                None,
            ),
            (
                0xD47,
                "ARM Cortex-A710",
                MicroArch::ArmCortexA710,
                "Cortex-A710",
                None,
            ),
            (
                0xD48,
                "ARM Cortex-X2",
                MicroArch::ArmCortexX2,
                "Cortex-X2",
                None,
            ),
            (
                0xD49,
                "ARM Neoverse N2",
                MicroArch::ArmNeoverseN2,
                "Neoverse N2",
                None,
            ),
            (
                0xD4A,
                "ARM Neoverse E1",
                MicroArch::ArmNeoverseE1,
                "Neoverse E1",
                None,
            ),
            (
                0xD4B,
                "ARM Cortex-A78C",
                MicroArch::ArmCortexA78C,
                "Cortex-A78C",
                None,
            ),
            (
                0xD4C,
                "ARM Cortex-X1C",
                MicroArch::ArmCortexX1C,
                "Cortex-X1C",
                None,
            ),
            (
                0xD4D,
                "ARM Cortex-A715",
                MicroArch::ArmCortexA715,
                "Cortex-A715",
                None,
            ),
            (
                0xD4E,
                "ARM Cortex-X3",
                MicroArch::ArmCortexX3,
                "Cortex-X3",
                None,
            ),
            (
                0xD4F,
                "ARM Neoverse V2",
                MicroArch::ArmNeoverseV2,
                "Neoverse V2",
                None,
            ),
            (
                0xD80,
                "ARM Cortex-A520",
                MicroArch::ArmCortexA520,
                "Cortex-A520",
                None,
            ),
            (
                0xD81,
                "ARM Cortex-A720",
                MicroArch::ArmCortexA720,
                "Cortex-A720",
                None,
            ),
            (
                0xD82,
                "ARM Cortex-X4",
                MicroArch::ArmCortexX4,
                "Cortex-X4",
                None,
            ),
            (
                0xD84,
                "ARM Neoverse V3",
                MicroArch::ArmNeoverseV3,
                "Neoverse V3",
                None,
            ),
            (
                0xD85,
                "ARM Cortex-X925",
                MicroArch::ArmCortexX925,
                "Cortex-X925",
                None,
            ),
            (
                0xD87,
                "ARM Cortex-A725",
                MicroArch::ArmCortexA725,
                "Cortex-A725",
                None,
            ),
            (
                0xD8E,
                "ARM Neoverse N3",
                MicroArch::ArmNeoverseN3,
                "Neoverse N3",
                None,
            ),
            (
                0xD8F,
                "ARM Cortex-A320",
                MicroArch::ArmCortexA320,
                "Cortex-A320",
                None,
            ),
        ];
        Self::find_impl(part, Implementer::Arm, PARTS)
    }

    /// Apple implementer (0x61) part lookups.
    ///
    /// References:
    /// - util-linux: https://github.com/util-linux/util-linux/blob/master/sys-utils/lscpu-arm.c
    /// - pytorch/cpuinfo: https://github.com/pytorch/cpuinfo/blob/main/src/arm/uarch.c
    fn find_apple(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (0x000, "Apple Swift", MicroArch::AppleSwift, "Swift", None),
            (
                0x001,
                "Apple Cyclone",
                MicroArch::AppleCyclone,
                "Cyclone",
                None,
            ),
            (
                0x002,
                "Apple Typhoon",
                MicroArch::AppleTyphoon,
                "Typhoon",
                None,
            ),
            (
                0x004,
                "Apple Twister",
                MicroArch::AppleTwister,
                "Twister",
                None,
            ),
            (
                0x006,
                "Apple Hurricane",
                MicroArch::AppleHurricane,
                "Hurricane",
                None,
            ),
            (
                0x008,
                "Apple Monsoon",
                MicroArch::AppleMonsoon,
                "Monsoon",
                None,
            ),
            (
                0x009,
                "Apple Mistral",
                MicroArch::AppleMistral,
                "Mistral",
                None,
            ),
            (
                0x00B,
                "Apple Vortex",
                MicroArch::AppleVortex,
                "Vortex",
                None,
            ),
            (
                0x00C,
                "Apple Tempest",
                MicroArch::AppleTempest,
                "Tempest",
                None,
            ),
            (
                0x012,
                "Apple Lightning",
                MicroArch::AppleLightning,
                "Lightning",
                None,
            ),
            (
                0x013,
                "Apple Thunder",
                MicroArch::AppleThunder,
                "Thunder",
                None,
            ),
            (
                0x020,
                "Apple A14",
                MicroArch::AppleIcestorm,
                "Icestorm",
                Some(N5),
            ),
            (
                0x021,
                "Apple A14",
                MicroArch::AppleFirestorm,
                "Firestorm",
                Some(N5),
            ),
            (
                0x022,
                "Apple M1",
                MicroArch::AppleIcestorm,
                "Tonga",
                Some(N5),
            ),
            (
                0x023,
                "Apple M1",
                MicroArch::AppleFirestorm,
                "Tonga",
                Some(N5),
            ),
            (
                0x024,
                "Apple M1 Pro",
                MicroArch::AppleIcestorm,
                "Jade Chop",
                Some(N5),
            ),
            (
                0x025,
                "Apple M1 Pro",
                MicroArch::AppleFirestorm,
                "Jade Chop",
                Some(N5),
            ),
            (
                0x028,
                "Apple M1 Max",
                MicroArch::AppleIcestorm,
                "Jade 1C",
                Some(N5),
            ),
            (
                0x029,
                "Apple M1 Max",
                MicroArch::AppleFirestorm,
                "Jade 1C",
                Some(N5),
            ),
            (
                0x030,
                "Apple A15",
                MicroArch::AppleBlizzard,
                "Blizzard",
                Some(N5),
            ),
            (
                0x031,
                "Apple A15",
                MicroArch::AppleAvalanche,
                "Avalanche",
                Some(N5),
            ),
            (
                0x032,
                "Apple M2",
                MicroArch::AppleBlizzard,
                "Staten",
                Some(N5),
            ),
            (
                0x033,
                "Apple M2",
                MicroArch::AppleAvalanche,
                "Staten",
                Some(N5),
            ),
            (
                0x034,
                "Apple M2 Pro",
                MicroArch::AppleBlizzard,
                "Rhodes Chop",
                Some(N5),
            ),
            (
                0x035,
                "Apple M2 Pro",
                MicroArch::AppleAvalanche,
                "Rhodes Chop",
                Some(N5),
            ),
            (
                0x036,
                "Apple A16",
                MicroArch::AppleSawtooth,
                "Sawtooth",
                Some(N4),
            ),
            (
                0x037,
                "Apple A16",
                MicroArch::AppleEverest,
                "Everest",
                Some(N4),
            ),
            (
                0x038,
                "Apple M2 Max",
                MicroArch::AppleBlizzard,
                "Rhodes 1C",
                Some(N5),
            ),
            (
                0x039,
                "Apple M2 Max",
                MicroArch::AppleAvalanche,
                "Rhodes 1C",
                Some(N5),
            ),
            (
                0x042,
                "Apple M3",
                MicroArch::AppleSawtooth,
                "Ibiza",
                Some(N3),
            ),
            (
                0x043,
                "Apple M3",
                MicroArch::AppleEverest,
                "Ibiza",
                Some(N3),
            ),
            (
                0x044,
                "Apple M3 Pro",
                MicroArch::AppleSawtooth,
                "Lobos",
                Some(N3),
            ),
            (
                0x045,
                "Apple M3 Pro",
                MicroArch::AppleEverest,
                "Lobos",
                Some(N3),
            ),
            (
                0x048,
                "Apple M3 Max",
                MicroArch::AppleSawtooth,
                "Palma",
                Some(N3),
            ),
            (
                0x049,
                "Apple M3 Max",
                MicroArch::AppleEverest,
                "Palma",
                Some(N3),
            ),
            (
                0x052,
                "Apple M4",
                MicroArch::AppleSawtooth,
                "Donan",
                Some(N3),
            ),
            (
                0x053,
                "Apple M4",
                MicroArch::AppleEverest,
                "Donan",
                Some(N3),
            ),
            (
                0x054,
                "Apple M4 Pro",
                MicroArch::AppleSawtooth,
                "Brava Chop",
                Some(N3),
            ),
            (
                0x055,
                "Apple M4 Pro",
                MicroArch::AppleEverest,
                "Brava Chop",
                Some(N3),
            ),
            (
                0x058,
                "Apple M4 Max",
                MicroArch::AppleSawtooth,
                "Brava",
                Some(N3),
            ),
            (
                0x059,
                "Apple M4 Max",
                MicroArch::AppleEverest,
                "Brava",
                Some(N3),
            ),
            (
                0x101,
                "Apple A18 Pro",
                MicroArch::AppleEverest,
                "Tahiti",
                Some(N3),
            ),
        ];
        Self::find_impl(part, Implementer::Apple, PARTS)
    }

    /// Qualcomm implementer (0x51) part lookups.
    ///
    /// References:
    /// - util-linux: https://github.com/util-linux/util-linux/blob/master/sys-utils/lscpu-arm.c
    /// - pytorch/cpuinfo: https://github.com/pytorch/cpuinfo/blob/main/src/arm/uarch.c
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
                MicroArch::ArmCortexA73,
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
        Self::find_impl(part, Implementer::Qualcomm, PARTS)
    }

    /// Samsung implementer (0x53) part lookups.
    fn find_samsung(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x001,
                "Exynos 8890",
                MicroArch::ExynosM1,
                "Exynos M1",
                Some(N14),
            ),
            (
                0x002,
                "Exynos 9810",
                MicroArch::ExynosM3,
                "Exynos M3",
                Some(N10),
            ),
            (
                0x003,
                "Exynos 9820",
                MicroArch::ExynosM4,
                "Exynos M4",
                Some(N8),
            ),
            (
                0x004,
                "Exynos 990",
                MicroArch::ExynosM5,
                "Exynos M5",
                Some(N7),
            ),
        ];
        Self::find_impl(part, Implementer::Samsung, PARTS)
    }

    /// Nvidia implementer (0x4E) part lookups.
    fn find_nvidia(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x000,
                "Tegra K1 / Denver",
                MicroArch::NvidiaDenver,
                "Denver",
                Some(N28),
            ),
            (
                0x003,
                "Tegra X2 / Denver 2",
                MicroArch::NvidiaDenver2,
                "Denver 2",
                Some(N16),
            ),
            (
                0x004,
                "Jetson Xavier / Carmel",
                MicroArch::NvidiaCarmel,
                "Carmel",
                Some(N12),
            ),
        ];
        Self::find_impl(part, Implementer::Nvidia, PARTS)
    }

    /// Ampere Computing implementer (0xC0) part lookups.
    fn find_ampere(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0xAC3,
                "AmpereOne",
                MicroArch::AmpereOne,
                "AmpereOne",
                Some(N7),
            ),
            (
                0xAC4,
                "AmpereOne-1a",
                MicroArch::AmpereOne,
                "AmpereOne-1a",
                Some(N5),
            ),
        ];
        Self::find_impl(part, Implementer::Ampere, PARTS)
    }

    /// HiSilicon implementer (0x48) part lookups.
    fn find_hisilicon(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0xD01,
                "Kunpeng 920",
                MicroArch::Kunpeng920,
                "TaiShan v110",
                Some(N7),
            ),
            (
                0xD02,
                "Kunpeng 920",
                MicroArch::Kunpeng920,
                "TaiShan v110",
                Some(N7),
            ),
            (
                0xD03,
                "Kunpeng 920",
                MicroArch::Kunpeng920,
                "TaiShan v110",
                Some(N7),
            ),
            (
                0xD06,
                "Kunpeng 950",
                MicroArch::Kunpeng950,
                "TaiShan v120",
                None,
            ),
        ];
        Self::find_impl(part, Implementer::HiSilicon, PARTS)
    }

    /// Fujitsu implementer (0x46) part lookups.
    fn find_fujitsu(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] =
            &[(0x001, "A64FX", MicroArch::FujitsuA64FX, "A64FX", Some(N7))];
        Self::find_impl(part, Implementer::Fujitsu, PARTS)
    }

    /// Broadcom implementer (0x42) part lookups.
    fn find_broadcom(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (0x00F, "Brahma B15", MicroArch::BrahmaB15, "B15", None),
            (0x100, "Brahma B53", MicroArch::BrahmaB53, "B53", None),
            (
                0x516,
                "ThunderX2",
                MicroArch::ThunderX2,
                "Vulcan",
                Some(N16),
            ),
        ];
        Self::find_impl(part, Implementer::Broadcom, PARTS)
    }

    /// Cavium implementer (0x43) part lookups.
    fn find_cavium(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x0A0,
                "ThunderX",
                MicroArch::ThunderX,
                "ThunderX",
                Some(N28),
            ),
            (
                0x0AF,
                "ThunderX2",
                MicroArch::ThunderX2,
                "ThunderX2",
                Some(N16),
            ),
            (
                0x0B0,
                "OcteonTX2",
                MicroArch::OcteonTX2,
                "OcteonTX2",
                Some(N7),
            ),
        ];
        Self::find_impl(part, Implementer::Cavium, PARTS)
    }

    /// Phytium implementer (0x70) part lookups.
    fn find_phytium(part: usize) -> Self {
        const PARTS: &[(usize, &str, MicroArch, &str, Option<&str>)] = &[
            (
                0x303,
                "Phytium FTC310",
                MicroArch::PhytiumFTC,
                "FTC310",
                None,
            ),
            (
                0x660,
                "Phytium FTC660",
                MicroArch::PhytiumFTC,
                "FTC660",
                None,
            ),
            (
                0x661,
                "Phytium FTC661",
                MicroArch::PhytiumFTC,
                "FTC661",
                None,
            ),
            (
                0x662,
                "Phytium FTC662",
                MicroArch::PhytiumFTC,
                "FTC662",
                None,
            ),
            (
                0x663,
                "Phytium FTC663",
                MicroArch::PhytiumFTC,
                "FTC663",
                None,
            ),
            (
                0x664,
                "Phytium FTC664",
                MicroArch::PhytiumFTC,
                "FTC664",
                None,
            ),
            (
                0x862,
                "Phytium FTC862",
                MicroArch::PhytiumFTC,
                "FTC862",
                None,
            ),
        ];
        Self::find_impl(part, Implementer::Phytium, PARTS)
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
        let cpu_e = CpuArch::find(IMPL_APPLE, 0x042, 0x0);
        assert_eq!(cpu_e.model.as_str(), "Apple M3");
        assert_eq!(cpu_e.micro_arch, MicroArch::AppleSawtooth);

        let cpu_p = CpuArch::find(IMPL_APPLE, 0x043, 0x0);
        assert_eq!(cpu_p.model.as_str(), "Apple M3");
        assert_eq!(cpu_p.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_m4_find() {
        let cpu_e = CpuArch::find(IMPL_APPLE, 0x052, 0x0);
        assert_eq!(cpu_e.model.as_str(), "Apple M4");
        assert_eq!(cpu_e.micro_arch, MicroArch::AppleSawtooth);

        let cpu_p = CpuArch::find(IMPL_APPLE, 0x053, 0x0);
        assert_eq!(cpu_p.model.as_str(), "Apple M4");
        assert_eq!(cpu_p.micro_arch, MicroArch::AppleEverest);
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
        let cpu = CpuArch::find(IMPL_ARM, 0xD0B, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A76");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA76);
    }

    #[test]
    fn test_arm_cortex_a73_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD09, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A73");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA73);
    }

    #[test]
    fn test_arm_cortex_a75_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD0A, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A75");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA75);
    }

    #[test]
    fn test_arm_cortex_a77_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD0D, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A77");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA77);
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
        let cpu = CpuArch::find(IMPL_ARM, 0xD44, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-X1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexX1);
    }

    #[test]
    fn test_arm_neoverse_n1_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD0C, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Neoverse N1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmNeoverseN1);
    }

    #[test]
    fn test_arm_neoverse_v1_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD40, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Neoverse V1");
        assert_eq!(cpu.micro_arch, MicroArch::ArmNeoverseV1);
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
    fn test_arm_cortex_a65_find() {
        let cpu = CpuArch::find(IMPL_ARM, 0xD06, 0x0);
        assert_eq!(cpu.model.as_str(), "ARM Cortex-A65");
        assert_eq!(cpu.micro_arch, MicroArch::ArmCortexA65);
    }

    #[test]
    fn test_qualcomm_oryon_find() {
        let cpu = CpuArch::find(IMPL_QUALCOMM, 0x001, 0x0);
        assert_eq!(cpu.model.as_str(), "Snapdragon X Elite");
        assert_eq!(cpu.micro_arch, MicroArch::QCOryon);
    }

    #[test]
    fn test_ampere_one_find() {
        let cpu = CpuArch::find(IMPL_AMPERE, 0xAC3, 0x0);
        assert_eq!(cpu.model.as_str(), "AmpereOne");
        assert_eq!(cpu.micro_arch, MicroArch::AmpereOne);
    }

    #[test]
    fn test_hisilicon_kunpeng920_find() {
        let cpu = CpuArch::find(IMPL_HISILICON, 0xD01, 0x0);
        assert_eq!(cpu.model.as_str(), "Kunpeng 920");
        assert_eq!(cpu.micro_arch, MicroArch::Kunpeng920);
    }

    #[test]
    fn test_samsung_exynos_m1_find() {
        let cpu = CpuArch::find(IMPL_SAMSUNG, 0x001, 0x0);
        assert_eq!(cpu.model.as_str(), "Exynos 8890");
        assert_eq!(cpu.micro_arch, MicroArch::ExynosM1);
    }
}
