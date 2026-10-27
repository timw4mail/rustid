#![cfg(uefi)]
//! Zero-dependency SMBIOS 2.x and 3.x parser for UEFI environment.

pub use crate::common::os::smbios::*;

#[cfg(uefi)]
use super::os::{EfiConfigurationTable, get_system_table};

/// SMBIOS 2.x 32-bit Table GUID: `{eb9d2d31-2d88-11d3-9a16-0090273fc14d}`
pub const SMBIOS_TABLE_GUID: super::os::EfiGuid = super::os::EfiGuid {
    a: 0xeb9d2d31,
    b: 0x2d88,
    c: 0x11d3,
    d: [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
};

/// SMBIOS 3.x 64-bit Table GUID: `{f2fd1544-9794-4a2c-992e-e5bbcf20e394}`
pub const SMBIOS3_TABLE_GUID: super::os::EfiGuid = super::os::EfiGuid {
    a: 0xf2fd1544,
    b: 0x9794,
    c: 0x4a2c,
    d: [0x99, 0x2e, 0xe5, 0xbb, 0xcf, 0x20, 0xe3, 0x94],
};

/// Locates and parses the SMBIOS table in a live UEFI environment.
#[cfg(uefi)]
pub fn detect_smbios() -> Option<SmbiosData> {
    let st = get_system_table();
    if st.is_null() {
        return None;
    }

    let entries_count = unsafe { (*st).number_of_table_entries };
    let config_tables = unsafe { (*st).configuration_table as *const EfiConfigurationTable };

    if config_tables.is_null() || entries_count == 0 {
        return None;
    }

    let find_table = |guid: super::os::EfiGuid,
                      get_addr_len: fn(*const u8) -> (u64, usize)|
     -> Option<SmbiosData> {
        for i in 0..entries_count {
            let entry = unsafe { *config_tables.add(i) };
            if entry.vendor_guid == guid && !entry.vendor_table.is_null() {
                let (table_addr, table_len) = get_addr_len(entry.vendor_table as *const u8);
                if table_addr != 0 && table_len > 0 && table_len <= 0x100000 {
                    let slice =
                        unsafe { core::slice::from_raw_parts(table_addr as *const u8, table_len) };
                    return Some(SmbiosData::parse(slice));
                }
            }
        }
        None
    };

    // 1. Try SMBIOS 3.x (64-bit table) first
    if let Some(data) = find_table(SMBIOS3_TABLE_GUID, |ptr| {
        let ep = unsafe { *(ptr as *const Smbios3EntryPoint) };
        if &ep.anchor == b"_SM3_" {
            (ep.table_address, ep.table_max_size as usize)
        } else {
            (0, 0)
        }
    }) {
        return Some(data);
    }

    // 2. Fall back to SMBIOS 2.x (32-bit table)
    find_table(SMBIOS_TABLE_GUID, |ptr| {
        let ep = unsafe { *(ptr as *const Smbios2EntryPoint) };
        if &ep.anchor == b"_SM_" {
            (ep.table_address as u64, ep.table_length as usize)
        } else {
            (0, 0)
        }
    })
}

/// Discovers the system name from SMBIOS in UEFI.
pub fn detect_smbios_system_name() -> Option<crate::common::SystemInfo> {
    #[cfg(uefi)]
    if let Some(smbios) = detect_smbios() {
        return smbios.get_system_name();
    }

    None
}
