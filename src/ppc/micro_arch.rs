use crate::common::constants::*;

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
    Ppc7445,
    Ppc7447,
    Ppc7447a,
    Ppc7448,
    Ppc7450,
    Ppc7451,
    Ppc7455,
    Ppc7457,
    Ppc7460,
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
            MicroArch::Ppc7445 => "PowerPC 7445 (G4)",
            MicroArch::Ppc7447 => "PowerPC 7447 (G4)",
            MicroArch::Ppc7447a => "PowerPC 7447A (G4)",
            MicroArch::Ppc7448 => "PowerPC 7448 (G4)",
            MicroArch::Ppc7450 => "PowerPC 7450 (G4)",
            MicroArch::Ppc7451 => "PowerPC 7451 (G4)",
            MicroArch::Ppc7455 => "PowerPC 7455 (G4)",
            MicroArch::Ppc7457 => "PowerPC 7457 (G4)",
            MicroArch::Ppc7460 => "PowerPC 7460 (G4)",
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
        let family = (pvr >> 24) as u8;
        let version = (pvr >> 16) as u16;

        match (family, version) {
            // IBM/Motorola PowerPC (family = 0x00)
            (0x00, 0x0001) => {
                Self::new("PowerPC 601", MicroArch::Ppc601, "601", 0x01, Some("0.6μm"))
            }
            (0x00, 0x0003) => {
                Self::new("PowerPC 603", MicroArch::Ppc603, "603", 0x03, Some("0.5μm"))
            }
            (0x00, 0x0004) => {
                Self::new("PowerPC 603e", MicroArch::Ppc603e, "603e", 0x04, Some(N350))
            }
            (0x00, 0x0006) => Self::new(
                "PowerPC 603eV",
                MicroArch::Ppc603ev,
                "603eV",
                0x06,
                Some(N250),
            ),
            (0x00, 0x0007) => Self::new("PowerPC 604", MicroArch::Ppc604, "604", 0x07, Some(N350)),
            (0x00, 0x0009) => {
                Self::new("PowerPC 604e", MicroArch::Ppc604e, "604e", 0x09, Some(N250))
            }
            (0x00, 0x000A) => {
                Self::new("PowerPC 604r", MicroArch::Ppc604r, "604r", 0x0A, Some(N250))
            }
            (0x00, 0x0013) => Self::new("PowerPC 620", MicroArch::Ppc620, "620", 0x13, Some(N350)),

            // PowerPC 750 (G3)
            (0x00, 0x0200) => Self::new(
                "PowerPC 750",
                MicroArch::Ppc750,
                "Arthur",
                0x200,
                Some(N260),
            ),
            (0x00, 0x0201) => {
                Self::new("PowerPC 750CX", MicroArch::Ppc750, "G3", 0x201, Some(N180))
            }
            (0x00, 0x0202) => {
                Self::new("PowerPC 750CXe", MicroArch::Ppc750, "G3", 0x202, Some(N180))
            }
            (0x00, 0x0203) => {
                Self::new("PowerPC 750FX", MicroArch::Ppc750, "G3", 0x203, Some(N180))
            }
            (0x00, 0x0204) => Self::new("PowerPC 750GX", MicroArch::Ppc750, "G3", 0x204, Some(N90)),
            (0x00, 0x0205) => Self::new("PowerPC 750L", MicroArch::Ppc750, "G3", 0x205, Some(N180)),

            // PowerPC 7400/7410 (G4)
            (0x00, 0x0308) => {
                Self::new("PowerPC 7400", MicroArch::Ppc7400, "Max", 0x308, Some(N220))
            }
            (0x00, 0x0309) => Self::new(
                "PowerPC 7410",
                MicroArch::Ppc7410,
                "Nitro",
                0x309,
                Some(N180),
            ),
            (0x00, 0x030C) => Self::new(
                "PowerPC 7447",
                MicroArch::Ppc7447,
                "Apollo 6",
                0x30C,
                Some(N130),
            ),
            (0x00, 0x030D) => Self::new(
                "PowerPC 7447A",
                MicroArch::Ppc7447a,
                "Apollo 7",
                0x30D,
                Some(N90),
            ),
            (0x00, 0x0351) => Self::new(
                "PowerPC 7450",
                MicroArch::Ppc7450,
                "Vger",
                0x351,
                Some(N180),
            ),
            (0x00, 0x0354) => Self::new(
                "PowerPC 7460",
                MicroArch::Ppc7460,
                "Apollo Pro",
                0x354,
                Some(N130),
            ),

            // PowerPC 970 / G5
            (0x00, 0x0039) => Self::new("PowerPC 970", MicroArch::Ppc970, "G5", 0x39, Some(N150)),
            (0x00, 0x003C) => {
                Self::new("PowerPC 970FX", MicroArch::Ppc970fx, "G5", 0x3C, Some(N90))
            }
            (0x00, 0x0044) => Self::new("PowerPC 970MP", MicroArch::Ppc970, "G5", 0x44, Some(N90)),

            // Apple PowerPC variants
            (0x00, 0x0033) => Self::new("Apple G4", MicroArch::Ppc7400, "Apollo", 0x33, Some(N180)),
            (0x00, 0x0045) => Self::new("Apple G5", MicroArch::Ppc970, "G5", 0x45, Some(N65)),
            (0x00, 0x0052) => Self::new("Apple G5", MicroArch::Ppc970fx, "G5", 0x52, Some(N65)),

            // 7451 family (version 0x02YY)
            (0x80, 0x0200..=0x0223) => Self::new(
                "PowerPC 7451",
                MicroArch::Ppc7451,
                "Vger",
                version,
                Some(N180),
            ),
            // 7445/7455 family (version 0x10YY)
            (0x80, 0x1000..=0x1014) => Self::new(
                "PowerPC 7455",
                MicroArch::Ppc7455,
                "Apollo 6",
                version,
                Some(N130),
            ),
            // 7447/7457 family (version 0x20YY)
            (0x80, 0x2000..=0x2022) => Self::new(
                "PowerPC 7457",
                MicroArch::Ppc7457,
                "Apollo 7",
                version,
                Some(N130),
            ),
            // 7447A family (version 0x30YY)
            (0x80, 0x3000..=0x3010) => Self::new(
                "PowerPC 7447A",
                MicroArch::Ppc7447a,
                "Apollo 7 PM",
                version,
                Some(N90),
            ),
            // 7448 family (version 0x40YY)
            (0x80, 0x4000..=0x4042) => Self::new(
                "PowerPC 7448",
                MicroArch::Ppc7448,
                "Apollo 8",
                version,
                Some(N90),
            ),

            // Apple "Scream" variants - specific revisions that override generic Freescale
            (0x80, 0x0101) => Self::new(
                "Apple G4",
                MicroArch::Ppc7445,
                "Scream",
                version,
                Some(N130),
            ),
            (0x80, 0x0102) => Self::new(
                "Apple G4",
                MicroArch::Ppc7445,
                "Scream",
                version,
                Some(N130),
            ),
            (0x80, 0x0103) => Self::new(
                "Apple G4",
                MicroArch::Ppc7455,
                "Scream",
                version,
                Some(N130),
            ),
            (0x80, 0x0104) => Self::new(
                "Apple G4",
                MicroArch::Ppc7455,
                "Scream",
                version,
                Some(N130),
            ),

            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppc750_lookup() {
        let cpu = CpuArch::find(0x00080200);
        assert_eq!(cpu.marketing_name, "PowerPC 750");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);
    }

    #[test]
    fn test_apple_g5_lookup() {
        let cpu = CpuArch::find(0x0045);
        assert_eq!(cpu.marketing_name, "Apple G5");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc970);
    }

    #[test]
    fn test_unknown_lookup() {
        let cpu = CpuArch::find(0xFFFF);
        assert_eq!(cpu.marketing_name, UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_ppc970fx_lookup() {
        let cpu = CpuArch::find(0x003C);
        assert_eq!(cpu.marketing_name, "PowerPC 970FX");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc970fx);
    }

    #[test]
    fn test_micro_arch_to_str() {
        assert_eq!(MicroArch::Ppc750.into(), "PowerPC 750 (G3)");
        assert_eq!(MicroArch::Unknown.into(), UNK);
    }

    #[test]
    fn test_freescale_7450_lookup() {
        let cpu = CpuArch::find(0x80000100);
        assert_eq!(cpu.marketing_name, "PowerPC 7450");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7450);
    }

    #[test]
    fn test_freescale_7451_lookup() {
        let cpu = CpuArch::find(0x80020000);
        assert_eq!(cpu.marketing_name, "PowerPC 7451");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7451);
    }

    #[test]
    fn test_freescale_7455_lookup() {
        let cpu = CpuArch::find(0x80100000);
        assert_eq!(cpu.marketing_name, "PowerPC 7455");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7455);
    }

    #[test]
    fn test_freescale_7447a_lookup() {
        let cpu = CpuArch::find(0x80300000);
        assert_eq!(cpu.marketing_name, "PowerPC 7447A");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7447a);
    }

    #[test]
    fn test_freescale_7448_lookup() {
        let cpu = CpuArch::find(0x80400000);
        assert_eq!(cpu.marketing_name, "PowerPC 7448");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7448);
    }

    #[test]
    fn test_apple_g4_7445_lookup() {
        let cpu = CpuArch::find(0x80100101);
        assert_eq!(cpu.marketing_name, "Apple G4");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7445);
    }

    #[test]
    fn test_apple_g4_7455_lookup() {
        let cpu = CpuArch::find(0x80100103);
        assert_eq!(cpu.marketing_name, "Apple G4");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc7455);
    }
}
