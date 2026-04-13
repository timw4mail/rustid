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

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn assoc(&self) -> u32 {
        self.assoc
    }

    pub fn kind(&self) -> CacheType {
        self.kind
    }

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

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[cfg(any(target_os = "linux", target_os = "windows", target_family = "unix"))]
impl Cache {
    pub fn detect() -> Option<Cache> {
        #[cfg(target_os = "linux")]
        {
            if let Some(cache) = Self::from_device_tree() {
                return Some(cache);
            }

            if let Some(cache) = Self::from_lscpu_command() {
                return Some(cache);
            }

            if let Some(cache) = Self::from_cpuinfo() {
                return Some(cache);
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn from_device_tree() -> Option<Cache> {
        use std::fs;
        use std::path::Path;

        // Try to read cache information from device tree
        let dt_root = Path::new("/proc/device-tree");
        if !dt_root.exists() {
            return None;
        }

        // Try to read cache properties
        let mut cache = Cache::default();
        let mut found_cache = false;

        // Try to read L1 cache
        if let Ok(l1_size) = fs::read_to_string(dt_root.join("l1-cache-size"))
            && let Ok(size) = l1_size.trim().parse::<u32>()
            && let Ok(l1_assoc) = fs::read_to_string(dt_root.join("l1-cache-associativity"))
            && let Ok(assoc) = l1_assoc.trim().parse::<u32>()
        {
            cache.l1 = Level1Cache::new_unified(size, assoc);
            found_cache = true;
        }

        // Try to read L2 cache
        if let Ok(l2_size) = fs::read_to_string(dt_root.join("l2-cache-size"))
            && let Ok(size) = l2_size.trim().parse::<u32>()
        {
            let mut l2_assoc = 0;
            if let Ok(assoc_str) = fs::read_to_string(dt_root.join("l2-cache-associativity"))
                && let Ok(assoc) = assoc_str.trim().parse::<u32>()
            {
                l2_assoc = assoc;
            }
            cache.l2 = Some(CacheLevel::new(size, CacheType::Unified, l2_assoc, 0));
            found_cache = true;
        }

        // Try to read L3 cache
        if let Ok(l3_size) = fs::read_to_string(dt_root.join("l3-cache-size"))
            && let Ok(size) = l3_size.trim().parse::<u32>()
        {
            let mut l3_assoc = 0;
            if let Ok(assoc_str) = fs::read_to_string(dt_root.join("l3-cache-associativity"))
                && let Ok(assoc) = assoc_str.trim().parse::<u32>()
            {
                l3_assoc = assoc;
            }
            cache.l3 = Some(CacheLevel::new(size, CacheType::Unified, l3_assoc, 0));
            found_cache = true;
        }

        if found_cache { Some(cache) } else { None }
    }

    #[cfg(target_os = "linux")]
    fn from_lscpu_command() -> Option<Cache> {
        let output = match std::process::Command::new("lscpu").arg("-C").output() {
            Ok(o) => o.stdout,
            Err(_) => return None,
        };

        let output_str = match String::from_utf8(output) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let mut cache = Cache::default();
        let mut found_cache = false;

        let lines: Vec<&str> = output_str.lines().collect();
        let table_keys: Vec<&str> = lines[0].split_whitespace().collect();

        for line in lines.into_iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let name = parts[table_keys.iter().position(|&x| x == "NAME")?];
            let size_str = parts[table_keys.iter().position(|&x| x == "ONE-SIZE")?];
            let ways_str = parts[table_keys.iter().position(|&x| x == "WAYS")?];

            // Parse size (e.g., "32K", "256K", "4M")
            let size_bytes: u32 = if let Some(stripped) = size_str.strip_suffix('K') {
                stripped.parse::<u32>().ok()? * 1024
            } else if let Some(stripped) = size_str.strip_suffix('M') {
                stripped.parse::<u32>().ok()? * 1024 * 1024
            } else {
                size_str.parse::<u32>().ok()? * 1024
            };

            let ways: u32 = ways_str.parse().unwrap_or(0);

            match name {
                "L1d" => {
                    cache.l1 = Level1Cache::Split {
                        data: CacheLevel::new(size_bytes, CacheType::Data, ways, 0),
                        instruction: CacheLevel::default(),
                    };
                    found_cache = true;
                }
                "L1i" => {
                    if let Level1Cache::Split { instruction, .. } = &mut cache.l1 {
                        instruction.size = size_bytes;
                        instruction.kind = CacheType::Instruction;
                        instruction.assoc = ways;
                    }
                }
                "L1" => {
                    cache.l1 = Level1Cache::Unified(CacheLevel::new_unified(size_bytes, ways));
                    found_cache = true;
                }
                "L2" => {
                    cache.l2 = Some(CacheLevel::new(size_bytes, CacheType::Unified, ways, 0));
                    found_cache = true;
                }
                "L3" => {
                    cache.l3 = Some(CacheLevel::new(size_bytes, CacheType::Unified, ways, 0));
                    found_cache = true;
                }
                _ => {}
            }
        }

        // Handle case where L1 is split but L1i wasn't in the output
        if let Level1Cache::Split { data, instruction } = &cache.l1
            && instruction.size == 0
            && data.size > 0
        {
            // Copy data settings to instruction
            cache.l1 = Level1Cache::Split {
                data: *data,
                instruction: CacheLevel::new(data.size, CacheType::Instruction, data.assoc, 0),
            };
        }

        if found_cache { Some(cache) } else { None }
    }

    #[cfg(target_os = "linux")]
    fn from_cpuinfo() -> Option<Cache> {
        use std::fs;

        let output = match fs::read_to_string("/proc/cpuinfo") {
            Ok(o) => o,
            Err(_) => return None,
        };

        let mut cache = Cache::default();
        let mut found_cache = false;
        let mut l1_size: u32 = 0;
        let mut l2_size: u32 = 0;
        let mut l2_assoc: u32 = 0;
        let mut l3_size: u32 = 0;
        let mut l3_assoc: u32 = 0;

        for line in output.lines() {
            let line = line.trim();

            // Look for cache size (typically reports L2 cache size in KB)
            if line.starts_with("cache") && line.contains("cache") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 2 {
                    continue;
                }

                // let key = parts[0].trim();
                let value = parts[1].trim().trim_end_matches(" KB");

                // Try to parse as L2 or L3 based on typicalPowerPC conventions
                // PowerPC /proc/cpuinfo often uses "cache size" for L2
                if let Ok(size) = value.parse::<u32>() {
                    let size_bytes = size * 1024;
                    if l2_size == 0 {
                        l2_size = size_bytes;
                        l2_assoc = 8; // Default assumption for PowerPC
                        found_cache = true;
                    } else if l3_size == 0 {
                        l3_size = size_bytes;
                        l3_assoc = 8;
                    }
                }
            }

            // Try to parse more specific cache info lines if available
            if (line.starts_with("l1-dcache-size") || line.starts_with("L1-dcache-size"))
                && let Some(size) = Self::parse_cache_value(line)
            {
                l1_size = size;
                if let Level1Cache::Split { data, .. } = &mut cache.l1 {
                    data.size = size;
                    found_cache = true;
                }
            }
            if (line.starts_with("l1-icache-size") || line.starts_with("L1-icache-size"))
                && let Some(size) = Self::parse_cache_value(line)
                && let Level1Cache::Split { instruction, .. } = &mut cache.l1
            {
                instruction.size = size;
                instruction.kind = CacheType::Instruction;
                found_cache = true;
            }
            if (line.starts_with("l2-cache-size") || line.starts_with("L2-cache-size"))
                && let Some(size) = Self::parse_cache_value(line)
            {
                l2_size = size;
                cache.l2 = Some(CacheLevel::new(size, CacheType::Unified, 8, 0));
                found_cache = true;
            }
            if (line.starts_with("l3-cache-size") || line.starts_with("L3-cache-size"))
                && let Some(size) = Self::parse_cache_value(line)
            {
                l3_size = size;
                cache.l3 = Some(CacheLevel::new(size, CacheType::Unified, 8, 0));
                found_cache = true;
            }
        }

        // If we found cache info from generic "cache size" but no structured info
        if found_cache && l1_size == 0 && l2_size > 0 {
            let l1_size = 32 * 1024; // Assume 32KB L1 for most PowerPC
            cache.l1 = Level1Cache::Split {
                data: CacheLevel::new(l1_size, CacheType::Data, 8, 0),
                instruction: CacheLevel::new(l1_size, CacheType::Instruction, 8, 0),
            };

            if l2_size > 0 {
                cache.l2 = Some(CacheLevel::new(l2_size, CacheType::Unified, l2_assoc, 0));
            }
            if l3_size > 0 {
                cache.l3 = Some(CacheLevel::new(l3_size, CacheType::Unified, l3_assoc, 0));
            }
        }

        if found_cache { Some(cache) } else { None }
    }

    #[cfg(target_os = "linux")]
    fn parse_cache_value(line: &str) -> Option<u32> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1]
            .trim()
            .trim_end_matches(" KB")
            .trim_end_matches("K");
        let size: u32 = value.parse().ok()?;
        Some(size * 1024)
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn from_windows() -> Option<Cache> {
        None
    }
}
