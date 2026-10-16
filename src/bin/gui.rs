#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(not(any(windows_os, target_os = "haiku", target_os = "linux")))]
fn main() {
    eprintln!("rustid-gui is currently supported on Windows, Haiku, and Linux targets.");
}

#[cfg(windows_os)]
use rustid::gui::windows as gui;

#[cfg(target_os = "haiku")]
use rustid::gui::haiku as gui;

#[cfg(target_os = "linux")]
use rustid::gui::linux as gui;

#[cfg(any(windows_os, target_os = "haiku", target_os = "linux"))]
fn main() {
    gui::run();
}
