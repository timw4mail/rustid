#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(not(windows_os))]
fn main() {
    eprintln!("rustid-gui is currently supported only on Windows targets.");
}

#[cfg(windows_os)]
#[path = "gui/windows/mod.rs"]
mod gui;

#[cfg(windows_os)]
fn main() {
    gui::run();
}
