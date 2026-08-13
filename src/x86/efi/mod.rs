#![cfg(target_os = "uefi")]
//! Zero-dependency UEFI environment support for rustid.

pub mod font;

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::ffi::c_void;
use core::fmt::Write;

pub type EfiStatus = usize;
pub type EfiHandle = *mut c_void;

pub const EFI_SUCCESS: EfiStatus = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfiGuid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x9042a9de,
    b: 0x23dc,
    c: 0x4a38,
    d: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub const EFI_UGA_DRAW_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x982c298b,
    b: 0xf4da,
    c: 0x4226,
    d: [0x9e, 0x46, 0x16, 0x9d, 0x36, 0x95, 0xaa, 0x62],
};

#[repr(C)]
pub struct EfiTableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset: unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, bool) -> EfiStatus,
    pub output_string:
        unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const u16) -> EfiStatus,
    pub test_string:
        unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const u16) -> EfiStatus,
    pub query_mode: unsafe extern "efiapi" fn(
        *mut SimpleTextOutputProtocol,
        usize,
        *mut usize,
        *mut usize,
    ) -> EfiStatus,
    pub set_mode: unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize) -> EfiStatus,
    pub set_attribute: unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize) -> EfiStatus,
    pub clear_screen: unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol) -> EfiStatus,
    pub set_cursor_position:
        unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize, usize) -> EfiStatus,
    pub enable_cursor: unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, bool) -> EfiStatus,
    pub mode: *mut c_void,
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
    pub query_mode: *const c_void,
    pub set_mode: *const c_void,
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

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfiMemoryType {
    EfiReservedMemoryType = 0,
    EfiLoaderCode = 1,
    EfiLoaderData = 2,
    EfiBootServicesCode = 3,
    EfiBootServicesData = 4,
    EfiRuntimeServicesCode = 5,
    EfiRuntimeServicesData = 6,
    EfiConventionalMemory = 7,
    EfiUnusableMemory = 8,
    EfiACPIReclaimMemory = 9,
    EfiACPIMemoryNVS = 10,
    EfiMemoryMappedIO = 11,
    EfiMemoryMappedIOPortSpace = 12,
    EfiPalCode = 13,
    EfiPersistentMemory = 14,
    EfiMaxMemoryType = 15,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct EfiInputKey {
    pub scan_code: u16,
    pub unicode_char: u16,
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset: unsafe extern "efiapi" fn(*mut SimpleTextInputProtocol, bool) -> EfiStatus,
    pub read_key_stroke:
        unsafe extern "efiapi" fn(*mut SimpleTextInputProtocol, *mut EfiInputKey) -> EfiStatus,
    pub wait_for_key: *mut c_void,
}

#[repr(C)]
pub struct BootServices {
    pub hdr: EfiTableHeader,
    pub raise_tpl: *const c_void,
    pub restore_tpl: *const c_void,
    pub allocate_pages: *const c_void,
    pub free_pages: *const c_void,
    pub get_memory_map: *const c_void,
    pub allocate_pool: unsafe extern "efiapi" fn(EfiMemoryType, usize, *mut *mut u8) -> EfiStatus,
    pub free_pool: unsafe extern "efiapi" fn(*mut u8) -> EfiStatus,
    pub create_event: *const c_void,
    pub set_timer: *const c_void,
    pub wait_for_event: *const c_void,
    pub signal_event: *const c_void,
    pub close_event: *const c_void,
    pub check_event: *const c_void,
    pub install_protocol_interface: *const c_void,
    pub reinstall_protocol_interface: *const c_void,
    pub uninstall_protocol_interface: *const c_void,
    pub handle_protocol: *const c_void,
    pub reserved: *const c_void,
    pub register_protocol_notify: *const c_void,
    pub locate_handle: *const c_void,
    pub locate_device_path: *const c_void,
    pub install_configuration_table: *const c_void,
    pub image_load: *const c_void,
    pub image_start: *const c_void,
    pub exit: *const c_void,
    pub image_unload: *const c_void,
    pub exit_boot_services: *const c_void,
    pub get_next_monotonic_count: *const c_void,
    pub stall: unsafe extern "efiapi" fn(usize) -> EfiStatus,
    pub set_watchdog_timer: *const c_void,
    pub connect_controller: *const c_void,
    pub disconnect_controller: *const c_void,
    pub open_protocol: *const c_void,
    pub close_protocol: *const c_void,
    pub open_protocol_information: *const c_void,
    pub protocols_per_handle: *const c_void,
    pub locate_handle_buffer: *const c_void,
    pub locate_protocol:
        unsafe extern "efiapi" fn(*const EfiGuid, *const c_void, *mut *mut c_void) -> EfiStatus,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfiResetType {
    EfiResetCold = 0,
    EfiResetWarm = 1,
    EfiResetShutdown = 2,
    EfiResetPlatformSpecific = 3,
}

#[repr(C)]
pub struct RuntimeServices {
    pub hdr: EfiTableHeader,
    pub get_time: *const c_void,
    pub set_time: *const c_void,
    pub get_wakeup_time: *const c_void,
    pub set_wakeup_time: *const c_void,
    pub set_virtual_address_map: *const c_void,
    pub convert_pointer: *const c_void,
    pub get_variable: *const c_void,
    pub get_next_variable_name: *const c_void,
    pub set_variable: *const c_void,
    pub get_next_high_mono_count: *const c_void,
    pub reset_system: unsafe extern "efiapi" fn(
        reset_type: EfiResetType,
        reset_status: EfiStatus,
        data_size: usize,
        reset_data: *const u16,
    ) -> !,
}

#[repr(C)]
pub struct EfiSystemTable {
    pub hdr: EfiTableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_in_handle: EfiHandle,
    pub con_in: *mut SimpleTextInputProtocol,
    pub console_out_handle: EfiHandle,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: EfiHandle,
    pub std_err: *mut SimpleTextOutputProtocol,
    pub runtime_services: *mut RuntimeServices,
    pub boot_services: *mut BootServices,
}

static mut SYSTEM_TABLE: *mut EfiSystemTable = core::ptr::null_mut();
static mut IMAGE_HANDLE: EfiHandle = core::ptr::null_mut();

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
    fg_color: 0x00FFFFFF,
    is_valid: false,
};

pub unsafe fn init_efi(image_handle: EfiHandle, system_table: *mut EfiSystemTable) {
    unsafe {
        IMAGE_HANDLE = image_handle;
        SYSTEM_TABLE = system_table;
    }
}

pub fn get_system_table() -> *mut EfiSystemTable {
    unsafe { SYSTEM_TABLE }
}

/// Initializes Graphics Output Protocol (GOP) or UGA Draw Protocol.
pub fn init_gfx() {
    let st = get_system_table();
    if st.is_null() {
        return;
    }
    let bs = unsafe { (*st).boot_services };
    if bs.is_null() {
        return;
    }

    // 1. Try GOP (UEFI 2.0+ / 64-bit EFI)
    let mut gop_ptr: *mut EfiGraphicsOutputProtocol = core::ptr::null_mut();
    let status = unsafe {
        ((*bs).locate_protocol)(
            &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
            core::ptr::null(),
            &mut gop_ptr as *mut _ as *mut *mut c_void,
        )
    };

    if status == EFI_SUCCESS && !gop_ptr.is_null() {
        let mode = unsafe { (*gop_ptr).mode };
        if !mode.is_null() {
            let info = unsafe { (*mode).info };
            let fb_base = unsafe { (*mode).frame_buffer_base };
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
                    GFX_STATE.fg_color = 0x00FFFFFF;
                    GFX_STATE.is_valid = true;

                    // Clear framebuffer screen to pitch black
                    let total_pixels = (stride * height) as usize;
                    for i in 0..total_pixels {
                        *GFX_STATE.fb.add(i) = 0x00000000;
                    }
                }
                return;
            }
        }
    }

    // 2. Try UGA (EFI 1.10 / 32-bit Apple EFI)
    let mut uga_ptr: *mut EfiUgaDrawProtocol = core::ptr::null_mut();
    let status_uga = unsafe {
        ((*bs).locate_protocol)(
            &EFI_UGA_DRAW_PROTOCOL_GUID,
            core::ptr::null(),
            &mut uga_ptr as *mut _ as *mut *mut c_void,
        )
    };

    if status_uga == EFI_SUCCESS && !uga_ptr.is_null() {
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

                // Clear screen to pitch black via UGA Fill
                let mut fill_color = 0x00000000u32;
                let _ = ((*uga_ptr).blt)(
                    uga_ptr,
                    &mut fill_color,
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
            for row in 0..16 {
                let py = GFX_STATE.cursor_y + row as u32;
                if py >= GFX_STATE.height {
                    break;
                }
                let byte = font_glyph[row];
                for col in 0..8 {
                    let px = GFX_STATE.cursor_x + col as u32;
                    if px >= GFX_STATE.width {
                        break;
                    }
                    let is_set = (byte & (1 << (7 - col))) != 0;
                    let pixel = if is_set {
                        GFX_STATE.fg_color
                    } else {
                        0x00000000
                    };
                    *GFX_STATE.fb.add((py * GFX_STATE.stride + px) as usize) = pixel;
                }
            }
        }
        // 2. UGA Blt (32-bit Apple EFI)
        else if !GFX_STATE.uga.is_null() {
            let mut glyph_pixels = [0u32; 128]; // 8x16 pixels
            for row in 0..16 {
                let byte = font_glyph[row];
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
                8 * 4,
            );
        }

        GFX_STATE.cursor_x += 8;
        if GFX_STATE.cursor_x + 8 > GFX_STATE.width {
            GFX_STATE.cursor_x = 16;
            GFX_STATE.cursor_y += 16;
        }
    }
}

/// Clears the console screen and initializes GOP/UGA framebuffer.
pub fn clear_screen_black() {
    init_gfx();

    let st = get_system_table();
    if st.is_null() {
        return;
    }
    let con_out = unsafe { (*st).con_out };
    let proto = if !con_out.is_null() {
        con_out
    } else {
        unsafe { (*st).std_err }
    };
    if !proto.is_null() {
        unsafe {
            let _ = ((*proto).set_attribute)(proto, 0x0F);
            let _ = ((*proto).clear_screen)(proto);
        }
    }
}

/// Prompts the user to press a key to shutdown.
pub fn wait_for_keypress(timeout_seconds: Option<u32>) {
    use crate::println;

    let st = get_system_table();
    if st.is_null() {
        return;
    }

    let con_in = unsafe { (*st).con_in };
    let bs = unsafe { (*st).boot_services };

    if con_in.is_null() {
        return;
    }

    // Reset console input buffer to clear any queued keypresses
    unsafe { ((*con_in).reset)(con_in, false) };

    if let Some(secs) = timeout_seconds {
        println!(
            "\nPress any key to shutdown (auto-shutdown in {}s)...",
            secs
        );
        let loops = secs * 10;
        for _ in 0..loops {
            let mut key = EfiInputKey::default();
            let status = unsafe { ((*con_in).read_key_stroke)(con_in, &mut key) };
            if status == EFI_SUCCESS {
                break;
            }
            if !bs.is_null() {
                unsafe { ((*bs).stall)(100_000) };
            }
        }
    } else {
        println!("\nPress any key to shutdown...");
        loop {
            let mut key = EfiInputKey::default();
            let status = unsafe { ((*con_in).read_key_stroke)(con_in, &mut key) };
            if status == EFI_SUCCESS {
                break;
            }
            if !bs.is_null() {
                unsafe { ((*bs).stall)(100_000) };
            }
        }
    }
}

/// Exits the EFI application or shuts down QEMU/system.
pub fn exit(code: u8) -> ! {
    // 1. Try QEMU ISA debug exit (port 0xf4)
    unsafe {
        asm!(
            "out 0xf4, al",
            in("al") code,
            options(nomem, nostack, preserves_flags)
        );
    }

    // 2. Try UEFI RuntimeServices ResetSystem shutdown
    let st = get_system_table();
    if !st.is_null() {
        let rs = unsafe { (*st).runtime_services };
        if !rs.is_null() {
            unsafe {
                ((*rs).reset_system)(
                    EfiResetType::EfiResetShutdown,
                    if code == 0 { 0 } else { 1 },
                    0,
                    core::ptr::null(),
                );
            }
        }
    }

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!("hlt")
        };
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!("hlt")
        };
    }
}

struct EfiAllocator;

unsafe impl GlobalAlloc for EfiAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let st = get_system_table();
        if st.is_null() {
            return core::ptr::null_mut();
        }
        let bs = unsafe { (*st).boot_services };
        if bs.is_null() {
            return core::ptr::null_mut();
        }
        let mut ptr: *mut u8 = core::ptr::null_mut();
        let status =
            unsafe { ((*bs).allocate_pool)(EfiMemoryType::EfiLoaderData, layout.size(), &mut ptr) };
        if status == EFI_SUCCESS {
            ptr
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let st = get_system_table();
        if st.is_null() {
            return;
        }
        let bs = unsafe { (*st).boot_services };
        if !bs.is_null() {
            unsafe { ((*bs).free_pool)(ptr) };
        }
    }
}

#[global_allocator]
static ALLOCATOR: EfiAllocator = EfiAllocator;

/// Custom panic handler for UEFI environment.
#[cfg(not(test))]
#[cold]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use crate::println;

    if let Some(location) = _info.location() {
        println!(
            "Panic in file '{}' at line {}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    } else {
        println!("Panic for unknown reason.");
    }

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!("hlt")
        };
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!("hlt")
        };
    }
}

/// Prints a formatted string to UEFI graphics display (GOP/UGA) or fallback ConOut.
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

    let is_gfx = unsafe { GFX_STATE.is_valid };

    let flush = |buf: &mut [u16; 256], idx: &mut usize| {
        // Only flush to ConOut if GFX framebuffer is NOT active (prevents duplicate text)
        if *idx > 0 && !proto.is_null() && !is_gfx {
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
                        0 => (0x0F, 0x00FFFFFF),       // White
                        32 => (0x0A, 0x0055FF55),      // Light Green
                        34 | 94 => (0x0B, 0x0055FFFF), // Light Cyan
                        33 | 93 => (0x0E, 0x00FFFF55), // Yellow
                        31 | 91 => (0x0C, 0x00FF5555), // Light Red
                        36 | 96 => (0x0B, 0x0055FFFF), // Light Cyan
                        90 => (0x08, 0x00888888),      // Dark Gray
                        _ => (0x0F, 0x00FFFFFF),
                    };
                    flush(&mut utf16_buf, &mut idx);
                    if !proto.is_null() && !is_gfx {
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
                if !proto.is_null() && !is_gfx {
                    unsafe { ((*proto).set_attribute)(proto, 0x0F) };
                }
                unsafe {
                    GFX_STATE.fg_color = 0x00FFFFFF;
                }
            }
            continue;
        }

        // Draw character to GOP / UGA graphics display
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
