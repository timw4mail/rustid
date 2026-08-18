//! Os-specific data gathering
use crate::common::TopologyTier;
use alloc::string::String;

#[cfg(bsd)]
pub mod bsd;

#[cfg(target_os = "uefi")]
pub mod efi;

pub mod common;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(any(bsd, target_os = "macos"))]
pub mod sysctl;

#[cfg(target_os = "haiku")]
pub mod haiku;

#[cfg(target_os = "windows")]
pub mod windows;

// ----------------------------------------------------------------------------

pub use common::*;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "haiku")]
pub use haiku::*;

#[cfg(any(bsd, target_os = "macos"))]
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

    fn get_socket_count() -> TopologyTier {
        TopologyTier::default()
    }
}
