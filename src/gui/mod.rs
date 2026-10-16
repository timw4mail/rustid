pub mod common;
pub use common::*;
pub mod styled_text;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(all(target_os = "macos", feature = "gui"))]
pub mod macos;

#[cfg(any(target_os = "haiku", test))]
pub mod haiku;

#[cfg(all(target_os = "linux", feature = "gui"))]
pub mod linux;
