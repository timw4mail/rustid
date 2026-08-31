#![cfg_attr(windows_os, windows_subsystem = "windows")]

#[cfg(not(windows_os))]
fn main() {
    eprintln!("rustid-gui is currently supported only on Windows targets.");
}

#[cfg(windows_os)]
mod gui {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicIsize, Ordering};

    use rustid::Cpu;
    #[allow(unused_imports)]
    use rustid::common::{CliFlags, CpuDisplay, Level1Cache, TCpuDisplay, TDetect, UNK};

    use windows::Win32::Foundation::{
        COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
    };
    use windows::Win32::Graphics::Gdi::{
        CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, CreateFontW, DEFAULT_CHARSET,
        DeleteObject, FW_NORMAL, GetDC, GetDeviceCaps, HBRUSH, HFONT, HGDIOBJ, LOGPIXELSX,
        OUT_DEFAULT_PRECIS, ReleaseDC, UpdateWindow,
    };
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, LoadLibraryW};
    use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW,
        RegQueryValueExW,
    };
    #[cfg(x86_cpu)]
    use windows::Win32::UI::Controls::Dialogs::{
        GetOpenFileNameW, GetSaveFileNameW, OFN_FILEMUSTEXIST, OFN_OVERWRITEPROMPT,
        OFN_PATHMUSTEXIST, OPENFILENAMEW,
    };
    use windows::Win32::UI::Controls::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_F5};
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::{PCWSTR, w};

    // Menu Item IDs
    const IDM_FILE_OPEN: u32 = 101;
    #[cfg(x86_cpu)]
    const IDM_FILE_EXPORT: u32 = 102;
    const IDM_FILE_COPY: u32 = 103;
    const IDM_FILE_REFRESH: u32 = 104;
    const IDM_FILE_EXIT: u32 = 105;

    const IDM_MODE_STANDARD: u32 = 201;
    const IDM_MODE_DEBUG: u32 = 202;
    const IDM_MODE_EVERYTHING: u32 = 203;
    #[cfg(x86_cpu)]
    const IDM_MODE_DUMP: u32 = 204;

    const IDM_OPT_COLOR: u32 = 301;
    const IDM_OPT_DARK_THEME: u32 = 302;
    const IDM_OPT_VERBOSE: u32 = 303;
    const IDM_OPT_COMPACT: u32 = 304;

    const IDM_HELP_ABOUT: u32 = 401;

    const IDC_STATUSBAR: i32 = 5001;
    const IDC_RICHEDIT: i32 = 5002;

    const CF_UNICODETEXT_FORMAT: u32 = 13;

    // RichEdit constants
    const EM_SETBKGNDCOLOR: u32 = WM_USER + 67;
    const EM_SETMARGINS: u32 = 0x00D3;
    const EM_EXLIMITTEXT: u32 = WM_USER + 53;
    const EC_LEFTMARGIN: usize = 0x0001;
    const EC_RIGHTMARGIN: usize = 0x0002;

    static FONT_MONO: AtomicIsize = AtomicIsize::new(0);
    static FONT_UI: AtomicIsize = AtomicIsize::new(0);

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum ViewMode {
        Standard,
        Debug,
        Everything,
        #[cfg(x86_cpu)]
        Dump,
    }

    struct AppState {
        hwnd_main: HWND,
        hwnd_edit: HWND,
        hwnd_status: HWND,
        hmenu: HMENU,
        dpi: u32,
        mode: ViewMode,
        color: bool,
        dark_theme: bool,
        custom_theme_set: bool,
        verbose: bool,
        compact: bool,
        loaded_file: Option<String>,
        current_plain_text: String,
    }

    fn is_system_dark_theme() -> bool {
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

    fn set_window_dark_titlebar(hwnd: HWND, dark: bool) {
        unsafe {
            let dark_val: i32 = if dark { 1 } else { 0 };
            if let Ok(dwmapi) = LoadLibraryW(w!("dwmapi.dll")) {
                type DwmSetWindowAttributeFn = unsafe extern "system" fn(
                    HWND,
                    u32,
                    *const c_void,
                    u32,
                )
                    -> windows::core::HRESULT;

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

    fn to_pcwstr(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn scale(val: i32, dpi: u32) -> i32 {
        (val * dpi as i32) / 96
    }

    fn send_msg(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
        unsafe { SendMessageW(hwnd, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }
    }

    fn init_dpi_awareness() {
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

    fn get_system_dpi() -> u32 {
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

    fn init_common_controls() {
        unsafe {
            let icc = INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_STANDARD_CLASSES | ICC_BAR_CLASSES,
            };
            let _ = InitCommonControlsEx(&icc);
            let _ = LoadLibraryW(w!("msftedit.dll"));
        }
    }

    fn create_fonts(dpi: u32) {
        unsafe {
            let old_mono = FONT_MONO.swap(0, Ordering::SeqCst);
            if old_mono != 0 {
                let _ = DeleteObject(HGDIOBJ(old_mono as *mut c_void));
            }
            let old_ui = FONT_UI.swap(0, Ordering::SeqCst);
            if old_ui != 0 {
                let _ = DeleteObject(HGDIOBJ(old_ui as *mut c_void));
            }

            let mono_height = -scale(14, dpi);
            let hfont_mono = CreateFontW(
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
            FONT_MONO.store(hfont_mono.0 as isize, Ordering::SeqCst);

            let ui_height = -scale(12, dpi);
            let hfont_ui = CreateFontW(
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
            FONT_UI.store(hfont_ui.0 as isize, Ordering::SeqCst);
        }
    }

    fn get_font_mono() -> HFONT {
        HFONT(FONT_MONO.load(Ordering::SeqCst) as *mut c_void)
    }

    fn get_font_ui() -> HFONT {
        HFONT(FONT_UI.load(Ordering::SeqCst) as *mut c_void)
    }

    fn set_control_font(hwnd: HWND, font: HFONT) {
        let _ = send_msg(hwnd, WM_SETFONT, font.0 as usize, 1);
    }

    fn copy_to_clipboard(hwnd_owner: HWND, text: &str) {
        unsafe {
            let text_u16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes_len = text_u16.len() * std::mem::size_of::<u16>();

            let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, bytes_len) else {
                return;
            };

            let ptr = GlobalLock(hmem) as *mut u16;
            if ptr.is_null() {
                return;
            }
            std::ptr::copy_nonoverlapping(text_u16.as_ptr(), ptr, text_u16.len());
            let _ = GlobalUnlock(hmem);

            if OpenClipboard(Some(hwnd_owner)).is_ok() {
                let _ = EmptyClipboard();
                let _ = SetClipboardData(
                    CF_UNICODETEXT_FORMAT,
                    Some(windows::Win32::Foundation::HANDLE(hmem.0)),
                );
                let _ = CloseClipboard();
            }
        }
    }

    #[cfg(x86_cpu)]
    fn open_dump_file_dialog(hwnd_parent: HWND) -> Option<String> {
        let mut file_buf = [0u16; 1024];
        let filter_str = "CPUID Dump (*.txt;*.dump)\0*.txt;*.dump\0All Files (*.*)\0*.*\0\0";
        let filter: Vec<u16> = filter_str.encode_utf16().collect();

        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd_parent,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            lpstrFile: windows::core::PWSTR(file_buf.as_mut_ptr()),
            nMaxFile: file_buf.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
            ..Default::default()
        };

        if unsafe { GetOpenFileNameW(&mut ofn) }.as_bool() {
            let len = file_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(file_buf.len());
            Some(String::from_utf16_lossy(&file_buf[..len]))
        } else {
            None
        }
    }

    #[cfg(x86_cpu)]
    fn export_dump_dialog(hwnd_parent: HWND, default_filename: &str) -> Option<String> {
        let mut file_buf = [0u16; 1024];
        let def_u16 = to_pcwstr(default_filename);
        let copy_len = def_u16.len().min(file_buf.len() - 1);
        file_buf[..copy_len].copy_from_slice(&def_u16[..copy_len]);

        let filter_str = "CPUID Dump (*.txt)\0*.txt\0All Files (*.*)\0*.*\0\0";
        let filter: Vec<u16> = filter_str.encode_utf16().collect();

        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd_parent,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            lpstrFile: windows::core::PWSTR(file_buf.as_mut_ptr()),
            nMaxFile: file_buf.len() as u32,
            lpstrDefExt: w!("txt"),
            Flags: OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT,
            ..Default::default()
        };

        if unsafe { GetSaveFileNameW(&mut ofn) }.as_bool() {
            let len = file_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(file_buf.len());
            Some(String::from_utf16_lossy(&file_buf[..len]))
        } else {
            None
        }
    }

    fn generate_report_plain(
        cpu: &Cpu,
        verbose: bool,
        compact: bool,
        is_from_dump: bool,
    ) -> String {
        let version_header = if is_from_dump {
            rustid::format_file_version()
        } else {
            rustid::format_version()
        };
        let flags = CliFlags {
            color: false,
            compact,
            verbose,
        };
        let sep = if compact { "\r\n" } else { "\r\n\r\n" };
        let table = cpu.render_table(flags);
        let crlf_table = table.replace("\r\n", "\n").replace('\n', "\r\n");
        format!("{}{}{}", version_header, sep, crlf_table)
    }

    fn generate_debug_info_plain(cpu: &Cpu) -> String {
        cpu.render_debug()
            .replace("\r\n", "\n")
            .replace('\n', "\r\n")
    }

    #[cfg(x86_cpu)]
    fn generate_dump_info_plain() -> String {
        use rustid::x86::{dump::dump_cpu, topology::Topology};
        let mut output = String::new();
        let topo = Topology::detect();
        let logical_cores = topo.threads.count as usize;
        for i in 0..logical_cores {
            dump_cpu(&mut output, i);
        }
        output.replace("\r\n", "\n").replace('\n', "\r\n")
    }

    fn rtf_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 10);
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '{' => out.push_str("\\{"),
                '}' => out.push_str("\\}"),
                '\r' => {}
                '\n' => out.push_str("\\par\r\n"),
                c if (c as u32) < 128 => out.push(c),
                c => {
                    let code = c as u32;
                    if code <= 0xFFFF {
                        out.push_str(&format!("\\u{}?", code as i16));
                    } else {
                        out.push('?');
                    }
                }
            }
        }
        out
    }

    fn to_rtf(plain_text: &str, dark_theme: bool, color: bool) -> String {
        if !color {
            let escaped = rtf_escape(plain_text);
            return if dark_theme {
                format!(
                    "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{{\\fonttbl{{\\f0\\fnil\\fcharset0 Consolas;}}}}{{\\colortbl ;\\red212\\green212\\blue212;}}\\viewkind4\\uc1\\f0\\fs22\\cf1 {}\\par}}",
                    escaped
                )
            } else {
                format!(
                    "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{{\\fonttbl{{\\f0\\fnil\\fcharset0 Consolas;}}}}{{\\colortbl ;\\red30\\green30\\blue30;}}\\viewkind4\\uc1\\f0\\fs22\\cf1 {}\\par}}",
                    escaped
                )
            };
        }

        // Palette definition
        // Color 1: Green (Main labels)
        // Color 2: Blue/Cyan (Sublabels / Header banner)
        // Color 3: Body text (Off-white or Charcoal)
        // Color 4: Warm highlight (Numbers, Hex, Features)
        // Color 5: Muted gray (Dividers)
        let color_tbl = if dark_theme {
            "{\\colortbl ;\\red115\\green218\\blue202;\\red125\\green207\\blue255;\\red212\\green212\\blue212;\\red255\\green158\\blue100;\\red86\\green95\\blue137;}"
        } else {
            "{\\colortbl ;\\red9\\green134\\blue88;\\red4\\green81\\blue165;\\red30\\green30\\blue30;\\red163\\green21\\blue21;\\red110\\green118\\blue129;}"
        };

        let mut rtf = format!(
            "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{{\\fonttbl{{\\f0\\fnil\\fcharset0 Consolas;}}}}{color_tbl}\\viewkind4\\uc1\\f0\\fs22 "
        );

        for line in plain_text.lines() {
            if line.trim().is_empty() {
                rtf.push_str("\\par\r\n");
                continue;
            }

            // Header line (e.g. --------------- Rustid ... ---------------)
            if line.starts_with("---------------") || line.starts_with("--------------------") {
                rtf.push_str(&format!("\\cf2 {}\\par\r\n", rtf_escape(line)));
                continue;
            }

            // Core # heading line
            if line.trim_start().starts_with("Core #") {
                rtf.push_str(&format!("\\cf2\\b {}\\b0\\par\r\n", rtf_escape(line)));
                continue;
            }

            // Standard line with labels: e.g. "        Vendor: AMD (AuthenticAMD)"
            if line.len() >= 16 && &line[14..16] == ": " {
                let label_part = &line[..14];
                let rest_part = &line[16..];

                // Check for inline sublabel e.g. "Frequency: Base: "
                if let Some(colon_pos) = rest_part.find(": ")
                    && colon_pos < 12
                {
                    let sub_label = &rest_part[..colon_pos];
                    let val = &rest_part[colon_pos + 2..];
                    rtf.push_str(&format!(
                        "\\cf1 {}: \\cf2 {}: \\cf3 {}\\par\r\n",
                        rtf_escape(label_part),
                        rtf_escape(sub_label),
                        rtf_escape(val)
                    ));
                    continue;
                }

                rtf.push_str(&format!(
                    "\\cf1 {}: \\cf3 {}\\par\r\n",
                    rtf_escape(label_part),
                    rtf_escape(rest_part)
                ));
                continue;
            }

            // Sublabel line e.g. "                L1i: ..." or "                (11, 15, ...)"
            if let Some(sub_rest) = line.strip_prefix("                ") {
                if let Some(colon_idx) = sub_rest.find(": ") {
                    let sub_lbl = &sub_rest[..colon_idx];
                    let sub_val = &sub_rest[colon_idx + 2..];
                    rtf.push_str(&format!(
                        "\\cf5                 \\cf2 {}: \\cf3 {}\\par\r\n",
                        rtf_escape(sub_lbl),
                        rtf_escape(sub_val)
                    ));
                    continue;
                } else if sub_rest.starts_with('(') {
                    rtf.push_str(&format!(
                        "\\cf5                 \\cf4 {}\\par\r\n",
                        rtf_escape(sub_rest)
                    ));
                    continue;
                }
            }

            // Default line
            rtf.push_str(&format!("\\cf3 {}\\par\r\n", rtf_escape(line)));
        }

        rtf.push('}');
        rtf
    }

    const EM_SETTEXTEX: u32 = WM_USER + 97;

    #[repr(C)]
    #[allow(clippy::upper_case_acronyms)]
    struct SETTEXTEX {
        flags: u32,
        codepage: u32,
    }

    fn set_richedit_content(hwnd_edit: HWND, doc: &str) {
        let doc_bytes: Vec<u8> = doc.bytes().chain(std::iter::once(0)).collect();
        let st = SETTEXTEX {
            flags: 0,
            codepage: 0,
        };
        unsafe {
            let _ = SendMessageW(
                hwnd_edit,
                EM_SETTEXTEX,
                Some(WPARAM(&st as *const _ as usize)),
                Some(LPARAM(doc_bytes.as_ptr() as isize)),
            );
        }
    }

    fn update_menu_checks(state: &AppState) {
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

        send_msg(state.hwnd_status, SB_SETTEXTW, 0, p1_u16.as_ptr() as isize);
        send_msg(state.hwnd_status, SB_SETTEXTW, 1, p2_u16.as_ptr() as isize);
        send_msg(state.hwnd_status, SB_SETTEXTW, 2, p3_u16.as_ptr() as isize);
    }

    fn render_current_text(state: &mut AppState) {
        #[cfg(x86_cpu)]
        if let Some(path) = &state.loaded_file {
            let dump = rustid::x86::provider::CpuDump::parse_file(path);
            rustid::x86::provider::set_cpuid_provider(dump);
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
                let report =
                    generate_report_plain(&cpu, state.verbose, state.compact, is_from_dump);
                let debug = generate_debug_info_plain(&cpu);
                format!("{}\r\n--------------------\r\n\r\n{}", report, debug)
            }
            #[cfg(x86_cpu)]
            ViewMode::Dump => generate_dump_info_plain(),
        };

        state.current_plain_text = plain_text;

        // Set background color for RichEdit
        let bg_color = if state.dark_theme {
            COLORREF(0x00261B1A) // dark background #1a1b26 (BGR: 0x261B1A)
        } else {
            COLORREF(0x00FFFFFF) // white
        };
        send_msg(state.hwnd_edit, EM_SETBKGNDCOLOR, 0, bg_color.0 as isize);

        // Format document and stream into RichEdit
        let doc = to_rtf(&state.current_plain_text, state.dark_theme, state.color);
        set_richedit_content(state.hwnd_edit, &doc);

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
                EC_LEFTMARGIN | EC_RIGHTMARGIN,
                (pad as isize) | ((pad as isize) << 16),
            );
        }
    }

    fn create_main_menu() -> HMENU {
        unsafe {
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
        }
    }

    unsafe extern "system" fn main_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;

            match msg {
                WM_CREATE => {
                    let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                    let state_raw = create_struct.lpCreateParams as *mut AppState;
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_raw as isize);
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
                                    if std::fs::write(&save_path, dump_content).is_ok() {
                                        let msg_text = format!(
                                            "CPUID dump successfully saved to:\n{}",
                                            save_path
                                        );
                                        let msg_u16 = to_pcwstr(&msg_text);
                                        let _ = MessageBoxW(
                                            Some(hwnd),
                                            PCWSTR(msg_u16.as_ptr()),
                                            w!("Export Complete"),
                                            MB_OK | MB_ICONINFORMATION,
                                        );
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
                                    "Rustid v{}\nMulti-architecture CPU detection tool\nRunning on {}-{}\n\nFeatures: Standard Table, Debug, Everything, CPUID Dump, Native Colors, Cross-Compiled.",
                                    env!("CARGO_PKG_VERSION"),
                                    std::env::consts::ARCH,
                                    std::env::consts::OS
                                );
                                let about_u16 = to_pcwstr(&about_text);
                                let _ = MessageBoxW(
                                    Some(hwnd),
                                    PCWSTR(about_u16.as_ptr()),
                                    w!("About Rustid"),
                                    MB_OK | MB_ICONINFORMATION,
                                );
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
                                    if let Some(save_path) = export_dump_dialog(hwnd, &default_name)
                                    {
                                        let dump_content = generate_dump_info_plain();
                                        let _ = std::fs::write(&save_path, dump_content);
                                    }
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
                    DefWindowProcW(hwnd, msg, wparam, lparam)
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
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0_isize as _);
                    }
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    pub fn run() {
        init_dpi_awareness();
        init_common_controls();

        unsafe {
            let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
            let class_name = w!("RustidModernMainWindowClass");

            let wc = WNDCLASSEXW {
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
                lpszClassName: class_name,
                hIconSm: HICON::default(),
            };

            let reg = RegisterClassExW(&wc);
            if reg == 0 {
                return;
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
            let title_u16 = to_pcwstr(&title);

            let hwnd_main = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
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
            .unwrap_or_default();

            if hwnd_main.is_invalid() {
                return;
            }

            let state = &mut *state_raw_ptr;
            state.hwnd_main = hwnd_main;

            // Create RichEdit Control (RichEdit50W from msftedit.dll)
            let hwnd_edit = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("RichEdit50W"),
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
            )
            .unwrap_or_default();
            set_control_font(hwnd_edit, get_font_mono());
            let _ = send_msg(hwnd_edit, EM_EXLIMITTEXT, 0, 0x7FFFFFFE);
            state.hwnd_edit = hwnd_edit;

            // Create Status Bar (msctls_statusbar32)
            let hwnd_status = CreateWindowExW(
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
            .unwrap_or_default();
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
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

fn main() {
    #[cfg(windows_os)]
    gui::run();
}
