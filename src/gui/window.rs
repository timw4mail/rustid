//! Window management, window procedure, control layout, rendering, and main loop.

use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};

use rustid::Cpu;
#[allow(unused_imports)]
use rustid::common::{CpuDisplay, TDetect};

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    COLOR_BTNFACE, CreateSolidBrush, DeleteObject, HBRUSH, HDC, HGDIOBJ, InvalidateRect,
    SetBkColor, SetTextColor, UpdateWindow,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_F5};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use super::dialogs::{copy_to_clipboard, read_file_to_string, write_string_to_file};
#[cfg(x86_cpu)]
use super::dialogs::{export_dump_dialog, open_dump_file_dialog};
use super::menu::*;
use super::rtf::*;
use super::state::*;
use super::theme::*;

const EM_STREAMIN: u32 = WM_USER + 73;
const SF_RTF: usize = 0x0002;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORSTATIC: u32 = 0x0138;

#[repr(C)]
struct EDITSTREAM {
    dw_cookie: usize,
    dw_error: u32,
    pfn_callback: Option<
        unsafe extern "system" fn(
            dw_cookie: usize,
            pb_buff: *mut u8,
            cb: i32,
            pcb: *mut i32,
        ) -> u32,
    >,
}

struct StreamCookie<'a> {
    data: &'a [u8],
    offset: usize,
}

unsafe extern "system" fn edit_stream_in_callback(
    dw_cookie: usize,
    pb_buff: *mut u8,
    cb: i32,
    pcb: *mut i32,
) -> u32 {
    if dw_cookie == 0 || pb_buff.is_null() || pcb.is_null() || cb <= 0 {
        return 0;
    }
    unsafe {
        let cookie = &mut *(dw_cookie as *mut StreamCookie);
        let remaining = &cookie.data[cookie.offset..];
        let to_copy = remaining.len().min(cb as usize);
        if to_copy > 0 {
            std::ptr::copy_nonoverlapping(remaining.as_ptr(), pb_buff, to_copy);
            cookie.offset += to_copy;
            *pcb = to_copy as i32;
        } else {
            *pcb = 0;
        }
    }
    0
}

static BRUSH_DARK: AtomicIsize = AtomicIsize::new(0);
static BRUSH_LIGHT: AtomicIsize = AtomicIsize::new(0);

fn get_dark_brush() -> HBRUSH {
    let raw = BRUSH_DARK.load(Ordering::Relaxed);
    if raw != 0 {
        HBRUSH(raw as *mut c_void)
    } else {
        unsafe {
            let b = CreateSolidBrush(COLORREF(0x00261B1A));
            BRUSH_DARK.store(b.0 as isize, Ordering::Relaxed);
            b
        }
    }
}

fn get_light_brush() -> HBRUSH {
    let raw = BRUSH_LIGHT.load(Ordering::Relaxed);
    if raw != 0 {
        HBRUSH(raw as *mut c_void)
    } else {
        unsafe {
            let b = CreateSolidBrush(COLORREF(0x00FFFFFF));
            BRUSH_LIGHT.store(b.0 as isize, Ordering::Relaxed);
            b
        }
    }
}

fn set_richedit_content(hwnd_edit: HWND, doc: &str, plain: &str) {
    if IS_RICHEDIT.load(Ordering::Relaxed) {
        // Universal RichEdit path: stream RTF via EM_STREAMIN (supported on RichEdit 1.0, 2.0A, 2.0W, 5.0W)
        let doc_bytes = doc.as_bytes();
        let mut cookie = StreamCookie {
            data: doc_bytes,
            offset: 0,
        };
        let mut es = EDITSTREAM {
            dw_cookie: &mut cookie as *mut _ as usize,
            dw_error: 0,
            pfn_callback: Some(edit_stream_in_callback),
        };
        let _ = send_msg(hwnd_edit, EM_STREAMIN, SF_RTF, &mut es as *mut _ as isize);
    } else {
        // Plain EDIT fallback: use WM_SETTEXT with plain text
        if IS_UNICODE.load(Ordering::Relaxed) {
            let plain_u16 = to_pcwstr(plain);
            unsafe {
                let _ = SendMessageW(
                    hwnd_edit,
                    WM_SETTEXT,
                    Some(WPARAM(0)),
                    Some(LPARAM(plain_u16.as_ptr() as isize)),
                );
            }
        } else {
            unsafe extern "system" {
                fn SendMessageA(hWnd: *mut c_void, Msg: u32, wParam: usize, lParam: isize)
                -> isize;
            }
            let text_a = std::ffi::CString::new(plain).unwrap_or_default();
            unsafe {
                SendMessageA(hwnd_edit.0, 0x000C, 0, text_a.as_ptr() as isize); // WM_SETTEXT
            }
        }
    }
}

fn update_status_bar(state: &AppState, cpu: &Cpu) {
    #[cfg(x86_cpu)]
    let model = cpu.display_model_string();
    #[cfg(not(x86_cpu))]
    let model = if !cpu.model.is_empty() {
        &cpu.model
    } else {
        "CPU"
    };
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let part1 = format!("{} ({}-{})", model, arch, os);
    let part2 = if let Some(path) = &state.loaded_file {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path);
        format!("Dump File: {}", filename)
    } else {
        "Live Hardware".to_string()
    };

    let mode_str = match state.mode {
        ViewMode::Standard => "Standard",
        ViewMode::Debug => "Debug (-d)",
        ViewMode::Everything => "Everything (-e)",
        #[cfg(x86_cpu)]
        ViewMode::Dump => "CPUID Dump (-r)",
    };

    let part3 = format!(
        "{} | Colors: {} | Theme: {}",
        mode_str,
        if state.color { "On" } else { "Off" },
        if state.dark_theme { "Dark" } else { "Light" }
    );

    let p1_u16 = to_pcwstr(&part1);
    let p2_u16 = to_pcwstr(&part2);
    let p3_u16 = to_pcwstr(&part3);

    if IS_UNICODE.load(Ordering::Relaxed) {
        send_msg(state.hwnd_status, SB_SETTEXTW, 0, p1_u16.as_ptr() as isize);
        send_msg(state.hwnd_status, SB_SETTEXTW, 1, p2_u16.as_ptr() as isize);
        send_msg(state.hwnd_status, SB_SETTEXTW, 2, p3_u16.as_ptr() as isize);
    } else {
        unsafe extern "system" {
            fn SendMessageA(hWnd: *mut c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
        }
        let p1_a = std::ffi::CString::new(part1).unwrap_or_default();
        let p2_a = std::ffi::CString::new(part2).unwrap_or_default();
        let p3_a = std::ffi::CString::new(part3).unwrap_or_default();
        unsafe {
            SendMessageA(state.hwnd_status.0, 0x0400 + 1, 0, p1_a.as_ptr() as isize);
            SendMessageA(state.hwnd_status.0, 0x0400 + 1, 1, p2_a.as_ptr() as isize);
            SendMessageA(state.hwnd_status.0, 0x0400 + 1, 2, p3_a.as_ptr() as isize);
        }
    }
}

fn render_current_text(state: &mut AppState) {
    #[cfg(x86_cpu)]
    if let Some(path) = &state.loaded_file {
        if let Some(contents) = read_file_to_string(path) {
            let dump = rustid::x86::provider::CpuDump::parse_str(&contents);
            rustid::x86::provider::set_cpuid_provider(dump);
        } else {
            rustid::x86::provider::reset_cpuid_provider();
        }
    } else {
        rustid::x86::provider::reset_cpuid_provider();
    }

    let cpu = Cpu::detect();

    let is_from_dump = state.loaded_file.is_some();
    let plain_text = match state.mode {
        ViewMode::Standard => {
            generate_report_plain(&cpu, state.verbose, state.compact, is_from_dump)
        }
        ViewMode::Debug => generate_debug_info_plain(&cpu),
        ViewMode::Everything => {
            let report = generate_report_plain(&cpu, state.verbose, state.compact, is_from_dump);
            let debug = generate_debug_info_plain(&cpu);
            format!("{}\r\n--------------------\r\n\r\n{}", report, debug)
        }
        #[cfg(x86_cpu)]
        ViewMode::Dump => generate_dump_info_plain(),
    };

    state.current_plain_text = plain_text;

    // Set background color for RichEdit (not supported on plain EDIT or ANSI path)
    if IS_RICHEDIT.load(Ordering::Relaxed) {
        let bg_color = if state.dark_theme {
            COLORREF(0x00261B1A) // dark background #1a1b26 (BGR: 0x261B1A)
        } else {
            COLORREF(0x00FFFFFF) // white
        };
        send_msg(state.hwnd_edit, EM_SETBKGNDCOLOR, 0, bg_color.0 as isize);
    }

    // Format document and stream into RichEdit (or plain text for ANSI/EDIT fallback)
    let doc = to_rtf(&state.current_plain_text, state.dark_theme, state.color);
    set_richedit_content(state.hwnd_edit, &doc, &state.current_plain_text);

    unsafe {
        let _ = InvalidateRect(Some(state.hwnd_edit), None, true);
    }

    update_menu_checks(state);
    update_status_bar(state, &cpu);

    if !state.hwnd_main.is_invalid() {
        set_window_dark_titlebar(state.hwnd_main, state.dark_theme);
    }
}

fn relayout_controls(state: &AppState, client_w: i32, client_h: i32) {
    let dpi = state.dpi;
    let margin = scale(8, dpi);
    let status_h = scale(24, dpi);

    unsafe {
        // Relayout Status Bar
        let _ = SetWindowPos(
            state.hwnd_status,
            None,
            0,
            client_h - status_h,
            client_w,
            status_h,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );

        // Configure status bar parts
        let part1_w = scale(320, dpi);
        let part2_w = part1_w + scale(180, dpi);
        let parts = [part1_w, part2_w, -1];
        let _ = send_msg(
            state.hwnd_status,
            SB_SETPARTS,
            parts.len(),
            parts.as_ptr() as isize,
        );

        // Relayout RichEdit Text Control
        let edit_y = margin;
        let edit_w = client_w - 2 * margin;
        let edit_h = (client_h - status_h - 2 * margin).max(scale(50, dpi));

        let _ = SetWindowPos(
            state.hwnd_edit,
            None,
            margin,
            edit_y,
            edit_w,
            edit_h,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );

        // Set padding margins inside the rich edit control
        let pad = scale(12, dpi);
        let _ = send_msg(
            state.hwnd_edit,
            EM_SETMARGINS,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize,
            (pad as isize) | ((pad as isize) << 16),
        );
    }
}

#[repr(C)]
struct WNDCLASSEXA {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u8,
    lpsz_class_name: *const u8,
    h_icon_sm: *mut c_void,
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state_ptr = get_window_state(hwnd);

        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                let state_raw = create_struct.lpCreateParams as *mut AppState;
                set_window_state(hwnd, state_raw as usize as _);
                LRESULT(0)
            }
            WM_COMMAND => {
                let cmd_id = (wparam.0 & 0xFFFF) as u32;

                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;

                    match cmd_id {
                        IDM_FILE_OPEN =>
                        {
                            #[cfg(x86_cpu)]
                            if let Some(path) = open_dump_file_dialog(hwnd) {
                                state.loaded_file = Some(path);
                                render_current_text(state);
                            }
                        }
                        #[cfg(x86_cpu)]
                        IDM_FILE_EXPORT => {
                            let cpu = Cpu::detect();
                            let default_name = format!(
                                "cpuid_dump_{}.txt",
                                cpu.display_model_string().replace([' ', '/', '\\'], "_")
                            );
                            if let Some(save_path) = export_dump_dialog(hwnd, &default_name) {
                                let dump_content = generate_dump_info_plain();
                                if write_string_to_file(&save_path, &dump_content) {
                                    let msg_text =
                                        format!("CPUID dump successfully saved to:\n{}", save_path);
                                    if IS_UNICODE.load(Ordering::Relaxed) {
                                        let msg_u16 = to_pcwstr(&msg_text);
                                        let _ = MessageBoxW(
                                            Some(hwnd),
                                            PCWSTR(msg_u16.as_ptr()),
                                            w!("Export Complete"),
                                            MB_OK | MB_ICONINFORMATION,
                                        );
                                    } else {
                                        unsafe extern "system" {
                                            fn MessageBoxA(
                                                hWnd: *mut c_void,
                                                lpText: *const u8,
                                                lpCaption: *const u8,
                                                uType: u32,
                                            ) -> i32;
                                        }
                                        let txt_a =
                                            std::ffi::CString::new(msg_text).unwrap_or_default();
                                        MessageBoxA(
                                            hwnd.0,
                                            txt_a.as_ptr() as *const u8,
                                            b"Export Complete\0".as_ptr(),
                                            0x00000040, // MB_OK | MB_ICONINFORMATION
                                        );
                                    }
                                }
                            }
                        }
                        IDM_FILE_COPY => {
                            copy_to_clipboard(hwnd, &state.current_plain_text);
                        }
                        IDM_FILE_REFRESH => {
                            state.loaded_file = None;
                            render_current_text(state);
                        }
                        IDM_FILE_EXIT => {
                            let _ = DestroyWindow(hwnd);
                        }
                        IDM_MODE_STANDARD => {
                            state.mode = ViewMode::Standard;
                            render_current_text(state);
                        }
                        IDM_MODE_DEBUG => {
                            state.mode = ViewMode::Debug;
                            render_current_text(state);
                        }
                        IDM_MODE_EVERYTHING => {
                            state.mode = ViewMode::Everything;
                            render_current_text(state);
                        }
                        #[cfg(x86_cpu)]
                        IDM_MODE_DUMP => {
                            state.mode = ViewMode::Dump;
                            render_current_text(state);
                        }
                        IDM_OPT_COLOR => {
                            state.color = !state.color;
                            render_current_text(state);
                        }
                        IDM_OPT_DARK_THEME => {
                            state.dark_theme = !state.dark_theme;
                            state.custom_theme_set = true;
                            render_current_text(state);
                        }
                        IDM_OPT_VERBOSE => {
                            state.verbose = !state.verbose;
                            render_current_text(state);
                        }
                        IDM_OPT_COMPACT => {
                            state.compact = !state.compact;
                            render_current_text(state);
                        }
                        IDM_HELP_ABOUT => {
                            let about_text = format!(
                                "Rustid v{}\nMulti-architecture CPU detection tool\nRunning on {}-{}",
                                env!("CARGO_PKG_VERSION"),
                                std::env::consts::ARCH,
                                std::env::consts::OS
                            );
                            if IS_UNICODE.load(Ordering::Relaxed) {
                                let about_u16 = to_pcwstr(&about_text);
                                let _ = MessageBoxW(
                                    Some(hwnd),
                                    PCWSTR(about_u16.as_ptr()),
                                    w!("About Rustid"),
                                    MB_OK | MB_ICONINFORMATION,
                                );
                            } else {
                                unsafe extern "system" {
                                    fn MessageBoxA(
                                        hWnd: *mut c_void,
                                        lpText: *const u8,
                                        lpCaption: *const u8,
                                        uType: u32,
                                    ) -> i32;
                                }
                                let txt_a = std::ffi::CString::new(about_text).unwrap_or_default();
                                MessageBoxA(
                                    hwnd.0,
                                    txt_a.as_ptr() as *const u8,
                                    b"About Rustid\0".as_ptr(),
                                    0x00000040,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let key = wparam.0;
                let is_ctrl = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    if key == VK_F5.0 as usize {
                        state.loaded_file = None;
                        render_current_text(state);
                        return LRESULT(0);
                    } else if is_ctrl {
                        match key as u8 {
                            b'O' => {
                                #[cfg(x86_cpu)]
                                if let Some(path) = open_dump_file_dialog(hwnd) {
                                    state.loaded_file = Some(path);
                                    render_current_text(state);
                                }
                                return LRESULT(0);
                            }
                            #[cfg(x86_cpu)]
                            b'S' => {
                                let cpu = Cpu::detect();
                                let default_name = format!(
                                    "cpuid_dump_{}.txt",
                                    cpu.display_model_string().replace(' ', "_")
                                );
                                if let Some(save_path) = export_dump_dialog(hwnd, &default_name) {
                                    let dump_content = generate_dump_info_plain();
                                    let _ = write_string_to_file(&save_path, &dump_content);
                                }
                                return LRESULT(0);
                            }
                            b'C' => {
                                copy_to_clipboard(hwnd, &state.current_plain_text);
                                return LRESULT(0);
                            }
                            b'1' => {
                                state.mode = ViewMode::Standard;
                                render_current_text(state);
                                return LRESULT(0);
                            }
                            b'2' => {
                                state.mode = ViewMode::Debug;
                                render_current_text(state);
                                return LRESULT(0);
                            }
                            b'3' => {
                                state.mode = ViewMode::Everything;
                                render_current_text(state);
                                return LRESULT(0);
                            }
                            #[cfg(x86_cpu)]
                            b'4' => {
                                state.mode = ViewMode::Dump;
                                render_current_text(state);
                                return LRESULT(0);
                            }
                            _ => {}
                        }
                    }
                }
                if IS_UNICODE.load(Ordering::Relaxed) {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                } else {
                    unsafe extern "system" {
                        fn DefWindowProcA(
                            hWnd: *mut c_void,
                            Msg: u32,
                            wParam: usize,
                            lParam: isize,
                        ) -> isize;
                    }
                    LRESULT(DefWindowProcA(hwnd.0, msg, wparam.0, lparam.0))
                }
            }
            WM_SIZE => {
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    let w = (lparam.0 & 0xFFFF) as i32;
                    let h = ((lparam.0 >> 16) & 0xFFFF) as i32;
                    relayout_controls(state, w, h);
                }
                LRESULT(0)
            }
            WM_GETMINMAXINFO => {
                let minmax = &mut *(lparam.0 as *mut MINMAXINFO);
                let dpi = if !state_ptr.is_null() {
                    (*state_ptr).dpi
                } else {
                    get_system_dpi()
                };
                minmax.ptMinTrackSize = POINT {
                    x: scale(580, dpi),
                    y: scale(380, dpi),
                };
                LRESULT(0)
            }
            WM_DPICHANGED => {
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    let new_dpi = (wparam.0 & 0xFFFF) as u32;
                    state.dpi = new_dpi;
                    create_fonts(new_dpi);

                    set_control_font(state.hwnd_edit, get_font_mono());
                    set_control_font(state.hwnd_status, get_font_ui());

                    let rect = &*(lparam.0 as *const RECT);
                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );

                    let mut client_rc = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client_rc);
                    relayout_controls(state, client_rc.right, client_rc.bottom);
                }
                LRESULT(0)
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    let hdc = HDC(wparam.0 as *mut c_void);
                    if state.dark_theme {
                        let _ = SetTextColor(hdc, COLORREF(0x00D4D4D4));
                        let _ = SetBkColor(hdc, COLORREF(0x00261B1A));
                        return LRESULT(get_dark_brush().0 as isize);
                    } else {
                        let _ = SetTextColor(hdc, COLORREF(0x001E1E1E));
                        let _ = SetBkColor(hdc, COLORREF(0x00FFFFFF));
                        return LRESULT(get_light_brush().0 as isize);
                    }
                }
                LRESULT(0)
            }
            WM_SETTINGCHANGE => {
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    if !state.custom_theme_set {
                        let sys_dark = is_system_dark_theme();
                        if state.dark_theme != sys_dark {
                            state.dark_theme = sys_dark;
                            render_current_text(state);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                if !state_ptr.is_null() {
                    let _ = Box::from_raw(state_ptr);
                    set_window_state(hwnd, 0);
                }
                let dark_b = BRUSH_DARK.swap(0, Ordering::SeqCst);
                if dark_b != 0 {
                    let _ = DeleteObject(HGDIOBJ(dark_b as *mut c_void));
                }
                let light_b = BRUSH_LIGHT.swap(0, Ordering::SeqCst);
                if light_b != 0 {
                    let _ = DeleteObject(HGDIOBJ(light_b as *mut c_void));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => {
                if IS_UNICODE.load(Ordering::Relaxed) {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                } else {
                    unsafe extern "system" {
                        fn DefWindowProcA(
                            hWnd: *mut c_void,
                            Msg: u32,
                            wParam: usize,
                            lParam: isize,
                        ) -> isize;
                    }
                    LRESULT(DefWindowProcA(hwnd.0, msg, wparam.0, lparam.0))
                }
            }
        }
    }
}

pub fn run() {
    init_dpi_awareness();
    init_common_controls();

    unsafe {
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let class_name_w = w!("RustidModernMainWindowClass");
        let class_name_a = b"RustidModernMainWindowClass\0";

        let wc_w = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(hinstance.0),
            hIcon: HICON::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH((COLOR_BTNFACE.0 + 1) as *mut c_void),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name_w,
            hIconSm: HICON::default(),
        };

        let reg = RegisterClassExW(&wc_w);
        if reg == 0 {
            // Windows 95 / 98 / ME fallback: Register ANSI window class
            IS_UNICODE.store(false, Ordering::Relaxed);
            unsafe extern "system" {
                fn RegisterClassExA(lpwcx: *const WNDCLASSEXA) -> u16;
                fn LoadCursorA(hInstance: *mut c_void, lpCursorName: *const u8) -> *mut c_void;
            }
            let wc_a = WNDCLASSEXA {
                cb_size: std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: 3, // CS_HREDRAW | CS_VREDRAW
                lpfn_wnd_proc: Some(main_wnd_proc),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: hinstance.0,
                h_icon: std::ptr::null_mut(),
                h_cursor: LoadCursorA(std::ptr::null_mut(), 32512 as *const u8),
                hbr_background: (COLOR_BTNFACE.0 + 1) as *mut c_void,
                lpsz_menu_name: std::ptr::null(),
                lpsz_class_name: class_name_a.as_ptr(),
                h_icon_sm: std::ptr::null_mut(),
            };
            let reg_a = RegisterClassExA(&wc_a);
            if reg_a == 0 {
                return;
            }
        }

        let dpi = get_system_dpi();
        create_fonts(dpi);

        let win_w = scale(820, dpi);
        let win_h = scale(640, dpi);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h - win_h) / 2;

        let hmenu_bar = create_main_menu();
        let is_dark = is_system_dark_theme();

        let app_state = Box::new(AppState {
            hwnd_main: HWND::default(),
            hwnd_edit: HWND::default(),
            hwnd_status: HWND::default(),
            hmenu: hmenu_bar,
            dpi,
            mode: ViewMode::Standard,
            color: true,
            dark_theme: is_dark,
            custom_theme_set: false,
            verbose: false,
            compact: false,
            loaded_file: None,
            current_plain_text: String::new(),
        });

        let state_raw_ptr = Box::into_raw(app_state);

        let title = format!(
            "Rustid {} - CPU Information ({}-{})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::ARCH,
            std::env::consts::OS
        );

        let hwnd_main = if IS_UNICODE.load(Ordering::Relaxed) {
            let title_u16 = to_pcwstr(&title);
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name_w,
                PCWSTR(title_u16.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                win_x,
                win_y,
                win_w,
                win_h,
                None,
                Some(hmenu_bar),
                Some(HINSTANCE(hinstance.0)),
                Some(state_raw_ptr as *const c_void),
            )
            .unwrap_or_default()
        } else {
            unsafe extern "system" {
                fn CreateWindowExA(
                    dwExStyle: u32,
                    lpClassName: *const u8,
                    lpWindowName: *const u8,
                    dwStyle: u32,
                    X: i32,
                    Y: i32,
                    nWidth: i32,
                    nHeight: i32,
                    hWndParent: *mut c_void,
                    hMenu: *mut c_void,
                    hInstance: *mut c_void,
                    lpParam: *mut c_void,
                ) -> *mut c_void;
            }
            let title_a = std::ffi::CString::new(title.clone()).unwrap_or_default();
            let h = CreateWindowExA(
                0,
                class_name_a.as_ptr(),
                title_a.as_ptr() as *const u8,
                0x00CF0000, // WS_OVERLAPPEDWINDOW
                win_x,
                win_y,
                win_w,
                win_h,
                std::ptr::null_mut(),
                hmenu_bar.0 as *mut c_void,
                hinstance.0,
                state_raw_ptr as *mut c_void,
            );
            HWND(h)
        };

        if hwnd_main.is_invalid() {
            return;
        }

        let state = &mut *state_raw_ptr;
        state.hwnd_main = hwnd_main;

        // Load RichEdit DLL so RichEdit20A/RichEdit20W classes are registered.
        // riched20.dll = RichEdit 2.0/3.0 (Win98+/NT4 SP3+).
        // riched32.dll = RichEdit 1.0 (Win95 original, registers "RICHEDIT" class).
        // We leak the handle intentionally — it must stay loaded for the window lifetime.
        {
            unsafe extern "system" {
                fn LoadLibraryA(lpFileName: *const u8) -> *mut c_void;
            }
            let mut riched_loaded = false;
            for dll in [b"riched20.dll\0".as_ptr(), b"riched32.dll\0".as_ptr()] {
                let h = LoadLibraryA(dll);
                if !h.is_null() {
                    riched_loaded = true;
                    break;
                }
            }
            IS_RICHEDIT.store(riched_loaded, Ordering::Relaxed);
        }

        // Create RichEdit / Edit Control with fallback for legacy Windows
        let mut hwnd_edit = HWND::default();
        if IS_UNICODE.load(Ordering::Relaxed) {
            // Unicode path: try RichEdit 5 → 2 → 1 → plain EDIT
            let edit_classes = [
                w!("RichEdit50W"),
                w!("RichEdit20W"),
                w!("RichEdit20A"),
                w!("RICHEDIT"),
                w!("EDIT"),
            ];
            for (i, class) in edit_classes.iter().enumerate() {
                if let Ok(h) = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    *class,
                    PCWSTR::null(),
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | WS_HSCROLL
                        | WS_TABSTOP
                        | WINDOW_STYLE(
                            (ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL | ES_AUTOHSCROLL) as u32,
                        ),
                    0,
                    0,
                    0,
                    0,
                    Some(hwnd_main),
                    Some(HMENU(IDC_RICHEDIT as *mut c_void)),
                    Some(HINSTANCE(hinstance.0)),
                    None,
                ) && !h.is_invalid()
                {
                    hwnd_edit = h;
                    // If we fell all the way to "EDIT", it's not a RichEdit
                    if i == edit_classes.len() - 1 {
                        IS_RICHEDIT.store(false, Ordering::Relaxed);
                    }
                    break;
                }
            }
        } else {
            unsafe extern "system" {
                fn CreateWindowExA(
                    dwExStyle: u32,
                    lpClassName: *const u8,
                    lpWindowName: *const u8,
                    dwStyle: u32,
                    X: i32,
                    Y: i32,
                    nWidth: i32,
                    nHeight: i32,
                    hWndParent: *mut c_void,
                    hMenu: *mut c_void,
                    hInstance: *mut c_void,
                    lpParam: *mut c_void,
                ) -> *mut c_void;
            }
            // ANSI path: try RichEdit 2.0A → 1.0 (RICHEDIT) → plain EDIT
            let edit_classes_a: &[&[u8]] = &[b"RichEdit20A\0", b"RICHEDIT\0", b"EDIT\0", b"Edit\0"];
            for (i, class) in edit_classes_a.iter().enumerate() {
                let h = CreateWindowExA(
                    0,
                    class.as_ptr(),
                    std::ptr::null(),
                    0x50300000 | 0x0004 | 0x0800 | 0x0040 | 0x0100, // WS_CHILD|WS_VISIBLE|WS_VSCROLL|WS_HSCROLL|ES_MULTILINE|ES_READONLY|ES_AUTOVSCROLL|ES_AUTOHSCROLL
                    0,
                    0,
                    0,
                    0,
                    hwnd_main.0,
                    IDC_RICHEDIT as *mut c_void,
                    hinstance.0,
                    std::ptr::null_mut(),
                );
                if !h.is_null() {
                    hwnd_edit = HWND(h);
                    // If we fell all the way to plain "EDIT" or "Edit", mark not richedit
                    if i >= edit_classes_a.len() - 2 {
                        IS_RICHEDIT.store(false, Ordering::Relaxed);
                    }
                    break;
                }
            }
        }

        set_control_font(hwnd_edit, get_font_mono());
        // EM_EXLIMITTEXT is a RichEdit-only message; skip on plain EDIT
        if IS_RICHEDIT.load(Ordering::Relaxed) {
            let _ = send_msg(hwnd_edit, EM_EXLIMITTEXT, 0, 0x7FFFFFFE);
        }
        state.hwnd_edit = hwnd_edit;

        // Create Status Bar (msctls_statusbar32)
        let hwnd_status = if IS_UNICODE.load(Ordering::Relaxed) {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("msctls_statusbar32"),
                PCWSTR::null(),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SBARS_SIZEGRIP),
                0,
                0,
                0,
                0,
                Some(hwnd_main),
                Some(HMENU(IDC_STATUSBAR as *mut c_void)),
                Some(HINSTANCE(hinstance.0)),
                None,
            )
            .unwrap_or_default()
        } else {
            unsafe extern "system" {
                fn CreateWindowExA(
                    dwExStyle: u32,
                    lpClassName: *const u8,
                    lpWindowName: *const u8,
                    dwStyle: u32,
                    X: i32,
                    Y: i32,
                    nWidth: i32,
                    nHeight: i32,
                    hWndParent: *mut c_void,
                    hMenu: *mut c_void,
                    hInstance: *mut c_void,
                    lpParam: *mut c_void,
                ) -> *mut c_void;
            }
            let h = CreateWindowExA(
                0,
                b"msctls_statusbar32\0".as_ptr(),
                std::ptr::null(),
                0x50000000 | 0x0100, // WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP
                0,
                0,
                0,
                0,
                hwnd_main.0,
                IDC_STATUSBAR as *mut c_void,
                hinstance.0,
                std::ptr::null_mut(),
            );
            HWND(h)
        };
        set_control_font(hwnd_status, get_font_ui());
        state.hwnd_status = hwnd_status;

        let mut client_rc = RECT::default();
        let _ = GetClientRect(hwnd_main, &mut client_rc);
        relayout_controls(state, client_rc.right, client_rc.bottom);

        // Initial render
        render_current_text(state);

        let _ = ShowWindow(hwnd_main, SW_SHOW);
        let _ = UpdateWindow(hwnd_main);

        let mut msg = MSG::default();
        if IS_UNICODE.load(Ordering::Relaxed) {
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        } else {
            unsafe extern "system" {
                fn GetMessageA(
                    lpMsg: *mut MSG,
                    hWnd: *mut c_void,
                    wMsgFilterMin: u32,
                    wMsgFilterMax: u32,
                ) -> i32;
                fn DispatchMessageA(lpMsg: *const MSG) -> isize;
            }
            while GetMessageA(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }
}
