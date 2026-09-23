//! File dialogs, alert helpers, and clipboard operations for Classic Mac GUI.

use super::ffi;
use alloc::ffi::CString;
use alloc::format;

pub fn copy_to_clipboard(text: &str) {
    if let Ok(c_text) = CString::new(text) {
        unsafe {
            ffi::mac_gui_copy_clipboard(c_text.as_ptr());
        }
    }
}

pub fn show_alert(title: &str, message: &str) {
    let c_title = CString::new(title).unwrap_or_default();
    let c_msg = CString::new(message).unwrap_or_default();
    unsafe {
        ffi::mac_gui_show_alert(c_title.as_ptr(), c_msg.as_ptr());
    }
}

pub fn show_about_dialog() {
    let about_text = format!(
        "Rustid v{}\nCPU identification tool for Classic Macintosh\nSystem 6 - Mac OS 9.2.2 (68k & PowerPC)",
        env!("CARGO_PKG_VERSION")
    );
    show_alert("About Rustid", &about_text);
}

pub fn export_report_dialog(default_filename: &str) {
    let c_def = CString::new(default_filename).unwrap_or_default();
    unsafe {
        ffi::mac_gui_save_file_dialog(c_def.as_ptr());
    }
}
