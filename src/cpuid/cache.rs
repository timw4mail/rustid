use crate::cpuid::brand::{VENDOR_AMD, VENDOR_CENTAUR};

use super::*;

const DATA_CACHE: u32 = 1;
const INSTRUCTION_CACHE: u32 = 2;
const UNIFIED_CACHE: u32 = 3;

const L1: u32 = 1;
const L2: u32 = 2;
const L3: u32 = 3;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum CacheType {
    Unified,
    Data,
    Instruction,
    #[default]
    Invalid,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CacheLevel {
    pub(crate) assoc: u32,
    pub(crate) size: u32,
    pub(crate) kind: CacheType,
    pub(crate) share_count: u32,
}

impl CacheLevel {
    pub fn new(size: u32, kind: CacheType, assoc: u32, share_count: u32) -> Self {
        CacheLevel {
            size,
            kind,
            assoc,
            share_count,
        }
    }

    pub fn no_count(size: u32, kind: CacheType, assoc: u32) -> Self {
        Self::new(size, kind, assoc, 0)
    }

    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Self::new(size, CacheType::Unified, assoc, 0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Level1Cache {
    Unified(CacheLevel),
    Split {
        data: CacheLevel,
        instruction: CacheLevel,
    },
}

impl Level1Cache {
    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Level1Cache::Unified(CacheLevel::new_unified(size, assoc))
    }

    pub fn is_unified(&self) -> bool {
        match self {
            Level1Cache::Unified(_) => true,
            Level1Cache::Split { .. } => false,
        }
    }

    pub fn is_split(&self) -> bool {
        !self.is_unified()
    }

    pub fn set_data(&mut self, size: u32, assoc: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.size = size;
            data.kind = CacheType::Data;
            data.assoc = assoc;
        }
    }

    pub fn set_data_share_count(&mut self, share_count: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.share_count = share_count;
        }
    }

    pub fn set_instruction(&mut self, size: u32, assoc: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.size = size;
            instruction.kind = CacheType::Instruction;
            instruction.assoc = assoc;
        }
    }

    pub fn set_instruction_share_count(&mut self, share_count: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.share_count = share_count;
        }
    }

    pub fn default_split() -> Self {
        Level1Cache::Split {
            data: CacheLevel::default(),
            instruction: CacheLevel::default(),
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            Level1Cache::Unified(level) => level.size,
            Level1Cache::Split { data, instruction } => data.size + instruction.size,
        }
    }
}

impl Default for Level1Cache {
    fn default() -> Self {
        Level1Cache::Unified(CacheLevel::default())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Cache {
    pub l1: Level1Cache,
    pub l2: Option<CacheLevel>,
    pub l3: Option<CacheLevel>,
}

impl Cache {
    pub fn new(l1: Level1Cache, l2: Option<CacheLevel>, l3: Option<CacheLevel>) -> Cache {
        Cache { l1, l2, l3 }
    }

    pub fn detect() -> Option<Self> {
        match vendor_str().as_str() {
            VENDOR_AMD => match is_valid_leaf(EXT_LEAF_1D) {
                true => Cache::detect_general(EXT_LEAF_1D),
                false => Cache::detect_ext_5_6(),
            },
            VENDOR_CENTAUR => match is_valid_leaf(LEAF_4) {
                true => Cache::detect_general(LEAF_4),
                false => Cache::detect_ext_5_6(),
            },
            _ => {
                // The 1-bit cache descriptors are on LEAF 0x2, but
                // the extended cache topology is on LEAF 0x4.
                // We want to use the extended cache topology if
                // it exists.
                match max_leaf() {
                    LEAF_2..LEAF_4 => Cache::detect_fallback(),
                    LEAF_4.. => Cache::detect_general(LEAF_4),
                    _ => None,
                }
            }
        }
    }

    fn detect_ext_5_6() -> Option<Self> {
        let res5 = x86_cpuid(EXT_LEAF_5);
        let res6 = x86_cpuid(EXT_LEAF_6);

        let mut c = Cache {
            l1: Level1Cache::default_split(),
            ..Cache::default()
        };

        let afn = match vendor_str().as_str() {
            super::brand::VENDOR_AMD => Self::amd_assoc,
            _ => Self::centaur_assoc,
        };

        let l1dassoc = afn((res5.ecx >> 16) & 0x1F);
        let l1iassoc = afn((res5.edx >> 16) & 0x1F);
        c.l1.set_data((res5.ecx >> 24) * 1024, l1dassoc);
        c.l1.set_instruction((res5.edx >> 24) * 1024, l1iassoc);

        let l2assoc = afn((res6.ecx >> 12) & 0x1F);
        let l2size = (res6.ecx >> 16) * 1024;
        let l3assoc = afn((res6.edx >> 12) & 0x1F);
        let l3size = (res6.edx >> 18) * 512 * 1024;

        if l2size != 0 {
            c.l2 = Some(CacheLevel::new_unified(l2size, l2assoc));
        }

        if l3size != 0 {
            c.l3 = Some(CacheLevel::new_unified(l3size, l3assoc));
        }

        Some(c)
    }

    fn centaur_assoc(reg: u32) -> u32 {
        match reg {
            0 | 0xF => 0,
            n => n,
        }
    }

    fn amd_assoc(reg: u32) -> u32 {
        match reg {
            0 | 0xF => 0,
            n => 1 << n,
        }
    }

    /// Get cache information via 1-bit descriptors
    ///
    /// See https://sandpile.org/x86/cpuid.htm#level_0000_0002h
    fn detect_fallback() -> Option<Self> {
        use heapless::Vec;

        let mut c = Cache::default();

        let res = x86_cpuid(LEAF_2);
        let iteration_count = res.eax & 0xFF;
        let mut desc_list: Vec<u32, 32> = Vec::new();

        for i in 0..=iteration_count {
            let res = x86_cpuid_count(LEAF_2, i);
            let valid_eax = (res.eax >> 31) == 0;
            if !valid_eax {
                break;
            }

            for offset in [8u32, 16, 24] {
                let desc = (res.eax >> offset) & 0xFF;
                if desc != 0 {
                    let _ = desc_list.push(desc);
                }
            }

            let valid_ebx = (res.ebx >> 31) == 0;
            if !valid_ebx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.ebx >> offset) & 0xFF;
                if desc != 0 {
                    let _ = desc_list.push(desc);
                }
            }

            let valid_ecx = (res.ecx >> 31) == 0;
            if !valid_ecx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.ecx >> offset) & 0xFF;
                if desc != 0 {
                    let _ = desc_list.push(desc);
                }
            }

            let valid_edx = (res.edx >> 31) == 0;
            if !valid_edx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.edx >> offset) & 0xFF;
                if desc != 0 {
                    let _ = desc_list.push(desc);
                }
            }
        }

        for desc in desc_list.iter() {
            match desc {
                // code L1 cache, 8 KB, 4 ways, 32 byte lines
                0x06 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(8 * 1024, 4);
                }
                // code L1 cache, 16 KB, 4 ways, 32 byte lines
                0x08 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(16 * 1024, 4);
                }
                // code L1 cache, 32 KB, 4 ways, 64 byte lines
                0x09 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(32 * 1024, 4);
                }
                // data L1 cache, 8 KB, 2 ways, 32 byte lines
                0x0A => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(8 * 1024, 2);
                }
                // data L1 cache, 16 KB, 4 ways, 32 byte lines
                // data L1 cache, 16 KB, 4 ways, 64 byte lines (ECC)
                0x0C | 0x0D => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 4);
                }
                // data L1 cache, 24 KB, 6 ways, 64 byte lines
                0x0E => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(24 * 1024, 6);
                }
                // data L1 cache, 16 KB, 4 ways, 32 byte lines (IA-64)
                0x10 | 0x15 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 4);
                }
                // data L1 cache, 16 KB, 4 ways, 64 byte lines (IA-64)
                0x12 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 4);
                }
                // data L1 cache, 16 KB, 8 ways, 64 byte lines (IA-64)
                0x13 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 8);
                }
                // data L1 cache, 32 KB, 4 ways, 64 byte lines (IA-64)
                0x14 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(32 * 1024, 4);
                }
                // data L1 cache, 32 KB, 8 ways, 64 byte lines (IA-64)
                0x16 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(32 * 1024, 8);
                }
                // data L1 cache, 32 KB, 16 ways, 64 byte lines (IA-64)
                0x17 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(32 * 1024, 16);
                }
                // data L1 cache, 64 KB, 4 ways, 64 byte lines (IA-64)
                0x18 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(64 * 1024, 4);
                }
                // data L1 cache, 64 KB, 8 ways, 64 byte lines (IA-64)
                0x19 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(64 * 1024, 8);
                }
                // code and data L2 cache, 96 KB, 6 ways, 64 byte lines (IA-64)
                0x1A => {
                    c.l2 = Some(CacheLevel::no_count(96 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 128 KB, 2 ways, 64 byte lines
                0x1D => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 256 KB, 8 ways, 64 byte lines
                0x21 => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 512 KB, 4 ways (!), 64 byte lines, dual-sectored
                0x22 => {
                    c.l3 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 8 ways, 64 byte lines, dual-sectored
                0x23 => {
                    c.l3 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 8));
                }
                //  code and data L2 cache, 1024 KB, 16 ways, 64 byte lines
                0x24 => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 2048 KB, 8 ways, 64 byte lines, dual-sectored
                0x25 => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 4096 KB, 8 ways, 64 byte lines, dual-sectored
                0x29 => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 8));
                }
                // data L1 cache, 32 KB, 8 ways, 64 byte lines
                0x2C => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(32 * 1024, 8);
                }
                // code L1 cache, 32 KB, 8 ways, 64 byte lines
                0x30 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(32 * 1024, 8);
                }
                // code and data L2 cache, 128 KB, 4 ways, 64 byte lines, sectored
                0x39 => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 192 KB, 6 ways, 64 byte lines, sectored
                0x3A => {
                    c.l2 = Some(CacheLevel::no_count(192 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 128 KB, 2 ways, 64 byte lines, sectored
                0x3B => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 256 KB, 4 ways, 64 byte lines, sectored
                0x3C => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 384 KB, 6 ways, 64 byte lines, sectored
                0x3D => {
                    c.l2 = Some(CacheLevel::no_count(384 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 512 KB, 4 ways, 64 byte lines, sectored
                0x3E => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 128 KB, 4 ways, 32 byte lines
                0x41 => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 256 KB, 4 ways, 32 byte lines
                0x42 => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 512 KB, 4 ways, 32 byte lines
                0x43 => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 1024 KB, 4 ways, 32 byte lines
                0x44 => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 2048 KB, 4 ways, 32 byte lines
                0x45 => {
                    c.l2 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 4096 KB, 4 ways, 64 byte lines
                0x46 => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 8192 KB, 8 ways, 64 byte lines
                0x47 => {
                    c.l3 = Some(CacheLevel::no_count(8192 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 3072 KB, 12 ways, 64 byte lines
                0x48 => {
                    c.l2 = Some(CacheLevel::no_count(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines (P4) or
                // code and data L2 cache, 4096 KB, 16 ways, 64 byte lines (Core 2)
                0x49 => {
                    let sig = CpuSignature::detect();

                    // Core/Core 2 has signatures like (0, 6, 0, 15, 6) and (0, 6, 1, 7, 0)
                    match (sig.family, sig.extended_model, sig.model) {
                        // Core 2
                        (6, 0, 13..) | (6, 1, 7) => {
                            c.l2 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                        }
                        // P4
                        _ => {
                            c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                        }
                    }
                }
                // code and data L3 cache, 6144 KB, 12 ways, 64 byte lines
                0x4A => {
                    c.l3 = Some(CacheLevel::no_count(6144 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 8192 KB, 16 ways, 64 byte lines
                0x4B => {
                    c.l3 = Some(CacheLevel::no_count(8192 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 12288 KB, 12 ways, 64 byte lines
                0x4C => {
                    c.l3 = Some(CacheLevel::no_count(12288 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 16384 KB, 16 ways, 64 byte lines
                0x4D => {
                    c.l3 = Some(CacheLevel::no_count(16384 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 2048 KB, 16 ways, 64 byte lines
                0x4E => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines
                0x4F => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                }
                // data L1 cache, 16 KB, 8 ways, 64 byte lines, sectored
                0x60 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 8);
                }
                // data L1 cache, 8 KB, 4 ways, 64 byte lines, sectored
                0x66 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(8 * 1024, 4);
                }
                // data L1 cache, 16 KB, 4 ways, 64 byte lines, sectored
                0x67 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(16 * 1024, 4);
                }
                // data L1 cache, 32 KB, 4 ways, 64 byte lines, sectored
                0x68 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(32 * 1024, 4);
                }
                // code L1 cache, 16 KB, 4 ways, 64 byte lines, sectored (IA-64)
                0x77 => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(16 * 1024, 4);
                }
                // code and data L2 cache, 1024 KB, 4 ways, 64 byte lines
                0x78 => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 128 KB, 8 ways, 64 byte lines, dual-sectored
                0x79 => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 64 byte lines, dual-sectored
                0x7A => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 8 ways, 64 byte lines, dual-sectored
                0x7B => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 64 byte lines, dual-sectored
                0x7C => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 2048 KB, 8 ways, 64 byte lines
                0x7D => {
                    c.l2 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 128 byte lines (IA-64)
                0x7E => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 2 ways, 64 byte lines
                0x7F => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 512 KB, 8 ways, 64 byte lines
                0x80 => {
                    #[cfg(target_arch = "x86")]
                    if super::is_cyrix() {
                        // code and data L1 cache, 16 KB, 4 ways, 16 byte lines
                        c.l1 = Level1Cache::Unified(CacheLevel::no_count(
                            16 * 1024,
                            CacheType::Unified,
                            4,
                        ));
                        continue;
                    }

                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 128 KB, 8 ways, 32 byte lines
                0x81 => {
                    c.l2 = Some(CacheLevel::no_count(128 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 32 byte lines
                0x82 => {
                    c.l2 = Some(CacheLevel::no_count(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 8 ways, 32 byte lines
                0x83 => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 32 byte lines
                0x84 => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 2048 KB, 8 ways, 32 byte lines
                0x85 => {
                    c.l2 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 4 ways, 64 byte lines
                0x86 => {
                    c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 64 byte lines
                0x87 => {
                    c.l2 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 2048 KB, 4 ways, 64 byte lines (IA-64)
                0x88 => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 4096 KB, 4 ways, 64 byte lines (IA-64)
                0x89 => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 8192 KB, 4 ways, 64 byte lines (IA-64)
                0x8A => {
                    c.l3 = Some(CacheLevel::no_count(8192 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 3072 KB, 12 ways, 128 byte lines (IA-64)
                0x8D => {
                    c.l3 = Some(CacheLevel::no_count(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 512 KB, 4 ways, 64 byte lines
                0xD0 => {
                    c.l3 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 4 ways, 64 byte lines
                0xD1 => {
                    c.l3 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 2048 KB, 4 ways, 64 byte lines
                0xD2 => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 8 ways, 64 byte lines
                0xD6 => {
                    c.l3 = Some(CacheLevel::no_count(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 2048 KB, 8 ways, 64 byte lines
                0xD7 => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 4096 KB, 8 ways, 64 byte lines
                0xD8 => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 1536 KB, 12 ways, 64 byte lines
                0xDC => {
                    c.l3 = Some(CacheLevel::no_count(1536 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 3072 KB, 12 ways, 64 byte lines
                0xDD => {
                    c.l3 = Some(CacheLevel::no_count(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 6144 KB, 12 ways, 64 byte lines
                0xDE => {
                    c.l3 = Some(CacheLevel::no_count(6144 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 2048 KB, 16 ways, 64 byte lines
                0xE2 => {
                    c.l3 = Some(CacheLevel::no_count(2048 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines
                0xE3 => {
                    c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 8192 KB, 16 ways, 64 byte lines
                0xE4 => {
                    c.l3 = Some(CacheLevel::no_count(8192 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 12288 KB, 24 ways, 64 byte lines
                0xEA => {
                    c.l3 = Some(CacheLevel::no_count(12288 * 1024, CacheType::Unified, 24));
                }
                // code and data L3 cache, 18432 KB, 24 ways, 64 byte lines
                0xEB => {
                    c.l3 = Some(CacheLevel::no_count(18432 * 1024, CacheType::Unified, 24));
                }
                // code and data L3 cache, 24576 KB, 24 ways, 64 byte lines
                0xEC => {
                    c.l3 = Some(CacheLevel::no_count(24576 * 1024, CacheType::Unified, 24));
                }
                _ => continue,
            }
        }

        if c == Cache::default() { None } else { Some(c) }
    }

    fn detect_general(leaf: u32) -> Option<Self> {
        let mut c = Cache::default();

        for level in 0u32..32 {
            let res = x86_cpuid_count(leaf, level);
            let cache_type = res.eax & 0xF;

            // If cache_type is 0, the cache type is invalid
            if cache_type == 0 {
                break;
            }

            let cache_level = (res.eax >> 5) & 0x7;
            let share_count = ((res.eax >> 14) & 0x7) + 1;
            let cache_sets = res.ecx + 1;
            let cache_line_size = (res.ebx & 0xFFF) + 1;
            let cache_partitions = ((res.ebx >> 12) & 0x3FF) + 1;
            let cache_ways_of_associativity = ((res.ebx >> 22) & 0x3FF) + 1;

            let cache_size =
                cache_sets * cache_partitions * cache_ways_of_associativity * cache_line_size;

            // If cache size is 0, the entry is probably invalid
            if cache_size == 0 {
                break;
            }

            match cache_type {
                DATA_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_data(cache_size, cache_ways_of_associativity);
                        c.l1.set_data_share_count(share_count)
                    }
                }
                INSTRUCTION_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_instruction(cache_size, cache_ways_of_associativity);
                        c.l1.set_instruction_share_count(share_count);
                    }
                }
                UNIFIED_CACHE => match cache_level {
                    L1 => {
                        c.l1 = Level1Cache::new_unified(cache_size, cache_ways_of_associativity);
                    }
                    L2 => {
                        c.l2 = Some(CacheLevel::new(
                            cache_size,
                            CacheType::Unified,
                            cache_ways_of_associativity,
                            share_count,
                        ));
                    }
                    L3 => {
                        c.l3 = Some(CacheLevel::new(
                            cache_size,
                            CacheType::Unified,
                            cache_ways_of_associativity,
                            share_count,
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if c == Cache::default() { None } else { Some(c) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_level() {
        let cl = CacheLevel::new(1024, CacheType::Unified, 8, 1);
        assert_eq!(cl.size, 1024);
        assert_eq!(cl.kind, CacheType::Unified);
        assert_eq!(cl.assoc, 8);
        assert_eq!(cl.share_count, 1);
    }

    #[test]
    fn test_l1_cache_unified() {
        let l1 = Level1Cache::new_unified(2048, 4);
        assert!(l1.is_unified());
        assert!(!l1.is_split());
        assert_eq!(l1.size(), 2048);
    }

    #[test]
    fn test_l1_cache_split() {
        let mut l1 = Level1Cache::default_split();
        assert!(l1.is_split());
        assert!(!l1.is_unified());
        assert_eq!(l1.size(), 0);

        l1.set_data(1024, 8);
        l1.set_instruction(1024, 4);

        assert_eq!(l1.size(), 2048);

        if let Level1Cache::Split { data, instruction } = l1 {
            assert_eq!(data.size, 1024);
            assert_eq!(data.assoc, 8);
            assert_eq!(instruction.size, 1024);
            assert_eq!(instruction.assoc, 4);
        } else {
            panic!("Expected split cache");
        }
    }

    #[test]
    fn test_cache_new() {
        let l1 = Level1Cache::new_unified(1, 1);
        let l2 = Some(CacheLevel::new(2, CacheType::Unified, 2, 2));
        let l3 = Some(CacheLevel::new(3, CacheType::Unified, 3, 3));
        let cache = Cache::new(l1, l2, l3);

        assert_eq!(cache.l1, l1);
        assert_eq!(cache.l2, l2);
        assert_eq!(cache.l3, l3);
    }

    #[test]
    fn test_centaur_assoc() {
        assert_eq!(Cache::centaur_assoc((0x40040140 >> 16) & 0x1F), 4);
    }

    #[test]
    fn test_amd_assoc() {
        assert_eq!(Cache::amd_assoc((0x00000000 >> 16) & 0xF), 0);
        assert_eq!(Cache::amd_assoc((0x00010000 >> 16) & 0xF), 2);
        assert_eq!(Cache::amd_assoc((0x00020000 >> 16) & 0xF), 4);
        assert_eq!(Cache::amd_assoc((0x00030000 >> 16) & 0xF), 8);
        assert_eq!(Cache::amd_assoc((0x000F0000 >> 16) & 0xF), 0);
    }

    #[test]
    fn test_amd_assoc_k5() {
        assert_eq!(Cache::amd_assoc((0x20020220 >> 16) & 0xF), 4);
    }
}
