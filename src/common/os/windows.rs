use crate::common::{
    DataSource, OS, TDetect, TOSData, TopologyCount, TopologyTier, is_generic_value,
    is_known_hypervisor_vendor,
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

/// Helper function to read a string registry value (REG_SZ or REG_EXPAND_SZ).
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

    let wchar_count = (size as usize + 1) / 2;
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
    if res.is_err() || (dw_type != REG_SZ && dw_type != REG_EXPAND_SZ) {
        return None;
    }

    while let Some(&0) = buf.last() {
        buf.pop();
    }
    let s = String::from_utf16_lossy(&buf);
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

impl TOSData for OS {
    fn get_system_name() -> Option<String> {
        let mut hkey = HKEY::default();
        let subkey = w!(r"HARDWARE\DESCRIPTION\System\BIOS");
        let result =
            unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, None, KEY_READ, &mut hkey) };

        if result.is_ok() {
            let product_name = read_reg_string(hkey, "SystemProductName");
            let manufacturer = read_reg_string(hkey, "SystemManufacturer");
            let board_vendor = read_reg_string(hkey, "BaseBoardManufacturer");
            let board_product = read_reg_string(hkey, "BaseBoardProduct");

            let _ = unsafe { RegCloseKey(hkey) };

            if let Some(prod) = product_name
                && !is_generic_value(&prod)
            {
                if let Some(mfr) = manufacturer
                    && !is_generic_value(&mfr)
                {
                    if is_known_hypervisor_vendor(&mfr)
                        || !prod.to_lowercase().contains(&mfr.to_lowercase())
                    {
                        return Some(format!("{mfr} {prod}"));
                    }
                }
                return Some(prod);
            }

            if let Some(board_prod) = board_product
                && !is_generic_value(&board_prod)
            {
                if let Some(board_mfr) = board_vendor
                    && !is_generic_value(&board_mfr)
                    && !board_prod
                        .to_lowercase()
                        .contains(&board_mfr.to_lowercase())
                {
                    return Some(format!("{board_mfr} {board_prod}"));
                }
                return Some(board_prod);
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
                    return TopologyTier::new(package_count, DataSource::WindowsRegistry);
                }
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
            source: DataSource::WindowsRegistry,
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
            source: DataSource::WindowsRegistry,
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
                    CacheData => CacheType::Data,
                    CacheInstruction => CacheType::Instruction,
                    CacheUnified => CacheType::Unified,
                    _ => {
                        offset += item.Size as usize;
                        continue;
                    }
                };
                let mask = cache_rel.GroupMask.Mask;
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
}
