#![cfg(target_os = "uefi")]
//! Zero-dependency SMBIOS 2.x and 3.x parser for UEFI environment.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::common::os::{is_generic_value, is_known_hypervisor_vendor};

#[cfg(target_os = "uefi")]
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

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Smbios2EntryPoint {
    pub anchor: [u8; 4], // "_SM_"
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub max_structure_size: u16,
    pub entry_point_revision: u8,
    pub formatted_area: [u8; 5],
    pub dmi_anchor: [u8; 5], // "_DMI_"
    pub dmi_checksum: u8,
    pub table_length: u16,
    pub table_address: u32,
    pub number_of_structures: u16,
    pub bcd_revision: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Smbios3EntryPoint {
    pub anchor: [u8; 5], // "_SM3_"
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub doc_rev: u8,
    pub entry_point_revision: u8,
    pub reserved: u8,
    pub table_max_size: u32,
    pub table_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SmbiosStructureHeader {
    pub type_: u8,
    pub length: u8,
    pub handle: u16,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosSystemInfo {
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub version: Option<String>,
    pub family: Option<String>,
    pub sku_number: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosBoardInfo {
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosChassisInfo {
    pub manufacturer: Option<String>,
    pub chassis_type: u8,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosProcessorInfo {
    pub socket_designation: Option<String>,
    pub processor_type: u8,
    pub processor_family: u8,
    pub manufacturer: Option<String>,
    pub version: Option<String>,
    pub voltage: u8,
    pub external_clock_mhz: u16,
    pub max_speed_mhz: u16,
    pub current_speed_mhz: u16,
    pub status: u8,
    pub is_populated: bool,
    pub is_enabled: bool,
    pub core_count: u32,
    pub core_enabled: u32,
    pub thread_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosBiosInfo {
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub release_date: Option<String>,
}

pub struct SmbiosRawStructure<'a> {
    pub header: SmbiosStructureHeader,
    pub formatted: &'a [u8],
    pub string_table: &'a [u8],
}

impl<'a> SmbiosRawStructure<'a> {
    /// Extracts a 1-based indexed string from the structure's string table.
    pub fn get_string(&self, idx: u8) -> Option<String> {
        if idx == 0 {
            return None;
        }

        let mut current_idx = 1u8;
        let mut start = 0;

        while start < self.string_table.len() {
            if let Some(end_rel) = self.string_table[start..].iter().position(|&b| b == 0) {
                let end = start + end_rel;
                if current_idx == idx {
                    let s_bytes = &self.string_table[start..end];
                    if let Ok(s) = core::str::from_utf8(s_bytes) {
                        let trimmed = s.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                    return None;
                }
                current_idx += 1;
                start = end + 1;
            } else {
                break;
            }
        }

        None
    }
}

pub struct SmbiosTableParser<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> SmbiosTableParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for SmbiosTableParser<'a> {
    type Item = SmbiosRawStructure<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 4 > self.data.len() {
            return None;
        }

        let type_ = self.data[self.offset];
        let length = self.data[self.offset + 1];
        let handle = u16::from_le_bytes([self.data[self.offset + 2], self.data[self.offset + 3]]);

        if length < 4 || self.offset + (length as usize) > self.data.len() {
            return None;
        }

        // Structure Type 127 indicates End of Table
        if type_ == 127 {
            return None;
        }

        let header = SmbiosStructureHeader {
            type_,
            length,
            handle,
        };

        let formatted = &self.data[self.offset..self.offset + (length as usize)];
        let strings_start = self.offset + (length as usize);

        // Find double-null termination of string area
        let mut strings_end = strings_start;
        while strings_end + 1 < self.data.len() {
            if self.data[strings_end] == 0 && self.data[strings_end + 1] == 0 {
                strings_end += 2;
                break;
            }
            strings_end += 1;
        }

        if strings_end > self.data.len() {
            strings_end = self.data.len();
        }

        let string_table = if strings_end > strings_start {
            &self.data[strings_start..strings_end]
        } else {
            &[]
        };

        self.offset = strings_end;

        Some(SmbiosRawStructure {
            header,
            formatted,
            string_table,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosData {
    pub system: Option<SmbiosSystemInfo>,
    pub board: Option<SmbiosBoardInfo>,
    pub chassis: Option<SmbiosChassisInfo>,
    pub bios: Option<SmbiosBiosInfo>,
    pub processors: Vec<SmbiosProcessorInfo>,
}

impl SmbiosData {
    /// Parses SMBIOS table data from a raw byte slice.
    pub fn parse(table_data: &[u8]) -> Self {
        let mut data = SmbiosData::default();
        let parser = SmbiosTableParser::new(table_data);

        for s in parser {
            match s.header.type_ {
                0 => {
                    // Type 0: BIOS Information
                    if s.formatted.len() >= 0x09 {
                        let vendor = s.get_string(s.formatted[0x04]);
                        let version = s.get_string(s.formatted[0x05]);
                        let release_date = s.get_string(s.formatted[0x08]);
                        data.bios = Some(SmbiosBiosInfo {
                            vendor,
                            version,
                            release_date,
                        });
                    }
                }
                1 => {
                    // Type 1: System Information
                    if s.formatted.len() >= 0x08 {
                        let manufacturer = s.get_string(s.formatted[0x04]);
                        let product_name = s.get_string(s.formatted[0x05]);
                        let version = s.get_string(s.formatted[0x06]);
                        let sku_number = if s.formatted.len() >= 0x1A {
                            s.get_string(s.formatted[0x19])
                        } else {
                            None
                        };
                        let family = if s.formatted.len() >= 0x1B {
                            s.get_string(s.formatted[0x1A])
                        } else {
                            None
                        };

                        data.system = Some(SmbiosSystemInfo {
                            manufacturer,
                            product_name,
                            version,
                            family,
                            sku_number,
                        });
                    }
                }
                2 => {
                    // Type 2: Baseboard (Motherboard) Information
                    if s.formatted.len() >= 0x07 {
                        let manufacturer = s.get_string(s.formatted[0x04]);
                        let product_name = s.get_string(s.formatted[0x05]);
                        let version = s.get_string(s.formatted[0x06]);

                        data.board = Some(SmbiosBoardInfo {
                            manufacturer,
                            product_name,
                            version,
                        });
                    }
                }
                3 => {
                    // Type 3: System Enclosure or Chassis
                    if s.formatted.len() >= 0x06 {
                        let manufacturer = s.get_string(s.formatted[0x04]);
                        let raw_type = s.formatted[0x05];
                        let chassis_type = raw_type & 0x7F;
                        let version = s.get_string(s.formatted[0x06]);

                        data.chassis = Some(SmbiosChassisInfo {
                            manufacturer,
                            chassis_type,
                            version,
                        });
                    }
                }
                4 => {
                    // Type 4: Processor Information
                    if s.formatted.len() >= 0x1A {
                        let socket_designation = s.get_string(s.formatted[0x04]);
                        let processor_type = s.formatted[0x05];
                        let processor_family = s.formatted[0x06];
                        let manufacturer = s.get_string(s.formatted[0x07]);
                        let version = s.get_string(s.formatted[0x10]);
                        let voltage = s.formatted[0x11];
                        let external_clock_mhz =
                            u16::from_le_bytes([s.formatted[0x12], s.formatted[0x13]]);
                        let max_speed_mhz =
                            u16::from_le_bytes([s.formatted[0x14], s.formatted[0x15]]);
                        let current_speed_mhz =
                            u16::from_le_bytes([s.formatted[0x16], s.formatted[0x17]]);
                        let status = s.formatted[0x18];
                        let is_populated = (status & 0x40) != 0;
                        let cpu_status = status & 0x07;
                        let is_enabled = cpu_status == 1 || cpu_status == 0;

                        let mut core_count = if s.formatted.len() >= 0x24 {
                            s.formatted[0x23] as u32
                        } else {
                            0
                        };
                        let mut core_enabled = if s.formatted.len() >= 0x25 {
                            s.formatted[0x24] as u32
                        } else {
                            0
                        };
                        let mut thread_count = if s.formatted.len() >= 0x26 {
                            s.formatted[0x25] as u32
                        } else {
                            0
                        };

                        if s.formatted.len() >= 0x30 {
                            if core_count == 0xFF {
                                core_count =
                                    u16::from_le_bytes([s.formatted[0x2A], s.formatted[0x2B]])
                                        as u32;
                            }
                            if core_enabled == 0xFF {
                                core_enabled =
                                    u16::from_le_bytes([s.formatted[0x2C], s.formatted[0x2D]])
                                        as u32;
                            }
                            if thread_count == 0xFF {
                                thread_count =
                                    u16::from_le_bytes([s.formatted[0x2E], s.formatted[0x2F]])
                                        as u32;
                            }
                        }

                        data.processors.push(SmbiosProcessorInfo {
                            socket_designation,
                            processor_type,
                            processor_family,
                            manufacturer,
                            version,
                            voltage,
                            external_clock_mhz,
                            max_speed_mhz,
                            current_speed_mhz,
                            status,
                            is_populated,
                            is_enabled,
                            core_count,
                            core_enabled,
                            thread_count,
                        });
                    }
                }
                _ => {}
            }
        }

        data
    }

    /// Checks if the machine is a laptop / notebook based on chassis type or product name.
    pub fn is_laptop(&self) -> bool {
        if let Some(chassis) = &self.chassis {
            match chassis.chassis_type {
                0x08 | 0x09 | 0x0A | 0x0B | 0x0E | 0x1E | 0x1F | 0x20 => return true,
                _ => {}
            }
        }
        if let Some(sys) = &self.system {
            if let Some(prod) = &sys.product_name {
                let p = prod.to_ascii_lowercase();
                if p.contains("laptop")
                    || p.contains("notebook")
                    || p.contains("macbook")
                    || p.contains("thinkpad")
                    || p.contains("elitebook")
                    || p.contains("latitude")
                    || p.contains("inspiron")
                    || p.contains("zenbook")
                {
                    return true;
                }
            }
            if let Some(fam) = &sys.family {
                let f = fam.to_ascii_lowercase();
                if f.contains("laptop")
                    || f.contains("notebook")
                    || f.contains("macbook")
                    || f.contains("thinkpad")
                    || f.contains("elitebook")
                    || f.contains("latitude")
                    || f.contains("inspiron")
                    || f.contains("zenbook")
                {
                    return true;
                }
            }
        }
        false
    }

    /// Determines the best human-readable system name matching Linux/macOS heuristics.
    pub fn get_system_name(&self) -> Option<String> {
        // 1. Check Type 1 Product Family / Product Name
        if let Some(sys) = &self.system {
            if let Some(prod) = &sys.product_name {
                if !is_generic_value(prod) {
                    // For known hypervisors, fold in vendor (e.g., "QEMU Standard PC...")
                    if let Some(mfg) = &sys.manufacturer {
                        if is_known_hypervisor_vendor(mfg)
                            && !prod
                                .to_ascii_lowercase()
                                .contains(&mfg.to_ascii_lowercase())
                        {
                            return Some(alloc::format!("{mfg} {prod}"));
                        }
                    }
                    return Some(prod.clone());
                }
            }

            if let Some(fam) = &sys.family {
                if !is_generic_value(fam) {
                    return Some(fam.clone());
                }
            }

            // Hypervisor vendor only
            if let Some(mfg) = &sys.manufacturer {
                if is_known_hypervisor_vendor(mfg) {
                    if let Some(prod) = &sys.product_name {
                        return Some(alloc::format!("{mfg} {prod}"));
                    }
                    return Some(mfg.clone());
                }
            }
        }

        // 2. Fallback to Type 2 Baseboard Vendor + Product (e.g. ASUS/Gigabyte white-box desktop)
        if let Some(board) = &self.board {
            if let (Some(mfg), Some(prod)) = (&board.manufacturer, &board.product_name) {
                if !is_generic_value(prod) {
                    if prod
                        .to_ascii_lowercase()
                        .contains(&mfg.to_ascii_lowercase())
                    {
                        return Some(prod.clone());
                    } else {
                        return Some(alloc::format!("{mfg} {prod}"));
                    }
                }
            } else if let Some(prod) = &board.product_name {
                if !is_generic_value(prod) {
                    return Some(prod.clone());
                }
            }
        }

        None
    }
}

/// Locates and parses the SMBIOS table in a live UEFI environment.
#[cfg(target_os = "uefi")]
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

    // 1. Try SMBIOS 3.x (64-bit table) first
    for i in 0..entries_count {
        let entry = unsafe { *config_tables.add(i) };
        if entry.vendor_guid == SMBIOS3_TABLE_GUID && !entry.vendor_table.is_null() {
            let ep_ptr = entry.vendor_table as *const Smbios3EntryPoint;
            let anchor = unsafe { (*ep_ptr).anchor };
            if &anchor == b"_SM3_" {
                let table_addr = unsafe { (*ep_ptr).table_address };
                let table_len = unsafe { (*ep_ptr).table_max_size } as usize;
                if table_addr != 0 && table_len > 0 && table_len <= 0x100000 {
                    let slice =
                        unsafe { core::slice::from_raw_parts(table_addr as *const u8, table_len) };
                    return Some(SmbiosData::parse(slice));
                }
            }
        }
    }

    // 2. Fall back to SMBIOS 2.x (32-bit table)
    for i in 0..entries_count {
        let entry = unsafe { *config_tables.add(i) };
        if entry.vendor_guid == SMBIOS_TABLE_GUID && !entry.vendor_table.is_null() {
            let ep_ptr = entry.vendor_table as *const Smbios2EntryPoint;
            let anchor = unsafe { (*ep_ptr).anchor };
            if &anchor == b"_SM_" {
                let table_addr = unsafe { (*ep_ptr).table_address };
                let table_len = unsafe { (*ep_ptr).table_length } as usize;
                if table_addr != 0 && table_len > 0 && table_len <= 0x100000 {
                    let slice =
                        unsafe { core::slice::from_raw_parts(table_addr as *const u8, table_len) };
                    return Some(SmbiosData::parse(slice));
                }
            }
        }
    }

    None
}

/// Discovers the system name from SMBIOS in UEFI.
pub fn detect_smbios_system_name() -> Option<String> {
    #[cfg(target_os = "uefi")]
    if let Some(smbios) = detect_smbios() {
        return smbios.get_system_name();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smbios_type_1_apple_parse() {
        // Construct a synthetic SMBIOS Type 1 table for MacBookPro18,3
        let mut table = Vec::new();
        // Type 1 header: type=1, len=27, handle=1
        table.extend_from_slice(&[1, 27, 1, 0]);
        // Formatted area: mfg=1, prod=2, ver=3, sn=4, uuid=[0;16], wake=0, sku=0, family=5
        table.push(1); // 0x04: manufacturer string 1 ("Apple Inc.")
        table.push(2); // 0x05: product name string 2 ("MacBookPro18,3")
        table.push(3); // 0x06: version string 3 ("1.0")
        table.push(4); // 0x07: serial number string 4
        table.extend_from_slice(&[0u8; 16]); // 0x08..0x17 UUID
        table.push(3); // 0x18: Wakeup type
        table.push(0); // 0x19: SKU
        table.push(5); // 0x1A: Family ("MacBook Pro")

        // Strings
        table.extend_from_slice(b"Apple Inc.\0");
        table.extend_from_slice(b"MacBookPro18,3\0");
        table.extend_from_slice(b"1.0\0");
        table.extend_from_slice(b"C02XXXXX\0");
        table.extend_from_slice(b"MacBook Pro\0");
        table.push(0); // End of strings (double null)

        // Type 127 end of table
        table.extend_from_slice(&[127, 4, 2, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert!(smbios.system.is_some());
        let sys = smbios.system.as_ref().unwrap();
        assert_eq!(sys.manufacturer.as_deref(), Some("Apple Inc."));
        assert_eq!(sys.product_name.as_deref(), Some("MacBookPro18,3"));
        assert_eq!(sys.family.as_deref(), Some("MacBook Pro"));

        let name = smbios.get_system_name();
        assert_eq!(name.as_deref(), Some("MacBookPro18,3"));
    }

    #[test]
    fn test_smbios_type_2_motherboard_fallback() {
        // Construct a synthetic SMBIOS Type 1 with generic product, Type 2 with ASUS motherboard
        let mut table = Vec::new();
        // Type 1: System Product Name (generic)
        table.extend_from_slice(&[1, 8, 1, 0, 1, 2, 0, 0]);
        table.extend_from_slice(b"System manufacturer\0");
        table.extend_from_slice(b"System Product Name\0");
        table.push(0);

        // Type 2: Baseboard info
        table.extend_from_slice(&[2, 8, 2, 0, 1, 2, 3, 0]);
        table.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
        table.extend_from_slice(b"ROG STRIX Z790-E GAMING WIFI\0");
        table.extend_from_slice(b"Rev 1.xx\0");
        table.push(0);

        // Type 127
        table.extend_from_slice(&[127, 4, 3, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        let name = smbios.get_system_name();
        assert_eq!(
            name.as_deref(),
            Some("ASUSTeK COMPUTER INC. ROG STRIX Z790-E GAMING WIFI")
        );
    }

    #[test]
    fn test_smbios_type_4_processor_info() {
        let mut table = Vec::new();
        // Type 4: Processor Information (length 0x30 for full SMBIOS 3.x)
        let mut type4 = vec![0u8; 0x30];
        type4[0] = 4; // type
        type4[1] = 0x30; // length
        type4[4] = 1; // socket designation string 1
        type4[5] = 3; // central processor
        type4[6] = 0xBE; // Core i7
        type4[7] = 2; // manufacturer string 2
        type4[0x10] = 3; // version string 3
        type4[0x12..0x14].copy_from_slice(&100u16.to_le_bytes()); // 100 MHz external clock
        type4[0x14..0x16].copy_from_slice(&5400u16.to_le_bytes()); // 5400 MHz max speed
        type4[0x16..0x18].copy_from_slice(&3400u16.to_le_bytes()); // 3400 MHz current speed
        type4[0x23] = 16; // 16 cores
        type4[0x24] = 16; // 16 enabled
        type4[0x25] = 24; // 24 threads

        table.extend_from_slice(&type4);
        table.extend_from_slice(b"LGA1700\0");
        table.extend_from_slice(b"Intel(R) Corporation\0");
        table.extend_from_slice(b"13th Gen Intel(R) Core(TM) i7-13700K\0");
        table.push(0);

        table.extend_from_slice(&[127, 4, 2, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert_eq!(smbios.processors.len(), 1);
        let proc = &smbios.processors[0];
        assert_eq!(proc.socket_designation.as_deref(), Some("LGA1700"));
        assert_eq!(proc.current_speed_mhz, 3400);
        assert_eq!(proc.max_speed_mhz, 5400);
        assert_eq!(proc.external_clock_mhz, 100);
        assert_eq!(proc.core_count, 16);
        assert_eq!(proc.thread_count, 24);
    }

    #[test]
    fn test_smbios_type_3_chassis_and_is_laptop() {
        let mut table = Vec::new();
        // Type 3: Enclosure/Chassis info: type=3, len=9, handle=1
        // Formatted: mfg=1, type=0x0A (Notebook), ver=2
        table.extend_from_slice(&[3, 9, 1, 0, 1, 0x0A, 2, 0, 0]);
        table.extend_from_slice(b"Apple Inc.\0");
        table.extend_from_slice(b"MacBookPro\0");
        table.push(0);

        // Type 127
        table.extend_from_slice(&[127, 4, 2, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert!(smbios.chassis.is_some());
        let chassis = smbios.chassis.as_ref().unwrap();
        assert_eq!(chassis.chassis_type, 0x0A);
        assert_eq!(chassis.manufacturer.as_deref(), Some("Apple Inc."));
        assert!(smbios.is_laptop());
    }

    #[test]
    fn test_smbios_unpopulated_socket() {
        let mut table = Vec::new();
        // Type 4: Socket 0 (Populated, Enabled)
        let mut type4_cpu0 = vec![0u8; 0x20];
        type4_cpu0[0] = 4;
        type4_cpu0[1] = 0x20;
        type4_cpu0[4] = 1; // "CPU0"
        type4_cpu0[5] = 3;
        type4_cpu0[0x18] = 0x41; // Populated (bit 6 = 1), CPU Enabled (status = 1)
        table.extend_from_slice(&type4_cpu0);
        table.extend_from_slice(b"CPU0\0");
        table.push(0);

        // Type 4: Socket 1 (Unpopulated)
        let mut type4_cpu1 = vec![0u8; 0x20];
        type4_cpu1[0] = 4;
        type4_cpu1[1] = 0x20;
        type4_cpu1[4] = 1; // "CPU1"
        type4_cpu1[5] = 3;
        type4_cpu1[0x18] = 0x00; // Unpopulated (bit 6 = 0), CPU Status = 0
        table.extend_from_slice(&type4_cpu1);
        table.extend_from_slice(b"CPU1\0");
        table.push(0);

        // Type 127
        table.extend_from_slice(&[127, 4, 3, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert_eq!(smbios.processors.len(), 2);
        assert!(smbios.processors[0].is_populated);
        assert!(smbios.processors[0].is_enabled);
        assert!(!smbios.processors[1].is_populated);
    }
}
