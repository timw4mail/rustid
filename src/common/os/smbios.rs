//! Architecture-independent SMBIOS 2.x and 3.x table parsing and SystemInfo resolution.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::common::os::{is_generic_value, parse_apple_model};
use crate::common::{DataSource, SystemInfo};

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

impl SmbiosChassisInfo {
    pub fn is_laptop(&self) -> bool {
        // SMBIOS Chassis Types 8..=14: Portable, Laptop, Notebook, Hand Held, Sub Notebook, etc.
        // Also 30..=32: Tablet, Convertible, Detachable
        matches!(self.chassis_type, 8..=14 | 30..=32)
    }
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
                4 if s.formatted.len() >= 0x1A => {
                    // Type 4: Processor Information
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
                    let cpu_status = status & 0x07;
                    let is_populated = (status & 0x40) != 0 || cpu_status == 1;
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
                _ => {}
            }
        }

        data
    }

    /// Checks if the machine is a laptop / notebook based on chassis type or product name.
    pub fn is_laptop(&self) -> bool {
        if let Some(chassis) = &self.chassis
            && chassis.is_laptop()
        {
            return true;
        }

        const LAPTOP_KEYWORDS: &[&str] = &[
            "laptop",
            "notebook",
            "macbook",
            "thinkpad",
            "elitebook",
            "latitude",
            "inspiron",
            "zenbook",
        ];

        let matches_keyword = |s: &str| {
            let lower = s.to_ascii_lowercase();
            LAPTOP_KEYWORDS.iter().any(|k| lower.contains(k))
        };

        if let Some(sys) = &self.system {
            if let Some(prod) = &sys.product_name
                && matches_keyword(prod)
            {
                return true;
            }
            if let Some(fam) = &sys.family
                && matches_keyword(fam)
            {
                return true;
            }
        }
        false
    }

    /// Determines the best human-readable system name matching Linux/macOS/Windows heuristics.
    pub fn get_system_name(&self) -> Option<SystemInfo> {
        let sys_prod = self.system.as_ref().and_then(|s| s.product_name.as_deref());
        let sys_family = self.system.as_ref().and_then(|s| s.family.as_deref());
        let sys_mfr = self.system.as_ref().and_then(|s| s.manufacturer.as_deref());
        let bios_ver = self.bios.as_ref().and_then(|b| b.version.as_deref());
        let bios_vendor = self.bios.as_ref().and_then(|b| b.vendor.as_deref());
        let board_prod = self.board.as_ref().and_then(|b| b.product_name.as_deref());
        let board_mfr = self.board.as_ref().and_then(|b| b.manufacturer.as_deref());

        // 1. Check for Apple Mac hardware model identifier first across all SMBIOS fields
        for candidate in [sys_prod, bios_ver, sys_family, board_prod]
            .into_iter()
            .flatten()
        {
            if let Some(mac) = parse_apple_model(candidate) {
                return Some(SystemInfo::new(
                    Some("Apple Inc.".to_string()),
                    DataSource::Smbios("SMBIOS Type 1:Manufacturer"),
                    Some(mac),
                    DataSource::Smbios("SMBIOS Type 1:Product Name"),
                ));
            }
        }

        let is_apple = [sys_mfr, board_mfr, bios_ver, bios_vendor, sys_prod, sys_family]
            .into_iter()
            .flatten()
            .any(|s| {
                let s_lower = s.to_ascii_lowercase();
                s_lower.starts_with("apple")
                    || s_lower.contains("apple")
                    || parse_apple_model(s).is_some()
                    || s.starts_with("Mac-")
            });

        // Skip SMBIOS Family on Apple hardware (where Family is often firmware metadata like "4 4 A  ")
        if !is_apple
            && let Some(fam) = sys_family
            && !is_generic_value(fam)
        {
            return Some(SystemInfo::new(
                sys_mfr.map(ToString::to_string),
                DataSource::Smbios("SMBIOS Type 1:Manufacturer"),
                Some(fam.to_string()),
                DataSource::Smbios("SMBIOS Type 1:Family"),
            ));
        }

        if !is_apple
            && let Some(prod) = sys_prod
            && !is_generic_value(prod)
        {
            return Some(SystemInfo::new(
                sys_mfr.map(ToString::to_string),
                DataSource::Smbios("SMBIOS Type 1:Manufacturer"),
                Some(prod.to_string()),
                DataSource::Smbios("SMBIOS Type 1:Product Name"),
            ));
        }

        if !is_apple
            && let Some(board) = board_prod
            && !is_generic_value(board)
        {
            return Some(SystemInfo::new(
                board_mfr.map(ToString::to_string),
                DataSource::Smbios("SMBIOS Type 2:BaseBoardManufacturer"),
                Some(board.to_string()),
                DataSource::Smbios("SMBIOS Type 2:BaseBoardProduct"),
            ));
        }

        None
    }
}

/// Robustly parse an SMBIOS buffer (with or without 8-byte RawSMBIOSData header or at arbitrary offset).
pub fn parse_smbios_buffer(buf: &[u8]) -> Option<SystemInfo> {
    if buf.len() < 8 {
        return None;
    }

    // 1. Try standard 8-byte RawSMBIOSData header
    let smbios = SmbiosData::parse(&buf[8..]);
    if let Some(name) = smbios.get_system_name() {
        return Some(name);
    }

    // 2. Try raw structures at offset 0
    let smbios = SmbiosData::parse(buf);
    if let Some(name) = smbios.get_system_name() {
        return Some(name);
    }

    // 3. Scan for Type 1 structure header in buffer
    for i in 0..buf.len().saturating_sub(8) {
        if buf[i] == 1
            && buf[i + 1] >= 8
            && (buf[i + 1] as usize) < 64
        {
            let smbios = SmbiosData::parse(&buf[i..]);
            if let Some(name) = smbios.get_system_name() {
                return Some(name);
            }
        }
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
        table.extend_from_slice(&[1, 27, 1, 0]);
        table.push(1); // 0x04: manufacturer string 1 ("Apple Inc.")
        table.push(2); // 0x05: product name string 2 ("MacBookPro18,3")
        table.push(3); // 0x06: version string 3 ("1.0")
        table.push(4); // 0x07: serial number string 4
        table.extend_from_slice(&[0u8; 16]); // UUID
        table.push(3); // Wakeup type
        table.push(0); // SKU
        table.push(5); // Family ("MacBook Pro")

        table.extend_from_slice(b"Apple Inc.\0");
        table.extend_from_slice(b"MacBookPro18,3\0");
        table.extend_from_slice(b"1.0\0");
        table.extend_from_slice(b"C02XXXXX\0");
        table.extend_from_slice(b"MacBook Pro\0");
        table.push(0);

        table.extend_from_slice(&[127, 4, 2, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert!(smbios.system.is_some());
        let sys = smbios.system.as_ref().expect("system info should be present");
        assert_eq!(sys.manufacturer.as_deref(), Some("Apple Inc."));
        assert_eq!(sys.product_name.as_deref(), Some("MacBookPro18,3"));
        assert_eq!(sys.family.as_deref(), Some("MacBook Pro"));

        let name = smbios.get_system_name();
        assert_eq!(
            name.and_then(|s| s.display_name()).as_deref(),
            Some("MacBookPro18,3")
        );
    }

    #[test]
    fn test_smbios_type_2_motherboard_fallback() {
        let mut table = Vec::new();
        table.extend_from_slice(&[1, 8, 1, 0, 1, 2, 0, 0]);
        table.extend_from_slice(b"System manufacturer\0");
        table.extend_from_slice(b"System Product Name\0");
        table.push(0);

        table.extend_from_slice(&[2, 8, 2, 0, 1, 2, 3, 0]);
        table.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
        table.extend_from_slice(b"ROG STRIX Z790-E GAMING WIFI\0");
        table.extend_from_slice(b"Rev 1.xx\0");
        table.push(0);

        table.extend_from_slice(&[127, 4, 3, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert!(smbios.board.is_some());
        let board = smbios.board.as_ref().expect("board info should be present");
        assert_eq!(board.manufacturer.as_deref(), Some("ASUSTeK COMPUTER INC."));
        assert_eq!(
            board.product_name.as_deref(),
            Some("ROG STRIX Z790-E GAMING WIFI")
        );

        let name = smbios.get_system_name();
        assert_eq!(
            name.and_then(|s| s.display_name()).as_deref(),
            Some("ASUSTeK COMPUTER INC. ROG STRIX Z790-E GAMING WIFI")
        );
    }

    #[test]
    fn test_parse_smbios_buffer() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]); // 8-byte header

        buf.push(1); // Type 1
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // "Apple Inc."
        buf.push(2); // "MacBook4,1"
        buf.push(3); // "1.0"
        buf.extend_from_slice(&[0; 19]);
        buf.push(4); // "MacBook"

        buf.extend_from_slice(b"Apple Inc.\0");
        buf.extend_from_slice(b"MacBook4,1\0");
        buf.extend_from_slice(b"1.0\0");
        buf.extend_from_slice(b"MacBook\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf).and_then(|s| s.display_name()),
            Some("MacBook4,1".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_family_preferred() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]);

        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // "LENOVO"
        buf.push(2); // "20L5CTO1WW"
        buf.push(0);
        buf.extend_from_slice(&[0; 19]);
        buf.push(3); // "ThinkPad T480"

        buf.extend_from_slice(b"LENOVO\0");
        buf.extend_from_slice(b"20L5CTO1WW\0");
        buf.extend_from_slice(b"ThinkPad T480\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf).and_then(|s| s.display_name()),
            Some("LENOVO ThinkPad T480".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_baseboard_fallback() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]);

        buf.push(1);
        buf.push(0x08);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1);
        buf.push(2);
        buf.push(0);
        buf.push(0);
        buf.extend_from_slice(b"System manufacturer\0");
        buf.extend_from_slice(b"System Product Name\0");
        buf.push(0);

        buf.push(2);
        buf.push(0x08);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1);
        buf.push(2);
        buf.push(0);
        buf.push(0);
        buf.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
        buf.extend_from_slice(b"ROG STRIX B550-F GAMING\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf).and_then(|s| s.display_name()),
            Some("ASUSTeK COMPUTER INC. ROG STRIX B550-F GAMING".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_2008_macbook() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]);

        buf.push(0);
        buf.push(0x18);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1);
        buf.push(2);
        buf.extend_from_slice(&[0; 18]);
        buf.extend_from_slice(b"Apple Inc.\0");
        buf.extend_from_slice(b"MB41.88Z.00C1.B00.0802091544\0");
        buf.push(0);

        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.extend_from_slice(&[0; 19]);
        buf.push(4);
        buf.extend_from_slice(b"Apple Inc.\0");
        buf.extend_from_slice(b"Mac-F4208CC8\0");
        buf.extend_from_slice(b"1.0\0");
        buf.extend_from_slice(b"4 4 A  \0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf).and_then(|s| s.display_name()),
            Some("MacBook4,1".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_2008_macbook_control_chars() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]);

        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(0);
        buf.push(1);
        buf.push(0);
        buf.extend_from_slice(&[0; 19]);
        buf.push(2);
        buf.extend_from_slice(b"Mac-F4208CC8\0");
        buf.extend_from_slice(b"4\x084\x08A\x04\x05\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        let parsed = parse_smbios_buffer(&buf);
        assert!(parsed.is_some());
        let info = parsed.expect("smbios buffer should parse successfully");
        assert_eq!(info.display_name(), Some("MacBook4,1".to_string()));
        assert_eq!(info.model.as_deref(), Some("MacBook4,1"));
        assert_eq!(
            info.model_source,
            DataSource::Smbios("SMBIOS Type 1:Product Name")
        );
        assert_eq!(info.vendor, Some("Apple Inc.".to_string()));
        assert_eq!(
            info.vendor_source,
            DataSource::Smbios("SMBIOS Type 1:Manufacturer")
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

        table.extend_from_slice(&[127, 4, 2, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert!(smbios.chassis.is_some());
        let chassis = smbios.chassis.as_ref().expect("chassis info should be present");
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

        table.extend_from_slice(&[127, 4, 3, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert_eq!(smbios.processors.len(), 2);
        assert!(smbios.processors[0].is_populated);
        assert!(smbios.processors[0].is_enabled);
        assert!(!smbios.processors[1].is_populated);
    }

    #[test]
    fn test_smbios_dual_socket_apple_server() {
        let mut table = Vec::new();
        // Type 1: System Info ("Apple Inc.", "Xserve1,1")
        table.extend_from_slice(&[1, 8, 1, 0, 1, 2, 0, 0]);
        table.extend_from_slice(b"Apple Inc.\0");
        table.extend_from_slice(b"Xserve1,1\0");
        table.push(0);

        // Type 4: Socket 0 (CPU0, 0x20 legacy length for SMBIOS 2.4)
        let mut type4_cpu0 = vec![0u8; 0x20];
        type4_cpu0[0] = 4;
        type4_cpu0[1] = 0x20;
        type4_cpu0[4] = 1; // "CPU0"
        type4_cpu0[5] = 3;
        type4_cpu0[0x18] = 0x41; // Populated & Enabled
        table.extend_from_slice(&type4_cpu0);
        table.extend_from_slice(b"CPU0\0");
        table.push(0);

        // Type 4: Socket 1 (CPU1, 0x20 legacy length for SMBIOS 2.4)
        let mut type4_cpu1 = vec![0u8; 0x20];
        type4_cpu1[0] = 4;
        type4_cpu1[1] = 0x20;
        type4_cpu1[4] = 1; // "CPU1"
        type4_cpu1[5] = 3;
        type4_cpu1[0x18] = 0x41; // Populated & Enabled
        table.extend_from_slice(&type4_cpu1);
        table.extend_from_slice(b"CPU1\0");
        table.push(0);

        table.extend_from_slice(&[127, 4, 3, 0, 0, 0]);

        let smbios = SmbiosData::parse(&table);
        assert_eq!(smbios.processors.len(), 2);
        assert!(smbios.processors[0].is_populated);
        assert!(smbios.processors[0].is_enabled);
        assert!(smbios.processors[1].is_populated);
        assert!(smbios.processors[1].is_enabled);

        let populated = smbios
            .processors
            .iter()
            .filter(|p| p.is_populated && p.is_enabled)
            .count();
        assert_eq!(populated, 2);
        assert!(!smbios.is_laptop());
    }
}
