#![cfg(target_os = "uefi")]
//! EFI display driver and graphics framebuffer rendering for rustid.

use core::ffi::c_void;
use core::fmt::Write;

use super::font;
use super::os::{EFI_SUCCESS, EfiGuid, EfiStatus, get_system_table, locate_protocol_compat};

pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x9042a9de,
    b: 0x23dc,
    c: 0x4a38,
    d: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub const EFI_APPLE_FRAMEBUFFER_INFO_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0xe316e100,
    b: 0x07e4,
    c: 0x457d,
    d: [0x9b, 0x5e, 0xcc, 0xaa, 0xcb, 0xd0, 0xdd, 0x18],
};

pub const EFI_UGA_DRAW_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x982c298b,
    b: 0xf4fa,
    c: 0x4226,
    d: [0x9e, 0x46, 0x16, 0x9d, 0x36, 0x95, 0xaa, 0x62],
};

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

#[repr(C)]
pub struct EfiAppleFramebufferInfoProtocol {
    pub get_framebuffer_info: unsafe extern "efiapi" fn(
        *mut EfiAppleFramebufferInfoProtocol,
        *mut *mut u8,
        *mut u32,
        *mut u32,
        *mut u32,
        *mut u32,
    ) -> EfiStatus,
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

#[cfg(target_arch = "x86")]
pub unsafe fn sync_gma950_display_plane() {
    unsafe fn pci_outl(port: u16, val: u32) {
        unsafe {
            core::arch::asm!("out dx, eax", in("dx") port, in("eax") val,
                options(nomem, nostack, preserves_flags));
        }
    }
    unsafe fn pci_inl(port: u16) -> u32 {
        let v: u32;
        unsafe {
            core::arch::asm!("in eax, dx", out("eax") v, in("dx") port,
                options(nomem, nostack, preserves_flags));
        }
        v
    }
    unsafe fn pci_read32(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
        let addr = 0x8000_0000u32
            | ((bus as u32) << 16)
            | ((dev as u32) << 11)
            | ((func as u32) << 8)
            | (offset as u32 & 0xFC);
        unsafe {
            pci_outl(0xCF8, addr);
            pci_inl(0xCFC)
        }
    }

    unsafe {
        let mch_id = pci_read32(0, 0, 0, 0x00);
        if (mch_id & 0xFFFF) == 0x8086 {
            let mmio_base = pci_read32(0, 2, 0, 0x10) & 0xFFFFFFF0;
            if mmio_base != 0 {
                let mmio = mmio_base as *mut u32;
                let dspasurf_ptr = mmio.add(0x70184 / 4);
                let dspbsurf_ptr = mmio.add(0x71184 / 4);

                let active_surf = dspasurf_ptr.read_volatile() & 0xFFFFF000;
                if active_surf != 0 {
                    if !GFX_STATE.fb.is_null() {
                        let target_addr = GFX_STATE.fb as u32;
                        dspasurf_ptr.write_volatile(target_addr);
                        dspbsurf_ptr.write_volatile(target_addr);
                    } else {
                        GFX_STATE.fb = active_surf as *mut u32;
                    }
                }
            }
        }
    }
}

/// Initializes Graphics Output Protocol (GOP), Apple Framebuffer Info, or UGA Draw Protocol.
pub fn init_gfx() {
    let st = get_system_table();
    if st.is_null() {
        return;
    }

    // 1. Try GOP (UEFI 2.0+ / 64-bit EFI)
    let gop_ptr = unsafe { locate_protocol_compat(&EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID) }
        as *mut EfiGraphicsOutputProtocol;

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

    // 2. Try UGA (EFI 1.10 / 32-bit Apple EFI)
    let uga_ptr =
        unsafe { locate_protocol_compat(&EFI_UGA_DRAW_PROTOCOL_GUID) } as *mut EfiUgaDrawProtocol;

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
                let _ = ((*uga_ptr).set_mode)(uga_ptr, width, height, depth, refresh);
                GFX_STATE.fb = core::ptr::null_mut();
                GFX_STATE.uga = uga_ptr;
                GFX_STATE.width = width;
                GFX_STATE.height = height;
                GFX_STATE.stride = width;
                GFX_STATE.cursor_x = 16;
                GFX_STATE.cursor_y = 16;
                GFX_STATE.fg_color = 0xFFFFFFFF;
                GFX_STATE.is_valid = true;

                // Clear screen row by row to pitch black via EfiUgaBltBufferToVideo
                let black_row = [0xFF000000u32; 2560];
                let row_pixels = (width as usize).min(black_row.len());
                for y in 0..height {
                    let _ = ((*uga_ptr).blt)(
                        uga_ptr,
                        black_row.as_ptr() as *mut u32,
                        EfiUgaBltOperation::EfiUgaBltBufferToVideo,
                        0,
                        0,
                        0,
                        y as usize,
                        row_pixels,
                        1,
                        row_pixels * 4,
                    );
                }
            }
            return;
        }
    }

    // 3. Try Apple Framebuffer Info (32-bit & 64-bit Apple EFI)
    let apple_fb_ptr = unsafe { locate_protocol_compat(&EFI_APPLE_FRAMEBUFFER_INFO_PROTOCOL_GUID) }
        as *mut EfiAppleFramebufferInfoProtocol;

    if !apple_fb_ptr.is_null() {
        let mut fb_base: *mut u8 = core::ptr::null_mut();
        let mut fb_size = 0u32;
        let mut depth = 0u32;
        let mut width = 0u32;
        let mut height = 0u32;
        let res = unsafe {
            ((*apple_fb_ptr).get_framebuffer_info)(
                apple_fb_ptr,
                &mut fb_base,
                &mut fb_size,
                &mut depth,
                &mut width,
                &mut height,
            )
        };
        if res == EFI_SUCCESS && !fb_base.is_null() && width > 0 && height > 0 {
            unsafe {
                GFX_STATE.fb = fb_base as *mut u32;
                GFX_STATE.uga = core::ptr::null_mut();
                GFX_STATE.width = width;
                GFX_STATE.height = height;
                GFX_STATE.stride = width;
                GFX_STATE.cursor_x = 16;
                GFX_STATE.cursor_y = 16;
                GFX_STATE.fg_color = 0xFFFFFFFF;
                GFX_STATE.is_valid = true;

                // Clear framebuffer screen to pitch black
                let total_pixels = (width * height) as usize;
                for i in 0..total_pixels {
                    *GFX_STATE.fb.add(i) = 0xFF000000;
                }
            }
            return;
        }
    }

    // 4. Intel GMA 950 direct PCI framebuffer (32-bit Apple EFI — Core Duo/Core 2 Duo Macs).
    //    When all EFI protocol lookups fail (as they do on Apple EFI 1.10), read the
    //    Intel i945GM/i945G display registers directly via PCI config space + MMIO.
    //    All 32-bit Apple EFI machines (MacBook 1,1/2,1; Mac mini 1,1; Xserve 1,1) have GMA 950.
    #[cfg(target_arch = "x86")]
    if unsafe { !GFX_STATE.is_valid } {
        unsafe fn pci_outl(port: u16, val: u32) {
            unsafe {
                core::arch::asm!("out dx, eax", in("dx") port, in("eax") val,
                    options(nomem, nostack, preserves_flags));
            }
        }
        unsafe fn pci_inl(port: u16) -> u32 {
            let v: u32;
            unsafe {
                core::arch::asm!("in eax, dx", out("eax") v, in("dx") port,
                    options(nomem, nostack, preserves_flags));
            }
            v
        }
        unsafe fn pci_read32(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
            let addr = 0x8000_0000u32
                | ((bus as u32) << 16)
                | ((dev as u32) << 11)
                | ((func as u32) << 8)
                | (offset as u32 & 0xFC);
            unsafe {
                pci_outl(0xCF8, addr);
                pci_inl(0xCFC)
            }
        }

        unsafe {
            // Verify Intel MCH at Bus 0, Dev 0, Func 0
            let mch_id = pci_read32(0, 0, 0, 0x00);
            if (mch_id & 0xFFFF) == 0x8086 {
                // BSM (Base Stolen Memory) at MCH offset 0x5C — i945GM/G
                // Bits [31:14] = physical base address of stolen memory (16 KB aligned)
                let bsm = pci_read32(0, 0, 0, 0x5C) & 0xFFFFC000;

                // GMA 950 MMIO base — Bus 0, Dev 2, Func 0, BAR 0 (offset 0x10)
                let mmio_base = pci_read32(0, 2, 0, 0x10) & 0xFFFFFFF0;

                if bsm != 0 && mmio_base != 0 {
                    // PIPEASRC (0x6001C): bits [28:16] = width-1, bits [12:0] = height-1
                    let pipeasrc = (mmio_base as *const u32).add(0x6001C / 4).read_volatile();
                    let width = ((pipeasrc >> 16) & 0x1FFF) + 1;
                    let height = (pipeasrc & 0x1FFF) + 1;

                    // DSPASTRIDE (0x70188): bytes per scan line
                    let stride_bytes =
                        (mmio_base as *const u32).add(0x70188 / 4).read_volatile() & 0x0000FFFF;
                    let stride_pixels = if stride_bytes >= 4 {
                        stride_bytes / 4
                    } else {
                        width
                    };

                    if (640..=2560).contains(&width) && (480..=1600).contains(&height) {
                        GFX_STATE.fb = bsm as *mut u32;
                        GFX_STATE.uga = core::ptr::null_mut();
                        GFX_STATE.width = width;
                        GFX_STATE.height = height;
                        GFX_STATE.stride = stride_pixels;
                        GFX_STATE.cursor_x = 16;
                        GFX_STATE.cursor_y = 16;
                        GFX_STATE.fg_color = 0xFFFFFFFF;
                        GFX_STATE.is_valid = true;

                        // Clear framebuffer to black
                        let total = (stride_pixels * height) as usize;
                        for i in 0..total {
                            *GFX_STATE.fb.add(i) = 0xFF00_0000;
                        }
                    }
                }
            }
        }
    }

    // Flush Apple firmware display driver by calling con_out.output_string with space
    // to force Apple's video driver FlushDisplay() to update the GPU surface register (DSPASURF).
    if unsafe { GFX_STATE.is_valid } {
        let con_out = unsafe { (*st).con_out };
        if !con_out.is_null() {
            let space_buf = [0x0020u16, 0u16];
            unsafe {
                let _ = ((*con_out).output_string)(con_out, space_buf.as_ptr());
            }
        }
    }

    #[cfg(target_arch = "x86")]
    unsafe {
        sync_gma950_display_plane();
    }

    // 5. No graphics protocol found — fall back to ConsoleControl text mode so that
    //    ConOut->output_string() produces visible output on Apple EFI (which boots in
    //    graphics mode where ConOut is silenced). Only reached when is_valid=false.
    if unsafe { !GFX_STATE.is_valid } {
        let cc_ptr = unsafe { locate_protocol_compat(&EFI_CONSOLE_CONTROL_PROTOCOL_GUID) }
            as *mut EfiConsoleControlProtocol;
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

        // 1. Direct Linear Framebuffer (GOP / Apple Framebuffer Info)
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
                        0xFF000000
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
                32, // Explicit 32 bytes stride (8 pixels * 4 bytes/pixel)
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

    // Suppress ConOut whenever ANY graphics protocol is active (GOP, UGA, or Apple Framebuffer).
    // On 32-bit Apple EFI, calling ConOut alongside UGA blt corrupts the display.
    let suppress_conout = unsafe { GFX_STATE.is_valid };

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
