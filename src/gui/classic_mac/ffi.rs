//! FFI declarations and callbacks for the Classic Macintosh C GUI bridge.

use super::styled_text::{RgbColor, TextRun};
use core::ffi::c_char;

pub type CmdCallback = unsafe extern "C" fn(cmd_id: u32);
pub type FileCallback = unsafe extern "C" fn(path: *const c_char, is_save: bool);
pub type QuitCallback = unsafe extern "C" fn();

unsafe extern "C" {
    pub fn mac_gui_init(title: *const c_char, width: i16, height: i16) -> bool;

    pub fn mac_gui_set_callbacks(
        on_cmd: Option<CmdCallback>,
        on_file: Option<FileCallback>,
        on_quit: Option<QuitCallback>,
    );

    pub fn mac_gui_set_text(
        text: *const c_char,
        length: u32,
        runs: *const TextRun,
        run_count: u32,
        bg_color: RgbColor,
    );

    pub fn mac_gui_set_status(part1: *const c_char, part2: *const c_char, part3: *const c_char);

    pub fn mac_gui_set_menu_checks(
        mode_cmd_id: u32,
        color: bool,
        dark_theme: bool,
        verbose: bool,
        compact: bool,
    );

    pub fn mac_gui_open_file_dialog();
    pub fn mac_gui_save_file_dialog(default_filename: *const c_char);
    pub fn mac_gui_copy_clipboard(text: *const c_char);
    pub fn mac_gui_show_alert(title: *const c_char, message: *const c_char);
    pub fn mac_gui_run();
}
