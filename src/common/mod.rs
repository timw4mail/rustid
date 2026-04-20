pub mod cache;

#[cfg(not(target_os = "none"))]
pub mod cores;

pub mod constants;

pub use cache::*;

#[cfg(not(target_os = "none"))]
pub use cores::*;

pub use constants::*;

pub trait TCpu {
    /// Detect the CPU
    fn detect() -> Self;

    /// Display the Rust debug output of the CPU object
    fn debug(&self);

    /// Display the CPU information in a table format
    fn display_table(&self);
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

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
impl From<String> for CoreType {
    fn from(val: String) -> Self {
        Self::from(val.as_str())
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
impl From<CoreType> for String {
    fn from(val: CoreType) -> String {
        let s: &str = val.into();

        String::from(s)
    }
}

#[cfg_attr(
    all(target_os = "none", not(feature = "debug")),
    derive(Default, PartialEq)
)]
#[cfg_attr(
    any(not(target_os = "none"), feature = "debug"),
    derive(Debug, Default, PartialEq)
)]
pub struct CpuArch {
    pub kind: CoreType,

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub name: Option<String>,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub name: Option<crate::cpuid::Str<70>>,

    pub cache: Option<Cache>,

    pub count: usize,
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
