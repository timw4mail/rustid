//! Os-specific data gathering
use crate::common::TopologyTier;
use alloc::string::String;

#[cfg(bsd)]
pub mod bsd;

#[cfg(uefi)]
pub mod efi;

pub mod common;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(any(bsd, target_os = "macos"))]
pub mod sysctl;

#[cfg(any(target_os = "haiku", test))]
pub mod haiku;

#[cfg(target_os = "windows")]
pub mod windows;

// ----------------------------------------------------------------------------

pub use common::*;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(any(target_os = "haiku", test))]
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
