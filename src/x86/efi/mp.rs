#![cfg(target_os = "uefi")]
//! EFI MP (MultiProcessor) Services Protocol implementation for core enumeration and targeted execution.

use core::ffi::c_void;

use super::os::{EFI_SUCCESS, EfiGuid, EfiStatus, get_system_table, locate_protocol_compat};

/// UEFI PI 1.2+ MP Services Protocol GUID: `{3fdda605-a76e-4f46-ad29-12f4531b3d08}`
pub const EFI_MP_SERVICES_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x3fdda605,
    b: 0xa76e,
    c: 0x4f46,
    d: [0xad, 0x29, 0x12, 0xf4, 0x53, 0x1b, 0x3d, 0x08],
};

/// Framework / Legacy EFI 1.10 MP Services Protocol GUID: `{3fdda604-8630-43c3-9802-be90f0aa52fb}`
pub const FRAMEWORK_EFI_MP_SERVICES_PROTOCOL_GUID: EfiGuid = EfiGuid {
    a: 0x3fdda604,
    b: 0x8630,
    c: 0x43c3,
    d: [0x98, 0x02, 0xbe, 0x90, 0xf0, 0xaa, 0x52, 0xfb],
};

pub const PROCESSOR_AS_BSP_BIT: u32 = 0x00000001;
pub const PROCESSOR_ENABLED_BIT: u32 = 0x00000002;
pub const PROCESSOR_HEALTH_STATUS_BIT: u32 = 0x00000004;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct EfiCpuPhysicalLocation {
    pub package: u32,
    pub core: u32,
    pub thread: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct EfiProcessorInformation {
    pub processor_id: u64,
    pub status_flag: u32,
    pub location: EfiCpuPhysicalLocation,
    pub extended_information: [u32; 6],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct FrameworkProcessorContext {
    pub processor_id: u64,
    pub status_flag: u8,
    pub reserved: [u8; 3],
    pub location: EfiCpuPhysicalLocation,
    pub health_flags: u32,
}

pub type EfiApProcedure = unsafe extern "efiapi" fn(*mut c_void);

#[repr(C)]
pub struct EfiMpServicesProtocol {
    pub get_number_of_processors:
        unsafe extern "efiapi" fn(*mut EfiMpServicesProtocol, *mut usize, *mut usize) -> EfiStatus,
    pub get_processor_info: unsafe extern "efiapi" fn(
        *mut EfiMpServicesProtocol,
        usize,
        *mut EfiProcessorInformation,
    ) -> EfiStatus,
    pub startup_all_aps: unsafe extern "efiapi" fn(
        *mut EfiMpServicesProtocol,
        EfiApProcedure,
        bool,
        *mut c_void,
        usize,
        *mut c_void,
        *mut *mut usize,
    ) -> EfiStatus,
    pub startup_this_ap: unsafe extern "efiapi" fn(
        *mut EfiMpServicesProtocol,
        EfiApProcedure,
        usize,
        *mut c_void,
        usize,
        *mut c_void,
        *mut bool,
    ) -> EfiStatus,
    pub switch_bsp: *const c_void,
    pub enable_disable_ap: *const c_void,
    pub who_am_i: unsafe extern "efiapi" fn(*mut EfiMpServicesProtocol, *mut usize) -> EfiStatus,
}

#[repr(C)]
pub struct FrameworkMpServicesProtocol {
    pub get_number_of_processors: unsafe extern "efiapi" fn(
        *mut FrameworkMpServicesProtocol,
        *mut usize,
        *mut usize,
    ) -> EfiStatus,
    pub get_processor_context: unsafe extern "efiapi" fn(
        *mut FrameworkMpServicesProtocol,
        usize,
        *mut usize,
        *mut FrameworkProcessorContext,
    ) -> EfiStatus,
    pub startup_all_aps: unsafe extern "efiapi" fn(
        *mut FrameworkMpServicesProtocol,
        EfiApProcedure,
        bool,
        *mut c_void,
        usize,
        *mut c_void,
        *mut *mut usize,
    ) -> EfiStatus,
    pub startup_this_ap: unsafe extern "efiapi" fn(
        *mut FrameworkMpServicesProtocol,
        EfiApProcedure,
        usize,
        *mut c_void,
        usize,
        *mut c_void,
        *mut bool,
    ) -> EfiStatus,
    pub switch_bsp: *const c_void,
    pub enable_disable_ap: *const c_void,
    pub who_am_i:
        unsafe extern "efiapi" fn(*mut FrameworkMpServicesProtocol, *mut usize) -> EfiStatus,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct EfiProcessorInfo {
    pub index: usize,
    pub processor_id: u64,
    pub is_bsp: bool,
    pub is_enabled: bool,
    pub is_healthy: bool,
    pub location: EfiCpuPhysicalLocation,
}

#[derive(Copy, Clone)]
pub enum EfiMpKind {
    Pi(*mut EfiMpServicesProtocol),
    Framework(*mut FrameworkMpServicesProtocol),
}

#[derive(Copy, Clone)]
pub struct EfiMpServices {
    kind: EfiMpKind,
}

struct ApClosureContext<'a> {
    func: &'a mut dyn FnMut(),
}

unsafe extern "efiapi" fn ap_procedure_thunk(arg: *mut c_void) {
    if !arg.is_null() {
        let ctx = unsafe { &mut *(arg as *mut ApClosureContext) };
        (ctx.func)();
    }
}

impl EfiMpServices {
    /// Discovers and connects to the active EFI MP Services Protocol.
    pub fn detect() -> Option<Self> {
        let st = get_system_table();
        if st.is_null() {
            return None;
        }

        // 1. Try UEFI PI 1.2+ MP Services Protocol
        let pi_ptr = unsafe { locate_protocol_compat(&EFI_MP_SERVICES_PROTOCOL_GUID) }
            as *mut EfiMpServicesProtocol;
        if !pi_ptr.is_null() {
            return Some(Self {
                kind: EfiMpKind::Pi(pi_ptr),
            });
        }

        // 2. Fall back to Framework / EFI 1.10 MP Services Protocol
        let fw_ptr = unsafe { locate_protocol_compat(&FRAMEWORK_EFI_MP_SERVICES_PROTOCOL_GUID) }
            as *mut FrameworkMpServicesProtocol;
        if !fw_ptr.is_null() {
            return Some(Self {
                kind: EfiMpKind::Framework(fw_ptr),
            });
        }

        None
    }

    /// Returns the total number of logical processors and enabled processors.
    pub fn processor_counts(&self) -> (usize, usize) {
        let mut total = 1usize;
        let mut enabled = 1usize;
        let status = unsafe {
            match self.kind {
                EfiMpKind::Pi(proto) => {
                    ((*proto).get_number_of_processors)(proto, &mut total, &mut enabled)
                }
                EfiMpKind::Framework(proto) => {
                    ((*proto).get_number_of_processors)(proto, &mut total, &mut enabled)
                }
            }
        };
        if status == EFI_SUCCESS {
            (total, enabled)
        } else {
            (1, 1)
        }
    }

    /// Returns the total number of logical processors available.
    pub fn processor_count(&self) -> usize {
        self.processor_counts().0
    }

    /// Retrieves detailed information for a specific logical processor by index.
    pub fn get_processor_info(&self, index: usize) -> Option<EfiProcessorInfo> {
        match self.kind {
            EfiMpKind::Pi(proto) => {
                let mut raw_info = EfiProcessorInformation::default();
                let status = unsafe { ((*proto).get_processor_info)(proto, index, &mut raw_info) };
                if status == EFI_SUCCESS {
                    Some(EfiProcessorInfo {
                        index,
                        processor_id: raw_info.processor_id,
                        is_bsp: (raw_info.status_flag & PROCESSOR_AS_BSP_BIT) != 0,
                        is_enabled: (raw_info.status_flag & PROCESSOR_ENABLED_BIT) != 0,
                        is_healthy: (raw_info.status_flag & PROCESSOR_HEALTH_STATUS_BIT) != 0,
                        location: raw_info.location,
                    })
                } else {
                    None
                }
            }
            EfiMpKind::Framework(proto) => {
                let mut ctx = FrameworkProcessorContext::default();
                let mut buf_len = core::mem::size_of::<FrameworkProcessorContext>();
                let status = unsafe {
                    ((*proto).get_processor_context)(proto, index, &mut buf_len, &mut ctx)
                };
                if status == EFI_SUCCESS {
                    Some(EfiProcessorInfo {
                        index,
                        processor_id: ctx.processor_id,
                        is_bsp: ctx.status_flag != 0,
                        is_enabled: true,
                        is_healthy: (ctx.health_flags & 0x04) != 0 || ctx.health_flags != 0,
                        location: ctx.location,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Returns the index of the logical processor currently executing.
    pub fn who_am_i(&self) -> usize {
        let mut index = 0usize;
        let status = unsafe {
            match self.kind {
                EfiMpKind::Pi(proto) => ((*proto).who_am_i)(proto, &mut index),
                EfiMpKind::Framework(proto) => ((*proto).who_am_i)(proto, &mut index),
            }
        };
        if status == EFI_SUCCESS { index } else { 0 }
    }

    /// Calculates the number of unique physical sockets/packages in the system.
    pub fn socket_count(&self) -> usize {
        let count = self.processor_count();
        let mut max_package = 0u32;
        let mut found_any = false;

        for i in 0..count {
            if let Some(info) = self.get_processor_info(i) {
                if info.location.package > max_package {
                    max_package = info.location.package;
                }
                found_any = true;
            }
        }

        if found_any {
            (max_package as usize) + 1
        } else {
            1
        }
    }

    /// Executes a closure on a specific logical processor `proc_idx`.
    ///
    /// If `proc_idx` is the current processor (BSP), the closure is executed immediately.
    /// If `proc_idx` is an AP, execution is dispatched via `StartupThisAP` with a blocking timeout.
    pub fn run_on_processor<F: FnOnce()>(&self, proc_idx: usize, f: F) {
        let current = self.who_am_i();
        if proc_idx == current {
            f();
            return;
        }

        let mut opt_f = Some(f);
        let mut wrapper = || {
            if let Some(func) = opt_f.take() {
                func();
            }
        };
        let mut ctx = ApClosureContext { func: &mut wrapper };
        let mut finished = false;
        let timeout_us = 2_000_000usize; // 2-second timeout

        let _ = unsafe {
            match self.kind {
                EfiMpKind::Pi(proto) => ((*proto).startup_this_ap)(
                    proto,
                    ap_procedure_thunk,
                    proc_idx,
                    core::ptr::null_mut(), // Synchronous blocking execution (NULL event)
                    timeout_us,
                    &mut ctx as *mut _ as *mut c_void,
                    &mut finished,
                ),
                EfiMpKind::Framework(proto) => ((*proto).startup_this_ap)(
                    proto,
                    ap_procedure_thunk,
                    proc_idx,
                    core::ptr::null_mut(), // Synchronous blocking execution (NULL event)
                    timeout_us,
                    &mut ctx as *mut _ as *mut c_void,
                    &mut finished,
                ),
            }
        };
    }
}
