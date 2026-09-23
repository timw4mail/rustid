//! Window management, event loop, and render pipeline for Classic Macintosh GUI (68k & PowerPC).

use alloc::ffi::CString;
use core::ffi::c_char;
use std::sync::Mutex;

use crate::Cpu;
use crate::common::cpu::TDetect;
use crate::gui::ReportSource;
use crate::gui::common;

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
    let cpu = Cpu::detect();
    let source = ReportSource::LiveHardware;

    state.current_plain_text = common::build_view_text(&cpu, state.mode, state.flags, source);

    let (bg_color, runs) =
        parse_text_runs(&state.current_plain_text, state.theme, state.flags.color);

    let c_text = CString::new(state.current_plain_text.clone()).unwrap_or_default();
    unsafe {
        ffi::mac_gui_set_text(
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
    let (part1, part2, part3) = common::format_status_parts(
        cpu,
        None,
        state.mode,
        state.flags,
        state.theme,
    );

    let c_p1 = CString::new(part1).unwrap_or_default();
    let c_p2 = CString::new(part2).unwrap_or_default();
    let c_p3 = CString::new(part3).unwrap_or_default();

    unsafe {
        ffi::mac_gui_set_status(c_p1.as_ptr(), c_p2.as_ptr(), c_p3.as_ptr());
    }
}

fn update_menu_checks(state: &AppState) {
    let active_mode_id = match state.mode {
        ViewMode::Standard => IDM_MODE_STANDARD,
        ViewMode::Debug => IDM_MODE_DEBUG,
        ViewMode::Everything => IDM_MODE_EVERYTHING,
        #[cfg(x86_cpu)]
        ViewMode::Dump => IDM_MODE_STANDARD,
    };

    unsafe {
        ffi::mac_gui_set_menu_checks(
            active_mode_id,
            state.flags.color,
            state.theme.is_dark(),
            state.flags.verbose,
            state.flags.compact,
        );
    }
}

unsafe extern "C" fn on_command_callback(cmd_id: u32) {
    with_state(|state| match cmd_id {
        IDM_FILE_EXPORT => {
            export_report_dialog("rustid_report.txt");
        }
        IDM_FILE_COPY => {
            copy_to_clipboard(&state.current_plain_text);
        }
        IDM_FILE_REFRESH => {
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
        IDM_OPT_COLOR => {
            state.flags.color = !state.flags.color;
            render_current_text(state);
        }
        IDM_OPT_DARK_THEME => {
            state.theme.toggle();
            render_current_text(state);
        }
        IDM_OPT_VERBOSE => {
            state.flags.verbose = !state.flags.verbose;
            render_current_text(state);
        }
        IDM_OPT_COMPACT => {
            state.flags.compact = !state.flags.compact;
            render_current_text(state);
        }
        IDM_HELP_ABOUT => {
            show_about_dialog();
        }
        _ => (),
    });
}

unsafe extern "C" fn on_file_callback(_path: *const c_char, _is_save: bool) {
    // Export report handler if needed
}

unsafe extern "C" fn on_quit_callback() {
    // Cleanup on exit
}

pub fn run() {
    if let Ok(mut guard) = APP_STATE.lock() {
        *guard = Some(AppState::default());
    }

    let title = CString::new("Rustid").unwrap_or_default();
    let init_ok = unsafe { ffi::mac_gui_init(title.as_ptr(), 540, 360) };

    if !init_ok {
        return;
    }

    unsafe {
        ffi::mac_gui_set_callbacks(
            Some(on_command_callback),
            Some(on_file_callback),
            Some(on_quit_callback),
        );
    }

    with_state(|state| {
        render_current_text(state);
    });

    unsafe {
        ffi::mac_gui_run();
    }
}
