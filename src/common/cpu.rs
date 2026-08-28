use crate::common::cache::Cache;
use crate::common::topology::{Speed, Topology};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub trait TDetect {
    fn detect() -> Self;
}

#[derive(Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum CoreType {
    Super,
    #[default]
    Performance,
    Efficiency,
}

impl From<&str> for CoreType {
    fn from(val: &str) -> Self {
        match val {
            "Super" => CoreType::Super,
            "Performance" => CoreType::Performance,
            "Efficiency" => CoreType::Efficiency,
            _ => CoreType::Performance,
        }
    }
}

impl From<CoreType> for &str {
    fn from(val: CoreType) -> &'static str {
        match val {
            CoreType::Super => "Super",
            CoreType::Performance => "Performance",
            CoreType::Efficiency => "Efficiency",
        }
    }
}

impl From<String> for CoreType {
    fn from(val: String) -> Self {
        Self::from(val.as_str())
    }
}

/// Information about a specific core type/cluster in the CPU.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CpuCore<M = &'static str> {
    /// Classification of this core (Performance, Efficiency, Super)
    pub kind: CoreType,
    /// Microarchitecture variant of this core type
    pub micro_arch: M,
    /// Marketing or core codename (e.g., "Golden Cove", "Cortex-A78", "U74")
    pub name: Option<String>,
    /// Core implementer / designer (e.g., "ARM", "Nvidia", "Apple")
    pub implementer: Option<String>,
    /// Cache hierarchy specific to this core cluster
    pub cache: Option<Cache>,
    /// Clock speed for this specific core cluster (base and boost frequencies in MHz)
    pub speed: Option<Speed>,
    /// Number of physical cores in this cluster
    pub count: u32,
    /// Number of logical threads in this cluster
    pub threads: u32,
}

/// Unified CPU representation across all hardware architectures.
#[derive(Debug, Default, PartialEq)]
pub struct Cpu<E = (), M = &'static str> {
    /// The system name, if applicable
    pub system: Option<String>,
    /// CPU vendor name
    pub vendor: String,
    /// CPU model name
    pub model: String,
    /// CPU topology details (sockets, dies, cores, threads, speed, cache)
    pub topology: Topology,
    /// Per-core-cluster breakdown of CPU cores
    pub cores: Vec<CpuCore<M>>,
    /// Detected CPU features
    pub features: BTreeMap<&'static str, String>,
    /// Architecture-specific extension data
    pub extra: E,
}

impl<E, M> Cpu<E, M> {
    /// Total sockets (at least 1)
    pub fn total_sockets(&self) -> u32 {
        self.topology.sockets.count.max(1)
    }

    /// Returns true if this CPU has multiple core types (hybrid architecture).
    pub fn is_hybrid(&self) -> bool {
        self.cores.len() > 1
    }

    /// Total physical cores across all clusters
    pub fn total_cores(&self) -> u32 {
        let sum: u32 = self.cores.iter().map(|c| c.count).sum();
        if sum > 0 {
            sum
        } else {
            self.topology.cores.count.max(1)
        }
    }

    /// Total logical threads across all clusters
    pub fn total_threads(&self) -> u32 {
        let sum: u32 = self.cores.iter().map(|c| c.threads).sum();
        if sum > 0 {
            sum
        } else {
            self.topology.threads.count.max(1)
        }
    }
}

impl<E, M> core::ops::Deref for Cpu<E, M> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.extra
    }
}

impl<E, M> core::ops::DerefMut for Cpu<E, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.extra
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_type_from_str_performance() {
        assert_eq!(CoreType::from("Performance"), CoreType::Performance);
    }

    #[test]
    fn test_core_type_from_str_efficiency() {
        assert_eq!(CoreType::from("Efficiency"), CoreType::Efficiency);
    }

    #[test]
    fn test_core_type_from_str_super() {
        assert_eq!(CoreType::from("Super"), CoreType::Super);
    }

    #[test]
    fn test_core_type_from_str_unknown_defaults_to_performance() {
        assert_eq!(CoreType::from("Unknown"), CoreType::Performance);
    }

    #[test]
    fn test_core_type_from_string() {
        let s = String::from("Efficiency");
        assert_eq!(CoreType::from(s), CoreType::Efficiency);
    }

    #[test]
    fn test_core_type_into_str() {
        let s: &str = CoreType::Super.into();
        assert_eq!(s, "Super");
        let s: &str = CoreType::Performance.into();
        assert_eq!(s, "Performance");
        let s: &str = CoreType::Efficiency.into();
        assert_eq!(s, "Efficiency");
    }
}
