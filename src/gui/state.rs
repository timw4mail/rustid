//! Application state, view mode, control constants, and window state helpers.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, GWLP_USERDATA, HMENU, SetWindowLongPtrW, WM_USER,
};

pub const IDC_STATUSBAR: i32 = 5001;
pub const IDC_RICHEDIT: i32 = 5002;

pub const CF_UNICODETEXT_FORMAT: u32 = 13;

// RichEdit constants
pub const EM_SETBKGNDCOLOR: u32 = WM_USER + 67;
pub const EM_EXLIMITTEXT: u32 = WM_USER + 53;

pub static IS_UNICODE: AtomicBool = AtomicBool::new(true);

// True when the edit control is a real RichEdit (not a plain EDIT fallback).
// Used to gate EM_SETTEXTEX / EM_SETBKGNDCOLOR / EM_EXLIMITTEXT.
pub static IS_RICHEDIT: AtomicBool = AtomicBool::new(true);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewMode {
    Standard,
    Debug,
    Everything,
    #[cfg(x86_cpu)]
    Dump,
}

pub struct AppState {
    pub hwnd_main: HWND,
    pub hwnd_edit: HWND,
    pub hwnd_status: HWND,
    pub hmenu: HMENU,
    pub dpi: u32,
    pub mode: ViewMode,
    pub color: bool,
    pub dark_theme: bool,
    pub custom_theme_set: bool,
    pub verbose: bool,
    pub compact: bool,
    pub loaded_file: Option<String>,
    pub current_plain_text: String,
}

pub unsafe fn set_window_state(hwnd: HWND, ptr: isize) {
    if IS_UNICODE.load(Ordering::Relaxed) {
        #[cfg(target_pointer_width = "64")]
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr);
        }
        #[cfg(target_pointer_width = "32")]
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as i32);
        }
    } else {
        unsafe extern "system" {
            fn SetWindowLongA(hWnd: *mut c_void, nIndex: i32, dwNewLong: i32) -> i32;
        }
        unsafe {
            SetWindowLongA(hwnd.0, -21, ptr as i32);
        }
    }
}

pub unsafe fn get_window_state(hwnd: HWND) -> *mut AppState {
    let ptr: isize = if IS_UNICODE.load(Ordering::Relaxed) {
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as isize }
    } else {
        unsafe extern "system" {
            fn GetWindowLongA(hWnd: *mut c_void, nIndex: i32) -> i32;
        }
        unsafe { GetWindowLongA(hwnd.0, -21) as isize }
    };
    ptr as *mut AppState
}
