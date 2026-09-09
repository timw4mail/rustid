//! Window management, event loop, and render pipeline for Haiku GUI.

use std::ffi::{CStr, CString, c_char};
use std::sync::Mutex;

use crate::Cpu;
#[allow(unused_imports)]
use crate::common::{CpuDisplay, TDetect};

use super::dialogs::*;
use super::ffi;
use super::menu::*;
use super::state::{AppState, ViewMode};
use super::styled_text::*;

static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);

fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AppState) -> R,
{
    if let Ok(mut guard) = APP_STATE.lock()
        && let Some(ref mut state) = *guard
    {
        Some(f(state))
    } else {
        None
    }
}

pub fn render_current_text(state: &mut AppState) {
    #[cfg(x86_cpu)]
    if let Some(path) = &state.loaded_file {
        if let Some(contents) = read_file_to_string(path) {
            let dump = crate::x86::provider::CpuDump::parse_str(&contents);
            crate::x86::provider::set_cpuid_provider(dump);
        } else {
            crate::x86::provider::reset_cpuid_provider();
        }
    } else {
        crate::x86::provider::reset_cpuid_provider();
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
            format!("{}\n--------------------\n\n{}", report, debug)
        }
        #[cfg(x86_cpu)]
        ViewMode::Dump => generate_dump_info_plain(),
    };

    state.current_plain_text = plain_text;

    let (bg_color, runs) =
        parse_text_runs(&state.current_plain_text, state.dark_theme, state.color);

    let c_text = CString::new(state.current_plain_text.clone()).unwrap_or_default();
    unsafe {
        ffi::haiku_gui_set_text(
            c_text.as_ptr(),
            state.current_plain_text.len() as u32,
            runs.as_ptr(),
            runs.len() as u32,
            bg_color,
        );
    }

    update_status_bar(state, &cpu);
    update_menu_checks(state);
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

    let c_p1 = CString::new(part1).unwrap_or_default();
    let c_p2 = CString::new(part2).unwrap_or_default();
    let c_p3 = CString::new(part3).unwrap_or_default();

    unsafe {
        ffi::haiku_gui_set_status(c_p1.as_ptr(), c_p2.as_ptr(), c_p3.as_ptr());
    }
}

fn update_menu_checks(state: &AppState) {
    let active_mode_id = match state.mode {
        ViewMode::Standard => IDM_MODE_STANDARD,
        ViewMode::Debug => IDM_MODE_DEBUG,
        ViewMode::Everything => IDM_MODE_EVERYTHING,
        #[cfg(x86_cpu)]
        ViewMode::Dump => IDM_MODE_DUMP,
    };

    unsafe {
        ffi::haiku_gui_set_menu_checks(
            active_mode_id,
            state.color,
            state.dark_theme,
            state.verbose,
            state.compact,
        );
    }
}

unsafe extern "C" fn on_command_callback(cmd_id: u32) {
    with_state(|state| match cmd_id {
        #[cfg(x86_cpu)]
        IDM_FILE_OPEN => {
            open_dump_file_dialog();
        }
        #[cfg(x86_cpu)]
        IDM_FILE_EXPORT => {
            let cpu = Cpu::detect();
            let default_name = format!(
                "cpuid_dump_{}.txt",
                cpu.display_model_string().replace([' ', '/', '\\'], "_")
            );
            export_dump_dialog(&default_name);
        }
        IDM_FILE_COPY => {
            copy_to_clipboard(&state.current_plain_text);
        }
        IDM_FILE_REFRESH => {
            state.loaded_file = None;
            render_current_text(state);
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
            show_about_dialog();
        }
        _ => {}
    });
}

unsafe extern "C" fn on_file_callback(path_ptr: *const c_char, is_save: bool) {
    if path_ptr.is_null() {
        return;
    }

    let path = match unsafe { CStr::from_ptr(path_ptr) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return,
    };

    if is_save {
        #[cfg(x86_cpu)]
        {
            let dump_content = generate_dump_info_plain();
            if write_string_to_file(&path, &dump_content) {
                let msg = format!("CPUID dump successfully saved to:\n{}", path);
                show_alert("Export Complete", &msg);
            }
        }
    } else {
        with_state(|state| {
            state.loaded_file = Some(path);
            render_current_text(state);
        });
    }
}

unsafe extern "C" fn on_quit_callback() {
    if let Ok(mut guard) = APP_STATE.lock() {
        *guard = None;
    }
}

pub fn run() {
    let title = format!(
        "Rustid {} ({}-{})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH,
        std::env::consts::OS
    );

    let c_title = CString::new(title).unwrap_or_default();
    let win_w = 820.0f32;
    let win_h = 640.0f32;
    let min_w = 580.0f32;
    let min_h = 380.0f32;

    let init_ok = unsafe { ffi::haiku_gui_init(c_title.as_ptr(), win_w, win_h, min_w, min_h) };

    if !init_ok {
        eprintln!("Failed to initialize Haiku GUI.");
        return;
    }

    unsafe {
        ffi::haiku_gui_set_callbacks(
            Some(on_command_callback),
            Some(on_file_callback),
            Some(on_quit_callback),
        );
    }

    let mut state = AppState::default();
    render_current_text(&mut state);

    if let Ok(mut guard) = APP_STATE.lock() {
        *guard = Some(state);
    }

    unsafe {
        ffi::haiku_gui_run();
    }
}
