use crate::common::{
    DataSource, OS, SystemInfo, TDetect, TOSData, TopologyCount, TopologyTier, is_generic_value,
    parse_apple_model,
};
use windows::Win32::System::Registry::*;
use windows::Win32::System::SystemInformation::*;
use windows::core::{HSTRING, PCWSTR, w};

/// Clean up processor name strings (removing trademark symbols like (TM), (R), extra spaces).
pub fn clean_soc_name(raw: &str) -> String {
    let s = raw.replace("(TM)", "").replace("(R)", "");
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.trim().chars() {
        if c.is_whitespace() {
            if !last_was_space && !out.is_empty() {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    out
}

/// Helper function to read a string registry value (REG_SZ, REG_EXPAND_SZ, or REG_MULTI_SZ).
pub fn read_reg_string(hkey: HKEY, value_name: &str) -> Option<String> {
    let mut size = 0u32;
    let mut dw_type = REG_NONE;
    let val_name_hstring = HSTRING::from(value_name);
    let val_pcwstr = PCWSTR(val_name_hstring.as_ptr());

    let res = unsafe {
        RegQueryValueExW(
            hkey,
            val_pcwstr,
            None,
            Some(&mut dw_type),
            None,
            Some(&mut size),
        )
    };
    if res.is_err() || size == 0 {
        return None;
    }

    let wchar_count = (size as usize).div_ceil(2);
    let mut buf = vec![0u16; wchar_count];
    let res = unsafe {
        RegQueryValueExW(
            hkey,
            val_pcwstr,
            None,
            Some(&mut dw_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut size),
        )
    };
    if res.is_err() || (dw_type != REG_SZ && dw_type != REG_EXPAND_SZ && dw_type != REG_MULTI_SZ) {
        return None;
    }

    while let Some(&0) = buf.last() {
        buf.pop();
    }
    let s = String::from_utf16_lossy(&buf);
    let s = if dw_type == REG_MULTI_SZ {
        s.split('\0')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        s
    };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Helper function to read a binary registry value (REG_BINARY).
pub fn read_reg_binary(hkey: HKEY, value_name: &str) -> Option<Vec<u8>> {
    let mut size = 0u32;
    let mut dw_type = REG_NONE;
    let val_name_hstring = HSTRING::from(value_name);
    let val_pcwstr = PCWSTR(val_name_hstring.as_ptr());

    let res = unsafe {
        RegQueryValueExW(
            hkey,
            val_pcwstr,
            None,
            Some(&mut dw_type),
            None,
            Some(&mut size),
        )
    };
    if res.is_err() || size == 0 {
        return None;
    }

    let mut buf = vec![0u8; size as usize];
    let res = unsafe {
        RegQueryValueExW(
            hkey,
            val_pcwstr,
            None,
            Some(&mut dw_type),
            Some(buf.as_mut_ptr()),
            Some(&mut size),
        )
    };
    if res.is_err() {
        return None;
    }

    Some(buf)
}

struct BiosRegistrySources {
    mfr: &'static str,
    prod: &'static str,
    family: &'static str,
    bios_ver: &'static str,
    board_mfr: &'static str,
    board_prod: &'static str,
}

const BIOS_REG_SOURCES: BiosRegistrySources = BiosRegistrySources {
    mfr: r"HARDWARE\DESCRIPTION\System\BIOS:SystemManufacturer",
    prod: r"HARDWARE\DESCRIPTION\System\BIOS:SystemProductName",
    family: r"HARDWARE\DESCRIPTION\System\BIOS:SystemFamily",
    bios_ver: r"HARDWARE\DESCRIPTION\System\BIOS:BIOSVersion",
    board_mfr: r"HARDWARE\DESCRIPTION\System\BIOS:BaseBoardManufacturer",
    board_prod: r"HARDWARE\DESCRIPTION\System\BIOS:BaseBoardProduct",
};

const SYSINFO_REG_SOURCES: BiosRegistrySources = BiosRegistrySources {
    mfr: r"SYSTEM\CurrentControlSet\Control\SystemInformation:SystemManufacturer",
    prod: r"SYSTEM\CurrentControlSet\Control\SystemInformation:SystemProductName",
    family: r"SYSTEM\CurrentControlSet\Control\SystemInformation:SystemFamily",
    bios_ver: r"SYSTEM\CurrentControlSet\Control\SystemInformation:BIOSVersion",
    board_mfr: r"SYSTEM\CurrentControlSet\Control\SystemInformation:BaseBoardManufacturer",
    board_prod: r"SYSTEM\CurrentControlSet\Control\SystemInformation:BaseBoardProduct",
};

/// Helper to extract system product/board name from a standard BIOS / SystemInformation registry key.
fn get_system_from_bios_subkey(
    subkey: PCWSTR,
    sources: &BiosRegistrySources,
) -> Option<SystemInfo> {
    let mut hkey = HKEY::default();
    let result = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, None, KEY_READ, &mut hkey) };

    if result.is_ok() {
        let family =
            read_reg_string(hkey, "SystemFamily").or_else(|| read_reg_string(hkey, "Family"));
        let product_name = read_reg_string(hkey, "SystemProductName")
            .or_else(|| read_reg_string(hkey, "SystemModel"))
            .or_else(|| read_reg_string(hkey, "Model"));
        let manufacturer = read_reg_string(hkey, "SystemManufacturer")
            .or_else(|| read_reg_string(hkey, "Manufacturer"));
        let board_vendor = read_reg_string(hkey, "BaseBoardManufacturer")
            .or_else(|| read_reg_string(hkey, "BoardManufacturer"));
        let board_product = read_reg_string(hkey, "BaseBoardProduct")
            .or_else(|| read_reg_string(hkey, "BoardProduct"));
        let bios_ver = read_reg_string(hkey, "BIOSVersion")
            .or_else(|| read_reg_string(hkey, "SystemBiosVersion"));

        let _ = unsafe { RegCloseKey(hkey) };

        for s in [&product_name, &bios_ver, &family, &board_product]
            .into_iter()
            .flatten()
        {
            if let Some(mac) = parse_apple_model(s) {
                return Some(SystemInfo::new(
                    Some("Apple Inc.".to_string()),
                    DataSource::WindowsRegistry(sources.mfr),
                    Some(mac),
                    DataSource::WindowsRegistry(sources.bios_ver),
                ));
            }
        }

        let is_apple = [
            &manufacturer,
            &board_vendor,
            &product_name,
            &family,
            &bios_ver,
        ]
        .into_iter()
        .flatten()
        .any(|s| {
            let s_lower = s.to_ascii_lowercase();
            s_lower.starts_with("apple")
                || s_lower.contains("apple")
                || parse_apple_model(s).is_some()
                || s.starts_with("Mac-")
        });

        if !is_apple
            && let Some(fam) = family
            && !is_generic_value(&fam)
        {
            return Some(SystemInfo::new(
                manufacturer,
                DataSource::WindowsRegistry(sources.mfr),
                Some(fam),
                DataSource::WindowsRegistry(sources.family),
            ));
        }

        if !is_apple
            && let Some(prod) = product_name
            && !is_generic_value(&prod)
        {
            return Some(SystemInfo::new(
                manufacturer,
                DataSource::WindowsRegistry(sources.mfr),
                Some(prod),
                DataSource::WindowsRegistry(sources.prod),
            ));
        }

        if !is_apple
            && let Some(board_prod) = board_product
            && !is_generic_value(&board_prod)
        {
            return Some(SystemInfo::new(
                board_vendor,
                DataSource::WindowsRegistry(sources.board_mfr),
                Some(board_prod),
                DataSource::WindowsRegistry(sources.board_prod),
            ));
        }
    }

    None
}

/// Helper to parse model & manufacturer from oeminfo.ini content.
pub fn parse_oeminfo_content(content: &str) -> Option<SystemInfo> {
    let mut mfr: Option<String> = None;
    let mut model: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if k.eq_ignore_ascii_case("Manufacturer") && !is_generic_value(v) {
                mfr = Some(v.to_string());
            } else if k.eq_ignore_ascii_case("Model") && !is_generic_value(v) {
                model = Some(v.to_string());
            }
        }
    }
    if let Some(m) = model {
        return Some(SystemInfo::new(
            mfr,
            DataSource::WindowsRegistry(r"oeminfo.ini:Manufacturer"),
            Some(m),
            DataSource::WindowsRegistry(r"oeminfo.ini:Model"),
        ));
    } else if let Some(f) = mfr {
        return Some(SystemInfo::new(
            Some(f),
            DataSource::WindowsRegistry(r"oeminfo.ini:Manufacturer"),
            None,
            DataSource::DefaultValue,
        ));
    }
    None
}

/// Reads OEM information from oeminfo.ini (standard on Windows 9x, 2000, XP).
fn read_oeminfo_ini() -> Option<SystemInfo> {
    let windir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("WINDIR"))
        .ok()?;
    let candidates = [
        format!(r"{}\system32\oeminfo.ini", windir),
        format!(r"{}\system\oeminfo.ini", windir),
        format!(r"{}\oeminfo.ini", windir),
    ];
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Some(sys) = parse_oeminfo_content(&content)
        {
            return Some(sys);
        }
    }
    None
}

#[repr(C)]
#[derive(Copy, Clone)]
struct SystemLogicalProcessorInformationLegacy {
    processor_mask: usize,
    relationship: u32,
    data: [u64; 2],
}

type PfnGetLogicalProcessorInformation = unsafe extern "system" fn(
    *mut SystemLogicalProcessorInformationLegacy,
    *mut u32,
) -> windows::core::BOOL;

/// Dynamically calls GetLogicalProcessorInformation (Windows XP SP3 / Server 2003 / Vista).
fn get_legacy_logical_processor_information() -> Option<Vec<SystemLogicalProcessorInformationLegacy>>
{
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::core::s;

    unsafe {
        let k32 = GetModuleHandleA(s!("kernel32.dll")).ok()?;
        let pfn = GetProcAddress(k32, s!("GetLogicalProcessorInformation"))?;
        let pfn: PfnGetLogicalProcessorInformation = std::mem::transmute(pfn);

        let mut length = 0u32;
        let _ = pfn(std::ptr::null_mut(), &mut length);
        if length == 0 {
            return None;
        }

        let item_size = std::mem::size_of::<SystemLogicalProcessorInformationLegacy>();
        let count = (length as usize) / item_size;
        let mut buffer = vec![std::mem::zeroed::<SystemLogicalProcessorInformationLegacy>(); count];
        let res = pfn(buffer.as_mut_ptr(), &mut length);
        if res.as_bool() { Some(buffer) } else { None }
    }
}

/// Gets the total number of logical processors reported by Windows NT / 2000 / XP / 9x.
fn get_legacy_processor_count() -> u32 {
    let num_procs = unsafe {
        let mut sys_info = std::mem::zeroed();
        GetSystemInfo(&mut sys_info);
        sys_info.dwNumberOfProcessors
    };

    let mut hkey = HKEY::default();
    let cp_subkey = w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor");
    let mut subkeys_count = 0u32;
    let res = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, cp_subkey, None, KEY_READ, &mut hkey) };
    if res.is_ok() {
        let _ = unsafe {
            RegQueryInfoKeyW(
                hkey,
                None,
                None,
                None,
                Some(&mut subkeys_count),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        };
        let _ = unsafe { RegCloseKey(hkey) };
    }

    num_procs.max(subkeys_count).max(1)
}

/// Detects Apple Mac model identifier across Windows registry keys and BIOS version strings.
fn detect_apple_mac_model() -> Option<SystemInfo> {
    let mut hkey = HKEY::default();

    // 1. Check HARDWARE\DESCRIPTION\System\BIOS
    if unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!(r"HARDWARE\DESCRIPTION\System\BIOS"),
            None,
            KEY_READ,
            &mut hkey,
        )
    }
    .is_ok()
    {
        let prod = read_reg_string(hkey, "SystemProductName");
        let bios_ver = read_reg_string(hkey, "BIOSVersion");
        let family = read_reg_string(hkey, "SystemFamily");
        let sys_ver = read_reg_string(hkey, "SystemVersion");
        let board_prod = read_reg_string(hkey, "BaseBoardProduct");
        let _ = unsafe { RegCloseKey(hkey) };

        for s in [&prod, &family, &sys_ver, &board_prod, &bios_ver]
            .into_iter()
            .flatten()
        {
            if let Some(mac) = parse_apple_model(s) {
                return Some(SystemInfo::new(
                    Some("Apple Inc.".to_string()),
                    DataSource::WindowsRegistry(
                        r"HARDWARE\DESCRIPTION\System\BIOS:SystemManufacturer",
                    ),
                    Some(mac),
                    DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System\BIOS:BIOSVersion"),
                ));
            }
        }
    }

    // 2. Check SYSTEM\CurrentControlSet\Control\SystemInformation
    if unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!(r"SYSTEM\CurrentControlSet\Control\SystemInformation"),
            None,
            KEY_READ,
            &mut hkey,
        )
    }
    .is_ok()
    {
        let prod = read_reg_string(hkey, "SystemProductName");
        let bios_ver = read_reg_string(hkey, "BIOSVersion");
        let _ = unsafe { RegCloseKey(hkey) };

        for s in [&prod, &bios_ver].into_iter().flatten() {
            if let Some(mac) = parse_apple_model(s) {
                return Some(SystemInfo::new(
                    Some("Apple Inc.".to_string()),
                    DataSource::WindowsRegistry(
                        r"SYSTEM\CurrentControlSet\Control\SystemInformation:SystemManufacturer",
                    ),
                    Some(mac),
                    DataSource::WindowsRegistry(
                        r"SYSTEM\CurrentControlSet\Control\SystemInformation:BIOSVersion",
                    ),
                ));
            }
        }
    }

    // 3. Check HARDWARE\DESCRIPTION\System (Identifier & SystemBiosVersion)
    if unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!(r"HARDWARE\DESCRIPTION\System"),
            None,
            KEY_READ,
            &mut hkey,
        )
    }
    .is_ok()
    {
        let id = read_reg_string(hkey, "Identifier");
        let bios_ver = read_reg_string(hkey, "SystemBiosVersion");
        let _ = unsafe { RegCloseKey(hkey) };

        if let Some(s) = &bios_ver {
            if let Some(mac) = parse_apple_model(s) {
                return Some(SystemInfo::new(
                    Some("Apple Inc.".to_string()),
                    DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:Identifier"),
                    Some(mac),
                    DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:SystemBiosVersion"),
                ));
            }
            for part in s.split_whitespace() {
                if let Some(mac) = parse_apple_model(part) {
                    return Some(SystemInfo::new(
                        Some("Apple Inc.".to_string()),
                        DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:Identifier"),
                        Some(mac),
                        DataSource::WindowsRegistry(
                            r"HARDWARE\DESCRIPTION\System:SystemBiosVersion",
                        ),
                    ));
                }
            }
        }
        if let Some(s) = &id
            && let Some(mac) = parse_apple_model(s)
        {
            return Some(SystemInfo::new(
                Some("Apple Inc.".to_string()),
                DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:Identifier"),
                Some(mac),
                DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:Identifier"),
            ));
        }
    }

    None
}

type PfnGetSystemFirmwareTable = unsafe extern "system" fn(u32, u32, *mut u8, u32) -> u32;

/// Parse raw SMBIOS structures for system name (matches "System Model" in msinfo32).
fn parse_smbios_structures_for_system_name(data: &[u8]) -> Option<SystemInfo> {
    let mut offset = 0;
    let mut bios_ver: Option<String> = None;
    let mut sys_family: Option<String> = None;
    let mut sys_prod: Option<String> = None;
    let mut sys_mfr: Option<String> = None;
    let mut board_prod: Option<String> = None;
    let mut board_mfr: Option<String> = None;

    while offset + 4 <= data.len() {
        let type_ = data[offset];
        let length = data[offset + 1] as usize;
        if length < 4 || offset + length > data.len() {
            break;
        }

        let formatted = &data[offset..offset + length];
        let mut string_offset = offset + length;
        let mut strings: Vec<String> = Vec::new();

        while string_offset < data.len() {
            if data[string_offset] == 0 {
                string_offset += 1;
                break;
            }
            let start = string_offset;
            while string_offset < data.len() && data[string_offset] != 0 {
                string_offset += 1;
            }
            let s = String::from_utf8_lossy(&data[start..string_offset]).to_string();
            strings.push(s);
            if string_offset < data.len() && data[string_offset] == 0 {
                string_offset += 1;
            }
        }

        let get_str = |idx: u8| -> Option<String> {
            if idx > 0 && (idx as usize) <= strings.len() {
                let s = strings[idx as usize - 1].trim().to_string();
                if !s.is_empty() { Some(s) } else { None }
            } else {
                None
            }
        };

        match type_ {
            0 => {
                if formatted.len() >= 6 {
                    bios_ver = get_str(formatted[5]);
                }
            }
            1 => {
                if formatted.len() >= 6 {
                    sys_mfr = get_str(formatted[4]);
                    sys_prod = get_str(formatted[5]);
                }
                if formatted.len() >= 0x1B {
                    sys_family = get_str(formatted[0x1A]);
                }
            }
            2 => {
                if formatted.len() >= 6 {
                    board_mfr = get_str(formatted[4]);
                    board_prod = get_str(formatted[5]);
                }
            }
            127 => break,
            _ => {}
        }

        offset = string_offset;
    }

    // Check for Apple Mac hardware model identifier first across all SMBIOS fields
    for candidate in [&sys_prod, &bios_ver, &sys_family, &board_prod]
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

    let is_apple = [&sys_mfr, &board_mfr, &bios_ver, &sys_prod, &sys_family]
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
        && !is_generic_value(&fam)
    {
        return Some(SystemInfo::new(
            sys_mfr,
            DataSource::Smbios("SMBIOS Type 1:Manufacturer"),
            Some(fam),
            DataSource::Smbios("SMBIOS Type 1:Family"),
        ));
    }

    if !is_apple
        && let Some(prod) = sys_prod
        && !is_generic_value(&prod)
    {
        return Some(SystemInfo::new(
            sys_mfr,
            DataSource::Smbios("SMBIOS Type 1:Manufacturer"),
            Some(prod),
            DataSource::Smbios("SMBIOS Type 1:Product Name"),
        ));
    }

    if !is_apple
        && let Some(board) = board_prod
        && !is_generic_value(&board)
    {
        return Some(SystemInfo::new(
            board_mfr,
            DataSource::Smbios("SMBIOS Type 2:BaseBoardManufacturer"),
            Some(board),
            DataSource::Smbios("SMBIOS Type 2:BaseBoardProduct"),
        ));
    }

    None
}

/// Reads SMBIOS table via GetSystemFirmwareTable (Windows Vista / 7 / 8 / 10 / 11 / Server 2003 SP1+).
fn get_system_name_from_firmware_table() -> Option<SystemInfo> {
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::core::s;

    unsafe {
        let k32 = GetModuleHandleA(s!("kernel32.dll")).ok()?;
        let pfn = GetProcAddress(k32, s!("GetSystemFirmwareTable"))?;
        let pfn: PfnGetSystemFirmwareTable = std::mem::transmute(pfn);

        let rsmb = u32::from_be_bytes(*b"RSMB");
        let size = pfn(rsmb, 0, std::ptr::null_mut(), 0);
        if size <= 8 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let bytes_written = pfn(rsmb, 0, buf.as_mut_ptr(), size);
        if bytes_written <= 8 {
            return None;
        }

        let smbios_data = &buf[8..bytes_written as usize];
        parse_smbios_structures_for_system_name(smbios_data)
    }
}

/// Robustly parse an SMBIOS buffer (with or without header or at arbitrary offset).
fn parse_smbios_buffer(buf: &[u8]) -> Option<SystemInfo> {
    if buf.len() < 8 {
        return None;
    }

    // 1. Try standard 8-byte RawSMBIOSData header
    if let Some(name) = parse_smbios_structures_for_system_name(&buf[8..]) {
        return Some(name);
    }

    // 2. Try raw structures at offset 0
    if let Some(name) = parse_smbios_structures_for_system_name(buf) {
        return Some(name);
    }

    // 3. Scan for Type 1 structure header in buffer
    for i in 0..buf.len().saturating_sub(8) {
        if buf[i] == 1
            && buf[i + 1] >= 8
            && (buf[i + 1] as usize) < 64
            && let Some(name) = parse_smbios_structures_for_system_name(&buf[i..])
        {
            return Some(name);
        }
    }

    None
}

/// Reads SMBIOS table from mssmbios registry cache (standard on Windows 2000 and Windows XP).
fn get_system_name_from_mssmbios_reg() -> Option<SystemInfo> {
    let subkeys = [
        w!(r"SYSTEM\CurrentControlSet\Services\mssmbios\Data"),
        w!(r"SYSTEM\ControlSet001\Services\mssmbios\Data"),
        w!(r"SYSTEM\CurrentControlSet\Services\mssmbios"),
        w!(r"SYSTEM\ControlSet001\Services\mssmbios"),
        w!(r"SYSTEM\CurrentControlSet\Control\SystemInformation"),
        w!(r"HARDWARE\DESCRIPTION\System\BIOS"),
        w!(r"HARDWARE\DESCRIPTION\System"),
    ];

    for subkey in subkeys {
        let mut hkey = HKEY::default();
        if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, None, KEY_READ, &mut hkey) }.is_ok() {
            let bin_data = read_reg_binary(hkey, "SMBiosData")
                .or_else(|| read_reg_binary(hkey, "SMBIOSTableData"))
                .or_else(|| read_reg_binary(hkey, "SMBIOSData"))
                .or_else(|| read_reg_binary(hkey, "SMBiosTable"))
                .or_else(|| read_reg_binary(hkey, "TableData"));
            let _ = unsafe { RegCloseKey(hkey) };

            if let Some(buf) = bin_data
                && let Some(name) = parse_smbios_buffer(&buf)
            {
                return Some(name);
            }
        }
    }
    None
}

impl TOSData for OS {
    fn get_system_name() -> Option<SystemInfo> {
        // 0. Apple Mac hardware running Windows (must precede generic PC SMBIOS heuristics)
        if let Some(mac_model) = detect_apple_mac_model() {
            return Some(mac_model);
        }

        // 1. Raw SMBIOS table via GetSystemFirmwareTable (same API used by msinfo32 "System Model")
        if let Some(sys) = get_system_name_from_firmware_table() {
            return Some(sys);
        }

        // 2. Raw SMBIOS table via mssmbios registry cache (Windows 2000 / XP)
        if let Some(sys) = get_system_name_from_mssmbios_reg() {
            return Some(sys);
        }

        // 3. Modern BIOS key (Windows Vista, 7, 8, 10, 11)
        if let Some(sys) =
            get_system_from_bios_subkey(w!(r"HARDWARE\DESCRIPTION\System\BIOS"), &BIOS_REG_SOURCES)
        {
            return Some(sys);
        }

        // 4. Windows 2000 / XP SystemInformation key
        if let Some(sys) = get_system_from_bios_subkey(
            w!(r"SYSTEM\CurrentControlSet\Control\SystemInformation"),
            &SYSINFO_REG_SOURCES,
        ) {
            return Some(sys);
        }

        // 4. OEMInformation registry key
        let mut hkey = HKEY::default();
        let oem_subkey = w!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\OEMInformation");
        if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, oem_subkey, None, KEY_READ, &mut hkey) }
            .is_ok()
        {
            let model = read_reg_string(hkey, "Model");
            let manufacturer = read_reg_string(hkey, "Manufacturer");
            let _ = unsafe { RegCloseKey(hkey) };

            if let Some(m) = model
                && !is_generic_value(&m)
            {
                return Some(SystemInfo::new(
                    manufacturer,
                    DataSource::WindowsRegistry(
                        r"SOFTWARE\Microsoft\Windows\CurrentVersion\OEMInformation:Manufacturer",
                    ),
                    Some(m),
                    DataSource::WindowsRegistry(
                        r"SOFTWARE\Microsoft\Windows\CurrentVersion\OEMInformation:Model",
                    ),
                ));
            }
        }

        // 5. OEM info INI file (Windows 9x, 2000, XP)
        if let Some(sys) = read_oeminfo_ini() {
            return Some(sys);
        }

        // 6. Hardware Description Identifier / BIOS Version string (non-Apple fallback)
        let desc_subkey = w!(r"HARDWARE\DESCRIPTION\System");
        if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, desc_subkey, None, KEY_READ, &mut hkey) }
            .is_ok()
        {
            let id = read_reg_string(hkey, "Identifier");
            let bios_ver = read_reg_string(hkey, "SystemBiosVersion");
            let _ = unsafe { RegCloseKey(hkey) };

            let is_apple = id
                .as_deref()
                .map(|s| s.to_uppercase().starts_with("APPLE"))
                .or_else(|| {
                    bios_ver
                        .as_deref()
                        .map(|s| s.to_uppercase().starts_with("APPLE"))
                })
                .unwrap_or(false);

            if !is_apple {
                if let Some(id_str) = id
                    && !is_generic_value(&id_str)
                    && id_str != "AT/AT COMPATIBLE"
                    && id_str != "PC-9800"
                {
                    return Some(SystemInfo::from_model(
                        id_str,
                        DataSource::WindowsRegistry(r"HARDWARE\DESCRIPTION\System:Identifier"),
                    ));
                }

                if let Some(ver) = bios_ver
                    && !is_generic_value(&ver)
                    && ver.len() > 3
                    && !ver
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '/' || c == '-' || c == '.')
                {
                    return Some(SystemInfo::from_model(
                        ver,
                        DataSource::WindowsRegistry(
                            r"HARDWARE\DESCRIPTION\System:SystemBiosVersion",
                        ),
                    ));
                }
            }
        }

        None
    }

    fn get_soc() -> Option<String> {
        let mut hkey = HKEY::default();
        let subkey = w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
        let result =
            unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, None, KEY_READ, &mut hkey) };

        if result.is_ok() {
            let proc_name = read_reg_string(hkey, "ProcessorNameString");
            let _ = unsafe { RegCloseKey(hkey) };

            if let Some(name) = proc_name
                && !is_generic_value(&name)
            {
                return Some(clean_soc_name(&name));
            }
        }

        None
    }

    fn get_socket_count() -> TopologyTier {
        // 1. Try modern GetLogicalProcessorInformationEx (Windows 7+)
        let mut length = 0u32;
        unsafe {
            let _ = GetLogicalProcessorInformationEx(RelationProcessorPackage, None, &mut length);
        }
        if length > 0 {
            let mut buffer = vec![0u8; length as usize];
            let ptr = buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
            let res = unsafe {
                GetLogicalProcessorInformationEx(RelationProcessorPackage, Some(ptr), &mut length)
            };
            if res.is_ok() {
                let mut package_count = 0u32;
                let mut offset = 0usize;
                while offset < length as usize {
                    let item_ptr = unsafe {
                        buffer.as_ptr().add(offset)
                            as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX
                    };
                    let item = unsafe { &*item_ptr };
                    if item.Size == 0 {
                        break;
                    }
                    if item.Relationship == RelationProcessorPackage {
                        package_count += 1;
                    }
                    offset += item.Size as usize;
                }
                if package_count > 0 {
                    return TopologyTier::new(
                        package_count,
                        DataSource::WindowsRegistry("GetLogicalProcessorInformationEx"),
                    );
                }
            }
        }

        // 2. Try legacy GetLogicalProcessorInformation (Windows XP SP3 / Server 2003 / Vista)
        if let Some(entries) = get_legacy_logical_processor_information() {
            let package_count = entries
                .iter()
                .filter(|e| e.relationship == 3) // RelationProcessorPackage = 3
                .count() as u32;
            if package_count > 0 {
                return TopologyTier::new(
                    package_count,
                    DataSource::WindowsRegistry("GetLogicalProcessorInformation"),
                );
            }
        }

        // 3. Windows 2000 / NT4 / XP RTM fallback: compute sockets from total processors
        let total_procs = get_legacy_processor_count();
        if total_procs > 1 {
            #[cfg(x86_cpu)]
            {
                let threads_per_pkg = crate::x86::cpuid_threads_per_package().max(1);
                let sockets = (total_procs / threads_per_pkg).max(1);
                return TopologyTier::new(sockets, DataSource::WindowsRegistry("CentralProcessor"));
            }
            #[cfg(not(x86_cpu))]
            {
                return TopologyTier::new(
                    total_procs,
                    DataSource::WindowsRegistry("CentralProcessor"),
                );
            }
        }

        TopologyTier::new(1, DataSource::DefaultValue)
    }
}

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();
        let mut cores = 0u32;
        let mut threads = 0u32;

        let mut length = 0u32;
        unsafe {
            let _ = GetLogicalProcessorInformationEx(RelationProcessorCore, None, &mut length);
        }
        if length > 0 {
            let mut buffer = vec![0u8; length as usize];
            let ptr = buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
            let res = unsafe {
                GetLogicalProcessorInformationEx(RelationProcessorCore, Some(ptr), &mut length)
            };
            if res.is_ok() {
                let mut offset = 0usize;
                while offset < length as usize {
                    let item_ptr = unsafe {
                        buffer.as_ptr().add(offset)
                            as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX
                    };
                    let item = unsafe { &*item_ptr };
                    if item.Size == 0 {
                        break;
                    }
                    if item.Relationship == RelationProcessorCore {
                        cores += 1;
                        let proc_rel = unsafe { &item.Anonymous.Processor };
                        for g in 0..proc_rel.GroupCount as usize {
                            let mask = proc_rel.GroupMask[g].Mask;
                            threads += mask.count_ones();
                        }
                    }
                    offset += item.Size as usize;
                }
            }
        }

        // Fallback to legacy GetLogicalProcessorInformation (Win XP SP3 / Server 2003 / Vista)
        if (cores == 0 || threads == 0)
            && let Some(entries) = get_legacy_logical_processor_information()
        {
            for e in entries.iter().filter(|e| e.relationship == 0) {
                // RelationProcessorCore = 0
                cores += 1;
                threads += e.processor_mask.count_ones();
            }
        }

        // Fallback for Windows 2000 / NT4 / XP RTM
        if threads == 0 {
            threads = get_legacy_processor_count();
        }
        if cores == 0 {
            #[cfg(x86_cpu)]
            {
                let threads_per_core = crate::x86::cpuid_threads_per_core().max(1);
                cores = (threads / threads_per_core).max(1);
            }
            #[cfg(not(x86_cpu))]
            {
                cores = threads;
            }
        }

        if cores == 0 {
            cores = 1;
        }
        if threads == 0 {
            threads = cores;
        }

        TopologyCount {
            sockets,
            cores,
            threads,
            source: DataSource::WindowsRegistry("GetLogicalProcessorInformation"),
        }
    }
}

use crate::common::{Cache, CacheLevel, CacheType, Level1Cache};

#[cfg(not(x86_cpu))]
impl Cache {
    pub fn detect() -> Option<Cache> {
        Self::from_windows()
    }
}

impl Cache {
    pub fn from_windows() -> Option<Cache> {
        let mut length = 0u32;
        unsafe {
            let _ = GetLogicalProcessorInformationEx(RelationCache, None, &mut length);
        }
        if length == 0 {
            return None;
        }

        let mut buffer = vec![0u8; length as usize];
        let ptr = buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
        let res =
            unsafe { GetLogicalProcessorInformationEx(RelationCache, Some(ptr), &mut length) };
        if res.is_err() {
            return None;
        }

        let mut cache = Cache {
            source: DataSource::WindowsRegistry("GetLogicalProcessorInformationEx"),
            ..Default::default()
        };
        let mut found_cache = false;

        let mut offset = 0usize;
        while offset < length as usize {
            let item_ptr = unsafe {
                buffer.as_ptr().add(offset) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX
            };
            let item = unsafe { &*item_ptr };
            if item.Size == 0 {
                break;
            }

            if item.Relationship == RelationCache {
                let cache_rel = unsafe { &item.Anonymous.Cache };
                let level = cache_rel.Level;
                let assoc = cache_rel.Associativity as u32;
                let size_bytes = cache_rel.CacheSize;
                let cache_type = match cache_rel.Type {
                    c if c == CacheData => CacheType::Data,
                    c if c == CacheInstruction => CacheType::Instruction,
                    c if c == CacheUnified => CacheType::Unified,
                    _ => {
                        offset += item.Size as usize;
                        continue;
                    }
                };
                let mask = unsafe { cache_rel.Anonymous.GroupMask.Mask };
                let share_count = mask.count_ones();

                match level {
                    1 => match cache_type {
                        CacheType::Unified => {
                            cache.l1 = Level1Cache::Unified(CacheLevel::new(
                                size_bytes,
                                cache_type,
                                assoc,
                                share_count,
                            ));
                            found_cache = true;
                        }
                        CacheType::Data => {
                            match &mut cache.l1 {
                                Level1Cache::Split { data, .. } => {
                                    *data =
                                        CacheLevel::new(size_bytes, cache_type, assoc, share_count);
                                }
                                _ => {
                                    cache.l1 = Level1Cache::Split {
                                        data: CacheLevel::new(
                                            size_bytes,
                                            CacheType::Data,
                                            assoc,
                                            share_count,
                                        ),
                                        instruction: CacheLevel::default(),
                                    };
                                }
                            }
                            found_cache = true;
                        }
                        CacheType::Instruction => {
                            match &mut cache.l1 {
                                Level1Cache::Split { instruction, .. } => {
                                    *instruction =
                                        CacheLevel::new(size_bytes, cache_type, assoc, share_count);
                                }
                                _ => {
                                    cache.l1 = Level1Cache::Split {
                                        data: CacheLevel::default(),
                                        instruction: CacheLevel::new(
                                            size_bytes,
                                            CacheType::Instruction,
                                            assoc,
                                            share_count,
                                        ),
                                    };
                                }
                            }
                            found_cache = true;
                        }
                        _ => {}
                    },
                    2 => {
                        cache.l2 =
                            Some(CacheLevel::new(size_bytes, cache_type, assoc, share_count));
                        found_cache = true;
                    }
                    3 => {
                        cache.l3 =
                            Some(CacheLevel::new(size_bytes, cache_type, assoc, share_count));
                        found_cache = true;
                    }
                    _ => {}
                }
            }

            offset += item.Size as usize;
        }

        if found_cache { Some(cache) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_soc_name() {
        assert_eq!(
            clean_soc_name("Snapdragon(TM) X Elite - X1E80100 - Qualcomm(R) Oryon(TM) CPU"),
            "Snapdragon X Elite - X1E80100 - Qualcomm Oryon CPU"
        );
        assert_eq!(
            clean_soc_name("Snapdragon(R) 8cx Gen 3 @ 3.0 GHz"),
            "Snapdragon 8cx Gen 3 @ 3.0 GHz"
        );
    }

    #[test]
    fn test_parse_oeminfo_content() {
        let content = r#"
[General]
Manufacturer=Dell Computer Corporation
Model=Dimension XPS T500
SupportURL=http://support.dell.com
"#;
        assert_eq!(
            parse_oeminfo_content(content).and_then(|s| s.display_name()),
            Some("Dell Computer Corporation Dimension XPS T500".to_string())
        );

        let content_model_contains_mfr = r#"
[General]
Manufacturer=IBM
Model=IBM ThinkPad T42
"#;
        assert_eq!(
            parse_oeminfo_content(content_model_contains_mfr).and_then(|s| s.display_name()),
            Some("IBM ThinkPad T42".to_string())
        );

        let content_generic = r#"
[General]
Manufacturer=To be filled by O.E.M.
Model=To be filled by O.E.M.
"#;
        assert_eq!(parse_oeminfo_content(content_generic), None);
    }

    #[test]
    fn test_parse_apple_model() {
        assert_eq!(
            parse_apple_model("MB41.88Z.00C1.B00.0802091544"),
            Some("MacBook4,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MBP31.88Z.0070.B00.0706281432"),
            Some("MacBookPro3,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MBA11.88Z.00BB.B00.0803171226"),
            Some("MacBookAir1,1".to_string())
        );
        assert_eq!(
            parse_apple_model("IM81.88Z.00C1.B00.0802091544"),
            Some("iMac8,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MM21.88Z.009A.B00.0706281359"),
            Some("Macmini2,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MP31.88Z.006C.B05.0802291410"),
            Some("MacPro3,1".to_string())
        );
        assert_eq!(
            parse_apple_model("MacBook4,1"),
            Some("MacBook4,1".to_string())
        );
        assert_eq!(parse_apple_model("APPLE  - c1"), None);
        assert_eq!(parse_apple_model("American Megatrends Inc."), None);
    }

    #[test]
    fn test_parse_smbios_buffer() {
        let mut buf = Vec::new();
        // 8-byte RawSMBIOSData header
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]);

        // SMBIOS Type 1: System Information
        buf.push(1); // Type 1
        buf.push(0x1B); // Formatted length (27 bytes)
        buf.extend_from_slice(&[0, 0]); // Handle
        buf.push(1); // 0x04: Manufacturer string 1 ("Apple Inc.")
        buf.push(2); // 0x05: Product Name string 2 ("MacBook4,1")
        buf.push(3); // 0x06: Version string 3 ("1.0")
        buf.extend_from_slice(&[0; 19]); // Fill up to 0x1A
        buf.push(4); // 0x1A: Family string 4 ("MacBook")

        // Strings
        buf.extend_from_slice(b"Apple Inc.\0");
        buf.extend_from_slice(b"MacBook4,1\0");
        buf.extend_from_slice(b"1.0\0");
        buf.extend_from_slice(b"MacBook\0");
        buf.push(0); // Double null terminator

        // End of table (Type 127)
        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf, "test").and_then(|s| s.display_name()),
            Some("MacBook4,1".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_family_preferred() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]); // 8-byte header

        // Type 1 with Family
        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // Mfr: LENOVO
        buf.push(2); // Prod: 20L5CTO1WW
        buf.push(0); // Version
        buf.extend_from_slice(&[0; 19]);
        buf.push(3); // Family: ThinkPad T480

        buf.extend_from_slice(b"LENOVO\0");
        buf.extend_from_slice(b"20L5CTO1WW\0");
        buf.extend_from_slice(b"ThinkPad T480\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        assert_eq!(
            parse_smbios_buffer(&buf, "test").and_then(|s| s.display_name()),
            Some("LENOVO ThinkPad T480".to_string())
        );
    }

    #[test]
    fn test_parse_smbios_buffer_baseboard_fallback() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]); // 8-byte header

        // Type 1: Generic System Product Name
        buf.push(1);
        buf.push(0x08);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // Mfr: System manufacturer
        buf.push(2); // Prod: System Product Name
        buf.push(0);
        buf.push(0);
        buf.extend_from_slice(b"System manufacturer\0");
        buf.extend_from_slice(b"System Product Name\0");
        buf.push(0);

        // Type 2: Baseboard
        buf.push(2);
        buf.push(0x08);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // Board Mfr: ASUSTeK COMPUTER INC.
        buf.push(2); // Board Prod: ROG STRIX B550-F GAMING
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
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]); // 8-byte header

        // Type 0: BIOS Information with Apple EFI BIOS Version "MB41.88Z..."
        buf.push(0);
        buf.push(0x18);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // Vendor: "Apple Inc."
        buf.push(2); // BIOS Version: "MB41.88Z.00C1.B00.0802091544"
        buf.extend_from_slice(&[0; 18]);
        buf.extend_from_slice(b"Apple Inc.\0");
        buf.extend_from_slice(b"MB41.88Z.00C1.B00.0802091544\0");
        buf.push(0);

        // Type 1: System Information with garbage family "4 4 A  "
        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(1); // Mfr: "Apple Inc."
        buf.push(2); // Prod: "Mac-F4208CC8"
        buf.push(3); // Version: "1.0"
        buf.extend_from_slice(&[0; 19]);
        buf.push(4); // Family: "4 4 A  "
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
        buf.extend_from_slice(&[1, 2, 4, 0, 50, 0, 0, 0]); // 8-byte header

        // Type 1: System Information with garbage family "4\u{8}4\u{8}A\u{4}\u{5}" and Board ID "Mac-F4208CC8"
        buf.push(1);
        buf.push(0x1B);
        buf.extend_from_slice(&[0, 0]);
        buf.push(0); // Mfr: None (0)
        buf.push(1); // Prod: "Mac-F4208CC8"
        buf.push(0); // Version: None
        buf.extend_from_slice(&[0; 19]);
        buf.push(2); // Family: "4\u{8}4\u{8}A\u{4}\u{5}"
        buf.extend_from_slice(b"Mac-F4208CC8\0");
        buf.extend_from_slice(b"4\x084\x08A\x04\x05\0");
        buf.push(0);

        buf.extend_from_slice(&[127, 4, 0, 0, 0, 0]);

        let parsed = parse_smbios_buffer(&buf);
        assert!(parsed.is_some());
        let info = parsed.unwrap();
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
}
