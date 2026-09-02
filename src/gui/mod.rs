//! Top-level GUI module for rustid-gui.

pub mod dialogs;
pub mod menu;
pub mod rtf;
pub mod shims;
pub mod state;
pub mod theme;
pub mod window;

pub use window::run;
