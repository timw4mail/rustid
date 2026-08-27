pub mod cache;

pub mod constants;

pub mod display;

pub mod os;

pub use cache::*;

pub use constants::*;

pub use display::*;

pub use os::*;

use alloc::string::String;

pub fn ucfirst(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn cleanup_soc_vendor(s: &str) -> String {
    let lower = s.to_lowercase();
    let vendor = match lower.as_str() {
        "allwinner" | "sunxi" => "Allwinner",
        "amlogic" | "meson" => "Amlogic",
        "apple" => "Apple",
        "bigtreetech" => "BigTreeTech",
        "brcm" | "broadcom" => "Broadcom",
        "hisilicon" | "hi" => "HiSilicon",
        "mediatek" | "mtk" => "MediaTek",
        "nxp" | "freescale" => "NXP",
        "qcom" | "qualcomm" => "Qualcomm",
        "raspberrypi" => "Raspberry Pi",
        "realtek" => "Realtek",
        "renesas" => "Renesas",
        "rk" | "rockchip" => "Rockchip",
        "samsung" | "exynos" => "Samsung",
        "st" | "stmicro" => "STMicroelectronics",
        "ti" => "Texas Instruments",
        "xilinx" => "Xilinx",
        _ => return ucfirst(s),
    };

    String::from(vendor)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CliFlags {
    pub compact: bool,
    pub color: bool,
    pub verbose: bool,
}

pub trait TDetect {
    fn detect() -> Self;
}

pub trait TCpuDisplay: TDetect {
    /// Display the Rust debug output of the CPU object
    fn debug(&self);

    /// Display the CPU information in a table format
    fn display_table(&self, flags: CliFlags);
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
    /// Per-core-cluster breakdown of CPU cores
    pub cores: alloc::vec::Vec<CpuCore<M>>,
    /// Detected CPU features
    pub features: alloc::collections::BTreeMap<&'static str, String>,
    /// Architecture-specific extension data
    pub extra: E,
}

impl<E, M> Cpu<E, M> {
    /// Returns true if this CPU has multiple core types (hybrid architecture).
    pub fn is_hybrid(&self) -> bool {
        self.cores.len() > 1
    }

    /// Total physical cores across all clusters
    pub fn total_cores(&self) -> u32 {
        self.cores.iter().map(|c| c.count).sum()
    }

    /// Total logical threads across all clusters
    pub fn total_threads(&self) -> u32 {
        self.cores.iter().map(|c| c.threads).sum()
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
    fn test_ucfirst_empty() {
        assert_eq!(ucfirst(""), "");
    }

    #[test]
    fn test_ucfirst_already_upper() {
        assert_eq!(ucfirst("Hello"), "Hello");
    }

    #[test]
    fn test_ucfirst_lowercase() {
        assert_eq!(ucfirst("hello"), "Hello");
    }

    #[test]
    fn test_ucfirst_single_char() {
        assert_eq!(ucfirst("a"), "A");
    }

    #[test]
    fn test_cleanup_soc_vendor_brcm() {
        assert_eq!(cleanup_soc_vendor("brcm"), "Broadcom");
    }

    #[test]
    fn test_cleanup_soc_vendor_qcom() {
        assert_eq!(cleanup_soc_vendor("qcom"), "Qualcomm");
    }

    #[test]
    fn test_cleanup_soc_vendor_rk() {
        assert_eq!(cleanup_soc_vendor("rk"), "Rockchip");
    }

    #[test]
    fn test_cleanup_soc_vendor_allwinner() {
        assert_eq!(cleanup_soc_vendor("sunxi"), "Allwinner");
    }

    #[test]
    fn test_cleanup_soc_vendor_raspberrypi() {
        assert_eq!(cleanup_soc_vendor("raspberrypi"), "Raspberry Pi");
    }

    #[test]
    fn test_cleanup_soc_vendor_bigtreetech() {
        assert_eq!(cleanup_soc_vendor("bigtreetech"), "BigTreeTech");
    }

    #[test]
    fn test_cleanup_soc_vendor_other() {
        assert_eq!(cleanup_soc_vendor("unknown_vendor"), "Unknown_vendor");
    }

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
    fn test_cli_flags_default() {
        let f = CliFlags::default();
        assert!(!f.color);
        assert!(!f.verbose);
    }

    #[test]
    fn test_cli_flags_explicit() {
        let f = CliFlags {
            color: true,
            compact: true,
            verbose: true,
        };
        assert!(f.color);
        assert!(f.compact);
        assert!(f.verbose);
    }
}
