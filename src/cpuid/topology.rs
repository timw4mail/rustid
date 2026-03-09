use super::brand::{CpuBrand, VENDOR_AMD, VENDOR_INTEL};
use crate::cpuid::{EXT_LEAF_1D, LEAF_2, get_ht, has_ht, max_extended_leaf, vendor_str};

#[allow(unused_imports)]
use super::{EXT_LEAF_5, EXT_LEAF_6, LEAF_4, LEAF_16, max_leaf, x86_cpuid, x86_cpuid_count};

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
}

impl CacheLevel {
    pub fn new(size: u32, kind: CacheType, assoc: u32) -> Self {
        CacheLevel { size, kind, assoc }
    }

    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Self::new(size, CacheType::Unified, assoc)
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

    pub fn set_instruction(&mut self, size: u32, assoc: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.size = size;
            instruction.kind = CacheType::Instruction;
            instruction.assoc = assoc;
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
        // Check for support for the Intel method
        match CpuBrand::detect() {
            CpuBrand::Intel => Self::detect_intel(),
            CpuBrand::AMD => Self::detect_amd(),
            _ => Self::detect_intel(),
        }
    }

    fn detect_amd() -> Option<Self> {
        if max_extended_leaf() >= EXT_LEAF_1D {
            Cache::detect_general(EXT_LEAF_1D)
        } else {
            Cache::detect_amd_fallback()
        }
    }

    fn detect_amd_fallback() -> Option<Self> {
        let res5 = x86_cpuid(EXT_LEAF_5);
        let res6 = x86_cpuid(EXT_LEAF_6);

        let mut c = Cache {
            l1: Level1Cache::default_split(),
            ..Cache::default()
        };

        let l1dassoc = (res5.ecx >> 16) + 1;
        let l1iassoc = (res5.edx >> 16) + 1;
        c.l1.set_data((res5.ecx >> 24) * 1024, l1dassoc);
        c.l1.set_instruction((res5.edx >> 24) * 1024, l1iassoc);

        let l2assoc = (res6.ecx >> 12) + 1;
        let l2size = (res6.ecx >> 16) * 1024;
        let l3assoc = (res6.edx >> 12) + 1;
        let l3size = (res6.edx >> 18) * 512 * 1024;

        if l2size != 0 {
            c.l2 = Some(CacheLevel::new_unified(l2size, l2assoc));
        }

        if l3size != 0 {
            c.l3 = Some(CacheLevel::new_unified(l3size, l3assoc));
        }

        Some(c)
    }

    fn detect_intel() -> Option<Self> {
        if max_leaf() >= LEAF_4 {
            Cache::detect_general(LEAF_4)
        } else if max_leaf() >= LEAF_2 {
            Cache::detect_fallback()
        } else {
            None
        }
    }

    /// Get cache information via 1-bit descriptors
    ///
    /// See https://sandpile.org/x86/cpuid.htm#level_0000_0002h
    fn detect_fallback() -> Option<Self> {
        use heapless::Vec;

        let mut c = Cache::default();

        if max_leaf() < LEAF_2 {
            return None;
        }

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
                    c.l2 = Some(CacheLevel::new(96 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 128 KB, 2 ways, 64 byte lines
                0x1D => {
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 256 KB, 8 ways, 64 byte lines
                0x21 => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 512 KB, 4 ways (!), 64 byte lines, dual-sectored
                0x22 => {
                    c.l3 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 8 ways, 64 byte lines, dual-sectored
                0x23 => {
                    c.l3 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8));
                }
                //  code and data L2 cache, 1024 KB, 16 ways, 64 byte lines
                0x24 => {
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 2048 KB, 8 ways, 64 byte lines, dual-sectored
                0x25 => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 4096 KB, 8 ways, 64 byte lines, dual-sectored
                0x29 => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 8));
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
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 192 KB, 6 ways, 64 byte lines, sectored
                0x3A => {
                    c.l2 = Some(CacheLevel::new(192 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 128 KB, 2 ways, 64 byte lines, sectored
                0x3B => {
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 256 KB, 4 ways, 64 byte lines, sectored
                0x3C => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 384 KB, 6 ways, 64 byte lines, sectored
                0x3D => {
                    c.l2 = Some(CacheLevel::new(384 * 1024, CacheType::Unified, 6));
                }
                // code and data L2 cache, 512 KB, 4 ways, 64 byte lines, sectored
                0x3E => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 128 KB, 4 ways, 32 byte lines
                0x41 => {
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 256 KB, 4 ways, 32 byte lines
                0x42 => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 512 KB, 4 ways, 32 byte lines
                0x43 => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 1024 KB, 4 ways, 32 byte lines
                0x44 => {
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 2048 KB, 4 ways, 32 byte lines
                0x45 => {
                    c.l2 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 4096 KB, 4 ways, 64 byte lines
                0x46 => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 8192 KB, 8 ways, 64 byte lines
                0x47 => {
                    c.l3 = Some(CacheLevel::new(8192 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 3072 KB, 12 ways, 64 byte lines
                0x48 => {
                    c.l2 = Some(CacheLevel::new(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines (P4) or
                // code and data L2 cache, 4096 KB, 16 ways, 64 byte lines (Core 2)
                0x49 => {
                    if c.l3.is_some() {
                        c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 16));
                    } else {
                        c.l2 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 16));
                    }
                }
                // code and data L3 cache, 6144 KB, 12 ways, 64 byte lines
                0x4A => {
                    c.l3 = Some(CacheLevel::new(6144 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 8192 KB, 16 ways, 64 byte lines
                0x4B => {
                    c.l3 = Some(CacheLevel::new(8192 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 12288 KB, 12 ways, 64 byte lines
                0x4C => {
                    c.l3 = Some(CacheLevel::new(12288 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 16384 KB, 16 ways, 64 byte lines
                0x4D => {
                    c.l3 = Some(CacheLevel::new(16384 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 2048 KB, 16 ways, 64 byte lines
                0x4E => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines
                0x4F => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 16));
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
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 128 KB, 8 ways, 64 byte lines, dual-sectored
                0x79 => {
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 64 byte lines, dual-sectored
                0x7A => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 8 ways, 64 byte lines, dual-sectored
                0x7B => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 64 byte lines, dual-sectored
                0x7C => {
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 2048 KB, 8 ways, 64 byte lines
                0x7D => {
                    c.l2 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 128 byte lines (IA-64)
                0x7E => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 2 ways, 64 byte lines
                0x7F => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 2));
                }
                // code and data L2 cache, 512 KB, 8 ways, 64 byte lines
                0x80 => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 128 KB, 8 ways, 32 byte lines
                0x81 => {
                    c.l2 = Some(CacheLevel::new(128 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 256 KB, 8 ways, 32 byte lines
                0x82 => {
                    c.l2 = Some(CacheLevel::new(256 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 8 ways, 32 byte lines
                0x83 => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 32 byte lines
                0x84 => {
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 2048 KB, 8 ways, 32 byte lines
                0x85 => {
                    c.l2 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L2 cache, 512 KB, 4 ways, 64 byte lines
                0x86 => {
                    c.l2 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L2 cache, 1024 KB, 8 ways, 64 byte lines
                0x87 => {
                    c.l2 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 2048 KB, 4 ways, 64 byte lines (IA-64)
                0x88 => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 4096 KB, 4 ways, 64 byte lines (IA-64)
                0x89 => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 8192 KB, 4 ways, 64 byte lines (IA-64)
                0x8A => {
                    c.l3 = Some(CacheLevel::new(8192 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 3072 KB, 12 ways, 128 byte lines (IA-64)
                0x8D => {
                    c.l3 = Some(CacheLevel::new(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 512 KB, 4 ways, 64 byte lines
                0xD0 => {
                    c.l3 = Some(CacheLevel::new(512 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 4 ways, 64 byte lines
                0xD1 => {
                    c.l3 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 2048 KB, 4 ways, 64 byte lines
                0xD2 => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 4));
                }
                // code and data L3 cache, 1024 KB, 8 ways, 64 byte lines
                0xD6 => {
                    c.l3 = Some(CacheLevel::new(1024 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 2048 KB, 8 ways, 64 byte lines
                0xD7 => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 4096 KB, 8 ways, 64 byte lines
                0xD8 => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 8));
                }
                // code and data L3 cache, 1536 KB, 12 ways, 64 byte lines
                0xDC => {
                    c.l3 = Some(CacheLevel::new(1536 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 3072 KB, 12 ways, 64 byte lines
                0xDD => {
                    c.l3 = Some(CacheLevel::new(3072 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 6144 KB, 12 ways, 64 byte lines
                0xDE => {
                    c.l3 = Some(CacheLevel::new(6144 * 1024, CacheType::Unified, 12));
                }
                // code and data L3 cache, 2048 KB, 16 ways, 64 byte lines
                0xE2 => {
                    c.l3 = Some(CacheLevel::new(2048 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 4096 KB, 16 ways, 64 byte lines
                0xE3 => {
                    c.l3 = Some(CacheLevel::new(4096 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 8192 KB, 16 ways, 64 byte lines
                0xE4 => {
                    c.l3 = Some(CacheLevel::new(8192 * 1024, CacheType::Unified, 16));
                }
                // code and data L3 cache, 12288 KB, 24 ways, 64 byte lines
                0xEA => {
                    c.l3 = Some(CacheLevel::new(12288 * 1024, CacheType::Unified, 24));
                }
                // code and data L3 cache, 18432 KB, 24 ways, 64 byte lines
                0xEB => {
                    c.l3 = Some(CacheLevel::new(18432 * 1024, CacheType::Unified, 24));
                }
                // code and data L3 cache, 24576 KB, 24 ways, 64 byte lines
                0xEC => {
                    c.l3 = Some(CacheLevel::new(24576 * 1024, CacheType::Unified, 24));
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
            let cache_sets = res.ecx + 1;
            let cache_line_size = (res.ebx & 0xFFF) + 1;
            let cache_partitions = ((res.ebx >> 12) & 0x3FF) + 1;
            let cache_ways_of_associativity = ((res.ebx >> 22) & 0x3FF) + 1;

            let cache_size =
                cache_sets * cache_partitions * cache_ways_of_associativity * cache_line_size;

            match cache_type {
                DATA_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_data(cache_size, cache_ways_of_associativity);
                    }
                }
                INSTRUCTION_CACHE => {
                    if cache_level == 1 {
                        if c.l1.is_unified() {
                            c.l1 = Level1Cache::default_split();
                        }

                        c.l1.set_instruction(cache_size, cache_ways_of_associativity);
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
                        ));
                    }
                    L3 => {
                        c.l3 = Some(CacheLevel::new(
                            cache_size,
                            CacheType::Unified,
                            cache_ways_of_associativity,
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

#[derive(Debug, Default)]
#[cfg(not(target_os = "none"))]
pub struct Speed {
    pub base: u32,
    pub boost: u32,
    pub measured: bool,
}

#[cfg(not(target_os = "none"))]
impl Speed {
    pub fn detect() -> Self {
        match vendor_str().as_str() {
            VENDOR_INTEL => {
                if max_leaf() < LEAF_16 {
                    return Speed::measure();
                }

                let res = x86_cpuid(LEAF_16);

                let base = res.eax;
                let boost = res.ebx;

                if base == 0 {
                    return Speed::measure();
                }

                Speed {
                    base,
                    boost,
                    measured: false,
                }
            }
            _ => Speed::measure(),
        }
    }

    fn measure() -> Self {
        if !super::has_tsc() {
            return Speed::default();
        }

        let freq = measure_tsc_frequency();
        if freq == 0 {
            return Speed::default();
        }

        Speed {
            base: freq,
            boost: freq,
            measured: true,
        }
    }
}

#[cfg(not(target_os = "none"))]
fn measure_tsc_frequency() -> u32 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::_rdtsc as rdtsc;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::_rdtsc as rdtsc;

    const MHZ_DIVISOR: u64 = 1_000_000;

    use core::time::Duration;

    let start_tsc = unsafe { rdtsc() };
    let start_time = std::time::Instant::now();

    let end_time = start_time + Duration::from_millis(10);

    while std::time::Instant::now() < end_time {
        core::hint::spin_loop();
    }

    let end_tsc = unsafe { rdtsc() };

    let elapsed = start_time.elapsed().as_nanos() as u64;
    let tsc_delta = end_tsc - start_tsc;

    if elapsed == 0 {
        return 0;
    }

    let freq_mhz = (tsc_delta * MHZ_DIVISOR) / elapsed;

    (freq_mhz / 1000) as u32
}

#[derive(Debug, Default)]
pub struct Topology {
    pub cores: u32,
    pub threads: u32,

    #[cfg(not(target_os = "none"))]
    pub speed: Speed,

    pub cache: Option<Cache>,
}

impl Topology {
    pub fn detect_core_count() -> u32 {
        match vendor_str().as_str() {
            VENDOR_INTEL => {
                if max_leaf() < LEAF_4 {
                    return 1;
                }

                let res = x86_cpuid(LEAF_4);

                (res.ebx >> 26) + 1
            }
            VENDOR_AMD => {
                if get_ht() != 0 {
                    return super::logical_cores() / (get_ht() + 1);
                };

                1
            }
            _ => {
                if has_ht() {
                    return super::logical_cores() / (get_ht() + 1);
                };

                1
            }
        }
    }

    #[cfg(not(target_os = "none"))]
    pub fn detect() -> Self {
        let threads = super::logical_cores();
        let cores = Self::detect_core_count();
        let speed = Speed::detect();
        let cache = Cache::detect();

        Topology {
            cores,
            threads,
            speed,
            cache,
        }
    }

    #[cfg(target_os = "none")]
    pub fn detect() -> Self {
        let mut t = Topology::default();
        t.cache = Cache::detect();

        return t;
    }
}
