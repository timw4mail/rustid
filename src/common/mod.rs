pub mod cache;

pub mod constants;

#[cfg(not(dos))]
pub mod count;

pub mod display;

pub mod os;

pub use cache::*;

pub use constants::*;

#[cfg(not(dos))]
pub use count::*;

pub use display::*;

#[cfg(not(dos))]
pub use os::*;

use alloc::string::String;

#[derive(Debug, Default, Clone, Copy)]
pub struct CliFlags {
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
#[derive(Debug, Default, PartialEq)]
pub struct Speed {
    /// Base frequency in MHz
    pub base: u32,
    /// Boost frequency in MHz
    pub boost: u32,
    /// Whether the frequency was measured (vs reported by CPU)
    pub measured: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TopologyCount {
    pub sockets: u32,
    pub cores: u32,
    pub threads: u32,
}

impl Default for TopologyCount {
    fn default() -> Self {
        TopologyCount {
            sockets: 1,
            cores: 1,
            threads: 1,
        }
    }
}

/// Where did this cpu information come from?
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum DataSource {
    /// A default value , when lookup fails
    #[default]
    DefaultValue,
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
    Sysctrl,
    /// value from system call
    SystemCall,
    /// value from Windows registry
    WindowsRegistry,
}
