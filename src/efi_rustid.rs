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
    rustid::x86::efi::clear_screen_black();

    use rustid::common::{CliFlags, TCpuDisplay, TDetect};
    use rustid::{Cpu, version};

    version();
    rustid::x86::efi::print_firmware_header();
    let cpu = Cpu::detect();
    let flags = CliFlags {
        color: true,
        ..Default::default()
    };
    cpu.display_table(flags);

    let is_vm = rustid::x86::is_hypervisor_guest();
    if !is_vm {
        rustid::x86::efi::wait_for_keypress(None);
    }

    0
}

#[cfg(not(target_os = "uefi"))]
pub fn main() {}
