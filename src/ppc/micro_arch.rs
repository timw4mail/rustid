use crate::common::constants::*;
use crate::common::{Cache, CoreType, Speed};

#[derive(Debug, Clone, PartialEq)]
pub struct CpuCore {
    /// Classification of this core (Performance, Efficiency, Super)
    pub kind: CoreType,
    /// Microarchitecture variant of this core
    pub micro_arch: MicroArch,
    /// Human-readable marketing / core codename (e.g. "PowerPC 750 (G3)")
    pub name: Option<String>,
    /// Cache hierarchy for this core cluster
    pub cache: Option<Cache>,
    /// Clock speed (base and boost frequencies in MHz)
    pub speed: Option<Speed>,
    /// Physical core count
    pub count: u32,
    /// Logical thread count (e.g., taking SMT into account)
    pub threads: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroArch {
    Unknown,

    // IBM/Motorola Classic PowerPC
    Ppc601,
    Ppc603,
    Ppc603e,
    Ppc603ev,
    Ppc604,
    Ppc604e,
    Ppc604r,
    Ppc620,

    // IBM/Motorola G3/G4
    Ppc750,
    Ppc7400,
    Ppc7410,
    Ppc7447,
    Ppc7447a,
    Ppc7450,
    Ppc7455,
    Ppc7457,
    Ppc7460,
    Ppc7448,
    Ppc970,

    // IBM G5
    Ppc970fx,
}

impl From<MicroArch> for &'static str {
    fn from(ma: MicroArch) -> &'static str {
        match ma {
            MicroArch::Unknown => UNK,
            MicroArch::Ppc601 => "PowerPC 601",
            MicroArch::Ppc603 => "PowerPC 603",
            MicroArch::Ppc603e => "PowerPC 603e",
            MicroArch::Ppc603ev => "PowerPC 603eV",
            MicroArch::Ppc604 => "PowerPC 604",
            MicroArch::Ppc604e => "PowerPC 604e",
            MicroArch::Ppc604r => "PowerPC 604r",
            MicroArch::Ppc620 => "PowerPC 620",
            MicroArch::Ppc750 => "PowerPC 750 (G3)",
            MicroArch::Ppc7400 => "PowerPC 7400 (G4)",
            MicroArch::Ppc7410 => "PowerPC 7410 (G4)",
            MicroArch::Ppc7447 => "PowerPC 7447 (G4)",
            MicroArch::Ppc7447a => "PowerPC 7447A (G4)",
            MicroArch::Ppc7450 => "PowerPC 7450 (G4)",
            MicroArch::Ppc7455 => "PowerPC 7455 (G4)",
            MicroArch::Ppc7457 => "PowerPC 7457 (G4)",
            MicroArch::Ppc7460 => "PowerPC 7460 (G4)",
            MicroArch::Ppc7448 => "PowerPC 7448 (G4)",
            MicroArch::Ppc970 => "PowerPC 970 (G5)",
            MicroArch::Ppc970fx => "PowerPC 970FX (G5)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub marketing_name: &'static str,
    pub micro_arch: MicroArch,
    pub code_name: &'static str,
    pub pvr_version: u16,
    pub technology: Option<&'static str>,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::new(UNK, MicroArch::Unknown, UNK, 0, None)
    }
}

impl CpuArch {
    pub fn new(
        marketing_name: &'static str,
        micro_arch: MicroArch,
        code_name: &'static str,
        pvr_version: u16,
        technology: Option<&'static str>,
    ) -> Self {
        CpuArch {
            marketing_name,
            micro_arch,
            code_name,
            pvr_version,
            technology,
        }
    }

    pub fn find(pvr: u32) -> Self {
        let version = (pvr >> 16) as u16;
        let revision = (pvr & 0xFFFF) as u16;

        // See: <https://raw.githubusercontent.com/tycho/pearpc/master/doc/powerpc_pvr_list.txt>
        match version {
            // IBM/Motorola PowerPC
            0x0001 => Self::new("PowerPC 601", MicroArch::Ppc601, "601", 0x01, Some("0.6μm")),
            0x0003 => Self::new("PowerPC 603", MicroArch::Ppc603, "603", 0x03, Some("0.5μm")),
            0x0004 => Self::new("PowerPC 603e", MicroArch::Ppc603e, "603e", 0x04, Some(N350)),
            0x0006 => Self::new(
                "PowerPC 603eV",
                MicroArch::Ppc603ev,
                "603eV",
                0x06,
                Some(N250),
            ),
            0x0007 => Self::new("PowerPC 604", MicroArch::Ppc604, "604", 0x07, Some(N350)),
            0x0009 => Self::new("PowerPC 604e", MicroArch::Ppc604e, "604e", 0x09, Some(N250)),
            0x000A => Self::new("PowerPC 604r", MicroArch::Ppc604r, "604r", 0x0A, Some(N250)),
            0x0013 => Self::new("PowerPC 620", MicroArch::Ppc620, "620", 0x13, Some(N350)),

            // PowerPC 750 (G3)
            0x0008 => match revision {
                0x0201 | 0x2201 => {
                    Self::new("PowerPC 750CX", MicroArch::Ppc750, "G3", 0x201, Some(N180))
                }
                0x0202 | 0x2202 => {
                    Self::new("PowerPC 750CXe", MicroArch::Ppc750, "G3", 0x202, Some(N180))
                }
                0x0205 => Self::new("PowerPC 750L", MicroArch::Ppc750, "G3", 0x205, Some(N180)),
                _ => Self::new(
                    "PowerPC 750",
                    MicroArch::Ppc750,
                    "Arthur",
                    0x200,
                    Some(N260),
                ),
            },

            0x7000 => match revision {
                0x0204 => Self::new("PowerPC 750GX", MicroArch::Ppc750, "G3", 0x204, Some(N90)),
                _ => Self::new("PowerPC 750FX", MicroArch::Ppc750, "G3", 0x203, Some(N180)),
            },

            // PowerPC 7400 (G4)
            0x000C => match revision {
                0x0309 => Self::new(
                    "PowerPC 7410",
                    MicroArch::Ppc7410,
                    "Nitro",
                    0x309,
                    Some(N180),
                ),
                _ => Self::new("PowerPC 7400", MicroArch::Ppc7400, "Max", 0x308, Some(N220)),
            },

            // PowerPC 7450 / 7455 / 7457 / 7447 / 7447A / 7460 (G4)
            0x8000 => Self::new("PowerPC 7450", MicroArch::Ppc7450, "Max", 0x351, Some(N180)),
            0x8001 => Self::new(
                "PowerPC 7455",
                MicroArch::Ppc7455,
                "Apollo 6",
                0x352,
                Some(N150),
            ),
            0x8002 => Self::new(
                "PowerPC 7457",
                MicroArch::Ppc7457,
                "Apollo 7",
                0x353,
                Some(N130),
            ),
            0x8003 => match revision {
                0x030C => Self::new(
                    "PowerPC 7447",
                    MicroArch::Ppc7447,
                    "Apollo 7",
                    0x30C,
                    Some(N130),
                ),
                0x030D => Self::new(
                    "PowerPC 7447A",
                    MicroArch::Ppc7447a,
                    "Apollo 7",
                    0x30D,
                    Some(N90),
                ),
                _ => Self::new(
                    "PowerPC 7460",
                    MicroArch::Ppc7460,
                    "Apollo Pro",
                    0x354,
                    Some(N130),
                ),
            },
            0x8004 => Self::new(
                "PowerPC 7448",
                MicroArch::Ppc7448,
                "Apollo 8",
                0x8004,
                Some(N90),
            ),

            // PowerPC 970 / G5
            0x0039 => Self::new("PowerPC 970", MicroArch::Ppc970, "G5", 0x39, Some(N150)),
            0x003C => Self::new("PowerPC 970FX", MicroArch::Ppc970fx, "G5", 0x3C, Some(N90)),
            0x0044 => Self::new("PowerPC 970MP", MicroArch::Ppc970, "G5", 0x44, Some(N90)),

            // Apple PowerPC variants (based on IBM 7400/7410)
            // Apple uses version 0x0033 for some G4 chips
            0x0033 => Self::new("Apple G4", MicroArch::Ppc7400, "Max", 0x33, Some(N180)),

            // Apple G5 variants
            0x0045 => Self::new("Apple G5", MicroArch::Ppc970, "G5", 0x45, Some(N65)),
            0x0052 => Self::new("Apple G5", MicroArch::Ppc970fx, "G5", 0x52, Some(N65)),

            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppc_find_classic() {
        let cpu = CpuArch::find(0x0001_0002);
        assert_eq!(cpu.marketing_name, "PowerPC 601");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc601);

        let cpu = CpuArch::find(0x0003_0100);
        assert_eq!(cpu.marketing_name, "PowerPC 603");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc603);

        let cpu = CpuArch::find(0x0004_0000);
        assert_eq!(cpu.marketing_name, "PowerPC 603e");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc603e);

        let cpu = CpuArch::find(0x0007_0000);
        assert_eq!(cpu.marketing_name, "PowerPC 604");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc604);
    }

    #[test]
    fn test_ppc_find_g3() {
        let cpu = CpuArch::find(0x0008_0200);
        assert_eq!(cpu.marketing_name, "PowerPC 750");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);
        assert_eq!(cpu.code_name, "Arthur");

        let cpu = CpuArch::find(0x0008_0201);
        assert_eq!(cpu.marketing_name, "PowerPC 750CX");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);
        assert_eq!(cpu.code_name, "G3");

        let cpu = CpuArch::find(0x0008_0202);
        assert_eq!(cpu.marketing_name, "PowerPC 750CXe");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);

        let cpu = CpuArch::find(0x7000_0203);
        assert_eq!(cpu.marketing_name, "PowerPC 750FX");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);

        let cpu = CpuArch::find(0x7000_0204);
        assert_eq!(cpu.marketing_name, "PowerPC 750GX");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);
    }

    #[test]
    fn test_ppc_find_g4() {
        let cpu = CpuArch::find(0x000C_0209);
        assert_eq!(cpu.marketing_name, "PowerPC 7400");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7400);

        let cpu = CpuArch::find(0x000C_0309);
        assert_eq!(cpu.marketing_name, "PowerPC 7410");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7410);

        let cpu = CpuArch::find(0x8000_0351);
        assert_eq!(cpu.marketing_name, "PowerPC 7450");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7450);

        let cpu = CpuArch::find(0x8001_0352);
        assert_eq!(cpu.marketing_name, "PowerPC 7455");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7455);

        let cpu = CpuArch::find(0x8002_0353);
        assert_eq!(cpu.marketing_name, "PowerPC 7457");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7457);

        let cpu = CpuArch::find(0x8003_030C);
        assert_eq!(cpu.marketing_name, "PowerPC 7447");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7447);

        let cpu = CpuArch::find(0x8003_030D);
        assert_eq!(cpu.marketing_name, "PowerPC 7447A");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7447a);

        let cpu = CpuArch::find(0x8003_0354);
        assert_eq!(cpu.marketing_name, "PowerPC 7460");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7460);

        let cpu = CpuArch::find(0x8004_0201);
        assert_eq!(cpu.marketing_name, "PowerPC 7448");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7448);
    }

    #[test]
    fn test_ppc_find_g5() {
        let cpu = CpuArch::find(0x0039_0202);
        assert_eq!(cpu.marketing_name, "PowerPC 970");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc970);
    }

    #[test]
    fn test_ppc_core_and_topology() {
        use super::super::cpu::Cpu;
        use crate::common::{CoreType, DataSource, Speed};

        let core = CpuCore {
            kind: CoreType::Performance,
            micro_arch: MicroArch::Ppc970,
            name: Some("PowerPC 970".to_string()),
            cache: None,
            speed: Some(Speed {
                base: 2000,
                boost: 2000,
                measured: false,
            }),
            count: 2,
            threads: 2,
        };

        let cpu = Cpu {
            system: None,
            pvr: 0x0039_0202,
            version: 0x0039,
            revision: 0x0202,
            cpu_arch: CpuArch::find(0x0039_0202),
            cores: vec![core],
            clock_speed_source: DataSource::DefaultValue,
        };

        assert!(!cpu.is_hybrid());
        assert_eq!(cpu.total_cores(), 2);
        assert_eq!(cpu.total_threads(), 2);
    }
}
