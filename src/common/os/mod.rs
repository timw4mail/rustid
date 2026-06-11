//! Os-specific data gathering
use crate::common::TopologyTier;

#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
pub mod bsd;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(all(target_family = "unix", not(target_os = "haiku")))]
pub mod sysctl;

#[cfg(target_os = "haiku")]
pub mod haiku;

#[cfg(target_os = "windows")]
pub mod windows;

// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "haiku")]
pub use haiku::*;

#[cfg(all(target_family = "unix", not(target_os = "haiku")))]
pub use sysctl::*;

// ----------------------------------------------------------------------------

pub struct OS;

pub trait TOSData {
    fn get_socket_count() -> TopologyTier;
}
