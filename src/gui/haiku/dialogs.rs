//! File dialogs, alert helpers, and clipboard operations for Haiku GUI.

use super::ffi;
use std::ffi::CString;

pub fn copy_to_clipboard(text: &str) {
    if let Ok(c_text) = CString::new(text) {
        unsafe {
            ffi::haiku_gui_copy_clipboard(c_text.as_ptr());
        }
    }
}

pub fn show_alert(title: &str, message: &str) {
    let c_title = CString::new(title).unwrap_or_default();
    let c_msg = CString::new(message).unwrap_or_default();
    unsafe {
        ffi::haiku_gui_show_alert(c_title.as_ptr(), c_msg.as_ptr());
    }
}

pub fn show_about_dialog() {
    let about_text = format!(
        "Rustid v{}\nMulti-architecture CPU detection tool\nRunning on {}-{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    show_alert("About Rustid", &about_text);
}

#[cfg(x86_cpu)]
pub fn open_dump_file_dialog() {
    unsafe {
        ffi::haiku_gui_open_file_dialog();
    }
}

#[cfg(x86_cpu)]
pub fn export_dump_dialog(default_filename: &str) {
    let c_def = CString::new(default_filename).unwrap_or_default();
    unsafe {
        ffi::haiku_gui_save_file_dialog(c_def.as_ptr());
    }
}

#[cfg(x86_cpu)]
pub fn read_file_to_string(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(x86_cpu)]
pub fn write_string_to_file(path: &str, content: &str) -> bool {
    std::fs::write(path, content).is_ok()
}
