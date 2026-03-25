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
    /// Creates a new CacheLevel with the specified parameters.
    pub fn new(size: u32, kind: CacheType, assoc: u32, share_count: u32) -> Self {
        CacheLevel {
            size,
            kind,
            assoc,
            share_count,
        }
    }

    /// Creates a new CacheLevel without share count information.
    pub fn no_count(size: u32, kind: CacheType, assoc: u32) -> Self {
        Self::new(size, kind, assoc, 0)
    }

    /// Creates a new unified CacheLevel.
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
    /// Creates a new unified L1 cache.
    pub fn new_unified(size: u32, assoc: u32) -> Self {
        Level1Cache::Unified(CacheLevel::new_unified(size, assoc))
    }

    /// Returns true if the L1 cache is unified.
    pub fn is_unified(&self) -> bool {
        match self {
            Level1Cache::Unified(_) => true,
            Level1Cache::Split { .. } => false,
        }
    }

    /// Returns true if the L1 cache is split (separate I-cache and D-cache).
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
    pub fn default_split() -> Self {
        Level1Cache::Split {
            data: CacheLevel::default(),
            instruction: CacheLevel::default(),
        }
    }
    /// Returns the total size of the L1 cache in bytes.
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
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Cache {
    pub l1: Level1Cache,
    pub l2: Option<CacheLevel>,
    pub l3: Option<CacheLevel>,
}
