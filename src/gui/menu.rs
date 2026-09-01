//! Menu definitions, construction, and state updates.

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CheckMenuItem, CheckMenuRadioItem, CreateMenu, CreatePopupMenu, GetSubMenu,
    HMENU, MF_BYCOMMAND, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
};
use windows::core::{PCWSTR, w};

use super::state::{AppState, IS_UNICODE, ViewMode};

pub const IDM_FILE_OPEN: u32 = 101;
#[cfg(x86_cpu)]
pub const IDM_FILE_EXPORT: u32 = 102;
pub const IDM_FILE_COPY: u32 = 103;
pub const IDM_FILE_REFRESH: u32 = 104;
pub const IDM_FILE_EXIT: u32 = 105;

pub const IDM_MODE_STANDARD: u32 = 201;
pub const IDM_MODE_DEBUG: u32 = 202;
pub const IDM_MODE_EVERYTHING: u32 = 203;
#[cfg(x86_cpu)]
pub const IDM_MODE_DUMP: u32 = 204;

pub const IDM_OPT_COLOR: u32 = 301;
pub const IDM_OPT_DARK_THEME: u32 = 302;
pub const IDM_OPT_VERBOSE: u32 = 303;
pub const IDM_OPT_COMPACT: u32 = 304;

pub const IDM_HELP_ABOUT: u32 = 401;

pub fn create_main_menu() -> HMENU {
    unsafe {
        if IS_UNICODE.load(Ordering::Relaxed) {
            let hmenu_bar = CreateMenu().unwrap_or_default();

            // File Menu
            let hmenu_file = CreatePopupMenu().unwrap_or_default();
            let _ = AppendMenuW(
                hmenu_file,
                MF_STRING,
                IDM_FILE_OPEN as usize,
                w!("Open Dump...\tCtrl+O"),
            );
            #[cfg(x86_cpu)]
            let _ = AppendMenuW(
                hmenu_file,
                MF_STRING,
                IDM_FILE_EXPORT as usize,
                w!("Export CPUID Dump...\tCtrl+S"),
            );
            let _ = AppendMenuW(hmenu_file, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu_file,
                MF_STRING,
                IDM_FILE_COPY as usize,
                w!("Copy All Text\tCtrl+C"),
            );
            let _ = AppendMenuW(
                hmenu_file,
                MF_STRING,
                IDM_FILE_REFRESH as usize,
                w!("Refresh Hardware\tF5"),
            );
            let _ = AppendMenuW(hmenu_file, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu_file,
                MF_STRING,
                IDM_FILE_EXIT as usize,
                w!("Exit\tAlt+F4"),
            );
            let _ = AppendMenuW(hmenu_bar, MF_POPUP, hmenu_file.0 as usize, w!("&File"));

            // Mode Menu
            let hmenu_mode = CreatePopupMenu().unwrap_or_default();
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_MODE_STANDARD as usize,
                w!("Standard Summary\tCtrl+1"),
            );
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_MODE_DEBUG as usize,
                w!("Debug Information (-d)\tCtrl+2"),
            );
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_MODE_EVERYTHING as usize,
                w!("Everything (-e)\tCtrl+3"),
            );
            #[cfg(x86_cpu)]
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_MODE_DUMP as usize,
                w!("Raw CPUID Dump (-r)\tCtrl+4"),
            );
            let _ = AppendMenuW(hmenu_mode, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_OPT_COLOR as usize,
                w!("Colorized Output"),
            );
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_OPT_DARK_THEME as usize,
                w!("Dark Theme"),
            );
            let _ = AppendMenuW(hmenu_mode, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_OPT_VERBOSE as usize,
                w!("Verbose Mode (-v)"),
            );
            let _ = AppendMenuW(
                hmenu_mode,
                MF_STRING,
                IDM_OPT_COMPACT as usize,
                w!("Compact Mode (-c)"),
            );
            let _ = AppendMenuW(hmenu_bar, MF_POPUP, hmenu_mode.0 as usize, w!("&Mode"));

            // Help Menu
            let hmenu_help = CreatePopupMenu().unwrap_or_default();
            let _ = AppendMenuW(
                hmenu_help,
                MF_STRING,
                IDM_HELP_ABOUT as usize,
                w!("&About Rustid"),
            );
            let _ = AppendMenuW(hmenu_bar, MF_POPUP, hmenu_help.0 as usize, w!("&Help"));

            hmenu_bar
        } else {
            unsafe extern "system" {
                fn CreateMenu() -> *mut c_void;
                fn CreatePopupMenu() -> *mut c_void;
                fn AppendMenuA(
                    hMenu: *mut c_void,
                    uFlags: u32,
                    uIDNewItem: usize,
                    lpNewItem: *const u8,
                ) -> i32;
            }
            let hmenu_bar = CreateMenu();

            // File Menu
            let hmenu_file = CreatePopupMenu();
            AppendMenuA(
                hmenu_file,
                0,
                IDM_FILE_OPEN as usize,
                b"Open Dump...\tCtrl+O\0".as_ptr(),
            );
            #[cfg(x86_cpu)]
            AppendMenuA(
                hmenu_file,
                0,
                IDM_FILE_EXPORT as usize,
                b"Export CPUID Dump...\tCtrl+S\0".as_ptr(),
            );
            AppendMenuA(hmenu_file, 0x800, 0, std::ptr::null());
            AppendMenuA(
                hmenu_file,
                0,
                IDM_FILE_COPY as usize,
                b"Copy All Text\tCtrl+C\0".as_ptr(),
            );
            AppendMenuA(
                hmenu_file,
                0,
                IDM_FILE_REFRESH as usize,
                b"Refresh Hardware\tF5\0".as_ptr(),
            );
            AppendMenuA(hmenu_file, 0x800, 0, std::ptr::null());
            AppendMenuA(
                hmenu_file,
                0,
                IDM_FILE_EXIT as usize,
                b"Exit\tAlt+F4\0".as_ptr(),
            );
            AppendMenuA(hmenu_bar, 0x10, hmenu_file as usize, b"&File\0".as_ptr());

            // Mode Menu
            let hmenu_mode = CreatePopupMenu();
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_MODE_STANDARD as usize,
                b"Standard Summary\tCtrl+1\0".as_ptr(),
            );
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_MODE_DEBUG as usize,
                b"Debug Information (-d)\tCtrl+2\0".as_ptr(),
            );
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_MODE_EVERYTHING as usize,
                b"Everything (-e)\tCtrl+3\0".as_ptr(),
            );
            #[cfg(x86_cpu)]
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_MODE_DUMP as usize,
                b"Raw CPUID Dump (-r)\tCtrl+4\0".as_ptr(),
            );
            AppendMenuA(hmenu_mode, 0x800, 0, std::ptr::null());
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_OPT_COLOR as usize,
                b"Colorized Output\0".as_ptr(),
            );
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_OPT_DARK_THEME as usize,
                b"Dark Theme\0".as_ptr(),
            );
            AppendMenuA(hmenu_mode, 0x800, 0, std::ptr::null());
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_OPT_VERBOSE as usize,
                b"Verbose Mode (-v)\0".as_ptr(),
            );
            AppendMenuA(
                hmenu_mode,
                0,
                IDM_OPT_COMPACT as usize,
                b"Compact Mode (-c)\0".as_ptr(),
            );
            AppendMenuA(hmenu_bar, 0x10, hmenu_mode as usize, b"&Mode\0".as_ptr());

            // Help Menu
            let hmenu_help = CreatePopupMenu();
            AppendMenuA(
                hmenu_help,
                0,
                IDM_HELP_ABOUT as usize,
                b"&About Rustid\0".as_ptr(),
            );
            AppendMenuA(hmenu_bar, 0x10, hmenu_help as usize, b"&Help\0".as_ptr());

            HMENU(hmenu_bar)
        }
    }
}

pub fn update_menu_checks(state: &AppState) {
    unsafe {
        let hmenu_mode = GetSubMenu(state.hmenu, 1);
        if hmenu_mode.is_invalid() {
            return;
        }

        let active_mode_id = match state.mode {
            ViewMode::Standard => IDM_MODE_STANDARD,
            ViewMode::Debug => IDM_MODE_DEBUG,
            ViewMode::Everything => IDM_MODE_EVERYTHING,
            #[cfg(x86_cpu)]
            ViewMode::Dump => IDM_MODE_DUMP,
        };

        #[cfg(x86_cpu)]
        let last_mode_id = IDM_MODE_DUMP;
        #[cfg(not(x86_cpu))]
        let last_mode_id = IDM_MODE_EVERYTHING;

        let _ = CheckMenuRadioItem(
            hmenu_mode,
            IDM_MODE_STANDARD,
            last_mode_id,
            active_mode_id,
            MF_BYCOMMAND.0,
        );

        let _ = CheckMenuItem(
            hmenu_mode,
            IDM_OPT_COLOR,
            if state.color {
                MF_CHECKED.0
            } else {
                MF_UNCHECKED.0
            },
        );

        let _ = CheckMenuItem(
            hmenu_mode,
            IDM_OPT_DARK_THEME,
            if state.dark_theme {
                MF_CHECKED.0
            } else {
                MF_UNCHECKED.0
            },
        );

        let _ = CheckMenuItem(
            hmenu_mode,
            IDM_OPT_VERBOSE,
            if state.verbose {
                MF_CHECKED.0
            } else {
                MF_UNCHECKED.0
            },
        );

        let _ = CheckMenuItem(
            hmenu_mode,
            IDM_OPT_COMPACT,
            if state.compact {
                MF_CHECKED.0
            } else {
                MF_UNCHECKED.0
            },
        );
    }
}
