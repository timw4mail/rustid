//! Native Linux (GTK) GUI module for rustid.

#[cfg(target_os = "linux")]
pub mod dialogs;
#[cfg(target_os = "linux")]
pub mod ffi;
pub mod menu;
pub mod state;
#[cfg(target_os = "linux")]
pub mod window;

#[cfg(target_os = "linux")]
pub use window::run;
