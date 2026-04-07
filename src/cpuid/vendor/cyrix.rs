use super::TMicroArch;
use crate::cpuid::brand::{CpuBrand, VENDOR_CYRIX};
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::{CpuSignature, FeatureClass, Str, UNK, has_cx8, vendor_str};
use crate::sfmt;

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
        let model = Self::model_string();
        let code_name = Str::from(Self::codename());

        Cyrix {
            dir0,
            revision,
            stepping,
            multiplier,
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

    pub fn get_signature_from_device_id(dir0: u8) -> CpuSignature {
        if !crate::cpuid::is_cyrix() {
            return CpuSignature::default();
        }

        match dir0 {
            // Cx486DX/DX2/DX4
            0x1A | 0x1B | 0x1F => CpuSignature::new_synth(4, 8, 0),
            // Cx586
            0x28..=0x2E => CpuSignature::new_synth(4, 9, 0),
            // 6x86/6x86L(M1)
            0x30 | 0x31 | 0x34 | 0x35 => CpuSignature::new_synth(5, 2, 0),
            // MediaGX/Geode
            0x40..=0x47 => CpuSignature::new_synth(5, 4, 0),
            // 6x86MX/MII
            0x50..=0x5F => CpuSignature::new_synth(6, 0, 0),
            _ => CpuSignature::default(),
        }
    }

    pub fn get_feature_class() -> FeatureClass {
        if !crate::cpuid::is_cyrix() {
            return FeatureClass::i386;
        }

        let (dir0, _) = Self::get_device_ids();

        match dir0 {
            // 5x86 and earlier
            0x00..=0x2F => FeatureClass::i486,

            // 6x86/6x86L
            0x30..=0x3F => FeatureClass::i586,

            // MediaGX
            0x40..=0x4F => FeatureClass::i586,

            // 6x86MX/MII
            0x50..=0x5F => FeatureClass::i686,
            _ => FeatureClass::i386,
        }
    }

    /// Check if the Cyrix model likely has CPUID support
    /// that can be enabled, if cpuid support is currently disabled
    pub fn can_enable_cpuid() -> bool {
        // If it's not Cyrix, or cpuid is enabled, we don't care
        if crate::cpuid::has_cpuid() || !crate::cpuid::is_cyrix() {
            return false;
        }

        let (dir0, dir1) = Self::get_device_ids();
        let stepping = dir1 >> 4;

        // 5x86 can toggle cpuid if stepping is 1 or greater
        if dir0 >= 0x28 && dir0 <= 0x2F && stepping >= 1 {
            return true;
        }
        // 6x86/6x86L/6x86MX can toggle cpuid, earlier models can not
        // MediaGX always has cpuid enabled
        dir0 >= 0x30 && dir0 < 0x40
    }

    /// Get Cyrix processor model via registers
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    pub fn model_string() -> Str<64> {
        if !crate::cpuid::is_cyrix() {
            return Str::from(UNK);
        }

        let (dir0, dir1) = Self::get_device_ids();

        let dev_id = dir0;
        let model = match dev_id {
            // Cx486SLC/DLC/SRx/DRx (M0.5)
            0x00 => "Cx486 SLC",
            0x01 => "Cx486 DLC",
            0x02 => "Cx486 SLC2",
            0x03 => "Cx486 DLC2",
            0x04 => "Cx486SRx",
            0x05 => "Cx486DRx",
            0x06 => "Cx486SRx2",
            0x07 => "Cx486DRx2",

            // Cx486S (M0.6)
            0x10 => "Cx486S (B step)",
            0x11 => "Cx486S2 (B step)",
            0x12 => "Cx486Se (B step)",
            0x13 => "Cx486S2e (B step)",

            // Cx486DX/DX2 (M0.7)
            0x1A => "Cx486DX",
            0x1B => "Cx486DX2",
            0x1F => "Cx486DX4",

            // 5x86 (M0.9)
            0x28..=0x2F => "5x86",

            // 6x86 (M1)
            0x30 | 0x31 | 0x34 | 0x35 => {
                if dir1 > 0x21 || has_cx8() {
                    "6x86L"
                } else {
                    "6x86"
                }
            }

            // MediaGX (GXm)
            0x40..=0x46 => "MediaGX GXm",

            // 6x86MX (M2)
            0x50..=0x5F => "6x86MX/MII",

            _ => UNK,
        };

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
            0x2D | 0x2F | 0x35 | 0x53 | 0x5B => "3",
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
        if !crate::cpuid::is_cyrix() {
            return UNK;
        }

        let (dir0, _) = Cyrix::get_device_ids();

        match dir0 {
            0x01..=0x07 => "M0.5",
            0x10..=0x13 => "M0.6",
            0x1A | 0x1B | 0x1F => "M0.7",
            0x28..=0x2F => "M0.9",
            0x30 | 0x31 | 0x34 | 0x35 => "M1",
            0x40..=0x47 => "Gx86/GXm",
            0x50..=0x59 => "M2",
            _ => "Unknown",
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
