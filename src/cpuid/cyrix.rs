use super::{FeatureClass, UNK, has_cx8};

use crate::cpuid::brand::VENDOR_CYRIX;
use core::str::FromStr;
use heapless::{String, format};

#[derive(Debug, Default, Clone)]
pub struct Cyrix {
    pub dir0: u8,
    pub dir1: u8,
    pub revision: u8,
    pub stepping: u8,
    pub multiplier: String<4>,
    pub model: String<64>,
    pub code_name: String<64>,
}

impl Cyrix {
    pub fn detect() -> Cyrix {
        if !Self::is_cyrix() {
            return Cyrix::default();
        }

        let (dir0, dir1) = Self::get_device_ids();
        let revision = dir1 & 0x0F;
        let stepping = dir1 >> 4;
        let multiplier = Self::multiplier();
        let model = Self::model_string();
        let code_name = String::from_str(Self::codename()).unwrap();

        Cyrix {
            dir0,
            dir1,
            revision,
            stepping,
            multiplier,
            model,
            code_name,
        }
    }

    fn get_device_ids() -> (u8, u8) {
        if !Self::is_cyrix() {
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

        (dir0, dir1)
    }

    fn is_cyrix() -> bool {
        super::vendor_str().as_str() == VENDOR_CYRIX
    }

    pub fn get_feature_class() -> FeatureClass {
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
        // If this isn't a Cyrix CPU, we'll just assume CPUID can't be enabled
        if !Self::is_cyrix() {
            return false;
        }

        // If CPUID is already enabled, we don't need to enable it again
        if super::has_cpuid() {
            return false;
        }

        let (dir0, _) = Self::get_device_ids();

        // 5x86/6x86/6x86L/6x86MX/MediaGX can toggle cpuid, earlier models can not
        dir0 >= 0x28
    }

    /// Get Cyrix processor model via registers
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    pub fn model_string() -> String<64> {
        if !Self::is_cyrix() {
            return String::from_str(UNK).unwrap();
        }

        let (dir0, _) = Self::get_device_ids();

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
                if has_cx8() {
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

        let s = format!("Cyrix {}", model);

        s.unwrap()
    }

    /// Get bus multiplier for the current cpu
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    fn multiplier() -> String<4> {
        if !Self::is_cyrix() {
            return String::from_str("0").unwrap();
        }

        let (dir0, _) = Self::get_device_ids();

        let s = match dir0 {
            0x28 | 0x2A | 0x30 | 0x50 | 0x58 => "1",
            0x29 | 0x2B | 0x31 | 0x51 | 0x59 => "2",
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

        String::from_str(s).unwrap()
    }

    /// Get Cyrix processor model via registers
    ///
    /// See: https://www.ardent-tool.com/CPU/docs/Cyrix/detect.pdf
    pub fn codename() -> &'static str {
        if !Self::is_cyrix() {
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
