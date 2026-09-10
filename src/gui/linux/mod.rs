//! Native Linux (GTK3 via gtk-rs) GUI module for rustid.

pub mod dialogs;
pub mod menu;
pub mod state;
pub mod window;

pub use window::run;
