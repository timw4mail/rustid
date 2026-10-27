//! Menu definitions, construction, and state updates.

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CheckMenuItem, CheckMenuRadioItem, CreateMenu, CreatePopupMenu, EnableMenuItem,
    GetSubMenu, HMENU, MENU_ITEM_FLAGS, MF_BYCOMMAND, MF_CHECKED, MF_ENABLED, MF_GRAYED, MF_POPUP,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
};
use windows::core::PCWSTR;

use super::state::{AppState, IS_RICHEDIT, IS_UNICODE, ViewMode};
use super::theme::to_pcwstr;

#[cfg(x86_cpu)]
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

unsafe fn create_menu() -> HMENU {
    unsafe {
        if IS_UNICODE.load(Ordering::Relaxed) {
            CreateMenu().unwrap_or_default()
        } else {
            unsafe extern "system" {
                fn CreateMenu() -> *mut c_void;
            }
            HMENU(CreateMenu())
        }
    }
}

unsafe fn create_popup_menu() -> HMENU {
    unsafe {
        if IS_UNICODE.load(Ordering::Relaxed) {
            CreatePopupMenu().unwrap_or_default()
        } else {
            unsafe extern "system" {
                fn CreatePopupMenu() -> *mut c_void;
            }
            HMENU(CreatePopupMenu())
        }
    }
}

unsafe fn append_menu_item(
    hmenu: HMENU,
    flags: MENU_ITEM_FLAGS,
    id: usize,
    text: Option<&str>,
) {
    unsafe {
        if IS_UNICODE.load(Ordering::Relaxed) {
            if let Some(t) = text {
                let u16_text = to_pcwstr(t);
                let _ = AppendMenuW(hmenu, flags, id, PCWSTR(u16_text.as_ptr()));
            } else {
                let _ = AppendMenuW(hmenu, flags, id, PCWSTR::null());
            }
        } else {
            unsafe extern "system" {
                fn AppendMenuA(
                    hMenu: *mut c_void,
                    uFlags: u32,
                    uIDNewItem: usize,
                    lpNewItem: *const u8,
                ) -> i32;
            }
            if let Some(t) = text {
                let c_str = std::ffi::CString::new(t).unwrap_or_default();
                AppendMenuA(hmenu.0, flags.0, id, c_str.as_ptr() as *const u8);
            } else {
                AppendMenuA(hmenu.0, flags.0, id, std::ptr::null());
            }
        }
    }
}

pub fn create_main_menu() -> HMENU {
    unsafe {
        let hmenu_bar = create_menu();

        // File Menu
        let hmenu_file = create_popup_menu();
        #[cfg(x86_cpu)]
        append_menu_item(
            hmenu_file,
            MF_STRING,
            IDM_FILE_OPEN as usize,
            Some("Open Dump...\tCtrl+O"),
        );
        #[cfg(x86_cpu)]
        append_menu_item(
            hmenu_file,
            MF_STRING,
            IDM_FILE_EXPORT as usize,
            Some("Export CPUID Dump...\tCtrl+S"),
        );
        #[cfg(x86_cpu)]
        append_menu_item(hmenu_file, MF_SEPARATOR, 0, None);
        append_menu_item(
            hmenu_file,
            MF_STRING,
            IDM_FILE_COPY as usize,
            Some("Copy All Text\tCtrl+C"),
        );
        #[cfg(x86_cpu)]
        append_menu_item(
            hmenu_file,
            MF_STRING,
            IDM_FILE_REFRESH as usize,
            Some("Refresh Hardware\tF5"),
        );
        append_menu_item(hmenu_file, MF_SEPARATOR, 0, None);
        append_menu_item(
            hmenu_file,
            MF_STRING,
            IDM_FILE_EXIT as usize,
            Some("Exit\tAlt+F4"),
        );
        append_menu_item(
            hmenu_bar,
            MF_POPUP,
            hmenu_file.0 as usize,
            Some("&File"),
        );

        // Mode Menu
        let hmenu_mode = create_popup_menu();
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_MODE_STANDARD as usize,
            Some("Standard Summary\tCtrl+1"),
        );
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_MODE_DEBUG as usize,
            Some("Debug Information (-d)\tCtrl+2"),
        );
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_MODE_EVERYTHING as usize,
            Some("Everything (-e)\tCtrl+3"),
        );
        #[cfg(x86_cpu)]
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_MODE_DUMP as usize,
            Some("Raw CPUID Dump (-r)\tCtrl+4"),
        );
        append_menu_item(hmenu_mode, MF_SEPARATOR, 0, None);
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_OPT_COLOR as usize,
            Some("Colorized Output"),
        );
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_OPT_DARK_THEME as usize,
            Some("Dark Theme"),
        );
        append_menu_item(hmenu_mode, MF_SEPARATOR, 0, None);
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_OPT_VERBOSE as usize,
            Some("Verbose Mode (-v)"),
        );
        append_menu_item(
            hmenu_mode,
            MF_STRING,
            IDM_OPT_COMPACT as usize,
            Some("Compact Mode (-c)"),
        );
        append_menu_item(
            hmenu_bar,
            MF_POPUP,
            hmenu_mode.0 as usize,
            Some("&Mode"),
        );

        // Help Menu
        let hmenu_help = create_popup_menu();
        append_menu_item(
            hmenu_help,
            MF_STRING,
            IDM_HELP_ABOUT as usize,
            Some("&About Rustid"),
        );
        append_menu_item(
            hmenu_bar,
            MF_POPUP,
            hmenu_help.0 as usize,
            Some("&Help"),
        );

        hmenu_bar
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

        let check_item = |id: u32, checked: bool| {
            let flag = if checked {
                MF_CHECKED.0
            } else {
                MF_UNCHECKED.0
            };
            let _ = CheckMenuItem(hmenu_mode, id, flag);
        };

        let is_richedit = IS_RICHEDIT.load(Ordering::Relaxed);
        if is_richedit {
            let _ = EnableMenuItem(hmenu_mode, IDM_OPT_COLOR, MF_BYCOMMAND | MF_ENABLED);
            check_item(IDM_OPT_COLOR, state.flags.color);
        } else {
            let _ = EnableMenuItem(hmenu_mode, IDM_OPT_COLOR, MF_BYCOMMAND | MF_GRAYED);
            check_item(IDM_OPT_COLOR, false);
        }

        check_item(IDM_OPT_DARK_THEME, state.theme.is_dark());
        check_item(IDM_OPT_VERBOSE, state.flags.verbose);
        check_item(IDM_OPT_COMPACT, state.flags.compact);
    }
}
