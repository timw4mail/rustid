#![cfg(uefi)]
//! Core EFI operating system bindings and services for rustid.

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::ffi::c_void;

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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EfiConfigurationTable {
    pub vendor_guid: EfiGuid,
    pub vendor_table: *mut c_void,
}

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
    pub wait_for_event:
        unsafe extern "efiapi" fn(usize, *const *mut c_void, *mut usize) -> EfiStatus,
    pub signal_event: *const c_void,
    pub close_event: *const c_void,
    pub check_event: unsafe extern "efiapi" fn(*mut c_void) -> EfiStatus,
    pub install_protocol_interface: *const c_void,
    pub reinstall_protocol_interface: *const c_void,
    pub uninstall_protocol_interface: *const c_void,
    pub handle_protocol:
        unsafe extern "efiapi" fn(EfiHandle, *const EfiGuid, *mut *mut c_void) -> EfiStatus,
    pub reserved: *const c_void,
    pub register_protocol_notify: *const c_void,
    pub locate_handle: unsafe extern "efiapi" fn(
        i32,
        *const EfiGuid,
        *const c_void,
        *mut usize,
        *mut EfiHandle,
    ) -> EfiStatus,
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
    pub locate_handle_buffer: unsafe extern "efiapi" fn(
        i32,
        *const EfiGuid,
        *const c_void,
        *mut usize,
        *mut *mut EfiHandle,
    ) -> EfiStatus,
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
    pub number_of_table_entries: usize,
    pub configuration_table: *mut c_void,
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

#[derive(Debug, Copy, Clone)]
pub struct EfiFirmwareInfo {
    pub vendor: [char; 64],
    pub vendor_len: usize,
    pub revision: u32,
    pub is_apple: bool,
    pub is_efi_1_10: bool,
    pub is_apple_32bit: bool,
}

pub fn detect_firmware() -> EfiFirmwareInfo {
    let st = get_system_table();
    let mut info = EfiFirmwareInfo {
        vendor: ['\0'; 64],
        vendor_len: 0,
        revision: 0,
        is_apple: false,
        is_efi_1_10: false,
        is_apple_32bit: false,
    };
    if st.is_null() {
        return info;
    }
    unsafe {
        info.revision = (*st).hdr.revision;
        info.is_efi_1_10 = (info.revision >> 16) == 1;
        let vendor_ptr = (*st).firmware_vendor;
        if !vendor_ptr.is_null() {
            let mut i = 0;
            while i < 63 {
                let u = *vendor_ptr.add(i);
                if u == 0 {
                    break;
                }
                if let Some(ch) = char::from_u32(u as u32) {
                    info.vendor[i] = ch;
                }
                i += 1;
            }
            info.vendor_len = i;

            let apple_match = ['A', 'p', 'p', 'l', 'e'];
            let mut apple_idx = 0;
            let mut is_apple = false;
            for &ch in &info.vendor[..i] {
                if ch.to_ascii_uppercase() == apple_match[apple_idx].to_ascii_uppercase() {
                    apple_idx += 1;
                    if apple_idx == apple_match.len() {
                        is_apple = true;
                        break;
                    }
                } else {
                    apple_idx = 0;
                }
            }
            info.is_apple = is_apple;
        }
        #[cfg(target_arch = "x86")]
        {
            info.is_apple_32bit = info.is_apple && info.is_efi_1_10;
        }
    }
    info
}

/// Backward/forward compatible protocol locator (EFI 1.0, EFI 1.10, UEFI 2.x).
pub unsafe fn locate_protocol_compat(guid: &EfiGuid) -> *mut c_void {
    let st = get_system_table();
    if st.is_null() {
        return core::ptr::null_mut();
    }
    let bs = unsafe { (*st).boot_services };
    if bs.is_null() {
        return core::ptr::null_mut();
    }

    // 1. Check system table console handles first (HandleProtocol at 0x58 is safe on ALL EFI versions)
    let handles_to_check = unsafe {
        [
            (*st).console_out_handle,
            (*st).standard_error_handle,
            (*st).console_in_handle,
        ]
    };
    for &h in &handles_to_check {
        if !h.is_null() {
            let mut interface: *mut c_void = core::ptr::null_mut();
            let status = unsafe { ((*bs).handle_protocol)(h, guid, &mut interface) };
            if status == EFI_SUCCESS && !interface.is_null() {
                return interface;
            }
        }
    }

    // 2. LocateHandle (ByProtocol = 2) via offset 0x64 (safe on all EFI versions back to 2000)
    let mut handles = [core::ptr::null_mut::<c_void>(); 256];
    let mut buf_size = core::mem::size_of_val(&handles);
    let status = unsafe {
        ((*bs).locate_handle)(
            2, // ByProtocol
            guid,
            core::ptr::null(),
            &mut buf_size,
            handles.as_mut_ptr(),
        )
    };

    if status == EFI_SUCCESS && buf_size >= core::mem::size_of::<EfiHandle>() {
        let count = (buf_size / core::mem::size_of::<EfiHandle>()).min(handles.len());
        for &h in handles.iter().take(count) {
            if !h.is_null() {
                let mut ptr: *mut c_void = core::ptr::null_mut();
                let res = unsafe { ((*bs).handle_protocol)(h, guid, &mut ptr) };
                if res == EFI_SUCCESS && !ptr.is_null() {
                    return ptr;
                }
            }
        }
    }

    // 3. LocateHandle (AllHandles = 0) via offset 0x64
    let mut all_handles = [core::ptr::null_mut::<c_void>(); 256];
    let mut all_buf_size = core::mem::size_of_val(&all_handles);
    let all_status = unsafe {
        ((*bs).locate_handle)(
            0, // AllHandles
            core::ptr::null(),
            core::ptr::null(),
            &mut all_buf_size,
            all_handles.as_mut_ptr(),
        )
    };

    if all_status == EFI_SUCCESS && all_buf_size >= core::mem::size_of::<EfiHandle>() {
        let count = (all_buf_size / core::mem::size_of::<EfiHandle>()).min(all_handles.len());
        for &h in all_handles.iter().take(count) {
            if !h.is_null() {
                let mut ptr: *mut c_void = core::ptr::null_mut();
                let res = unsafe { ((*bs).handle_protocol)(h, guid, &mut ptr) };
                if res == EFI_SUCCESS && !ptr.is_null() {
                    return ptr;
                }
            }
        }
    }

    core::ptr::null_mut()
}

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

    if let Some(secs) = timeout_seconds {
        println!("\nPress any key to exit (auto-exit in {}s)...", secs);
        let loops = secs * 50; // 50 * 20ms = 1 sec
        let wait_key = unsafe { (*con_in).wait_for_key };
        for _ in 0..loops {
            if !wait_key.is_null() && !bs.is_null() {
                let _ = unsafe { ((*bs).check_event)(wait_key) };
            }
            if !bs.is_null() {
                unsafe { ((*bs).stall)(20_000) };
            }
            let mut key = EfiInputKey::default();
            let status = unsafe { ((*con_in).read_key_stroke)(con_in, &mut key) };
            if status == EFI_SUCCESS {
                break;
            }
        }
    } else {
        println!("\nPress any key to exit...");
        let wait_key = unsafe { (*con_in).wait_for_key };
        loop {
            if !wait_key.is_null() && !bs.is_null() {
                let _ = unsafe { ((*bs).check_event)(wait_key) };
            }
            if !bs.is_null() {
                unsafe { ((*bs).stall)(20_000) };
            }
            let mut key = EfiInputKey::default();
            let status = unsafe { ((*con_in).read_key_stroke)(con_in, &mut key) };
            if status == EFI_SUCCESS {
                break;
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

#[cfg(uefi)]
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
