#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(windows_os)]
#[path = "gui/windows/mod.rs"]
mod gui;

#[cfg(macos_os)]
#[path = "gui/macos/mod.rs"]
mod gui;

#[cfg(any(windows_os, macos_os))]
fn main() {
    gui::run();
}

#[cfg(not(any(windows_os, macos_os)))]
fn main() {
    eprintln!("rustid-gui is currently supported only on Windows and macOS targets.");
    std::process::exit(1);
}
