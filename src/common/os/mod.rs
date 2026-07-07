//! Os-specific data gathering
use crate::common::TopologyTier;
use alloc::string::String;

#[cfg(bsd)]
pub mod bsd;

#[cfg(any(target_os = "android", target_os = "linux"))]
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

#[cfg(bsd)]
pub use bsd::*;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "haiku")]
pub use haiku::*;

#[cfg(all(target_family = "unix", not(target_os = "haiku")))]
pub use sysctl::*;

// ----------------------------------------------------------------------------

pub struct OS;

pub trait TOSData {
    fn get_system_name() -> Option<String> {
        None
    }

    fn get_soc() -> Option<String> {
        None
    }

    fn get_socket_count() -> TopologyTier;
}
