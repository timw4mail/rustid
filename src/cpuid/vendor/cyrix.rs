use super::TMicroArch;
use crate::cpuid::brand::{CpuBrand, VENDOR_CYRIX};
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::{CpuSignature, FeatureClass, Str, UNK, has_cx8};
use crate::sfmt;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum CyrixModel {
    Slc,
    Dlc,
    Slc2,
    Dlc2,
    Srx,
    Drx,
    Srx2,
    Drx2,
    Cx486S,
    Cx486S2,
    Cx486Se,
    Cx486Se2,
    Cx486DX,
    Cx486DX2,
    Cx486DX4,
    Cx5x86,
    Cx6x86,
    Cx6x86L,
    MediaGx,
    M2,
    #[default]
    Unknown,
}

impl CyrixModel {
    pub fn detect() -> Self {
        let (dir0, dir1) = Cyrix::get_device_ids();

        match dir0 {
            // Cx486SLC/DLC/SRx/DRx (M0.5)
            0x00 => Self::Slc,
            0x01 => Self::Dlc,
            0x02 => Self::Slc2,
            0x03 => Self::Dlc2,
            0x04 => Self::Srx,
            0x05 => Self::Drx,
            0x06 => Self::Srx2,
            0x07 => Self::Drx2,

            // Cx486S (M0.6)
            0x10 => Self::Cx486S,
            0x11 => Self::Cx486S2,
            0x12 => Self::Cx486Se,
            0x13 => Self::Cx486Se2,

            // Cx486DX/DX2 (M0.7)
            0x1A => Self::Cx486DX,
            0x1B => Self::Cx486DX2,
            0x1F | 0x81 => Self::Cx486DX4,

            // 5x86 (M0.9)
            0x28..=0x2F => Self::Cx5x86,

            // 6x86 (M1)
            0x30 | 0x31 | 0x34 | 0x35 => {
                if dir1 > 0x21 || has_cx8() {
                    Self::Cx6x86L
                } else {
                    Self::Cx6x86
                }
            }

            // MediaGX (GXm)
            0x40..=0x46 => Self::MediaGx,

            // 6x86MX (M2)
            0x50..=0x5F => Self::M2,

            _ => Self::Unknown,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            // Cx486SLC/DLC/SRx/DRx (M0.5)
            Self::Slc => "Cx486 SLC",
            Self::Dlc => "Cx486 DLC",
            Self::Slc2 => "Cx486 SLC2",
            Self::Dlc2 => "Cx486 DLC2",
            Self::Srx => "Cx486SRx",
            Self::Drx => "Cx486DRx",
            Self::Srx2 => "Cx486SRx2",
            Self::Drx2 => "Cx486DRx2",

            // Cx486S (M0.6)
            Self::Cx486S => "Cx486S (B step)",
            Self::Cx486S2 => "Cx486S2 (B step)",
            Self::Cx486Se => "Cx486Se (B step)",
            Self::Cx486Se2 => "Cx486S2e (B step)",

            // Cx486DX/DX2 (M0.7)
            Self::Cx486DX => "Cx486DX",
            Self::Cx486DX2 => "Cx486DX2",
            Self::Cx486DX4 => "Cx486DX4",

            // 5x86 (M0.9)
            Self::Cx5x86 => "5x86",

            // 6x86 (M1)
            Self::Cx6x86 => "6x86",
            Self::Cx6x86L => "6x86L",

            // MediaGX (GXm)
            Self::MediaGx => "MediaGX GXm",

            // 6x86MX (M2)
            Self::M2 => "6x86MX/MII",

            Self::Unknown => UNK,
        }
    }
}

/// Cyrix-specific CPU identification and detection.
#[derive(Debug, Default, Clone)]
pub struct Cyrix {
    /// Device ID register 0
    pub dir0: u8,
    /// CPU revision
    pub revision: u8,
    /// CPU stepping
    pub stepping: u8,
    /// Bus multiplier factor
    pub multiplier: Str<4>,
    /// Model enum
    pub emodel: CyrixModel,
    /// Model name
    pub model: Str<64>,
    /// Code name
    pub code_name: Str<64>,
}

impl Cyrix {
    /// Detects and returns Cyrix CPU information.
    pub fn detect() -> Cyrix {
        if !crate::cpuid::is_cyrix() {
            return Cyrix::default();
        }

        let (dir0, dir1) = Self::get_device_ids();
        let revision = dir1 & 0x0F;
        let stepping = dir1 >> 4;
        let multiplier = Self::multiplier();
        let emodel = CyrixModel::detect();
        let model = Self::model_string();
        let code_name = Str::from(Self::codename());

        Cyrix {
            dir0,
            revision,
            stepping,
            multiplier,
            emodel,
            model,
            code_name,
        }
    }

    fn get_device_ids() -> (u8, u8) {
        if !crate::cpuid::is_cyrix() {
            return (0, 0);
        }

        const CYRIX_CCR_PORT: u16 = 0x22;
        const CYRIX_DATA_PORT: u16 = 0x23;

        fn read_ccr(index: u8) -> u8 {
            let result: u8;
            let idx = index;
            unsafe {
                core::arch::asm!(
                    "out dx, al",
                    in("dx") CYRIX_CCR_PORT,
                    in("al") idx,
                );
                core::arch::asm!(
                    "in al, dx",
                    in("dx") CYRIX_DATA_PORT,
                    out("al") result,
                );
            }
            result
        }

        let dir0 = read_ccr(0xFE);
        let dir1 = read_ccr(0xFF);

        let dir0 = if dir0 == 0xFF || dir0 == 0x00 {
            Self::get_device_id_from_signature()
        } else {
            dir0
        };

        (dir0, dir1)
    }

    fn get_device_id_from_signature() -> u8 {
        let signature = CpuSignature::detect();

        match (signature.family, signature.model) {
            (3, 2) => 0x01,
            (4, 5) => 0x10,
            (4, 8) => 0x1B,
            _ => 0,
        }
    }

    pub fn get_signature_from_device_id() -> CpuSignature {
        if !crate::cpuid::is_cyrix() {
            return CpuSignature::default();
        }

        match CyrixModel::detect() {
            CyrixModel::Cx486DX | CyrixModel::Cx486DX2 | CyrixModel::Cx486DX4 => {
                CpuSignature::new_synth(4, 8, 0)
            }
            CyrixModel::Cx5x86 => CpuSignature::new_synth(4, 9, 0),
            CyrixModel::Cx6x86 | CyrixModel::Cx6x86L => CpuSignature::new_synth(5, 2, 0),
            CyrixModel::MediaGx => CpuSignature::new_synth(5, 4, 0),
            CyrixModel::M2 => CpuSignature::new_synth(6, 0, 0),
            _ => CpuSignature::default(),
        }
    }

    pub fn get_feature_class() -> FeatureClass {
        if !crate::cpuid::is_cyrix() {
            return FeatureClass::i386;
        }

        match CyrixModel::detect() {
            CyrixModel::Cx6x86 | CyrixModel::Cx6x86L | CyrixModel::MediaGx => FeatureClass::i586,
            CyrixModel::M2 => FeatureClass::i686,
            _ => FeatureClass::i486,
        }
    }

    /// Check if the Cyrix model likely has CPUID support
    /// that can be enabled, if cpuid support is currently disabled
    pub fn can_enable_cpuid() -> bool {
        // If it's not Cyrix, or cpuid is enabled, we don't care
        if crate::cpuid::has_cpuid() || !crate::cpuid::is_cyrix() {
            return false;
        }

        let model = CyrixModel::detect();
        let (_, dir1) = Self::get_device_ids();
        let stepping = dir1 >> 4;

        // 5x86 can toggle cpuid if stepping is 1 or greater
        // 6x86/6x86L/6x86MX can toggle cpuid, earlier models can not
        // MediaGX always has cpuid enabled
        match model {
            CyrixModel::Cx5x86 => {
                if stepping >= 1 {
                    true
                } else {
                    false
                }
            }
            CyrixModel::Cx6x86 | CyrixModel::Cx6x86L | CyrixModel::M2 => true,
            _ => false,
        }
    }

    /// Get Cyrix processor model via registers
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    pub fn model_string() -> Str<64> {
        if !crate::cpuid::is_cyrix() {
            return Str::from(UNK);
        }

        let model = CyrixModel::detect().to_str();

        sfmt!("Cyrix {}", model).into()
    }

    /// Get bus multiplier for the current cpu
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    fn multiplier() -> Str<4> {
        if !crate::cpuid::is_cyrix() {
            return Str::from("0");
        }

        let (dir0, _) = Self::get_device_ids();

        let s = match dir0 {
            0x28 | 0x2A | 0x30 | 0x50 | 0x58 => "1",
            0x1B | 0x29 | 0x2B | 0x31 | 0x51 | 0x59 => "2",
            0x52 | 0x5A => "2.5",
            0x2D | 0x2F | 0x35 | 0x53 | 0x5B | 0x81 => "3",
            0x54 | 0x5C => "3.5",
            0x2C | 0x2E | 0x34 | 0x40 | 0x42 | 0x55 | 0x5D => "4",
            0x56 | 0x5E => "4.5",
            0x47 | 0x57 | 0x5F => "5",
            0x41 | 0x43 => "6",
            0x44 | 0x46 => "7",
            0x45 => "8",
            _ => "0",
        };

        Str::from(s)
    }

    /// Get Cyrix processor model via registers
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    pub fn codename() -> &'static str {
        match CyrixModel::detect() {
            CyrixModel::Slc
            | CyrixModel::Dlc
            | CyrixModel::Slc2
            | CyrixModel::Dlc2
            | CyrixModel::Srx
            | CyrixModel::Drx
            | CyrixModel::Srx2
            | CyrixModel::Drx2 => "M0.5",
            CyrixModel::Cx486S
            | CyrixModel::Cx486S2
            | CyrixModel::Cx486Se
            | CyrixModel::Cx486Se2 => "M0.6",
            CyrixModel::Cx486DX | CyrixModel::Cx486DX2 | CyrixModel::Cx486DX4 => "M0.7",
            CyrixModel::Cx5x86 => "M0.9",
            CyrixModel::Cx6x86 | CyrixModel::Cx6x86L => "M1",
            CyrixModel::MediaGx => "Gx86/Gxm",
            CyrixModel::M2 => "M2",
            CyrixModel::Unknown => UNK,
        }
    }
}

impl TMicroArch for Cyrix {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand = CpuBrand::Cyrix;
        let brand_arch = |ma: MicroArch, code_name: &'static str, tech: Option<&str>| -> CpuArch {
            CpuArch::new(
                model,
                ma,
                code_name,
                brand.to_brand_name(),
                VENDOR_CYRIX,
                tech,
            )
        };

        match (s.family, s.model, s.stepping) {
            (3, 2, _) => brand_arch(MicroArch::Cx486DLC, Cyrix::codename(), None),
            (4, 5, _) => brand_arch(MicroArch::Cx486S, Cyrix::codename(), None),
            (4, 8, _) => brand_arch(MicroArch::Cx486DX, Cyrix::codename(), None),
            (4, 9, _) => brand_arch(MicroArch::Cy5x86, Cyrix::codename(), None),
            (5, 2 | 3, _) => brand_arch(MicroArch::M1, Cyrix::codename(), None),
            (5, 4, _) => brand_arch(MicroArch::MediaGx, Cyrix::codename(), Some("350nm")),
            (6, 0, _) => brand_arch(MicroArch::M2, Cyrix::codename(), None),
            _ => brand_arch(MicroArch::Unknown, Cyrix::codename(), None),
        }
    }
}
