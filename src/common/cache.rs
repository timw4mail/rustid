use super::DataSource;

/// Cache type enumeration.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum CacheType {
    Unified,
    Data,
    Instruction,
    #[default]
    Invalid,
}

/// Represents a single level of cache (L1, L2, or L3).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CacheLevel {
    /// Cache associativity (number of ways)
    pub(crate) assoc: u32,
    /// Cache size in bytes
    pub(crate) size: u32,
    /// Type of cache (data, instruction, or unified)
    pub(crate) kind: CacheType,
    /// Number of cores sharing this cache
    pub(crate) share_count: u32,
}

impl CacheLevel {
    /// Creates a new `CacheLevel` with the specified parameters.
    #[must_use]
    pub fn new(size: u32, kind: CacheType, assoc: u32, share_count: u32) -> Self {
        CacheLevel {
            assoc,
            size,
            kind,
            share_count,
        }
    }

    /// Creates a new `CacheLevel` without share count information.
    #[must_use]
    pub fn no_count(size: u32, kind: CacheType, assoc: u32) -> Self {
        Self::new(size, kind, assoc, 0)
    }

    /// Creates a new unified `CacheLevel`.
    #[must_use]
    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Self::new(size, CacheType::Unified, assoc, 0)
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }

    #[must_use]
    pub fn assoc(&self) -> u32 {
        self.assoc
    }

    #[must_use]
    pub fn kind(&self) -> CacheType {
        self.kind
    }

    #[must_use]
    pub fn share_count(&self) -> u32 {
        self.share_count
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
    /// Creates a new unified L1 cache.
    #[must_use]
    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Level1Cache::Unified(CacheLevel::new_unified(size, assoc))
    }

    /// Returns true if the L1 cache is unified.
    #[must_use]
    pub fn is_unified(&self) -> bool {
        match self {
            Level1Cache::Unified(_) => true,
            Level1Cache::Split { .. } => false,
        }
    }

    /// Returns true if the L1 cache is split (separate I-cache and D-cache).
    #[must_use]
    pub fn is_split(&self) -> bool {
        !self.is_unified()
    }

    /// Sets the data cache size and associativity.
    pub fn set_data(&mut self, size: u32, assoc: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.size = size;
            data.kind = CacheType::Data;
            data.assoc = assoc;
        }
    }

    /// Sets the data cache share count (number of cores sharing the cache).
    pub fn set_data_share_count(&mut self, share_count: u32) {
        if let Level1Cache::Split { data, .. } = self {
            data.share_count = share_count;
        }
    }

    /// Sets the instruction cache size and associativity.
    pub fn set_instruction(&mut self, size: u32, assoc: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.size = size;
            instruction.kind = CacheType::Instruction;
            instruction.assoc = assoc;
        }
    }

    /// Sets the instruction cache share count (number of cores sharing the cache).
    pub fn set_instruction_share_count(&mut self, share_count: u32) {
        if let Level1Cache::Split { instruction, .. } = self {
            instruction.share_count = share_count;
        }
    }

    /// Creates a default split L1 cache configuration.
    #[must_use]
    pub fn default_split() -> Self {
        Level1Cache::Split {
            data: CacheLevel::default(),
            instruction: CacheLevel::default(),
        }
    }
    /// Returns the total size of the L1 cache in bytes.
    #[must_use]
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

/// Complete cache hierarchy information for a processor.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Cache {
    pub l1: Level1Cache,
    pub l2: Option<CacheLevel>,
    pub l3: Option<CacheLevel>,
    pub source: DataSource,
}

impl Cache {
    /// Detects cache using platform/OS specific information sources.
    #[must_use]
    pub fn detect_os() -> Option<Cache> {
        #[cfg(x86_cpu)]
        {
            if crate::x86::provider::info_source()
                == crate::x86::provider::CpuidInfoSource::DumpFile
            {
                return None;
            }
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        {
            Cache::from_sys_fs()
        }
        #[cfg(not(any(target_os = "android", target_os = "linux")))]
        {
            None
        }
    }

    /// Returns true if any active cache level is missing share_count information (share_count == 0).
    #[must_use]
    pub fn needs_share_count_fallback(&self) -> bool {
        match self.l1 {
            Level1Cache::Unified(l1) => {
                if l1.size > 0 && l1.share_count == 0 {
                    return true;
                }
            }
            Level1Cache::Split { data, instruction } => {
                if (data.size > 0 && data.share_count == 0)
                    || (instruction.size > 0 && instruction.share_count == 0)
                {
                    return true;
                }
            }
        }
        if let Some(l2) = self.l2
            && l2.size > 0
            && l2.share_count == 0
        {
            return true;
        }
        if let Some(l3) = self.l3
            && l3.size > 0
            && l3.share_count == 0
        {
            return true;
        }
        false
    }

    /// Merges missing share counts from another Cache (e.g. from platform/OS sources).
    pub fn merge_share_counts(&mut self, other: &Cache) {
        match (&mut self.l1, &other.l1) {
            (Level1Cache::Unified(l1), Level1Cache::Unified(ol1)) => {
                if l1.share_count == 0 && ol1.share_count > 0 {
                    l1.share_count = ol1.share_count;
                }
            }
            (
                Level1Cache::Split { data, instruction },
                Level1Cache::Split {
                    data: odata,
                    instruction: oinstruction,
                },
            ) => {
                if data.share_count == 0 && odata.share_count > 0 {
                    data.share_count = odata.share_count;
                }
                if instruction.share_count == 0 && oinstruction.share_count > 0 {
                    instruction.share_count = oinstruction.share_count;
                }
            }
            _ => {}
        }
        if let Some(l2) = &mut self.l2
            && l2.share_count == 0
            && let Some(ol2) = &other.l2
            && ol2.share_count > 0
        {
            l2.share_count = ol2.share_count;
        }
        if let Some(l3) = &mut self.l3
            && l3.share_count == 0
            && let Some(ol3) = &other.l3
            && ol3.share_count > 0
        {
            l3.share_count = ol3.share_count;
        }
    }

    /// Resolves missing share counts (`share_count == 0`) using platform-specific information
    /// sources first, and then applying final fallback rules.
    pub fn resolve_share_counts(&mut self, core_count: u32, thread_count: u32, socket_count: u32) {
        // 1. Try platform-specific information sources if share counts are missing
        if self.needs_share_count_fallback()
            && let Some(platform_cache) = Self::detect_os()
        {
            self.merge_share_counts(&platform_cache);
        }

        // 2. Final fallback logic for any remaining missing share counts
        self.resolve_share_counts_final_fallback(core_count, thread_count, socket_count);
    }

    /// Applies the final fallback rules for any remaining missing share counts:
    /// - L1i & L1d = core-count (per-core)
    /// - If L3, L2 = core-count (per-core)
    /// - Otherwise (L3 or L2 without L3) = shared per socket
    pub fn resolve_share_counts_final_fallback(
        &mut self,
        core_count: u32,
        thread_count: u32,
        socket_count: u32,
    ) {
        let core_count = core_count.max(1);
        let thread_count = thread_count.max(core_count);
        let socket_count = socket_count.max(1);

        let smt_width = (thread_count / core_count).max(1);
        let threads_per_socket = (thread_count / socket_count).max(1);
        let has_l3 = self.l3.as_ref().is_some_and(|l| l.size > 0);

        match &mut self.l1 {
            Level1Cache::Unified(l1) => {
                if l1.size > 0 && l1.share_count == 0 {
                    l1.share_count = smt_width;
                }
            }
            Level1Cache::Split { data, instruction } => {
                if data.size > 0 && data.share_count == 0 {
                    data.share_count = smt_width;
                }
                if instruction.size > 0 && instruction.share_count == 0 {
                    instruction.share_count = smt_width;
                }
            }
        }

        if let Some(l2) = &mut self.l2
            && l2.size > 0
            && l2.share_count == 0
        {
            if has_l3 {
                // "If L3, L2 = core-count" (per-core)
                l2.share_count = smt_width;
            } else {
                l2.share_count = threads_per_socket;
            }
        }

        if let Some(l3) = &mut self.l3
            && l3.size > 0
            && l3.share_count == 0
        {
            l3.share_count = threads_per_socket;
        }
    }
}

#[cfg(not(x86_cpu))]
impl Cache {
    #[cfg(not(any(target_os = "android", target_os = "linux")))]
    pub fn detect() -> Option<Cache> {
        None
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn from_windows() -> Option<Cache> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_level_no_count() {
        let cl = CacheLevel::no_count(512 * 1024, CacheType::Unified, 8);
        assert_eq!(cl.size, 512 * 1024);
        assert_eq!(cl.kind, CacheType::Unified);
        assert_eq!(cl.assoc, 8);
        assert_eq!(cl.share_count, 0);
    }

    #[test]
    fn test_cache_level_new_unified() {
        let cl = CacheLevel::new_unified(1024 * 1024, 16);
        assert_eq!(cl.size, 1024 * 1024);
        assert_eq!(cl.kind, CacheType::Unified);
        assert_eq!(cl.assoc, 16);
        assert_eq!(cl.share_count, 0);
    }

    #[test]
    fn test_cache_level_getters() {
        let cl = CacheLevel::new(8192, CacheType::Data, 4, 2);
        assert_eq!(cl.size(), 8192);
        assert_eq!(cl.assoc(), 4);
        assert_eq!(cl.kind(), CacheType::Data);
        assert_eq!(cl.share_count(), 2);
    }

    #[test]
    fn test_cache_level_default() {
        let cl = CacheLevel::default();
        assert_eq!(cl.size, 0);
        assert_eq!(cl.kind, CacheType::Invalid);
        assert_eq!(cl.assoc, 0);
        assert_eq!(cl.share_count, 0);
    }

    #[test]
    fn test_l1_cache_set_share_counts() {
        let mut l1 = Level1Cache::default_split();
        l1.set_data(16384, 8);
        l1.set_instruction(16384, 4);
        l1.set_data_share_count(2);
        l1.set_instruction_share_count(2);

        if let Level1Cache::Split { data, instruction } = l1 {
            assert_eq!(data.share_count, 2);
            assert_eq!(instruction.share_count, 2);
        } else {
            panic!("Expected split cache");
        }
    }

    #[test]
    fn test_l1_cache_default() {
        let l1 = Level1Cache::default();
        assert!(l1.is_unified());
        assert_eq!(l1.size(), 0);
    }

    #[test]
    fn test_cache_default() {
        let c = Cache::default();
        assert!(c.l1.is_unified());
        assert_eq!(c.l1.size(), 0);
        assert!(c.l2.is_none());
        assert!(c.l3.is_none());
    }

    #[test]
    fn test_l1_is_split() {
        let split = Level1Cache::default_split();
        assert!(split.is_split());
        assert!(!split.is_unified());
    }

    #[test]
    fn test_l1_is_unified() {
        let unified = Level1Cache::new_unified(1024, 4);
        assert!(unified.is_unified());
        assert!(!unified.is_split());
    }

    #[test]
    fn test_level1_cache_debug() {
        let l1 = Level1Cache::new_unified(4096, 2);
        assert_eq!(l1.size(), 4096);
    }

    #[test]
    fn test_cache_type_debug() {
        assert_eq!(format!("{:?}", CacheType::Unified), "Unified");
        assert_eq!(format!("{:?}", CacheType::Data), "Data");
        assert_eq!(format!("{:?}", CacheType::Instruction), "Instruction");
        assert_eq!(format!("{:?}", CacheType::Invalid), "Invalid");
    }

    #[test]
    fn test_resolve_share_counts_no_l3() {
        let mut c = Cache {
            l1: Level1Cache::Split {
                data: CacheLevel::no_count(32768, CacheType::Data, 8),
                instruction: CacheLevel::no_count(32768, CacheType::Instruction, 8),
            },
            l2: Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8)),
            l3: None,
            source: DataSource::DefaultValue,
        };

        // 4 cores, 4 threads, 1 socket (SMT = 1)
        c.resolve_share_counts_final_fallback(4, 4, 1);

        if let Level1Cache::Split { data, instruction } = c.l1 {
            assert_eq!(data.share_count, 1); // L1d = core-count (1 thread per core)
            assert_eq!(instruction.share_count, 1); // L1i = core-count (1 thread per core)
        } else {
            panic!("Expected split cache");
        }
        assert_eq!(c.l2.expect("L2 expected").share_count, 4); // L2 = per-socket (4 threads per socket)
    }

    #[test]
    fn test_resolve_share_counts_with_l3() {
        let mut c = Cache {
            l1: Level1Cache::Split {
                data: CacheLevel::no_count(32768, CacheType::Data, 8),
                instruction: CacheLevel::no_count(32768, CacheType::Instruction, 8),
            },
            l2: Some(CacheLevel::no_count(512 * 1024, CacheType::Unified, 8)),
            l3: Some(CacheLevel::no_count(
                8 * 1024 * 1024,
                CacheType::Unified,
                16,
            )),
            source: DataSource::DefaultValue,
        };

        // 4 cores, 8 threads, 1 socket (SMT = 2)
        c.resolve_share_counts_final_fallback(4, 8, 1);

        if let Level1Cache::Split { data, instruction } = c.l1 {
            assert_eq!(data.share_count, 2); // L1d = 2 threads per core
            assert_eq!(instruction.share_count, 2); // L1i = 2 threads per core
        } else {
            panic!("Expected split cache");
        }
        assert_eq!(c.l2.expect("L2 expected").share_count, 2); // L2 = per-core (2 threads per core, because L3 exists)
        assert_eq!(c.l3.expect("L3 expected").share_count, 8); // L3 = per-socket (8 threads per socket)
    }

    #[test]
    fn test_resolve_share_counts_already_set() {
        let mut c = Cache {
            l1: Level1Cache::Split {
                data: CacheLevel::new(32768, CacheType::Data, 8, 2),
                instruction: CacheLevel::new(32768, CacheType::Instruction, 8, 2),
            },
            l2: Some(CacheLevel::new(512 * 1024, CacheType::Unified, 8, 2)),
            l3: Some(CacheLevel::new(8 * 1024 * 1024, CacheType::Unified, 16, 8)),
            source: DataSource::DefaultValue,
        };

        c.resolve_share_counts_final_fallback(4, 8, 1);

        if let Level1Cache::Split { data, instruction } = c.l1 {
            assert_eq!(data.share_count, 2);
            assert_eq!(instruction.share_count, 2);
        }
        assert_eq!(c.l2.expect("L2 expected").share_count, 2);
        assert_eq!(c.l3.expect("L3 expected").share_count, 8);
    }
}
