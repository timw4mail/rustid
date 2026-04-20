use crate::arm::CoreType;
use crate::arm::brand::*;
use crate::common::constants::*;
use crate::common::{Cache, CacheLevel, CacheType, Level1Cache};

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

pub const UNK: &str = "Unknown";
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

    pub fn cache(&self) -> Option<Cache> {
        match self {
            MicroArch::Unknown => None,

            MicroArch::AppleFirestorm => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(131072, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(192 * 1024, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(12 * 1024 * 1024, CacheType::Unified, 8, 8)),
                l3: Some(CacheLevel::new(24 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::AppleIcestorm => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(131072, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::AppleAvalanche => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(131072, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(131072, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(16 * 1024 * 1024, CacheType::Unified, 8, 8)),
                l3: Some(CacheLevel::new(32 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::AppleBlizzard => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::AppleEverest => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(131072, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(131072, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(16 * 1024 * 1024, CacheType::Unified, 8, 8)),
                l3: Some(CacheLevel::new(32 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::AppleSawtooth => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA7 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA8 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(4096, CacheType::Data, 4, 1),
                    instruction: CacheLevel::new(4096, CacheType::Instruction, 4, 1),
                },
                l2: None,
                l3: None,
            }),

            MicroArch::ArmCortexA9 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA12 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA15 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(2 * 1024 * 1024, CacheType::Unified, 8, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA17 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA32 | MicroArch::ArmCortexA35 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA53 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA55 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA65 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(256 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA72 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA73 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: None,
            }),

            MicroArch::ArmCortexA75 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA76 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmCortexA77 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmCortexA78 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA510 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4, 4)),
                l3: Some(CacheLevel::new(2 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA520 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4, 4)),
                l3: Some(CacheLevel::new(2 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA710 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA715 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA720 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexA725 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::ArmCortexX1 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmCortexX2 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmCortexX3 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmCortexX4 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmNeoverseE1 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::ArmNeoverseN1 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmNeoverseN2 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmNeoverseV1 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 16, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::ArmNeoverseV2 => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 16, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 0)),
            }),

            MicroArch::QCScorpion => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(4096, CacheType::Data, 4, 1),
                    instruction: CacheLevel::new(4096, CacheType::Instruction, 4, 1),
                },
                l2: None,
                l3: None,
            }),

            MicroArch::QCKrait => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(4096, CacheType::Data, 4, 1),
                    instruction: CacheLevel::new(4096, CacheType::Instruction, 4, 1),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::QCKryo => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4, 4)),
                l3: None,
            }),

            MicroArch::QCFalkor => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(32768, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(32768, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: None,
            }),

            MicroArch::QCSaphira => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 4, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 4, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(4 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),

            MicroArch::QCOryon => Some(Cache {
                l1: Level1Cache::Split {
                    data: CacheLevel::new(65536, CacheType::Data, 8, 4),
                    instruction: CacheLevel::new(65536, CacheType::Instruction, 8, 4),
                },
                l2: Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8, 4)),
                l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 8, 0)),
            }),
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
            IMPL_QUALCOMM => Self::find_qualcomm(part),
            _ => Self {
                implementer: Implementer::from(implementer),
                ..Self::default()
            },
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

            // Cortex-A510 series
            0xD46 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A510",
                MicroArch::ArmCortexA510,
                "Cortex-A510",
                0xD46,
                None,
            ),

            // Cortex-A520 series
            0xD80 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A520",
                MicroArch::ArmCortexA520,
                "Cortex-A520",
                0xD80,
                None,
            ),

            // Cortex-A710 series
            0xD47 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A710",
                MicroArch::ArmCortexA710,
                "Cortex-A710",
                0xD47,
                None,
            ),

            // Cortex-A715 series
            0xD4D => Self::new(
                Implementer::Arm,
                "ARM Cortex-A715",
                MicroArch::ArmCortexA715,
                "Cortex-A715",
                0xD4D,
                None,
            ),

            // Cortex-A720 series
            0xD81 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A720",
                MicroArch::ArmCortexA720,
                "Cortex-A720",
                0xD81,
                None,
            ),

            // Cortex-A725 series
            0xD87 => Self::new(
                Implementer::Arm,
                "ARM Cortex-A725",
                MicroArch::ArmCortexA725,
                "Cortex-A725",
                0xD87,
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
            0xD48 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X2",
                MicroArch::ArmCortexX2,
                "Cortex-X2",
                0xD48,
                None,
            ),

            // Cortex-X3 series
            0xD4E => Self::new(
                Implementer::Arm,
                "ARM Cortex-X3",
                MicroArch::ArmCortexX3,
                "Cortex-X3",
                0xD4E,
                None,
            ),

            // Cortex-X4 series
            0xD82 => Self::new(
                Implementer::Arm,
                "ARM Cortex-X4",
                MicroArch::ArmCortexX4,
                "Cortex-X4",
                0xD82,
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
            0xD49 => Self::new(
                Implementer::Arm,
                "ARM Neoverse N2",
                MicroArch::ArmNeoverseN2,
                "Neoverse N2",
                0xD49,
                None,
            ),

            // Neoverse V1
            0xD44 => Self::new(
                Implementer::Arm,
                "ARM Neoverse V1",
                MicroArch::ArmNeoverseV1,
                "Neoverse V1",
                0xD44,
                None,
            ),

            // Neoverse V2
            0xD4F => Self::new(
                Implementer::Arm,
                "ARM Neoverse V2",
                MicroArch::ArmNeoverseV2,
                "Neoverse V2",
                0xD4F,
                None,
            ),

            _ => Self {
                implementer: Implementer::Arm,
                ..Self::default()
            },
        }
    }

    /// See <https://github.com/freebsd/freebsd-src/blob/main/sys/arm64/include/cpu.h>
    fn find_apple(part: usize) -> Self {
        match part {
            // M1
            0x022 => Self::new(
                Implementer::Apple,
                "Apple M1",
                MicroArch::AppleIcestorm,
                "Tonga",
                0x022,
                Some(N5),
            ),
            0x023 => Self::new(
                Implementer::Apple,
                "Apple M1",
                MicroArch::AppleFirestorm,
                "Tonga",
                0x023,
                Some(N5),
            ),
            0x024 => Self::new(
                Implementer::Apple,
                "Apple M1 Pro",
                MicroArch::AppleIcestorm,
                "Jade Chop",
                0x024,
                Some(N5),
            ),
            0x025 => Self::new(
                Implementer::Apple,
                "Apple M1 Pro",
                MicroArch::AppleFirestorm,
                "Jade Chop",
                0x025,
                Some(N5),
            ),
            0x028 => Self::new(
                Implementer::Apple,
                "Apple M1 Max",
                MicroArch::AppleIcestorm,
                "Jade 1C",
                0x028,
                Some(N5),
            ),
            0x029 => Self::new(
                Implementer::Apple,
                "Apple M1 Max",
                MicroArch::AppleFirestorm,
                "Jade 1C",
                0x029,
                Some(N5),
            ),

            // M2
            0x32 => Self::new(
                Implementer::Apple,
                "Apple M2",
                MicroArch::AppleBlizzard,
                "Staten",
                0x32,
                Some(N5),
            ),
            0x33 => Self::new(
                Implementer::Apple,
                "Apple M2",
                MicroArch::AppleAvalanche,
                "Staten",
                0x33,
                Some(N5),
            ),
            0x34 => Self::new(
                Implementer::Apple,
                "Apple M2 Pro",
                MicroArch::AppleBlizzard,
                "Rhodes Chop",
                0x34,
                Some(N5),
            ),
            0x35 => Self::new(
                Implementer::Apple,
                "Apple M2 Pro",
                MicroArch::AppleAvalanche,
                "Rhodes Chop",
                0x35,
                Some(N5),
            ),
            0x38 => Self::new(
                Implementer::Apple,
                "Apple M2 Max",
                MicroArch::AppleBlizzard,
                "Rhodes 1C",
                0x38,
                Some(N5),
            ),
            0x39 => Self::new(
                Implementer::Apple,
                "Apple M2 Max",
                MicroArch::AppleAvalanche,
                "Rhodes 1C",
                0x39,
                Some(N5),
            ),

            // M3
            0x42 => Self::new(
                Implementer::Apple,
                "Apple M3",
                MicroArch::AppleEverest,
                "Ibiza",
                0x42,
                Some(N3),
            ),
            0x43 => Self::new(
                Implementer::Apple,
                "Apple M3",
                MicroArch::AppleSawtooth,
                "Ibiza",
                0x43,
                Some(N3),
            ),
            0x44 => Self::new(
                Implementer::Apple,
                "Apple M3 Pro",
                MicroArch::AppleEverest,
                "Lobos",
                0x44,
                Some(N3),
            ),
            0x45 => Self::new(
                Implementer::Apple,
                "Apple M3 Pro",
                MicroArch::AppleSawtooth,
                "Lobos",
                0x45,
                Some(N3),
            ),
            0x48 => Self::new(
                Implementer::Apple,
                "Apple M3 Max",
                MicroArch::AppleEverest,
                "Palma",
                0x48,
                Some(N3),
            ),
            0x49 => Self::new(
                Implementer::Apple,
                "Apple M3 Max",
                MicroArch::AppleSawtooth,
                "Palma",
                0x49,
                Some(N3),
            ),

            // M4
            0x052 => Self::new(
                Implementer::Apple,
                "Apple M4",
                MicroArch::AppleEverest,
                "Donan",
                0x052,
                Some(N3),
            ),
            0x053 => Self::new(
                Implementer::Apple,
                "Apple M4",
                MicroArch::AppleSawtooth,
                "Donan",
                0x053,
                Some(N3),
            ),
            0x54 => Self::new(
                Implementer::Apple,
                "Apple M4 Pro",
                MicroArch::AppleEverest,
                "Brava Chop",
                0x54,
                Some(N3),
            ),
            0x55 => Self::new(
                Implementer::Apple,
                "Apple M4 Pro",
                MicroArch::AppleSawtooth,
                "Brava Chop",
                0x55,
                Some(N3),
            ),
            0x58 => Self::new(
                Implementer::Apple,
                "Apple M4 Max",
                MicroArch::AppleEverest,
                "Brava",
                0x58,
                Some(N3),
            ),
            0x59 => Self::new(
                Implementer::Apple,
                "Apple M4 Max",
                MicroArch::AppleSawtooth,
                "Brava",
                0x59,
                Some(N3),
            ),

            // A18 Pro
            0x101 => Self::new(
                Implementer::Apple,
                "Apple A18 Pro",
                MicroArch::AppleEverest,
                "Tahiti",
                0x101,
                Some(N3),
            ),

            _ => Self {
                implementer: Implementer::Apple,
                ..Self::default()
            },
        }
    }

    fn find_qualcomm(part: usize) -> Self {
        match part {
            0x001 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon X Elite",
                MicroArch::QCOryon,
                "Oryon",
                0x001,
                Some(N4),
            ),
            0x00F => Self::new(
                Implementer::Qualcomm,
                "Snapdragon S1/S2/S3",
                MicroArch::QCScorpion,
                "Scorpion",
                0x00F,
                Some("65-45nm"),
            ),
            0x02D => Self::new(
                Implementer::Qualcomm,
                "Snapdragon S4",
                MicroArch::QCScorpion,
                "Scorpion",
                0x02D,
                Some(N28),
            ),
            0x04D => Self::new(
                Implementer::Qualcomm,
                "Snapdragon S4 Plus/Pro",
                MicroArch::QCKrait,
                "Krait",
                0x04D,
                Some(N28),
            ),
            0x06F => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 800/801",
                MicroArch::QCKrait,
                "Krait 400",
                0x06F,
                Some(N28),
            ),
            0x201 | 0x205 | 0x211 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 820/821",
                MicroArch::QCKryo,
                "Kryo",
                part,
                Some(N14),
            ),
            0x800 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 835",
                MicroArch::QCFalkor,
                "Kryo 280 Gold",
                0x800,
                Some(N10),
            ),
            0x801 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 835",
                MicroArch::ArmCortexA53,
                "Kryo 280 Silver",
                0x801,
                Some(N10),
            ),
            0x802 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 845",
                MicroArch::ArmCortexA75,
                "Kryo 385 Gold",
                0x802,
                Some(N10),
            ),
            0x803 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 845",
                MicroArch::ArmCortexA55,
                "Kryo 385 Silver",
                0x803,
                Some(N10),
            ),
            0x804 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 855",
                MicroArch::ArmCortexA76,
                "Kryo 485 Gold",
                0x804,
                Some(N7),
            ),
            0x805 => Self::new(
                Implementer::Qualcomm,
                "Snapdragon 855",
                MicroArch::ArmCortexA55,
                "Kryo 485 Silver",
                0x805,
                Some(N7),
            ),
            0xC00 => Self::new(
                Implementer::Qualcomm,
                "Centriq 2400",
                MicroArch::QCFalkor,
                "Falkor",
                0xC00,
                Some(N10),
            ),
            0xC01 => Self::new(
                Implementer::Qualcomm,
                "Qualcomm Saphira",
                MicroArch::QCSaphira,
                "Saphira",
                0xC01,
                None,
            ),
            _ => Self {
                implementer: Implementer::Qualcomm,
                ..Self::default()
            },
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
    fn test_midr_parsing_m1() {
        let midr = Midr::new(0x611F0231);
        assert_eq!(midr.implementer, 0x61);
        assert_eq!(midr.variant, 0x1);
        assert_eq!(midr.architecture, 0xF);
        assert_eq!(midr.part, 0x023);
        assert_eq!(midr.revision, 0x1);
    }

    #[test]
    fn test_apple_m1_find() {
        let cpu = CpuArch::find(0x61, 0x022, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1");
        assert_eq!(cpu.micro_arch, MicroArch::AppleIcestorm);
    }

    #[test]
    fn test_apple_m1_pro_find() {
        let cpu = CpuArch::find(0x61, 0x024, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M1 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleIcestorm);
    }

    #[test]
    fn test_apple_m2_find() {
        let cpu = CpuArch::find(0x61, 0x032, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M2");
        assert_eq!(cpu.micro_arch, MicroArch::AppleBlizzard);
    }

    #[test]
    fn test_apple_m3_find() {
        let cpu = CpuArch::find(0x61, 0x042, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M3");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_m4_find() {
        let cpu = CpuArch::find(0x61, 0x052, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple M4");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
    }

    #[test]
    fn test_apple_a18_pro_find() {
        let cpu = CpuArch::find(0x61, 0x101, 0x0);
        assert_eq!(cpu.model.as_str(), "Apple A18 Pro");
        assert_eq!(cpu.micro_arch, MicroArch::AppleEverest);
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

    #[test]
    fn test_qualcomm_oryon_find() {
        let cpu = CpuArch::find(0x51, 0x001, 0x0);
        assert_eq!(cpu.model.as_str(), "Snapdragon X Elite");
        assert_eq!(cpu.micro_arch, MicroArch::QCOryon);
    }
}
