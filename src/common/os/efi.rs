#![cfg(target_os = "uefi")]

use crate::common::{OS, TOSData, TopologyTier};
use alloc::string::String;

impl TOSData for OS {
    fn get_system_name() -> Option<String> {
        crate::x86::efi::smbios::detect_smbios_system_name()
    }

    fn get_socket_count() -> TopologyTier {
        crate::x86::count::get_platform_socket_count()
    }
}
