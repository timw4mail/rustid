#![cfg(target_os = "uefi")]
//! Zero-dependency UEFI environment support for rustid.

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::ffi::c_void;
use core::fmt::Write;

pub type EfiStatus = usize;
pub type EfiHandle = *mut c_void;

pub const EFI_SUCCESS: EfiStatus = 0;

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
pub struct BootServices {
    pub hdr: EfiTableHeader,
    pub raise_tpl: *const c_void,
    pub restore_tpl: *const c_void,
    pub allocate_pages: *const c_void,
    pub free_pages: *const c_void,
    pub get_memory_map: *const c_void,
    pub allocate_pool: unsafe extern "efiapi" fn(EfiMemoryType, usize, *mut *mut u8) -> EfiStatus,
    pub free_pool: unsafe extern "efiapi" fn(*mut u8) -> EfiStatus,
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
    pub con_in: *mut c_void,
    pub console_out_handle: EfiHandle,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: EfiHandle,
    pub std_err: *mut SimpleTextOutputProtocol,
    pub runtime_services: *mut RuntimeServices,
    pub boot_services: *mut BootServices,
}

static mut SYSTEM_TABLE: *mut EfiSystemTable = core::ptr::null_mut();
static mut IMAGE_HANDLE: EfiHandle = core::ptr::null_mut();

pub unsafe fn init_efi(image_handle: EfiHandle, system_table: *mut EfiSystemTable) {
    unsafe {
        IMAGE_HANDLE = image_handle;
        SYSTEM_TABLE = system_table;
    }
}

pub fn get_system_table() -> *mut EfiSystemTable {
    unsafe { SYSTEM_TABLE }
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
        unsafe { asm!("hlt") };
        #[cfg(target_arch = "x86")]
        unsafe { asm!("hlt") };
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

/// Prints a formatted string to the UEFI console output.
pub fn _print_str(s: &str) {
    let st = get_system_table();
    if st.is_null() {
        return;
    }
    let con_out = unsafe { (*st).con_out };
    if con_out.is_null() {
        return;
    }

    let mut utf16_buf = [0u16; 128];
    let mut idx = 0;

    for ch in s.chars() {
        if ch == '\n' {
            if idx > 0 && utf16_buf[idx - 1] != ('\r' as u16) {
                utf16_buf[idx] = '\r' as u16;
                idx += 1;
                if idx >= utf16_buf.len() - 2 {
                    utf16_buf[idx] = 0;
                    unsafe { ((*con_out).output_string)(con_out, utf16_buf.as_ptr()) };
                    idx = 0;
                }
            }
        }

        let mut code_units = [0u16; 2];
        let encoded = ch.encode_utf16(&mut code_units);
        for &mut u in encoded {
            utf16_buf[idx] = u;
            idx += 1;
            if idx >= utf16_buf.len() - 2 {
                utf16_buf[idx] = 0;
                unsafe { ((*con_out).output_string)(con_out, utf16_buf.as_ptr()) };
                idx = 0;
            }
        }
    }

    if idx > 0 {
        utf16_buf[idx] = 0;
        unsafe { ((*con_out).output_string)(con_out, utf16_buf.as_ptr()) };
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
