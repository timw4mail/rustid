use super::*;
use crate::common::cache::*;

const DATA_CACHE: u32 = 1;
const INSTRUCTION_CACHE: u32 = 2;
const UNIFIED_CACHE: u32 = 3;

const L1: u32 = 1;
const L2: u32 = 2;
const L3: u32 = 3;

#[derive(Copy, Clone)]
enum CacheDescTarget {
    L1Instruction(u32, u32),
    L1Data(u32, u32),
    L2(u32, u32),
    L3(u32, u32),
}

impl Cache {
    pub fn new(l1: Level1Cache, l2: Option<CacheLevel>, l3: Option<CacheLevel>) -> Cache {
        Cache { l1, l2, l3 }
    }

    /// Detects and returns the cache configuration for this CPU.
    ///
    /// Returns `None` if cache information cannot be determined.
    pub fn detect() -> Option<Self> {
        if !has_cpuid() {
            return None;
        }

        match &*vendor_str() {
            VENDOR_AMD => match is_valid_leaf(EXT_LEAF_1D) {
                true => Cache::detect_general(EXT_LEAF_1D),
                false => Cache::detect_ext_5_6(),
            },
            VENDOR_CENTAUR => match is_valid_leaf(LEAF_4) {
                true => Cache::detect_general(LEAF_4),
                false => Cache::detect_ext_5_6(),
            },
            VENDOR_TRANSMETA => match is_valid_leaf(EXT_LEAF_6) {
                true => Cache::detect_ext_5_6(),
                false => Cache::detect_fallback(),
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

    /// Detect cache via extended leaves 5 and 6.
    fn detect_ext_5_6() -> Option<Self> {
        if !is_valid_leaf(EXT_LEAF_5) {
            return None;
        }

        let mut c = Cache {
            l1: Level1Cache::default_split(),
            l2: None,
            l3: None,
        };

        let res5 = x86_cpuid(EXT_LEAF_5);
        let l1dassoc = Self::assoc((res5.ecx >> 16) & 0x1F);
        let l1iassoc = Self::assoc((res5.edx >> 16) & 0x1F);
        c.l1.set_data((res5.ecx >> 24) * 1024, l1dassoc);
        c.l1.set_instruction((res5.edx >> 24) * 1024, l1iassoc);

        // Make sure to check for Extended Leaf 6 separately
        // Some CPUs only support EXT_LEAF_5
        if !is_valid_leaf(EXT_LEAF_6) {
            return Some(c);
        }

        let res6 = x86_cpuid(EXT_LEAF_6);

        let afn = match &*vendor_str() {
            VENDOR_AMD => Self::amd_assoc,
            _ => Self::assoc,
        };

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

    fn assoc(reg: u32) -> u32 {
        match reg {
            0xF => 1,
            n => n,
        }
    }

    /// See https://www.amd.com/content/dam/amd/en/documents/archived-tech-docs/design-guides/25481.pdf
    fn amd_assoc(reg: u32) -> u32 {
        match reg {
            2 => 2,
            4 => 4,
            6 => 8,
            8 => 16,
            10 => 32,
            11 => 48,
            12 => 64,
            13 => 96,
            14 => 128,

            // I don't care about the cpu being directly mapped, or fully associative
            1 | 15 => 1,
            _ => 0,
        }
    }

    /// Get cache information via 1-bit descriptors
    ///
    /// See https://sandpile.org/x86/cpuid.htm#level_0000_0002h
    fn detect_fallback() -> Option<Self> {
        use super::StaticVec;

        if !is_valid_leaf(LEAF_2) {
            return None;
        }

        let mut c = Cache::default();

        let res = x86_cpuid(LEAF_2);
        let iteration_count = res.eax & 0xFF;
        let mut desc_list: StaticVec<u32, 32> = StaticVec::new();

        for i in 0..=iteration_count {
            let res = x86_cpuid_count(LEAF_2, i);
            let valid_eax = (res.eax >> 31) == 0;
            if !valid_eax {
                break;
            }

            for offset in [8u32, 16, 24] {
                let desc = (res.eax >> offset) & 0xFF;
                if desc != 0 {
                    desc_list.push(desc);
                }
            }

            let valid_ebx = (res.ebx >> 31) == 0;
            if !valid_ebx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.ebx >> offset) & 0xFF;
                if desc != 0 {
                    desc_list.push(desc);
                }
            }

            let valid_ecx = (res.ecx >> 31) == 0;
            if !valid_ecx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.ecx >> offset) & 0xFF;
                if desc != 0 {
                    desc_list.push(desc);
                }
            }

            let valid_edx = (res.edx >> 31) == 0;
            if !valid_edx {
                break;
            }
            for offset in [0u32, 8, 16, 24] {
                let desc = (res.edx >> offset) & 0xFF;
                if desc != 0 {
                    desc_list.push(desc);
                }
            }
        }

        for desc in desc_list.iter() {
            Self::apply_descriptor(*desc, &mut c);
        }

        if c == Cache::default() {
            None
        } else {
            Some(c)
        }
    }

    const fn get_cache_desc_info(desc: u32) -> Option<CacheDescTarget> {
        match desc {
            // L1 Instruction cache
            // 8KB, 4-way
            0x06 => Some(CacheDescTarget::L1Instruction(8 * 1024, 4)),
            // 16KB, 4-way
            0x08 => Some(CacheDescTarget::L1Instruction(16 * 1024, 4)),
            // 32KB, 4-way
            0x09 => Some(CacheDescTarget::L1Instruction(32 * 1024, 4)),
            // 32KB, 8-way
            0x30 => Some(CacheDescTarget::L1Instruction(32 * 1024, 8)),

            // L1 Data cache
            // 8KB, 2-way
            0x0A => Some(CacheDescTarget::L1Data(8 * 1024, 2)),
            // 8KB, 4-way
            0x66 => Some(CacheDescTarget::L1Data(8 * 1024, 4)),
            // 16KB, 4-way
            0x0C | 0x0D | 0x67 => Some(CacheDescTarget::L1Data(16 * 1024, 4)),
            // 16KB, 8-way
            0x60 => Some(CacheDescTarget::L1Data(16 * 1024, 8)),
            // 24KB, 6-way
            0x0E => Some(CacheDescTarget::L1Data(24 * 1024, 6)),
            // 32KB, 4-way
            0x68 => Some(CacheDescTarget::L1Data(32 * 1024, 4)),
            // 32KB, 8-way
            0x2C => Some(CacheDescTarget::L1Data(32 * 1024, 8)),

            // L2 cache
            // 128KB, 2-way
            0x1D | 0x3B => Some(CacheDescTarget::L2(128 * 1024, 2)),
            // 128KB, 4-way
            0x39 | 0x41 => Some(CacheDescTarget::L2(128 * 1024, 4)),
            // 128KB, 8-way
            0x79 | 0x81 => Some(CacheDescTarget::L2(128 * 1024, 8)),
            // 192KB, 6-way
            0x3A => Some(CacheDescTarget::L2(192 * 1024, 6)),
            // 256KB, 2-way
            0x7F => Some(CacheDescTarget::L2(256 * 1024, 2)),
            // 256KB, 4-way
            0x3C | 0x42 => Some(CacheDescTarget::L2(256 * 1024, 4)),
            // 256KB, 8-way
            0x21 | 0x7A | 0x82 => Some(CacheDescTarget::L2(256 * 1024, 8)),
            // 384KB, 6-way
            0x3D => Some(CacheDescTarget::L2(384 * 1024, 6)),
            // 512KB, 2-way
            0x3E => Some(CacheDescTarget::L2(512 * 1024, 2)),
            // 512KB, 4-way
            0x43 | 0x78 | 0x86 => Some(CacheDescTarget::L2(512 * 1024, 4)),
            // 512KB, 8-way
            0x7B | 0x80 | 0x83 => Some(CacheDescTarget::L2(512 * 1024, 8)),
            // 1024KB, 4-way
            0x44 | 0x7C => Some(CacheDescTarget::L2(1024 * 1024, 4)),
            // 1024KB, 8-way
            0x84 | 0x87 => Some(CacheDescTarget::L2(1024 * 1024, 8)),
            // 1024KB, 16-way
            0x24 => Some(CacheDescTarget::L2(1024 * 1024, 16)),
            // 2048KB, 4-way
            0x45 => Some(CacheDescTarget::L2(2048 * 1024, 4)),
            // 2048KB, 8-way
            0x7D | 0x85 => Some(CacheDescTarget::L2(2048 * 1024, 8)),
            // 3072KB, 12-way
            0x48 => Some(CacheDescTarget::L2(3072 * 1024, 12)),

            // L3 cache
            // 512KB, 4-way
            0x22 | 0xD0 => Some(CacheDescTarget::L3(512 * 1024, 4)),
            // 512KB, 8-way
            0xD6 => Some(CacheDescTarget::L3(512 * 1024, 8)),
            // 1024KB, 4-way
            0x23 | 0xD1 => Some(CacheDescTarget::L3(1024 * 1024, 4)),
            // 1024KB, 8-way
            0xD7 => Some(CacheDescTarget::L3(1024 * 1024, 8)),
            // 1536KB, 12-way
            0xDC => Some(CacheDescTarget::L3(1536 * 1024, 12)),
            // 2048KB, 4-way
            0xD2 => Some(CacheDescTarget::L3(2048 * 1024, 4)),
            // 2048KB, 8-way
            0x25 => Some(CacheDescTarget::L3(2048 * 1024, 8)),
            // 2048KB, 16-way
            0x4E | 0xE2 => Some(CacheDescTarget::L3(2048 * 1024, 16)),
            // 3072KB, 12-way
            0xDD => Some(CacheDescTarget::L3(3072 * 1024, 12)),
            // 4096KB, 4-way
            0x29 | 0x46 => Some(CacheDescTarget::L3(4096 * 1024, 4)),
            // 4096KB, 8-way
            0xD8 => Some(CacheDescTarget::L3(4096 * 1024, 8)),
            // 4096KB, 16-way
            0x4F | 0xE3 => Some(CacheDescTarget::L3(4096 * 1024, 16)),
            // 6144KB, 12-way
            0x4A | 0xDE => Some(CacheDescTarget::L3(6144 * 1024, 12)),
            // 8192KB, 8-way
            0x47 => Some(CacheDescTarget::L3(8192 * 1024, 8)),
            // 8192KB, 16-way
            0x4B | 0xE4 => Some(CacheDescTarget::L3(8192 * 1024, 16)),
            // 12288KB, 12-way
            0x4C | 0xEA => Some(CacheDescTarget::L3(12288 * 1024, 12)),
            // 16384KB, 16-way
            0x4D => Some(CacheDescTarget::L3(16384 * 1024, 16)),
            // 18432KB, 24-way
            0xEB => Some(CacheDescTarget::L3(18432 * 1024, 24)),
            // 24576KB, 24-way
            0xEC => Some(CacheDescTarget::L3(24576 * 1024, 24)),

            _ => None,
        }
    }

    fn apply_descriptor(desc: u32, c: &mut Cache) {
        match desc {
            0x49 => {
                let sig = CpuSignature::detect();
                match (sig.family, sig.extended_model, sig.model) {
                    (6, 0, 13..) | (6, 1, 7) => {
                        c.l2 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                    }
                    _ => {
                        c.l3 = Some(CacheLevel::no_count(4096 * 1024, CacheType::Unified, 16));
                        return;
                    }
                }
            }
            0x80 => {
                #[cfg(target_arch = "x86")]
                if is_cyrix() {
                    c.l1 = Level1Cache::Unified(CacheLevel::no_count(
                        16 * 1024,
                        CacheType::Unified,
                        4,
                    ));
                    return;
                }
                c.l2 = Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8));
            }
            _ => {}
        }

        if let Some(info) = Self::get_cache_desc_info(desc) {
            match info {
                CacheDescTarget::L1Instruction(size, assoc) => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_instruction(size, assoc);
                }
                CacheDescTarget::L1Data(size, assoc) => {
                    if c.l1.is_unified() {
                        c.l1 = Level1Cache::default_split();
                    }
                    c.l1.set_data(size, assoc);
                }
                CacheDescTarget::L2(size, assoc) => {
                    c.l2 = Some(CacheLevel::no_count(size, CacheType::Unified, assoc));
                }
                CacheDescTarget::L3(size, assoc) => {
                    c.l3 = Some(CacheLevel::no_count(size, CacheType::Unified, assoc));
                }
            }
        }
    }

    /// Cache detection via deterministic cache parameters
    /// EXT_LEAF_1D for AMD
    /// LEAF_4 for Intel
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

        if c == Cache::default() {
            None
        } else {
            Some(c)
        }
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
    fn test_assoc() {
        assert_eq!(Cache::assoc((0x40040140 >> 16) & 0x1F), 4);
    }

    #[test]
    fn test_amd_assoc() {
        assert_eq!(Cache::amd_assoc(0), 0);
        assert_eq!(Cache::amd_assoc((0x00010000 >> 16) & 0xF), 1);
        assert_eq!(Cache::amd_assoc((0x00020000 >> 16) & 0xF), 2);
        assert_eq!(Cache::amd_assoc((0x00030000 >> 16) & 0xF), 0);
        assert_eq!(Cache::amd_assoc((0x00040000 >> 16) & 0xF), 4);
        assert_eq!(Cache::amd_assoc((0x00060000 >> 16) & 0xF), 8);
        assert_eq!(Cache::amd_assoc((0x000F0000 >> 16) & 0xF), 1);
    }

    #[test]
    #[ignore]
    fn test_amd_assoc_k5() {
        assert_eq!(Cache::amd_assoc((0x20020220 >> 16) & 0xF), 4);
    }
}
