pub mod cache;

#[cfg(not(target_os = "none"))]
pub mod cores;

pub use cache::*;

#[cfg(not(target_os = "none"))]
pub use cores::*;

pub const UNK: &str = "Unknown";

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

#[derive(Debug, Default, PartialEq)]
pub struct CpuCore {
    pub kind: CoreType,

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub name: Option<String>,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub name: Option<crate::cpuid::Str<64>>,

    pub cache: Option<Cache>,

    pub count: usize,
}
