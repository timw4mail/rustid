#![cfg(target_os = "uefi")]
//! EFI display driver and graphics framebuffer rendering for rustid.

use core::ffi::c_void;
use core::fmt::Write;

use super::font;
use super::os::{
    EFI_SUCCESS, EfiGuid, EfiHandle, EfiStatus, get_system_table, locate_protocol_compat,
};

pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x9042a9de,
    b: 0x23dc,
    c: 0x4a38,
    d: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub const EFI_APPLE_SCREEN_INFO_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0xe316e100,
    b: 0x0751,
    c: 0x4c49,
    d: [0x90, 0x56, 0x48, 0x6c, 0x7e, 0x47, 0x29, 0x03],
};

pub const EFI_UGA_DRAW_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x982c298b,
    b: 0xf4fa,
    c: 0x4226,
    d: [0x9e, 0x46, 0x16, 0x9d, 0x36, 0x95, 0xaa, 0x62],
};

#[repr(C)]
pub struct EfiAppleScreenInfoProtocol {
    pub get_info: unsafe extern "efiapi" fn(
        *mut EfiAppleScreenInfoProtocol,
        *mut u64,
        *mut u64,
        *mut u32,
        *mut u32,
        *mut u32,
        *mut u32,
    ) -> EfiStatus,
}

#[repr(C)]
pub struct EfiGraphicsOutputModeInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: u32,
    pub pixel_information: [u32; 4],
    pub pixels_per_scan_line: u32,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocolMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *mut EfiGraphicsOutputModeInformation,
    pub size_of_info: usize,
    pub frame_buffer_base: u64,
    pub frame_buffer_size: usize,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub query_mode: unsafe extern "efiapi" fn(
        *mut EfiGraphicsOutputProtocol,
        u32,
        *mut usize,
        *mut *mut EfiGraphicsOutputModeInformation,
    ) -> EfiStatus,
    pub set_mode: unsafe extern "efiapi" fn(*mut EfiGraphicsOutputProtocol, u32) -> EfiStatus,
    pub blt: *const c_void,
    pub mode: *mut EfiGraphicsOutputProtocolMode,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfiUgaBltOperation {
    EfiUgaVideoFill = 0,
    EfiUgaVideoToBltBuffer = 1,
    EfiUgaBltBufferToVideo = 2,
    EfiUgaVideoToVideo = 3,
    EfiUgaBltMax = 4,
}

#[repr(C)]
pub struct EfiUgaDrawProtocol {
    pub get_mode: unsafe extern "efiapi" fn(
        *mut EfiUgaDrawProtocol,
        *mut u32,
        *mut u32,
        *mut u32,
        *mut u32,
    ) -> EfiStatus,
    pub set_mode:
        unsafe extern "efiapi" fn(*mut EfiUgaDrawProtocol, u32, u32, u32, u32) -> EfiStatus,
    pub blt: unsafe extern "efiapi" fn(
        *mut EfiUgaDrawProtocol,
        *mut u32,
        EfiUgaBltOperation,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) -> EfiStatus,
}

pub struct FramebufferState {
    pub fb: *mut u32,
    pub uga: *mut EfiUgaDrawProtocol,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub fg_color: u32,
    pub is_valid: bool,
}

static mut GFX_STATE: FramebufferState = FramebufferState {
    fb: core::ptr::null_mut(),
    uga: core::ptr::null_mut(),
    width: 0,
    height: 0,
    stride: 0,
    cursor_x: 0,
    cursor_y: 0,
    fg_color: 0xFFFFFFFF,
    is_valid: false,
};

pub const EFI_CONSOLE_CONTROL_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0xf42f7782,
    b: 0x012e,
    c: 0x4c12,
    d: [0x99, 0x56, 0x49, 0xf9, 0x43, 0xcd, 0x8f, 0x7f],
};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfiConsoleControlScreenMode {
    EfiConsoleControlScreenText = 0,
    EfiConsoleControlScreenGraphics = 1,
    EfiConsoleControlScreenBootGraphic = 2,
    EfiConsoleControlScreenMaxValue = 3,
}

#[repr(C)]
pub struct EfiConsoleControlProtocol {
    pub get_mode: unsafe extern "efiapi" fn(
        *mut EfiConsoleControlProtocol,
        *mut EfiConsoleControlScreenMode,
        *mut bool,
        *mut bool,
    ) -> EfiStatus,
    pub set_mode: unsafe extern "efiapi" fn(
        *mut EfiConsoleControlProtocol,
        EfiConsoleControlScreenMode,
    ) -> EfiStatus,
    pub lock_std_in: *const c_void,
}

pub unsafe fn get_protocol_on_handle<T>(handle: EfiHandle, guid: &EfiGuid) -> *mut T {
    let st = get_system_table();
    if st.is_null() || handle.is_null() {
        return core::ptr::null_mut();
    }
    let bs = unsafe { (*st).boot_services };
    if bs.is_null() {
        return core::ptr::null_mut();
    }
    let mut interface: *mut c_void = core::ptr::null_mut();
    let status = unsafe { ((*bs).handle_protocol)(handle, guid, &mut interface) };
    if status == EFI_SUCCESS && !interface.is_null() {
        interface as *mut T
    } else {
        core::ptr::null_mut()
    }
}

/// Initializes Graphics Output Protocol (GOP), Apple Framebuffer Info, or UGA Draw Protocol.
pub fn init_gfx() {
    let st = get_system_table();
    if st.is_null() {
        return;
    }

    let console_handle = unsafe { (*st).console_out_handle };

    // Locate ConsoleControl for use in UGA/fallback paths.
    // Do NOT switch to graphics mode here — that silences ConOut.
    // GOP and Apple FB paths have a direct linear framebuffer and don't need ConsoleControl.
    // The UGA path will switch to text mode so ConOut->output_string is visible.
    let cc_ptr = unsafe { locate_protocol_compat(&EFI_CONSOLE_CONTROL_PROTOCOL_GUID) }
        as *mut EfiConsoleControlProtocol;

    // 1. Try GOP (UEFI 2.0+ / 64-bit EFI)
    let mut gop_ptr = if !console_handle.is_null() {
        unsafe {
            get_protocol_on_handle::<EfiGraphicsOutputProtocol>(
                console_handle,
                &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
            )
        }
    } else {
        core::ptr::null_mut()
    };
    if gop_ptr.is_null() {
        gop_ptr = unsafe { locate_protocol_compat(&EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID) }
            as *mut EfiGraphicsOutputProtocol;
    }

    if !gop_ptr.is_null() {
        let mode_ptr = unsafe { (*gop_ptr).mode };
        if !mode_ptr.is_null() {
            let max_mode = unsafe { (*mode_ptr).max_mode };
            let mut best_mode = unsafe { (*mode_ptr).mode };
            let mut best_pixels = 0u64;

            for m in 0..max_mode {
                let mut size_of_info = 0usize;
                let mut info_ptr: *mut EfiGraphicsOutputModeInformation = core::ptr::null_mut();
                let status = unsafe {
                    ((*gop_ptr).query_mode)(gop_ptr, m, &mut size_of_info, &mut info_ptr)
                };
                if status == EFI_SUCCESS && !info_ptr.is_null() {
                    let w = unsafe { (*info_ptr).horizontal_resolution } as u64;
                    let h = unsafe { (*info_ptr).vertical_resolution } as u64;
                    let pixels = w * h;
                    if pixels > best_pixels {
                        best_pixels = pixels;
                        best_mode = m;
                    }
                }
            }

            if best_mode != unsafe { (*mode_ptr).mode } {
                let _ = unsafe { ((*gop_ptr).set_mode)(gop_ptr, best_mode) };
            }

            let mode_ptr = unsafe { (*gop_ptr).mode };
            if !mode_ptr.is_null() {
                let info = unsafe { (*mode_ptr).info };
                let fb_base = unsafe { (*mode_ptr).frame_buffer_base };
                if !info.is_null() && fb_base != 0 {
                    let width = unsafe { (*info).horizontal_resolution };
                    let height = unsafe { (*info).vertical_resolution };
                    let stride = unsafe { (*info).pixels_per_scan_line };
                    let stride = if stride > 0 { stride } else { width };
                    unsafe {
                        GFX_STATE.fb = fb_base as *mut u32;
                        GFX_STATE.uga = core::ptr::null_mut();
                        GFX_STATE.width = width;
                        GFX_STATE.height = height;
                        GFX_STATE.stride = stride;
                        GFX_STATE.cursor_x = 16;
                        GFX_STATE.cursor_y = 16;
                        GFX_STATE.fg_color = 0xFFFFFFFF;
                        GFX_STATE.is_valid = true;

                        // Clear framebuffer screen to pitch black
                        let total_pixels = (stride * height) as usize;
                        for i in 0..total_pixels {
                            *GFX_STATE.fb.add(i) = 0xFF000000;
                        }
                    }
                    return;
                }
            }
        }
    }

    // 2. Try Apple Screen Info Protocol (32-bit & early 64-bit Apple EFI)
    let mut apple_screen_ptr = if !console_handle.is_null() {
        unsafe {
            get_protocol_on_handle::<EfiAppleScreenInfoProtocol>(
                console_handle,
                &EFI_APPLE_SCREEN_INFO_PROTOCOL_GUID,
            )
        }
    } else {
        core::ptr::null_mut()
    };
    if apple_screen_ptr.is_null() {
        apple_screen_ptr = unsafe { locate_protocol_compat(&EFI_APPLE_SCREEN_INFO_PROTOCOL_GUID) }
            as *mut EfiAppleScreenInfoProtocol;
    }

    if !apple_screen_ptr.is_null() {
        let mut base_addr = 0u64;
        let mut fb_size = 0u64;
        let mut bytes_per_row = 0u32;
        let mut width = 0u32;
        let mut height = 0u32;
        let mut depth = 0u32;
        let res = unsafe {
            ((*apple_screen_ptr).get_info)(
                apple_screen_ptr,
                &mut base_addr,
                &mut fb_size,
                &mut bytes_per_row,
                &mut width,
                &mut height,
                &mut depth,
            )
        };
        if res == EFI_SUCCESS && base_addr != 0 && width > 0 && height > 0 {
            let stride = if bytes_per_row >= 4 {
                bytes_per_row / 4
            } else {
                width
            };
            unsafe {
                GFX_STATE.fb = base_addr as usize as *mut u32;
                GFX_STATE.uga = core::ptr::null_mut();
                GFX_STATE.width = width;
                GFX_STATE.height = height;
                GFX_STATE.stride = stride;
                GFX_STATE.cursor_x = 16;
                GFX_STATE.cursor_y = 16;
                GFX_STATE.fg_color = 0xFFFFFFFF;
                GFX_STATE.is_valid = true;

                // Clear linear framebuffer to pitch black
                let total_pixels = (stride * height) as usize;
                for i in 0..total_pixels {
                    *GFX_STATE.fb.add(i) = 0xFF000000;
                }
            }
            return;
        }
    }

    // 3. Try UGA (EFI 1.10 / 32-bit Apple EFI)
    let mut uga_ptr = if !console_handle.is_null() {
        unsafe {
            get_protocol_on_handle::<EfiUgaDrawProtocol>(
                console_handle,
                &EFI_UGA_DRAW_PROTOCOL_GUID,
            )
        }
    } else {
        core::ptr::null_mut()
    };
    if uga_ptr.is_null() {
        uga_ptr = unsafe { locate_protocol_compat(&EFI_UGA_DRAW_PROTOCOL_GUID) }
            as *mut EfiUgaDrawProtocol;
    }

    if !uga_ptr.is_null() {
        let mut width = 0u32;
        let mut height = 0u32;
        let mut depth = 0u32;
        let mut refresh = 0u32;
        let res = unsafe {
            ((*uga_ptr).get_mode)(uga_ptr, &mut width, &mut height, &mut depth, &mut refresh)
        };
        if res == EFI_SUCCESS && width > 0 && height > 0 {
            unsafe {
                GFX_STATE.fb = core::ptr::null_mut();
                GFX_STATE.uga = uga_ptr;
                GFX_STATE.width = width;
                GFX_STATE.height = height;
                GFX_STATE.stride = width;
                GFX_STATE.cursor_x = 16;
                GFX_STATE.cursor_y = 16;
                GFX_STATE.fg_color = 0x00FFFFFF;
                GFX_STATE.is_valid = true;

                // Clear the screen to pitch black via EfiUgaVideoFill
                let black_pixel: u32 = 0x00000000;
                let _ = ((*uga_ptr).blt)(
                    uga_ptr,
                    &black_pixel as *const u32 as *mut u32,
                    EfiUgaBltOperation::EfiUgaVideoFill,
                    0,
                    0,
                    0,
                    0,
                    width as usize,
                    height as usize,
                    0,
                );
            }
            return;
        }
    }

    // 3. No graphics protocol found — fall back to ConsoleControl text mode.
    if unsafe { !GFX_STATE.is_valid } {
        if !cc_ptr.is_null() {
            unsafe {
                let _ = ((*cc_ptr).set_mode)(
                    cc_ptr,
                    EfiConsoleControlScreenMode::EfiConsoleControlScreenText,
                );
            }
        }
        if !st.is_null() {
            let con_out = unsafe { (*st).con_out };
            if !con_out.is_null() {
                unsafe {
                    let _ = ((*con_out).set_mode)(con_out, 0);
                    let _ = ((*con_out).clear_screen)(con_out);
                }
            }
        }
    }
}

/// Draws a single character to GOP linear framebuffer or UGA.
pub fn gfx_draw_char(c: char) {
    unsafe {
        if !GFX_STATE.is_valid {
            return;
        }

        if c == '\n' {
            GFX_STATE.cursor_x = 16;
            GFX_STATE.cursor_y += 16;
            return;
        }
        if c == '\r' {
            GFX_STATE.cursor_x = 16;
            return;
        }

        let ascii_idx = if (c as usize) >= 32 && (c as usize) <= 126 {
            c as usize - 32
        } else {
            '?' as usize - 32
        };

        let font_glyph = &font::FONT_8X16[ascii_idx * 16..(ascii_idx + 1) * 16];

        // 1. Direct Linear Framebuffer (GOP)
        if !GFX_STATE.fb.is_null() {
            for (row, &byte) in font_glyph.iter().enumerate() {
                let py = GFX_STATE.cursor_y + row as u32;
                if py >= GFX_STATE.height {
                    break;
                }
                for col in 0..8 {
                    let px = GFX_STATE.cursor_x + col as u32;
                    if px >= GFX_STATE.width {
                        break;
                    }
                    let is_set = (byte & (1 << (7 - col))) != 0;
                    let pixel = if is_set {
                        GFX_STATE.fg_color
                    } else {
                        0xFF000000
                    };
                    *GFX_STATE.fb.add((py * GFX_STATE.stride + px) as usize) = pixel;
                }
            }
            #[cfg(target_arch = "x86")]
            core::arch::asm!("sfence", options(nomem, nostack, preserves_flags));
        }
        // 2. UGA Blt (32-bit Apple EFI)
        else if !GFX_STATE.uga.is_null() {
            let mut glyph_pixels = [0u32; 128]; // 8x16 pixels
            for (row, &byte) in font_glyph.iter().enumerate() {
                for col in 0..8 {
                    let is_set = (byte & (1 << (7 - col))) != 0;
                    glyph_pixels[row * 8 + col] = if is_set {
                        GFX_STATE.fg_color
                    } else {
                        0x00000000
                    };
                }
            }
            let uga = GFX_STATE.uga;
            let _ = ((*uga).blt)(
                uga,
                glyph_pixels.as_mut_ptr(),
                EfiUgaBltOperation::EfiUgaBltBufferToVideo,
                0,
                0,
                GFX_STATE.cursor_x as usize,
                GFX_STATE.cursor_y as usize,
                8,
                16,
                0,
            );
        }

        GFX_STATE.cursor_x += 8;
        if GFX_STATE.cursor_x + 8 > GFX_STATE.width {
            GFX_STATE.cursor_x = 16;
            GFX_STATE.cursor_y += 16;
        }
    }
}

/// Clears the console screen via GOP/UGA framebuffer only.
/// Does NOT call ConOut->clear_screen() — Apple EFI freezes the cursor on that call.
pub fn clear_screen_black() {
    init_gfx();
}

/// Prints a formatted string to UEFI graphics display (GOP/Apple Framebuffer/UGA) or fallback ConOut.
pub fn _print_str(s: &str) {
    let st = get_system_table();
    let con_out = if !st.is_null() {
        unsafe { (*st).con_out }
    } else {
        core::ptr::null_mut()
    };
    let proto = if !con_out.is_null() {
        con_out
    } else if !st.is_null() {
        unsafe { (*st).std_err }
    } else {
        core::ptr::null_mut()
    };

    let mut utf16_buf = [0u16; 256];
    let mut idx = 0;
    let mut chars = s.chars().peekable();

    // Suppress ConOut only when we have a direct linear framebuffer (GOP / Apple FB).
    // In UGA mode (Apple 32-bit EFI), ConOut->output_string IS the visible output path —
    // Apple's UGA console splitter routes ConOut text to the display surface.
    // Individual UGA Blt calls for glyphs silently fail on Apple EFI 1.10.
    let suppress_conout = unsafe { GFX_STATE.is_valid && !GFX_STATE.fb.is_null() };

    let flush = |buf: &mut [u16; 256], idx: &mut usize| {
        if *idx > 0 && !proto.is_null() && !suppress_conout {
            buf[*idx] = 0;
            unsafe { ((*proto).output_string)(proto, buf.as_ptr()) };
            *idx = 0;
        } else {
            *idx = 0;
        }
    };

    while let Some(ch) = chars.next() {
        // Parse ANSI escape sequence \x1b[...m
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // Consume '['
            let mut code = 0u32;
            let mut has_code = false;
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    code = code * 10 + (c as u32 - '0' as u32);
                    has_code = true;
                    chars.next();
                } else if c == ';' || c == 'm' {
                    chars.next();
                    let (attr, gfx_color) = match code {
                        0 => (0x0F, 0xFFFFFFFF),       // White
                        32 => (0x0A, 0xFF55FF55),      // Light Green
                        34 | 94 => (0x0B, 0xFF55FFFF), // Light Cyan
                        33 | 93 => (0x0E, 0xFFFFFF55), // Yellow
                        31 | 91 => (0x0C, 0xFFFF5555), // Light Red
                        36 | 96 => (0x0B, 0xFF55FFFF), // Light Cyan
                        90 => (0x08, 0xFF888888),      // Dark Gray
                        _ => (0x0F, 0xFFFFFFFF),
                    };
                    flush(&mut utf16_buf, &mut idx);
                    if !proto.is_null() && !suppress_conout {
                        unsafe { ((*proto).set_attribute)(proto, attr) };
                    }
                    unsafe {
                        GFX_STATE.fg_color = gfx_color;
                    }
                    if c == 'm' {
                        break;
                    }
                    code = 0;
                    has_code = false;
                } else {
                    break;
                }
            }
            if !has_code && chars.peek() == Some(&'m') {
                chars.next();
                flush(&mut utf16_buf, &mut idx);
                if !proto.is_null() && !suppress_conout {
                    unsafe { ((*proto).set_attribute)(proto, 0x0F) };
                }
                unsafe {
                    GFX_STATE.fg_color = 0xFFFFFFFF;
                }
            }
            continue;
        }

        // Draw character to GOP / Apple Framebuffer / UGA graphics display
        gfx_draw_char(ch);

        if ch == '\n' {
            if idx == 0 || utf16_buf[idx - 1] != ('\r' as u16) {
                utf16_buf[idx] = '\r' as u16;
                idx += 1;
            }
            utf16_buf[idx] = '\n' as u16;
            idx += 1;
            flush(&mut utf16_buf, &mut idx);
            continue;
        }

        let mut code_units = [0u16; 2];
        let encoded = ch.encode_utf16(&mut code_units);
        for &mut u in encoded {
            utf16_buf[idx] = u;
            idx += 1;
            if idx >= utf16_buf.len() - 2 {
                flush(&mut utf16_buf, &mut idx);
            }
        }
    }

    flush(&mut utf16_buf, &mut idx);

    #[cfg(target_arch = "x86")]
    if unsafe { GFX_STATE.is_valid } {
        unsafe {
            core::arch::asm!("mfence", options(nomem, nostack, preserves_flags));
        }
    }
}

pub struct EfiWriter;

impl Write for EfiWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        _print_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($s:literal) => {
        $crate::x86::efi::_print_str($s)
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!(&mut $crate::x86::efi::EfiWriter {}, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($s:literal) => {
        {
            $crate::print!($s);
            $crate::print!("\r\n");
        }
    };
    ($($arg:tt)*) => {
        {
            $crate::print!($($arg)*);
            $crate::print!("\r\n");
        }
    };
}
