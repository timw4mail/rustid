#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(not(any(windows_os, target_os = "haiku", target_os = "linux", macos_os)))]
fn main() {
    eprintln!("rustid-gui is currently supported on Windows, Haiku, Linux, and macOS targets.");
}

#[cfg(windows_os)]
use rustid::gui::windows as gui;

#[cfg(macos_os)]
use rustid::gui::macos as gui;

#[cfg(target_os = "haiku")]
use rustid::gui::haiku as gui;

#[cfg(target_os = "linux")]
use rustid::gui::linux as gui;

#[cfg(any(windows_os, target_os = "haiku", target_os = "linux", macos_os))]
fn main() {
    gui::run();
}
