//! Os-specific data gathering

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(all(target_family = "unix", not(target_os = "haiku")))]
pub mod sysctl;

// ----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(all(target_family = "unix", not(target_os = "haiku")))]
pub use sysctl::*;

// ----------------------------------------------------------------------------
