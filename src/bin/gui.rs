#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
use rustid::gui::windows as gui;

#[cfg(target_os = "macos")]
use rustid::gui::macos as gui;

#[cfg(target_os = "haiku")]
use rustid::gui::haiku as gui;

fn main() {
    #[cfg(any(windows_os, macos_os, target_os = "haiku"))]
    gui::run();

    #[cfg(not(any(windows_os, macos_os, target_os = "haiku")))]
    {
        eprintln!("gui is currently supported only on Windows and macOS targets.");
        std::process::exit(1);
    }
}
