//! Native Haiku GUI module for rustid.

#[cfg(target_os = "haiku")]
pub mod dialogs;
#[cfg(target_os = "haiku")]
pub mod ffi;
pub mod menu;
pub mod state;
pub use crate::gui::styled_text;
#[cfg(target_os = "haiku")]
pub mod window;

#[cfg(target_os = "haiku")]
pub use window::run;
