use crate::common::cache::Cache;
use crate::common::combine_vendor_and_model;
use alloc::string::String;

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

/// The detected host system information with independent vendor and model data sources.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub vendor: Option<String>,
    pub vendor_source: DataSource,
    pub model: Option<String>,
    pub model_source: DataSource,
}

impl SystemInfo {
    pub fn new(
        vendor: Option<String>,
        vendor_source: DataSource,
        model: Option<String>,
        model_source: DataSource,
    ) -> Self {
        let vendor_source = if vendor.is_some() {
            vendor_source
        } else {
            DataSource::DefaultValue
        };
        let model_source = if model.is_some() {
            model_source
        } else {
            DataSource::DefaultValue
        };
        Self {
            vendor,
            vendor_source,
            model,
            model_source,
        }
    }

    pub fn from_model(model: impl Into<String>, source: DataSource) -> Self {
        Self {
            vendor: None,
            vendor_source: DataSource::DefaultValue,
            model: Some(model.into()),
            model_source: source,
        }
    }

    /// Combines vendor and model for canonical display.
    pub fn display_name(&self) -> Option<String> {
        let model = self.model.as_deref()?;
        let vendor = self.vendor.as_deref();
        Some(combine_vendor_and_model(vendor, model))
    }
}

/// Where did this cpu or system information come from?
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum DataSource {
    /// A default value, when lookup fails
    #[default]
    DefaultValue,
    /// Value from Android getprop shell tool with property name
    AndroidGetprop(&'static str),
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
    /// from device tree with node path
    DeviceTree(&'static str),
    /// sysinfo command on Haiku
    HaikuSysinfo,
    /// /proc/cpuinfo
    LinuxProcCpuinfo,
    /// Linux virtual /sys directory tree with sysfs path
    LinuxSysFs(&'static str),
    /// Determined from a set of pre-defined values
    LookupTable,
    /// Linux lscpu command
    Lscpu,
    /// x86 MpTable
    MpTable,
    /// value from SMBIOS / DMI table with structure description
    Smbios(&'static str),
    /// value from sysctl tool with sysctl name
    Sysctrl(&'static str),
    /// value from system call
    SystemCall,
    /// value from Windows registry with subkey / value path
    WindowsRegistry(&'static str),
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

    #[test]
    fn test_system_info_source_defaults_when_none() {
        let info = SystemInfo::new(
            None,
            DataSource::Smbios("test"),
            Some("Model X".to_string()),
            DataSource::LinuxSysFs("/sys/class/dmi/id/product_name"),
        );
        assert_eq!(info.vendor, None);
        assert_eq!(info.vendor_source, DataSource::DefaultValue);
        assert_eq!(info.model.as_deref(), Some("Model X"));
        assert_eq!(
            info.model_source,
            DataSource::LinuxSysFs("/sys/class/dmi/id/product_name")
        );

        let info2 = SystemInfo::new(
            Some("Vendor Y".to_string()),
            DataSource::Smbios("test"),
            None,
            DataSource::Smbios("test"),
        );
        assert_eq!(info2.vendor.as_deref(), Some("Vendor Y"));
        assert_eq!(info2.vendor_source, DataSource::Smbios("test"));
        assert_eq!(info2.model, None);
        assert_eq!(info2.model_source, DataSource::DefaultValue);
    }
}
