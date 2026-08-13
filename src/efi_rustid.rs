#![cfg_attr(all(not(test), target_os = "uefi"), no_std)]
#![cfg_attr(all(not(test), target_os = "uefi"), no_main)]

#[cfg(target_os = "uefi")]
extern crate alloc;

#[cfg(target_os = "uefi")]
#[unsafe(no_mangle)]
pub unsafe extern "efiapi" fn efi_main(
    image_handle: *mut core::ffi::c_void,
    system_table: *mut rustid::x86::efi::EfiSystemTable,
) -> usize {
    unsafe { rustid::x86::efi::init_efi(image_handle, system_table) };

    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::{Cpu, version};

    version();
    let cpu = Cpu::detect();
    let flags = CliFlags::default();
    cpu.display_table(flags);

    rustid::x86::efi::exit(0)
}

#[cfg(not(target_os = "uefi"))]
pub fn main() {}
