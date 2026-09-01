//! File dialogs and clipboard operations.

use windows::Win32::Foundation::HWND;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
#[cfg(x86_cpu)]
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_FILEMUSTEXIST, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    OPENFILENAMEW,
};
use windows::core::{PCWSTR, w};

use super::state::CF_UNICODETEXT_FORMAT;
use super::theme::to_pcwstr;

pub fn copy_to_clipboard(hwnd_owner: HWND, text: &str) {
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
pub fn open_dump_file_dialog(hwnd_parent: HWND) -> Option<String> {
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
pub fn export_dump_dialog(hwnd_parent: HWND, default_filename: &str) -> Option<String> {
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
