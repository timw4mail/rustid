#![cfg(target_os = "uefi")]
//! Zero-dependency UEFI environment support for rustid.

pub mod display;
pub mod font;
pub mod mp;
pub mod os;

pub use display::*;
pub use mp::*;
pub use os::*;
