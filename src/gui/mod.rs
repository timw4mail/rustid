pub mod common;
pub use common::*;
pub mod styled_text;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "haiku", test))]
pub mod haiku;

#[cfg(any(target_os = "linux", test))]
pub mod linux;
