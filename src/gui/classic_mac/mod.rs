//! Native Classic Macintosh GUI module for rustid (68k and PowerPC).

pub mod dialogs;
pub mod ffi;
pub mod menu;
pub mod state;
pub use crate::gui::styled_text;
pub mod window;

pub use window::run;
