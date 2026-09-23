//! Motorola 680x0 microarchitecture and CPU model identification.

use crate::common::constants::*;
use crate::common::UNK;

pub type CpuCore = crate::common::CpuCore<MicroArch>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    Unknown,

    // Generation 1 (Original 68000 Family)
    M68000,
    M68008,
    M68010,

    // Generation 2 (32-bit & Cache)
    M68020,
    M68EC020,

    // Generation 3 (On-chip MMU & Dual Cache)
    M68030,
    M68EC030,

    // Generation 4 (Integrated Pipelined Architecture)
    M68040,
    M68LC040,
    M68EC040,

    // Generation 5 (Superscalar Architecture)
    M68060,
    M68LC060,
    M68EC060,

    // ColdFire Architecture
    ColdFireV1,
    ColdFireV2,
    ColdFireV3,
    ColdFireV4,
}

impl From<MicroArch> for &'static str {
    fn from(ma: MicroArch) -> &'static str {
        match ma {
            MicroArch::Unknown => UNK,
            MicroArch::M68000 => "Motorola 68000",
            MicroArch::M68008 => "Motorola 68008",
            MicroArch::M68010 => "Motorola 68010",
            MicroArch::M68020 => "Motorola 68020",
            MicroArch::M68EC020 => "Motorola 68EC020",
            MicroArch::M68030 => "Motorola 68030",
            MicroArch::M68EC030 => "Motorola 68EC030",
            MicroArch::M68040 => "Motorola 68040",
            MicroArch::M68LC040 => "Motorola 68LC040",
            MicroArch::M68EC040 => "Motorola 68EC040",
            MicroArch::M68060 => "Motorola 68060",
            MicroArch::M68LC060 => "Motorola 68LC060",
            MicroArch::M68EC060 => "Motorola 68EC060",
            MicroArch::ColdFireV1 => "ColdFire V1",
            MicroArch::ColdFireV2 => "ColdFire V2",
            MicroArch::ColdFireV3 => "ColdFire V3",
            MicroArch::ColdFireV4 => "ColdFire V4",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FpuType {
    #[default]
    None,
    M68881,
    M68882,
    Integrated040,
    Integrated060,
}

impl From<FpuType> for &'static str {
    fn from(fpu: FpuType) -> &'static str {
        match fpu {
            FpuType::None => "None",
            FpuType::M68881 => "Motorola 68881",
            FpuType::M68882 => "Motorola 68882",
            FpuType::Integrated040 => "Integrated 68040 FPU",
            FpuType::Integrated060 => "Integrated 68060 FPU",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MmuType {
    #[default]
    None,
    Amu,
    M68851,
    Integrated030,
    Integrated040,
    Integrated060,
}

impl From<MmuType> for &'static str {
    fn from(mmu: MmuType) -> &'static str {
        match mmu {
            MmuType::None => "None",
            MmuType::Amu => "Apple AMU",
            MmuType::M68851 => "Motorola 68851 PMMU",
            MmuType::Integrated030 => "Integrated 68030 MMU",
            MmuType::Integrated040 => "Integrated 68040 MMU",
            MmuType::Integrated060 => "Integrated 68060 MMU",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub marketing_name: &'static str,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub cpu_id: u32,
    pub technology: Option<&'static str>,
    pub default_fpu: FpuType,
    pub default_mmu: MmuType,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(UNK, MicroArch::Unknown, UNK, 0, None, FpuType::None, MmuType::None)
    }
}

impl CpuArch {
    pub const fn new(
        marketing_name: &'static str,
        micro_arch: MicroArch,
        code_name: &'static str,
        cpu_id: u32,
        technology: Option<&'static str>,
        default_fpu: FpuType,
        default_mmu: MmuType,
    ) -> Self {
        CpuArch {
            marketing_name,
            micro_arch,
            code_name,
            cpu_id,
            technology,
            default_fpu,
            default_mmu,
        }
    }

    pub fn find_by_gestalt(cpu_type: u32, fpu_type: u32, mmu_type: u32) -> Self {
        let fpu = match fpu_type {
            1 => FpuType::M68881,
            2 => FpuType::M68882,
            3 => FpuType::Integrated040,
            _ => FpuType::None,
        };

        let mmu = match mmu_type {
            1 => MmuType::Amu,
            2 => MmuType::M68851,
            3 => MmuType::Integrated030,
            4 => MmuType::Integrated040,
            _ => MmuType::None,
        };

        match cpu_type {
            1 => Self::new("MC68000", MicroArch::M68000, "68000", 1, Some("3.5μm"), fpu, mmu),
            2 => Self::new("MC68010", MicroArch::M68010, "68010", 2, Some("3.0μm"), fpu, mmu),
            3 => Self::new("MC68020", MicroArch::M68020, "68020", 3, Some("1.5μm"), fpu, mmu),
            4 => {
                let mmu_actual = if mmu == MmuType::None {
                    MmuType::Integrated030
                } else {
                    mmu
                };
                Self::new(
                    "MC68030",
                    MicroArch::M68030,
                    "68030",
                    4,
                    Some("0.8μm"),
                    fpu,
                    mmu_actual,
                )
            }
            5 => {
                if fpu == FpuType::None {
                    Self::new(
                        "MC68LC040",
                        MicroArch::M68LC040,
                        "68LC040",
                        5,
                        Some("0.65μm"),
                        FpuType::None,
                        MmuType::Integrated040,
                    )
                } else {
                    Self::new(
                        "MC68040",
                        MicroArch::M68040,
                        "68040",
                        5,
                        Some("0.65μm"),
                        FpuType::Integrated040,
                        MmuType::Integrated040,
                    )
                }
            }
            6 => {
                if fpu == FpuType::None {
                    Self::new(
                        "MC68LC060",
                        MicroArch::M68LC060,
                        "68LC060",
                        6,
                        Some(N350),
                        FpuType::None,
                        MmuType::Integrated060,
                    )
                } else {
                    Self::new(
                        "MC68060",
                        MicroArch::M68060,
                        "68060",
                        6,
                        Some(N350),
                        FpuType::Integrated060,
                        MmuType::Integrated060,
                    )
                }
            }
            _ => Self::default(),
        }
    }
}
