pub const UNK: &str = "Unknown";

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
    Ppc970,

    // IBM G5 / Apple A10 (PowerPC 970FX variant)
    Ppc970fx,

    // Apple RISC Reduced
    AppleTitan,
    AppleApollo,
    AppleDiana,
    AppleApollo2,
    AppleDiana2,
}

impl From<MicroArch> for String {
    fn from(ma: MicroArch) -> String {
        let s = match ma {
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
            MicroArch::Ppc970 => "PowerPC 970 (G5)",
            MicroArch::Ppc970fx => "PowerPC 970FX (G5)",
            MicroArch::AppleTitan => "Apple Titan",
            MicroArch::AppleApollo => "Apple Apollo",
            MicroArch::AppleDiana => "Apple Diana",
            MicroArch::AppleApollo2 => "Apple Apollo 2",
            MicroArch::AppleDiana2 => "Apple Diana 2",
        };

        String::from(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuArch {
    pub marketing_name: String,
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
        marketing_name: &str,
        micro_arch: MicroArch,
        code_name: &'static str,
        pvr_version: u16,
        technology: Option<&'static str>,
    ) -> Self {
        CpuArch {
            marketing_name: String::from(marketing_name),
            micro_arch,
            code_name,
            pvr_version,
            technology,
        }
    }

    pub fn find(pvr: u32) -> Self {
        let version = (pvr >> 16) as u16;

        match version {
            // IBM/Motorola PowerPC
            0x0001 => Self::new("PowerPC 601", MicroArch::Ppc601, "601", 0x01, Some("0.6μm")),
            0x0003 => Self::new("PowerPC 603", MicroArch::Ppc603, "603", 0x03, Some("0.5μm")),
            0x0004 => Self::new(
                "PowerPC 603e",
                MicroArch::Ppc603e,
                "603e",
                0x04,
                Some("0.35μm"),
            ),
            0x0006 => Self::new(
                "PowerPC 603eV",
                MicroArch::Ppc603ev,
                "603eV",
                0x06,
                Some("0.25μm"),
            ),
            0x0007 => Self::new(
                "PowerPC 604",
                MicroArch::Ppc604,
                "604",
                0x07,
                Some("0.35μm"),
            ),
            0x0009 => Self::new(
                "PowerPC 604e",
                MicroArch::Ppc604e,
                "604e",
                0x09,
                Some("0.25μm"),
            ),
            0x000A => Self::new(
                "PowerPC 604r",
                MicroArch::Ppc604r,
                "604r",
                0x0A,
                Some("0.25μm"),
            ),
            0x0013 => Self::new(
                "PowerPC 620",
                MicroArch::Ppc620,
                "620",
                0x13,
                Some("0.35μm"),
            ),

            // PowerPC 750 (G3)
            0x0200 => Self::new(
                "PowerPC 750",
                MicroArch::Ppc750,
                "Arthur",
                0x200,
                Some("0.26μm"),
            ),
            0x0201 => Self::new(
                "PowerPC 750CX",
                MicroArch::Ppc750,
                "G3",
                0x201,
                Some("0.18μm"),
            ),
            0x0202 => Self::new(
                "PowerPC 750CXe",
                MicroArch::Ppc750,
                "G3",
                0x202,
                Some("0.18μm"),
            ),
            0x0203 => Self::new(
                "PowerPC 750FX",
                MicroArch::Ppc750,
                "G3",
                0x203,
                Some("0.18μm"),
            ),
            0x0204 => Self::new(
                "PowerPC 750GX",
                MicroArch::Ppc750,
                "G3",
                0x204,
                Some("90nm"),
            ),
            0x0205 => Self::new(
                "PowerPC 750L",
                MicroArch::Ppc750,
                "G3",
                0x205,
                Some("0.18μm"),
            ),

            // PowerPC 7400 (G4)
            0x0308 => Self::new(
                "PowerPC 7400",
                MicroArch::Ppc7400,
                "Max",
                0x308,
                Some("0.22μm"),
            ),
            0x0309 => Self::new(
                "PowerPC 7410",
                MicroArch::Ppc7410,
                "Nitro",
                0x309,
                Some("0.18μm"),
            ),
            0x030C => Self::new(
                "PowerPC 7447",
                MicroArch::Ppc7447,
                "Apollo 6",
                0x30C,
                Some("0.13μm"),
            ),
            0x030D => Self::new(
                "PowerPC 7447A",
                MicroArch::Ppc7447a,
                "Apollo 7",
                0x30D,
                Some("90nm"),
            ),
            0x0351 => Self::new(
                "PowerPC 7450",
                MicroArch::Ppc7450,
                "Max",
                0x351,
                Some("0.18μm"),
            ),
            0x0352 => Self::new(
                "PowerPC 7455",
                MicroArch::Ppc7455,
                "Apollo",
                0x352,
                Some("0.15μm"),
            ),
            0x0353 => Self::new(
                "PowerPC 7457",
                MicroArch::Ppc7457,
                "Apollo",
                0x353,
                Some("0.13μm"),
            ),
            0x0354 => Self::new(
                "PowerPC 7460",
                MicroArch::Ppc7460,
                "Apollo Pro",
                0x354,
                Some("0.13μm"),
            ),

            // PowerPC 970 / G5
            0x0039 => Self::new("PowerPC 970", MicroArch::Ppc970, "G5", 0x39, Some("0.15μm")),
            0x003C => Self::new(
                "PowerPC 970FX",
                MicroArch::Ppc970fx,
                "G5",
                0x3C,
                Some("90nm"),
            ),
            0x0044 => Self::new("PowerPC 970MP", MicroArch::Ppc970, "G5", 0x44, Some("90nm")),

            // Apple PowerPC variants (based on IBM 7400/7410)
            // Apple uses version 0x0033 for some G4 chips
            0x0033 => Self::new(
                "Apple G4",
                MicroArch::Ppc7400,
                "Apollo",
                0x33,
                Some("0.18μm"),
            ),

            // Apple G5 variants
            0x0045 => Self::new("Apple G5", MicroArch::Ppc970, "G5", 0x45, Some("65nm")),
            0x0052 => Self::new("Apple G5", MicroArch::Ppc970fx, "G5", 0x52, Some("65nm")),

            // Apple "Scream" chips - these are actually G4 derivatives
            // Version 0x8000 series used by Apple for some custom chips
            0x8000 => Self::new(
                "Apple G4",
                MicroArch::Ppc7400,
                "Scream",
                0x8000,
                Some("0.18μm"),
            ),
            0x8001 => Self::new(
                "Apple G4+",
                MicroArch::Ppc7447,
                "Scream",
                0x8001,
                Some("0.13μm"),
            ),
            0x8002 => Self::new(
                "Apple G4+",
                MicroArch::Ppc7447a,
                "Scream",
                0x8002,
                Some("90nm"),
            ),

            // Apple RISC (Titan/Apollo/Diana) - these are custom Apple cores
            // Based on IBM 7400 architecture with custom modifications
            0x8003 => Self::new(
                "Apple Titan",
                MicroArch::AppleTitan,
                "Titan",
                0x8003,
                Some("0.18μm"),
            ),
            0x8004 => Self::new(
                "Apple Apollo",
                MicroArch::AppleApollo,
                "Apollo",
                0x8004,
                Some("0.13μm"),
            ),
            0x8005 => Self::new(
                "Apple Diana",
                MicroArch::AppleDiana,
                "Diana",
                0x8005,
                Some("90nm"),
            ),
            0x8006 => Self::new(
                "Apple Apollo 2",
                MicroArch::AppleApollo2,
                "Apollo 2",
                0x8006,
                Some("65nm"),
            ),
            0x8007 => Self::new(
                "Apple Diana 2",
                MicroArch::AppleDiana2,
                "Diana 2",
                0x8007,
                Some("65nm"),
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
        let cpu = CpuArch::find(0x0200);
        assert_eq!(cpu.marketing_name.as_str(), "PowerPC 750");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc750);
        assert_eq!(cpu.code_name, "Arthur");
    }

    #[test]
    fn test_apple_g5_lookup() {
        let cpu = CpuArch::find(0x0045);
        assert_eq!(cpu.marketing_name.as_str(), "Apple G5");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc970);
    }

    #[test]
    fn test_apple_apollo_lookup() {
        let cpu = CpuArch::find(0x8004);
        assert_eq!(cpu.marketing_name.as_str(), "Apple Apollo");
        assert_eq!(cpu.micro_arch, MicroArch::AppleApollo);
    }

    #[test]
    fn test_unknown_lookup() {
        let cpu = CpuArch::find(0xFFFF);
        assert_eq!(cpu.marketing_name.as_str(), UNK);
        assert_eq!(cpu.micro_arch, MicroArch::Unknown);
    }

    #[test]
    fn test_ppc970fx_lookup() {
        let cpu = CpuArch::find(0x003C);
        assert_eq!(cpu.marketing_name.as_str(), "PowerPC 970FX");
        assert_eq!(cpu.micro_arch, MicroArch::Ppc970fx);
    }

    #[test]
    fn test_apple_diana2_lookup() {
        let cpu = CpuArch::find(0x8007);
        assert_eq!(cpu.marketing_name.as_str(), "Apple Diana 2");
        assert_eq!(cpu.micro_arch, MicroArch::AppleDiana2);
    }

    #[test]
    fn test_micro_arch_to_string() {
        assert_eq!(String::from(MicroArch::Ppc750), "PowerPC 750 (G3)");
        assert_eq!(String::from(MicroArch::AppleApollo), "Apple Apollo");
        assert_eq!(String::from(MicroArch::Unknown), UNK);
    }
}
