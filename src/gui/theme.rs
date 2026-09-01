//! DPI awareness, font caching, dark theme detection, and window message helpers.

use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DeleteObject,
    FW_NORMAL, GetDC, GetDeviceCaps, HFONT, HGDIOBJ, LOGPIXELSX, OUT_DEFAULT_PRECIS, ReleaseDC,
};
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW,
    RegQueryValueExW,
};
use windows::Win32::UI::Controls::{
    ICC_BAR_CLASSES, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx,
};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};
use windows::core::w;

use super::state::IS_UNICODE;

pub static FONT_MONO: AtomicIsize = AtomicIsize::new(0);
pub static FONT_UI: AtomicIsize = AtomicIsize::new(0);

pub fn is_system_dark_theme() -> bool {
    unsafe {
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let value_name = w!("AppsUseLightTheme");
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, None, KEY_READ, &mut hkey).is_ok() {
            let mut data = 1u32;
            let mut data_size = std::mem::size_of::<u32>() as u32;
            let mut val_type = REG_VALUE_TYPE::default();
            let status = RegQueryValueExW(
                hkey,
                value_name,
                None,
                Some(&mut val_type),
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_size),
            );
            let _ = RegCloseKey(hkey);
            if status.is_ok() {
                return data == 0;
            }
        }
        false
    }
}

pub fn set_window_dark_titlebar(hwnd: HWND, dark: bool) {
    unsafe {
        let dark_val: i32 = if dark { 1 } else { 0 };
        if let Ok(dwmapi) = LoadLibraryW(w!("dwmapi.dll")) {
            type DwmSetWindowAttributeFn =
                unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> windows::core::HRESULT;

            if let Some(proc) = windows::Win32::System::LibraryLoader::GetProcAddress(
                dwmapi,
                windows::core::s!("DwmSetWindowAttribute"),
            ) {
                let func: DwmSetWindowAttributeFn = std::mem::transmute(proc);
                // DWMWA_USE_IMMERSIVE_DARK_MODE = 20 (Windows 11 / Windows 10 2004+)
                let res = func(
                    hwnd,
                    20,
                    &dark_val as *const _ as *const c_void,
                    std::mem::size_of::<i32>() as u32,
                );
                if res.is_err() {
                    // DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1 = 19 (Windows 10 1809)
                    let _ = func(
                        hwnd,
                        19,
                        &dark_val as *const _ as *const c_void,
                        std::mem::size_of::<i32>() as u32,
                    );
                }
            }
        }
    }
}

pub fn to_pcwstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn scale(val: i32, dpi: u32) -> i32 {
    (val * dpi as i32) / 96
}

pub fn send_msg(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
    if IS_UNICODE.load(Ordering::Relaxed) {
        unsafe { SendMessageW(hwnd, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }
    } else {
        unsafe extern "system" {
            fn SendMessageA(hWnd: *mut c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
        }
        unsafe { LRESULT(SendMessageA(hwnd.0, msg, wparam, lparam)) }
    }
}

pub fn init_dpi_awareness() {
    unsafe {
        if let Ok(user32) = LoadLibraryW(w!("user32.dll")) {
            type SetProcessDpiAwarenessContextFn = unsafe extern "system" fn(isize) -> i32;
            if let Some(proc) = windows::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows::core::s!("SetProcessDpiAwarenessContext"),
            ) {
                let func: SetProcessDpiAwarenessContextFn = std::mem::transmute(proc);
                // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
                if func(-4) != 0 {
                    return;
                }
            }

            // Fallback for Windows Vista/7/8: SetProcessDPIAware
            type SetProcessDPIAwareFn = unsafe extern "system" fn() -> i32;
            if let Some(proc) = windows::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows::core::s!("SetProcessDPIAware"),
            ) {
                let func: SetProcessDPIAwareFn = std::mem::transmute(proc);
                let _ = func();
            }
        }
    }
}

pub fn get_system_dpi() -> u32 {
    unsafe {
        if let Ok(user32) = LoadLibraryW(w!("user32.dll")) {
            type GetDpiForSystemFn = unsafe extern "system" fn() -> u32;
            if let Some(proc) = windows::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows::core::s!("GetDpiForSystem"),
            ) {
                let func: GetDpiForSystemFn = std::mem::transmute(proc);
                let dpi = func();
                if dpi > 0 {
                    return dpi;
                }
            }
        }

        let hdc = GetDC(None);
        if !hdc.is_invalid() {
            let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32;
            let _ = ReleaseDC(None, hdc);
            if dpi > 0 {
                return dpi;
            }
        }

        96
    }
}

pub fn init_common_controls() {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES | ICC_BAR_CLASSES,
        };
        let _ = InitCommonControlsEx(&icc);

        #[link(name = "kernel32")]
        #[link(name = "comctl32")]
        unsafe extern "system" {
            fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
            fn InitCommonControls();
        }
        InitCommonControls();
        let _ = LoadLibraryA(b"msftedit.dll\0".as_ptr());
        let _ = LoadLibraryA(b"riched20.dll\0".as_ptr());
        let _ = LoadLibraryA(b"riched32.dll\0".as_ptr());
    }
}

pub fn create_fonts(dpi: u32) {
    unsafe {
        let old_mono = FONT_MONO.swap(0, Ordering::SeqCst);
        if old_mono != 0 {
            let _ = DeleteObject(HGDIOBJ(old_mono as *mut c_void));
        }
        let old_ui = FONT_UI.swap(0, Ordering::SeqCst);
        if old_ui != 0 {
            let _ = DeleteObject(HGDIOBJ(old_ui as *mut c_void));
        }

        let is_unicode = IS_UNICODE.load(Ordering::Relaxed);
        let mono_height = -scale(14, dpi);
        let ui_height = -scale(12, dpi);

        let (hfont_mono, hfont_ui) = if is_unicode {
            let hmono = CreateFontW(
                mono_height,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                0,
                w!("Consolas"),
            );
            let hui = CreateFontW(
                ui_height,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                0,
                w!("Segoe UI"),
            );
            (hmono, hui)
        } else {
            #[link(name = "gdi32")]
            unsafe extern "system" {
                fn CreateFontA(
                    cHeight: i32,
                    cWidth: i32,
                    cEscapement: i32,
                    cOrientation: i32,
                    cWeight: i32,
                    bItalic: u32,
                    bUnderline: u32,
                    bStrikeOut: u32,
                    iCharSet: u32,
                    iOutPrecision: u32,
                    iClipPrecision: u32,
                    iQuality: u32,
                    iPitchAndFamily: u32,
                    pszFaceName: *const u8,
                ) -> *mut c_void;
            }
            let hmono = CreateFontA(
                mono_height,
                0,
                0,
                0,
                400, // FW_NORMAL
                0,
                0,
                0,
                1, // DEFAULT_CHARSET
                0, // OUT_DEFAULT_PRECIS
                0, // CLIP_DEFAULT_PRECIS
                0, // DEFAULT_QUALITY
                0,
                b"Courier New\0".as_ptr(),
            );
            let hui = CreateFontA(
                ui_height,
                0,
                0,
                0,
                400, // FW_NORMAL
                0,
                0,
                0,
                1, // DEFAULT_CHARSET
                0, // OUT_DEFAULT_PRECIS
                0, // CLIP_DEFAULT_PRECIS
                0, // DEFAULT_QUALITY
                0,
                b"Tahoma\0".as_ptr(),
            );
            (HFONT(hmono), HFONT(hui))
        };

        FONT_MONO.store(hfont_mono.0 as isize, Ordering::SeqCst);
        FONT_UI.store(hfont_ui.0 as isize, Ordering::SeqCst);
    }
}

pub fn get_font_mono() -> HFONT {
    HFONT(FONT_MONO.load(Ordering::SeqCst) as *mut c_void)
}

pub fn get_font_ui() -> HFONT {
    HFONT(FONT_UI.load(Ordering::SeqCst) as *mut c_void)
}

pub fn set_control_font(hwnd: HWND, font: HFONT) {
    let _ = send_msg(hwnd, WM_SETFONT, font.0 as usize, 1);
}
