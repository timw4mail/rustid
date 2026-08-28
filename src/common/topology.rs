use crate::common::cache::Cache;

/// CPU speed information (base and boost frequencies).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Speed {
    /// Base frequency in MHz
    pub base: u32,
    /// Boost frequency in MHz
    pub boost: u32,
    /// Whether the frequency was measured (vs reported by CPU)
    pub measured: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TopologyTier {
    pub count: u32,
    pub source: DataSource,
}

impl TopologyTier {
    pub fn new(count: u32, source: DataSource) -> Self {
        Self { count, source }
    }
}

impl Default for TopologyTier {
    fn default() -> Self {
        Self {
            count: 1,
            source: DataSource::default(),
        }
    }
}

/// Complete CPU topology information including sockets, dies, cores, threads, speed, and cache.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Topology {
    /// Number of processor sockets
    pub sockets: TopologyTier,
    /// Number of dies per socket
    pub dies: TopologyTier,
    /// Number of physical cores
    pub cores: TopologyTier,
    /// Number of logical threads (includes SMT)
    pub threads: TopologyTier,
    /// CPU speed information
    pub speed: Speed,
    /// Cache hierarchy information
    pub cache: Option<Cache>,
}

impl Topology {
    pub fn new(sockets: u32, cores: u32, threads: u32) -> Self {
        Self {
            sockets: TopologyTier::new(sockets, DataSource::default()),
            cores: TopologyTier::new(cores, DataSource::default()),
            threads: TopologyTier::new(threads, DataSource::default()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TopologyCount {
    pub sockets: TopologyTier,
    pub cores: u32,
    pub threads: u32,
    pub source: DataSource,
}

impl Default for TopologyCount {
    fn default() -> Self {
        TopologyCount {
            sockets: TopologyTier::default(),
            cores: 1,
            threads: 1,
            source: DataSource::DefaultValue,
        }
    }
}

/// Where did this cpu information come from?
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum DataSource {
    /// A default value , when lookup fails
    #[default]
    DefaultValue,
    /// Value from Android getprop shell tool
    AndroidGetprop,
    /// Value generated from other inputs
    Calculated(&'static str),
    /// x86 cpuid instruction
    Cpuid,
    /// x86 cpuid instruction dump
    CpuidDump,
    /// Magic values from the cpu that need to be mapped to a readable value
    CpuLookupTable,
    /// model-specific registers (MSR)
    CpuMsr,
    /// value in cpu register on cpu reset
    CpuReset,
    /// from device tree
    DeviceTree,
    /// sysinfo command on Haiku
    HaikuSysinfo,
    /// /proc/cpuinfo
    LinuxProcCpuinfo,
    /// Linux virtual /sys directory tree
    LinuxSysFs,
    /// Determined from a set of pre-defined values
    LookupTable,
    /// Linux lscpu command
    Lscpu,
    /// x86 MpTable
    MpTable,
    /// value from sysctrl tool
    Sysctrl(&'static str),
    /// value from system call
    SystemCall,
    /// value from Windows registry
    WindowsRegistry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_tier_new() {
        let t = TopologyTier::new(4, DataSource::Cpuid);
        assert_eq!(t.count, 4);
        assert_eq!(t.source, DataSource::Cpuid);
    }

    #[test]
    fn test_topology_tier_default() {
        let t = TopologyTier::default();
        assert_eq!(t.count, 1);
        assert_eq!(t.source, DataSource::DefaultValue);
    }

    #[test]
    fn test_speed_default() {
        let s = Speed::default();
        assert_eq!(s.base, 0);
        assert_eq!(s.boost, 0);
        assert!(!s.measured);
    }

    #[test]
    fn test_speed_values() {
        let s = Speed {
            base: 2400,
            boost: 5000,
            measured: false,
        };
        assert_eq!(s.base, 2400);
        assert_eq!(s.boost, 5000);
        assert!(!s.measured);
    }

    #[test]
    fn test_speed_measured() {
        let s = Speed {
            base: 3000,
            boost: 3000,
            measured: true,
        };
        assert!(s.measured);
    }
}
