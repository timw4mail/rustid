//! File dialogs and clipboard operations.

use std::ffi::c_void;
use std::sync::atomic::Ordering;

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
#[cfg(x86_cpu)]
use windows::core::{PCWSTR, w};

use super::state::{CF_UNICODETEXT_FORMAT, IS_UNICODE};
#[cfg(x86_cpu)]
use super::theme::to_pcwstr;

const CF_TEXT_FORMAT: u32 = 1;
#[cfg(x86_cpu)]
const OPENFILENAME_SIZE_VERSION_400: u32 = 76;

#[cfg(x86_cpu)]
#[repr(C)]
#[allow(dead_code)]
struct OpenFileNameA {
    l_struct_size: u32,
    hwnd_owner: *mut c_void,
    h_instance: *mut c_void,
    lpstr_filter: *const u8,
    lpstr_custom_filter: *mut u8,
    n_max_cust_filter: u32,
    n_filter_index: u32,
    lpstr_file: *mut u8,
    n_max_file: u32,
    lpstr_file_title: *mut u8,
    n_max_file_title: u32,
    lpstr_initial_dir: *const u8,
    lpstr_title: *const u8,
    flags: u32,
    n_file_offset: u16,
    n_file_extension: u16,
    lpstr_def_ext: *const u8,
    l_cust_data: isize,
    lpfn_hook: Option<unsafe extern "system" fn(*mut c_void, u32, usize, isize) -> usize>,
    lp_template_name: *const u8,
}

#[link(name = "kernel32")]
#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GlobalFree(hMem: *mut c_void) -> *mut c_void;
    #[cfg(x86_cpu)]
    fn CreateFileA(
        lpFileName: *const u8,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut c_void,
    ) -> *mut c_void;
    #[cfg(x86_cpu)]
    fn GetFileSize(hFile: *mut c_void, lpFileSizeHigh: *mut u32) -> u32;
    #[cfg(x86_cpu)]
    fn ReadFile(
        hFile: *mut c_void,
        lpBuffer: *mut c_void,
        nNumberOfBytesToRead: u32,
        lpNumberOfBytesRead: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> bool;
    #[cfg(x86_cpu)]
    fn WriteFile(
        hFile: *mut c_void,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> bool;
    #[cfg(x86_cpu)]
    fn CloseHandle(hObject: *mut c_void) -> i32;
    #[cfg(x86_cpu)]
    fn GetOpenFileNameA(lpofn: *mut OpenFileNameA) -> i32;
    #[cfg(x86_cpu)]
    fn GetSaveFileNameA(lpofn: *mut OpenFileNameA) -> i32;
}

#[cfg(x86_cpu)]
pub fn read_file_to_string(path: &str) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(path) {
        return Some(content);
    }
    let path_a = std::ffi::CString::new(path).ok()?;
    unsafe {
        let hfile = CreateFileA(
            path_a.as_ptr() as *const u8,
            0x80000000, // GENERIC_READ
            1,          // FILE_SHARE_READ
            std::ptr::null_mut(),
            3,          // OPEN_EXISTING
            0x00000080, // FILE_ATTRIBUTE_NORMAL
            std::ptr::null_mut(),
        );
        if hfile == (-1isize as *mut c_void) || hfile.is_null() {
            return None;
        }
        let size = GetFileSize(hfile, std::ptr::null_mut());
        if size == 0xFFFFFFFF {
            let _ = CloseHandle(hfile);
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        let mut read = 0u32;
        let ok = ReadFile(
            hfile,
            buf.as_mut_ptr() as *mut c_void,
            size,
            &mut read,
            std::ptr::null_mut(),
        );
        let _ = CloseHandle(hfile);
        if ok {
            buf.truncate(read as usize);
            Some(String::from_utf8_lossy(&buf).to_string())
        } else {
            None
        }
    }
}

#[cfg(x86_cpu)]
pub fn write_string_to_file(path: &str, content: &str) -> bool {
    if std::fs::write(path, content).is_ok() {
        return true;
    }
    let path_a = match std::ffi::CString::new(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    unsafe {
        let hfile = CreateFileA(
            path_a.as_ptr() as *const u8,
            0x40000000, // GENERIC_WRITE
            0,
            std::ptr::null_mut(),
            2,          // CREATE_ALWAYS
            0x00000080, // FILE_ATTRIBUTE_NORMAL
            std::ptr::null_mut(),
        );
        if hfile == (-1isize as *mut c_void) || hfile.is_null() {
            return false;
        }
        let bytes = content.as_bytes();
        let mut written = 0u32;
        let ok = WriteFile(
            hfile,
            bytes.as_ptr() as *const c_void,
            bytes.len() as u32,
            &mut written,
            std::ptr::null_mut(),
        );
        let _ = CloseHandle(hfile);
        ok && written == bytes.len() as u32
    }
}

pub fn copy_to_clipboard(hwnd_owner: HWND, text: &str) {
    let crlf_text = text.replace("\r\n", "\n").replace('\n', "\r\n");

    unsafe {
        // Retry OpenClipboard up to 5 times in case another process holds it
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(Some(hwnd_owner)).is_ok() {
                opened = true;
                break;
            }
            windows::Win32::System::Threading::Sleep(10);
        }
        if !opened {
            return;
        }

        let _ = EmptyClipboard();

        let is_unicode = IS_UNICODE.load(Ordering::Relaxed);
        if is_unicode {
            let text_u16: Vec<u16> = crlf_text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes_len = text_u16.len() * std::mem::size_of::<u16>();

            if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, bytes_len) {
                let ptr = GlobalLock(hmem) as *mut u16;
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(text_u16.as_ptr(), ptr, text_u16.len());
                    let _ = GlobalUnlock(hmem);

                    if SetClipboardData(
                        CF_UNICODETEXT_FORMAT,
                        Some(windows::Win32::Foundation::HANDLE(hmem.0)),
                    )
                    .is_err()
                    {
                        let _ = GlobalFree(hmem.0);
                    }
                } else {
                    let _ = GlobalFree(hmem.0);
                }
            }
        } else {
            let text_bytes: Vec<u8> = crlf_text.bytes().chain(std::iter::once(0)).collect();
            if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, text_bytes.len()) {
                let ptr = GlobalLock(hmem) as *mut u8;
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), ptr, text_bytes.len());
                    let _ = GlobalUnlock(hmem);

                    if SetClipboardData(
                        CF_TEXT_FORMAT,
                        Some(windows::Win32::Foundation::HANDLE(hmem.0)),
                    )
                    .is_err()
                    {
                        let _ = GlobalFree(hmem.0);
                    }
                } else {
                    let _ = GlobalFree(hmem.0);
                }
            }
        }

        let _ = CloseClipboard();
    }
}

#[cfg(x86_cpu)]
pub fn open_dump_file_dialog(hwnd_parent: HWND) -> Option<String> {
    if IS_UNICODE.load(Ordering::Relaxed) {
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
    } else {
        let mut file_buf = [0u8; 1024];
        let filter = b"CPUID Dump (*.txt;*.dump)\0*.txt;*.dump\0All Files (*.*)\0*.*\0\0";

        let mut ofn = OpenFileNameA {
            l_struct_size: OPENFILENAME_SIZE_VERSION_400,
            hwnd_owner: hwnd_parent.0,
            h_instance: std::ptr::null_mut(),
            lpstr_filter: filter.as_ptr(),
            lpstr_custom_filter: std::ptr::null_mut(),
            n_max_cust_filter: 0,
            n_filter_index: 1,
            lpstr_file: file_buf.as_mut_ptr(),
            n_max_file: file_buf.len() as u32,
            lpstr_file_title: std::ptr::null_mut(),
            n_max_file_title: 0,
            lpstr_initial_dir: std::ptr::null(),
            lpstr_title: std::ptr::null(),
            flags: 0x00001000 | 0x00000800, // OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST
            n_file_offset: 0,
            n_file_extension: 0,
            lpstr_def_ext: std::ptr::null(),
            l_cust_data: 0,
            lpfn_hook: None,
            lp_template_name: std::ptr::null(),
        };

        if unsafe { GetOpenFileNameA(&mut ofn) } != 0 {
            let len = file_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(file_buf.len());
            Some(String::from_utf8_lossy(&file_buf[..len]).to_string())
        } else {
            None
        }
    }
}

#[cfg(x86_cpu)]
pub fn export_dump_dialog(hwnd_parent: HWND, default_filename: &str) -> Option<String> {
    if IS_UNICODE.load(Ordering::Relaxed) {
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
    } else {
        let mut file_buf = [0u8; 1024];
        let def_bytes = default_filename.as_bytes();
        let copy_len = def_bytes.len().min(file_buf.len() - 1);
        file_buf[..copy_len].copy_from_slice(&def_bytes[..copy_len]);

        let filter = b"CPUID Dump (*.txt)\0*.txt\0All Files (*.*)\0*.*\0\0";
        let def_ext = b"txt\0";

        let mut ofn = OpenFileNameA {
            l_struct_size: OPENFILENAME_SIZE_VERSION_400,
            hwnd_owner: hwnd_parent.0,
            h_instance: std::ptr::null_mut(),
            lpstr_filter: filter.as_ptr(),
            lpstr_custom_filter: std::ptr::null_mut(),
            n_max_cust_filter: 0,
            n_filter_index: 1,
            lpstr_file: file_buf.as_mut_ptr(),
            n_max_file: file_buf.len() as u32,
            lpstr_file_title: std::ptr::null_mut(),
            n_max_file_title: 0,
            lpstr_initial_dir: std::ptr::null(),
            lpstr_title: std::ptr::null(),
            flags: 0x00000800 | 0x00000002, // OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT
            n_file_offset: 0,
            n_file_extension: 0,
            lpstr_def_ext: def_ext.as_ptr(),
            l_cust_data: 0,
            lpfn_hook: None,
            lp_template_name: std::ptr::null(),
        };

        if unsafe { GetSaveFileNameA(&mut ofn) } != 0 {
            let len = file_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(file_buf.len());
            Some(String::from_utf8_lossy(&file_buf[..len]).to_string())
        } else {
            None
        }
    }
}
