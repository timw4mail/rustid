//! Windows compatibility shims and assembly hooks for legacy Windows (9x/NT).
#![allow(dead_code)]

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
core::arch::global_asm!(
    r#"
    .section .text
    .globl _SystemFunction036@8
    .def _SystemFunction036@8; .scl 2; .type 32; .endef
    _SystemFunction036@8:
        jmp _rustid_system_function_036@8

    .globl _ProcessPrng@8
    .def _ProcessPrng@8; .scl 2; .type 32; .endef
    _ProcessPrng@8:
        jmp _rustid_system_function_036@8

    .globl _WaitOnAddress@16
    .def _WaitOnAddress@16; .scl 2; .type 32; .endef
    _WaitOnAddress@16:
        jmp _rustid_wait_on_address@16

    .globl _WakeByAddressAll@4
    .def _WakeByAddressAll@4; .scl 2; .type 32; .endef
    _WakeByAddressAll@4:
        jmp _rustid_wake_by_address@4

    .globl _WakeByAddressSingle@4
    .def _WakeByAddressSingle@4; .scl 2; .type 32; .endef
    _WakeByAddressSingle@4:
        jmp _rustid_wake_by_address@4

    .globl _AddVectoredExceptionHandler@8
    .def _AddVectoredExceptionHandler@8; .scl 2; .type 32; .endef
    _AddVectoredExceptionHandler@8:
        jmp _rustid_add_vectored_exception_handler@8

    .globl _RemoveVectoredExceptionHandler@4
    .def _RemoveVectoredExceptionHandler@4; .scl 2; .type 32; .endef
    _RemoveVectoredExceptionHandler@4:
        jmp _rustid_remove_vectored_exception_handler@4

    .globl _FlsAlloc@4
    .def _FlsAlloc@4; .scl 2; .type 32; .endef
    _FlsAlloc@4:
        jmp _rustid_fls_alloc@4

    .globl _FlsFree@4
    .def _FlsFree@4; .scl 2; .type 32; .endef
    _FlsFree@4:
        jmp _rustid_fls_free@4

    .globl _FlsSetValue@8
    .def _FlsSetValue@8; .scl 2; .type 32; .endef
    _FlsSetValue@8:
        jmp _rustid_fls_set_value@8

    .globl _FlsGetValue@4
    .def _FlsGetValue@4; .scl 2; .type 32; .endef
    _FlsGetValue@4:
        jmp _rustid_fls_get_value@4

    .globl _InitOnceBeginInitialize@16
    .def _InitOnceBeginInitialize@16; .scl 2; .type 32; .endef
    _InitOnceBeginInitialize@16:
        jmp _rustid_init_once_begin_initialize@16

    .globl _InitOnceComplete@12
    .def _InitOnceComplete@12; .scl 2; .type 32; .endef
    _InitOnceComplete@12:
        jmp _rustid_init_once_complete@12

    .globl _IsThreadAFiber@0
    .def _IsThreadAFiber@0; .scl 2; .type 32; .endef
    _IsThreadAFiber@0:
        jmp _rustid_is_thread_a_fiber@0

    .globl _GetModuleHandleExW@12
    .def _GetModuleHandleExW@12; .scl 2; .type 32; .endef
    _GetModuleHandleExW@12:
        jmp _rustid_get_module_handle_ex_w@12

    .globl _SetThreadStackGuarantee@4
    .def _SetThreadStackGuarantee@4; .scl 2; .type 32; .endef
    _SetThreadStackGuarantee@4:
        jmp _rustid_set_thread_stack_guarantee@4

    .globl _GetFileInformationByHandleEx@16
    .def _GetFileInformationByHandleEx@16; .scl 2; .type 32; .endef
    _GetFileInformationByHandleEx@16:
        jmp _rustid_get_file_information_by_handle_ex@16

    .globl _SetFileInformationByHandle@16
    .def _SetFileInformationByHandle@16; .scl 2; .type 32; .endef
    _SetFileInformationByHandle@16:
        jmp _rustid_set_file_information_by_handle@16

    .globl _RtlCaptureContext@4
    .def _RtlCaptureContext@4; .scl 2; .type 32; .endef
    _RtlCaptureContext@4:
        jmp _rustid_rtl_capture_context@4

    .globl _GetLogicalProcessorInformationEx@12
    .def _GetLogicalProcessorInformationEx@12; .scl 2; .type 32; .endef
    _GetLogicalProcessorInformationEx@12:
        jmp _rustid_get_logical_processor_information_ex@12

    .globl _NtReadFile@36
    .def _NtReadFile@36; .scl 2; .type 32; .endef
    _NtReadFile@36:
        jmp _rustid_nt_read_file@36

    .globl _NtWriteFile@36
    .def _NtWriteFile@36; .scl 2; .type 32; .endef
    _NtWriteFile@36:
        jmp _rustid_nt_write_file@36

    .globl _RtlNtStatusToDosError@4
    .def _RtlNtStatusToDosError@4; .scl 2; .type 32; .endef
    _RtlNtStatusToDosError@4:
        jmp _rustid_rtl_nt_status_to_dos_error@4

    .globl _Module32FirstW@8
    .def _Module32FirstW@8; .scl 2; .type 32; .endef
    _Module32FirstW@8:
        jmp _rustid_module32_first_w@8

    .globl _Module32NextW@8
    .def _Module32NextW@8; .scl 2; .type 32; .endef
    _Module32NextW@8:
        jmp _rustid_module32_next_w@8

    .globl _Process32FirstW@8
    .def _Process32FirstW@8; .scl 2; .type 32; .endef
    _Process32FirstW@8:
        jmp _rustid_process32_first_w@8

    .globl _Process32NextW@8
    .def _Process32NextW@8; .scl 2; .type 32; .endef
    _Process32NextW@8:
        jmp _rustid_process32_next_w@8

    .globl _IsDebuggerPresent@0
    .def _IsDebuggerPresent@0; .scl 2; .type 32; .endef
    _IsDebuggerPresent@0:
        jmp _rustid_is_debugger_present@0

    .globl _TryEnterCriticalSection@4
    .def _TryEnterCriticalSection@4; .scl 2; .type 32; .endef
    _TryEnterCriticalSection@4:
        jmp _rustid_try_enter_critical_section@4

    .section .rdata,"dr"
    .globl __imp__SystemFunction036@8
    __imp__SystemFunction036@8:
        .long _SystemFunction036@8
    .globl __imp_SystemFunction036
    __imp_SystemFunction036:
        .long _SystemFunction036@8

    .globl __imp__ProcessPrng@8
    __imp__ProcessPrng@8:
        .long _ProcessPrng@8
    .globl __imp_ProcessPrng
    __imp_ProcessPrng:
        .long _ProcessPrng@8

    .globl __imp__WaitOnAddress@16
    __imp__WaitOnAddress@16:
        .long _WaitOnAddress@16
    .globl __imp_WaitOnAddress
    __imp_WaitOnAddress:
        .long _WaitOnAddress@16

    .globl __imp__WakeByAddressAll@4
    __imp__WakeByAddressAll@4:
        .long _WakeByAddressAll@4
    .globl __imp_WakeByAddressAll
    __imp_WakeByAddressAll:
        .long _WakeByAddressAll@4

    .globl __imp__WakeByAddressSingle@4
    __imp__WakeByAddressSingle@4:
        .long _WakeByAddressSingle@4
    .globl __imp_WakeByAddressSingle
    __imp_WakeByAddressSingle:
        .long _WakeByAddressSingle@4

    .globl __imp__AddVectoredExceptionHandler@8
    __imp__AddVectoredExceptionHandler@8:
        .long _AddVectoredExceptionHandler@8
    .globl __imp_AddVectoredExceptionHandler
    __imp_AddVectoredExceptionHandler:
        .long _AddVectoredExceptionHandler@8

    .globl __imp__RemoveVectoredExceptionHandler@4
    __imp__RemoveVectoredExceptionHandler@4:
        .long _RemoveVectoredExceptionHandler@4
    .globl __imp_RemoveVectoredExceptionHandler
    __imp_RemoveVectoredExceptionHandler:
        .long _RemoveVectoredExceptionHandler@4

    .globl __imp__FlsAlloc@4
    __imp__FlsAlloc@4:
        .long _FlsAlloc@4
    .globl __imp_FlsAlloc
    __imp_FlsAlloc:
        .long _FlsAlloc@4

    .globl __imp__FlsFree@4
    __imp__FlsFree@4:
        .long _FlsFree@4
    .globl __imp_FlsFree
    __imp_FlsFree:
        .long _FlsFree@4

    .globl __imp__FlsSetValue@8
    __imp__FlsSetValue@8:
        .long _FlsSetValue@8
    .globl __imp_FlsSetValue
    __imp_FlsSetValue:
        .long _FlsSetValue@8

    .globl __imp__FlsGetValue@4
    __imp__FlsGetValue@4:
        .long _FlsGetValue@4
    .globl __imp_FlsGetValue
    __imp_FlsGetValue:
        .long _FlsGetValue@4

    .globl __imp__InitOnceBeginInitialize@16
    __imp__InitOnceBeginInitialize@16:
        .long _InitOnceBeginInitialize@16
    .globl __imp_InitOnceBeginInitialize
    __imp_InitOnceBeginInitialize:
        .long _InitOnceBeginInitialize@16

    .globl __imp__InitOnceComplete@12
    __imp__InitOnceComplete@12:
        .long _InitOnceComplete@12
    .globl __imp_InitOnceComplete
    __imp_InitOnceComplete:
        .long _InitOnceComplete@12

    .globl __imp__IsThreadAFiber@0
    __imp__IsThreadAFiber@0:
        .long _IsThreadAFiber@0
    .globl __imp_IsThreadAFiber
    __imp_IsThreadAFiber:
        .long _IsThreadAFiber@0

    .globl __imp__GetModuleHandleExW@12
    __imp__GetModuleHandleExW@12:
        .long _GetModuleHandleExW@12
    .globl __imp_GetModuleHandleExW
    __imp_GetModuleHandleExW:
        .long _GetModuleHandleExW@12

    .globl __imp__SetThreadStackGuarantee@4
    __imp__SetThreadStackGuarantee@4:
        .long _SetThreadStackGuarantee@4
    .globl __imp_SetThreadStackGuarantee
    __imp_SetThreadStackGuarantee:
        .long _SetThreadStackGuarantee@4

    .globl __imp__GetFileInformationByHandleEx@16
    __imp__GetFileInformationByHandleEx@16:
        .long _GetFileInformationByHandleEx@16
    .globl __imp_GetFileInformationByHandleEx
    __imp_GetFileInformationByHandleEx:
        .long _GetFileInformationByHandleEx@16

    .globl __imp__SetFileInformationByHandle@16
    __imp__SetFileInformationByHandle@16:
        .long _SetFileInformationByHandle@16
    .globl __imp_SetFileInformationByHandle
    __imp_SetFileInformationByHandle:
        .long _SetFileInformationByHandle@16

    .globl __imp__RtlCaptureContext@4
    __imp__RtlCaptureContext@4:
        .long _RtlCaptureContext@4
    .globl __imp_RtlCaptureContext
    __imp_RtlCaptureContext:
        .long _RtlCaptureContext@4

    .globl __imp__GetLogicalProcessorInformationEx@12
    __imp__GetLogicalProcessorInformationEx@12:
        .long _GetLogicalProcessorInformationEx@12
    .globl __imp_GetLogicalProcessorInformationEx
    __imp_GetLogicalProcessorInformationEx:
        .long _GetLogicalProcessorInformationEx@12

    .globl __imp__NtReadFile@36
    __imp__NtReadFile@36:
        .long _NtReadFile@36
    .globl __imp_NtReadFile
    __imp_NtReadFile:
        .long _NtReadFile@36

    .globl __imp__NtWriteFile@36
    __imp__NtWriteFile@36:
        .long _NtWriteFile@36
    .globl __imp_NtWriteFile
    __imp_NtWriteFile:
        .long _NtWriteFile@36

    .globl __imp__RtlNtStatusToDosError@4
    __imp__RtlNtStatusToDosError@4:
        .long _RtlNtStatusToDosError@4
    .globl __imp_RtlNtStatusToDosError
    __imp_RtlNtStatusToDosError:
        .long _RtlNtStatusToDosError@4

    .globl __imp__Module32FirstW@8
    __imp__Module32FirstW@8:
        .long _Module32FirstW@8
    .globl __imp_Module32FirstW
    __imp_Module32FirstW:
        .long _Module32FirstW@8

    .globl __imp__Module32NextW@8
    __imp__Module32NextW@8:
        .long _Module32NextW@8
    .globl __imp_Module32NextW
    __imp_Module32NextW:
        .long _Module32NextW@8

    .globl __imp__Process32FirstW@8
    __imp__Process32FirstW@8:
        .long _Process32FirstW@8
    .globl __imp_Process32FirstW
    __imp_Process32FirstW:
        .long _Process32FirstW@8

    .globl __imp__Process32NextW@8
    __imp__Process32NextW@8:
        .long _Process32NextW@8
    .globl __imp_Process32NextW
    __imp_Process32NextW:
        .long _Process32NextW@8

    .globl __imp__IsDebuggerPresent@0
    __imp__IsDebuggerPresent@0:
        .long _IsDebuggerPresent@0
    .globl __imp_IsDebuggerPresent
    __imp_IsDebuggerPresent:
        .long _IsDebuggerPresent@0

    .globl __imp__TryEnterCriticalSection@4
    __imp__TryEnterCriticalSection@4:
        .long _TryEnterCriticalSection@4
    .globl __imp_TryEnterCriticalSection
    __imp_TryEnterCriticalSection:
        .long _TryEnterCriticalSection@4
    "#
);

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_system_function_036(
    buf: *mut core::ffi::c_void,
    len: u32,
) -> i32 {
    use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
    static PFN: AtomicUsize = AtomicUsize::new(0);
    static INITED: AtomicUsize = AtomicUsize::new(0);
    static SEED: AtomicU32 = AtomicU32::new(0x85ebca6b);

    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn GetTickCount() -> u32;
    }

    if INITED.load(Ordering::Relaxed) == 0 {
        unsafe {
            let advapi = GetModuleHandleA(b"advapi32.dll\0".as_ptr());
            if !advapi.is_null() {
                let p = GetProcAddress(advapi, b"SystemFunction036\0".as_ptr());
                if !p.is_null() {
                    PFN.store(p as usize, Ordering::Relaxed);
                }
            }
        }
        INITED.store(1, Ordering::Relaxed);
    }

    let pfn_addr = PFN.load(Ordering::Relaxed);
    if pfn_addr != 0 {
        let pfn: extern "system" fn(*mut core::ffi::c_void, u32) -> i32 =
            unsafe { std::mem::transmute(pfn_addr) };
        return pfn(buf, len);
    }

    // Windows 95 / 98 / ME fallback: LCG random generator seeded by GetTickCount
    let tick = unsafe { GetTickCount() };
    let p = buf as *mut u8;
    let mut current_seed = SEED
        .load(Ordering::Relaxed)
        .wrapping_mul(1664525)
        .wrapping_add(1013904223)
        .wrapping_add(tick);
    for i in 0..len as usize {
        current_seed = current_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        unsafe {
            *p.add(i) = (current_seed >> 16) as u8;
        }
    }
    SEED.store(current_seed, Ordering::Relaxed);
    1
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_wait_on_address(
    address: *const core::ffi::c_void,
    compare_address: *const core::ffi::c_void,
    address_size: usize,
    dw_milliseconds: u32,
) -> i32 {
    unsafe extern "system" {
        fn Sleep(dwMilliseconds: u32);
    }
    if address.is_null() || compare_address.is_null() {
        return 1;
    }
    let same = match address_size {
        1 => unsafe { *(address as *const u8) == *(compare_address as *const u8) },
        2 => unsafe { *(address as *const u16) == *(compare_address as *const u16) },
        4 => unsafe { *(address as *const u32) == *(compare_address as *const u32) },
        8 => unsafe { *(address as *const u64) == *(compare_address as *const u64) },
        _ => true,
    };
    if same {
        unsafe {
            Sleep(if dw_milliseconds == 0xFFFFFFFF {
                1
            } else {
                dw_milliseconds.max(1)
            });
        }
    }
    1
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_wake_by_address(_address: *const core::ffi::c_void) {}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_add_vectored_exception_handler(
    first: u32,
    handler: *const core::ffi::c_void,
) -> *mut core::ffi::c_void {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p = GetProcAddress(k32, b"AddVectoredExceptionHandler\0".as_ptr());
            if !p.is_null() {
                let pfn: extern "system" fn(
                    u32,
                    *const core::ffi::c_void,
                ) -> *mut core::ffi::c_void = std::mem::transmute(p);
                return pfn(first, handler);
            }
        }
    }
    1 as *mut core::ffi::c_void
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_remove_vectored_exception_handler(
    handle: *const core::ffi::c_void,
) -> u32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p = GetProcAddress(k32, b"RemoveVectoredExceptionHandler\0".as_ptr());
            if !p.is_null() {
                let pfn: extern "system" fn(*const core::ffi::c_void) -> u32 =
                    std::mem::transmute(p);
                return pfn(handle);
            }
        }
    }
    1
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_fls_alloc(_callback: *const core::ffi::c_void) -> u32 {
    unsafe extern "system" {
        fn TlsAlloc() -> u32;
    }
    unsafe { TlsAlloc() }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_fls_free(index: u32) -> i32 {
    unsafe extern "system" {
        fn TlsFree(dwTlsIndex: u32) -> i32;
    }
    unsafe { TlsFree(index) }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_fls_set_value(
    index: u32,
    lp_fls_data: *const core::ffi::c_void,
) -> i32 {
    unsafe extern "system" {
        fn TlsSetValue(dwTlsIndex: u32, lpTlsValue: *const core::ffi::c_void) -> i32;
    }
    unsafe { TlsSetValue(index, lp_fls_data) }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_fls_get_value(index: u32) -> *mut core::ffi::c_void {
    unsafe extern "system" {
        fn TlsGetValue(dwTlsIndex: u32) -> *mut core::ffi::c_void;
    }
    unsafe { TlsGetValue(index) }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_init_once_begin_initialize(
    init_once: *mut usize,
    _dw_flags: u32,
    f_pending: *mut i32,
    _lp_context: *mut *mut core::ffi::c_void,
) -> i32 {
    if init_once.is_null() {
        return 0;
    }
    use std::sync::atomic::{AtomicUsize, Ordering};
    let atom = unsafe { &*(init_once as *const AtomicUsize) };
    loop {
        let val = atom.load(Ordering::Acquire);
        if val == 2 {
            if !f_pending.is_null() {
                unsafe {
                    *f_pending = 0;
                }
            }
            return 1;
        }
        if val == 0 {
            if atom
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                if !f_pending.is_null() {
                    unsafe {
                        *f_pending = 1;
                    }
                }
                return 1;
            }
        } else {
            unsafe extern "system" {
                fn Sleep(ms: u32);
            }
            unsafe {
                Sleep(1);
            }
        }
    }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_init_once_complete(
    init_once: *mut usize,
    _dw_flags: u32,
    _lp_context: *const core::ffi::c_void,
) -> i32 {
    if init_once.is_null() {
        return 0;
    }
    use std::sync::atomic::{AtomicUsize, Ordering};
    let atom = unsafe { &*(init_once as *const AtomicUsize) };
    atom.store(2, Ordering::Release);
    1
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_is_thread_a_fiber() -> i32 {
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_get_module_handle_ex_w(
    _dw_flags: u32,
    lp_module_name: *const u16,
    ph_module: *mut *mut core::ffi::c_void,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleW(lpModuleName: *const u16) -> *mut core::ffi::c_void;
    }
    let h = unsafe { GetModuleHandleW(lp_module_name) };
    if !ph_module.is_null() {
        unsafe {
            *ph_module = h;
        }
    }
    if !h.is_null() { 1 } else { 0 }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_set_thread_stack_guarantee(_size: *mut u32) -> i32 {
    1
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_get_file_information_by_handle_ex(
    _h_file: *mut core::ffi::c_void,
    _info_class: u32,
    _lp_file_info: *mut core::ffi::c_void,
    _dw_buffer_size: u32,
) -> i32 {
    unsafe extern "system" {
        fn SetLastError(dwErrCode: u32);
    }
    unsafe {
        SetLastError(50);
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_set_file_information_by_handle(
    _h_file: *mut core::ffi::c_void,
    _info_class: u32,
    _lp_file_info: *mut core::ffi::c_void,
    _dw_buffer_size: u32,
) -> i32 {
    unsafe extern "system" {
        fn SetLastError(dwErrCode: u32);
    }
    unsafe {
        SetLastError(50);
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_rtl_capture_context(_context_record: *mut core::ffi::c_void) {}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_get_logical_processor_information_ex(
    _relationship: u32,
    _buffer: *mut core::ffi::c_void,
    _returned_length: *mut u32,
) -> i32 {
    unsafe extern "system" {
        fn SetLastError(dwErrCode: u32);
    }
    unsafe {
        SetLastError(50);
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_nt_read_file(
    file_handle: *mut core::ffi::c_void,
    _event: *mut core::ffi::c_void,
    _apc_routine: *mut core::ffi::c_void,
    _apc_context: *mut core::ffi::c_void,
    io_status_block: *mut usize,
    buffer: *mut core::ffi::c_void,
    length: u32,
    _byte_offset: *mut u64,
    _key: *mut u32,
) -> i32 {
    unsafe extern "system" {
        fn ReadFile(
            hFile: *mut core::ffi::c_void,
            lpBuffer: *mut core::ffi::c_void,
            nNumberOfBytesToRead: u32,
            lpNumberOfBytesRead: *mut u32,
            lpOverlapped: *mut core::ffi::c_void,
        ) -> bool;
    }
    let mut bytes_read: u32 = 0;
    let ok = unsafe {
        ReadFile(
            file_handle,
            buffer,
            length,
            &mut bytes_read,
            std::ptr::null_mut(),
        )
    };
    if !io_status_block.is_null() {
        unsafe {
            *io_status_block.add(1) = bytes_read as usize;
        }
    }
    if ok { 0 } else { -1073741823 }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_nt_write_file(
    file_handle: *mut core::ffi::c_void,
    _event: *mut core::ffi::c_void,
    _apc_routine: *mut core::ffi::c_void,
    _apc_context: *mut core::ffi::c_void,
    io_status_block: *mut usize,
    buffer: *const core::ffi::c_void,
    length: u32,
    _byte_offset: *mut u64,
    _key: *mut u32,
) -> i32 {
    unsafe extern "system" {
        fn WriteFile(
            hFile: *mut core::ffi::c_void,
            lpBuffer: *const core::ffi::c_void,
            nNumberOfBytesToWrite: u32,
            lpNumberOfBytesWritten: *mut u32,
            lpOverlapped: *mut core::ffi::c_void,
        ) -> bool;
    }
    let mut bytes_written: u32 = 0;
    let ok = unsafe {
        WriteFile(
            file_handle,
            buffer,
            length,
            &mut bytes_written,
            std::ptr::null_mut(),
        )
    };
    if !io_status_block.is_null() {
        unsafe {
            *io_status_block.add(1) = bytes_written as usize;
        }
    }
    if ok { 0 } else { -1073741823 }
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_rtl_nt_status_to_dos_error(status: i32) -> u32 {
    if status == 0 { 0 } else { 1 }
}

#[repr(C)]
struct RustidModuleEntry32A {
    dw_size: u32,
    th32_module_id: u32,
    th32_process_id: u32,
    glblcnt_usage: u32,
    proccnt_usage: u32,
    mod_base_addr: *mut u8,
    mod_base_size: u32,
    h_module: *mut core::ffi::c_void,
    sz_module: [u8; 256],
    sz_exe_path: [u8; 260],
}

#[repr(C)]
pub struct RustidModuleEntry32W {
    pub dw_size: u32,
    pub th32_module_id: u32,
    pub th32_process_id: u32,
    pub glblcnt_usage: u32,
    pub proccnt_usage: u32,
    pub mod_base_addr: *mut u8,
    pub mod_base_size: u32,
    pub h_module: *mut core::ffi::c_void,
    pub sz_module: [u16; 256],
    pub sz_exe_path: [u16; 260],
}

#[repr(C)]
struct RustidProcessEntry32A {
    dw_size: u32,
    cnt_usage: u32,
    th32_process_id: u32,
    th32_default_heap_id: usize,
    th32_module_id: u32,
    cnt_threads: u32,
    th32_parent_process_id: u32,
    pc_pri_class_base: i32,
    dw_flags: u32,
    sz_exe_file: [u8; 260],
}

#[repr(C)]
pub struct RustidProcessEntry32W {
    pub dw_size: u32,
    pub cnt_usage: u32,
    pub th32_process_id: u32,
    pub th32_default_heap_id: usize,
    pub th32_module_id: u32,
    pub cnt_threads: u32,
    pub th32_parent_process_id: u32,
    pub pc_pri_class_base: i32,
    pub dw_flags: u32,
    pub sz_exe_file: [u16; 260],
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_module32_first_w(
    h_snapshot: *mut core::ffi::c_void,
    lpme: *mut RustidModuleEntry32W,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn MultiByteToWideChar(
            code_page: u32,
            dw_flags: u32,
            lp_multi_byte_str: *const u8,
            cb_multi_byte: i32,
            lp_wide_char_str: *mut u16,
            cch_wide_char: i32,
        ) -> i32;
    }
    if lpme.is_null() {
        return 0;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p_w = GetProcAddress(k32, b"Module32FirstW\0".as_ptr());
            if !p_w.is_null() {
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidModuleEntry32W,
                ) -> i32 = std::mem::transmute(p_w);
                return pfn(h_snapshot, lpme);
            }
            let p_a = GetProcAddress(k32, b"Module32First\0".as_ptr());
            if !p_a.is_null() {
                let mut me_a: RustidModuleEntry32A = std::mem::zeroed();
                me_a.dw_size = std::mem::size_of::<RustidModuleEntry32A>() as u32;
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidModuleEntry32A,
                ) -> i32 = std::mem::transmute(p_a);
                if pfn(h_snapshot, &mut me_a) != 0 {
                    (*lpme).th32_module_id = me_a.th32_module_id;
                    (*lpme).th32_process_id = me_a.th32_process_id;
                    (*lpme).glblcnt_usage = me_a.glblcnt_usage;
                    (*lpme).proccnt_usage = me_a.proccnt_usage;
                    (*lpme).mod_base_addr = me_a.mod_base_addr;
                    (*lpme).mod_base_size = me_a.mod_base_size;
                    (*lpme).h_module = me_a.h_module;
                    MultiByteToWideChar(
                        0,
                        0,
                        me_a.sz_module.as_ptr(),
                        -1,
                        (*lpme).sz_module.as_mut_ptr(),
                        256,
                    );
                    MultiByteToWideChar(
                        0,
                        0,
                        me_a.sz_exe_path.as_ptr(),
                        -1,
                        (*lpme).sz_exe_path.as_mut_ptr(),
                        260,
                    );
                    return 1;
                }
            }
        }
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_module32_next_w(
    h_snapshot: *mut core::ffi::c_void,
    lpme: *mut RustidModuleEntry32W,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn MultiByteToWideChar(
            code_page: u32,
            dw_flags: u32,
            lp_multi_byte_str: *const u8,
            cb_multi_byte: i32,
            lp_wide_char_str: *mut u16,
            cch_wide_char: i32,
        ) -> i32;
    }
    if lpme.is_null() {
        return 0;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p_w = GetProcAddress(k32, b"Module32NextW\0".as_ptr());
            if !p_w.is_null() {
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidModuleEntry32W,
                ) -> i32 = std::mem::transmute(p_w);
                return pfn(h_snapshot, lpme);
            }
            let p_a = GetProcAddress(k32, b"Module32Next\0".as_ptr());
            if !p_a.is_null() {
                let mut me_a: RustidModuleEntry32A = std::mem::zeroed();
                me_a.dw_size = std::mem::size_of::<RustidModuleEntry32A>() as u32;
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidModuleEntry32A,
                ) -> i32 = std::mem::transmute(p_a);
                if pfn(h_snapshot, &mut me_a) != 0 {
                    (*lpme).th32_module_id = me_a.th32_module_id;
                    (*lpme).th32_process_id = me_a.th32_process_id;
                    (*lpme).glblcnt_usage = me_a.glblcnt_usage;
                    (*lpme).proccnt_usage = me_a.proccnt_usage;
                    (*lpme).mod_base_addr = me_a.mod_base_addr;
                    (*lpme).mod_base_size = me_a.mod_base_size;
                    (*lpme).h_module = me_a.h_module;
                    MultiByteToWideChar(
                        0,
                        0,
                        me_a.sz_module.as_ptr(),
                        -1,
                        (*lpme).sz_module.as_mut_ptr(),
                        256,
                    );
                    MultiByteToWideChar(
                        0,
                        0,
                        me_a.sz_exe_path.as_ptr(),
                        -1,
                        (*lpme).sz_exe_path.as_mut_ptr(),
                        260,
                    );
                    return 1;
                }
            }
        }
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_process32_first_w(
    h_snapshot: *mut core::ffi::c_void,
    lppe: *mut RustidProcessEntry32W,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn MultiByteToWideChar(
            code_page: u32,
            dw_flags: u32,
            lp_multi_byte_str: *const u8,
            cb_multi_byte: i32,
            lp_wide_char_str: *mut u16,
            cch_wide_char: i32,
        ) -> i32;
    }
    if lppe.is_null() {
        return 0;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p_w = GetProcAddress(k32, b"Process32FirstW\0".as_ptr());
            if !p_w.is_null() {
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidProcessEntry32W,
                ) -> i32 = std::mem::transmute(p_w);
                return pfn(h_snapshot, lppe);
            }
            let p_a = GetProcAddress(k32, b"Process32First\0".as_ptr());
            if !p_a.is_null() {
                let mut pe_a: RustidProcessEntry32A = std::mem::zeroed();
                pe_a.dw_size = std::mem::size_of::<RustidProcessEntry32A>() as u32;
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidProcessEntry32A,
                ) -> i32 = std::mem::transmute(p_a);
                if pfn(h_snapshot, &mut pe_a) != 0 {
                    (*lppe).cnt_usage = pe_a.cnt_usage;
                    (*lppe).th32_process_id = pe_a.th32_process_id;
                    (*lppe).th32_default_heap_id = pe_a.th32_default_heap_id;
                    (*lppe).th32_module_id = pe_a.th32_module_id;
                    (*lppe).cnt_threads = pe_a.cnt_threads;
                    (*lppe).th32_parent_process_id = pe_a.th32_parent_process_id;
                    (*lppe).pc_pri_class_base = pe_a.pc_pri_class_base;
                    (*lppe).dw_flags = pe_a.dw_flags;
                    MultiByteToWideChar(
                        0,
                        0,
                        pe_a.sz_exe_file.as_ptr(),
                        -1,
                        (*lppe).sz_exe_file.as_mut_ptr(),
                        260,
                    );
                    return 1;
                }
            }
        }
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_process32_next_w(
    h_snapshot: *mut core::ffi::c_void,
    lppe: *mut RustidProcessEntry32W,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn MultiByteToWideChar(
            code_page: u32,
            dw_flags: u32,
            lp_multi_byte_str: *const u8,
            cb_multi_byte: i32,
            lp_wide_char_str: *mut u16,
            cch_wide_char: i32,
        ) -> i32;
    }
    if lppe.is_null() {
        return 0;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p_w = GetProcAddress(k32, b"Process32NextW\0".as_ptr());
            if !p_w.is_null() {
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidProcessEntry32W,
                ) -> i32 = std::mem::transmute(p_w);
                return pfn(h_snapshot, lppe);
            }
            let p_a = GetProcAddress(k32, b"Process32Next\0".as_ptr());
            if !p_a.is_null() {
                let mut pe_a: RustidProcessEntry32A = std::mem::zeroed();
                pe_a.dw_size = std::mem::size_of::<RustidProcessEntry32A>() as u32;
                let pfn: extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut RustidProcessEntry32A,
                ) -> i32 = std::mem::transmute(p_a);
                if pfn(h_snapshot, &mut pe_a) != 0 {
                    (*lppe).cnt_usage = pe_a.cnt_usage;
                    (*lppe).th32_process_id = pe_a.th32_process_id;
                    (*lppe).th32_default_heap_id = pe_a.th32_default_heap_id;
                    (*lppe).th32_module_id = pe_a.th32_module_id;
                    (*lppe).cnt_threads = pe_a.cnt_threads;
                    (*lppe).th32_parent_process_id = pe_a.th32_parent_process_id;
                    (*lppe).pc_pri_class_base = pe_a.pc_pri_class_base;
                    (*lppe).dw_flags = pe_a.dw_flags;
                    MultiByteToWideChar(
                        0,
                        0,
                        pe_a.sz_exe_file.as_ptr(),
                        -1,
                        (*lppe).sz_exe_file.as_mut_ptr(),
                        260,
                    );
                    return 1;
                }
            }
        }
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_is_debugger_present() -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p = GetProcAddress(k32, b"IsDebuggerPresent\0".as_ptr());
            if !p.is_null() {
                let pfn: extern "system" fn() -> i32 = std::mem::transmute(p);
                return pfn();
            }
        }
    }
    0
}

#[cfg(all(target_os = "windows", target_arch = "x86", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn rustid_try_enter_critical_section(
    lp_critical_section: *mut core::ffi::c_void,
) -> i32 {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut core::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut core::ffi::c_void;
        fn EnterCriticalSection(lpCriticalSection: *mut core::ffi::c_void);
    }
    unsafe {
        let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if !k32.is_null() {
            let p = GetProcAddress(k32, b"TryEnterCriticalSection\0".as_ptr());
            if !p.is_null() {
                let pfn: extern "system" fn(*mut core::ffi::c_void) -> i32 = std::mem::transmute(p);
                return pfn(lp_critical_section);
            }
        }
        EnterCriticalSection(lp_critical_section);
        1
    }
}
