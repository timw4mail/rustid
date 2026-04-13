//! MultiProcessor (MP) table detection for x86 systems.
//!
//! This module implements scanning and parsing of the Intel MP specification
//! tables to determine multi-processor topology (sockets, cores).

/// MultiProcessor (MP) table information for multi-socket systems.
#[derive(Debug, Default)]
pub struct MpTable {
    /// Number of processor sockets
    pub sockets: u32,
}

impl MpTable {
    /// Returns the number of processor sockets.
    pub fn socket_count(&self) -> u32 {
        self.sockets
    }

    /// Detects the number of sockets by reading the specified file.
    #[cfg(not(target_os = "none"))]
    pub fn detect_file(file: &str) -> MpTable {
        use std::collections::HashSet;

        let mut table = MpTable { sockets: 1 };

        // Fallback: /proc/cpuinfo unique physical ids
        if let Ok(content) = std::fs::read_to_string(file) {
            let mut entries = 0;
            let mut physical_ids = HashSet::new();
            let mut core_ids = HashSet::new();

            for line in content.lines() {
                if line.starts_with("physical id")
                    && let Some(id) = line.split(':').nth(1)
                {
                    physical_ids.insert(id.trim());
                    entries += 1;
                }

                if line.starts_with("core id")
                    && let Some(id) = line.split(':').nth(1)
                {
                    core_ids.insert(id.trim());
                }
            }

            // For the Pentium Pro, all the rules seem to be broken.
            // There might be multiple entries in /proc/cpuinfo, all with identical ids
            if physical_ids.len() == 1 && core_ids.len() == 1 && entries != 1 {
                table.sockets = entries;
            } else {
                table.sockets = physical_ids.len() as u32;
            }
        }

        table
    }
}

#[cfg(not(any(target_os = "none", target_os = "linux")))]
impl MpTable {
    pub fn detect() -> MpTable {
        MpTable { sockets: 1 }
    }
}

#[cfg(target_os = "linux")]
impl MpTable {
    /// Detects the number of sockets on Linux by parsing /proc/cpuinfo.
    pub fn detect() -> MpTable {
        Self::detect_file("/proc/cpuinfo")
    }
}

/// MP Floating Pointer Structure signature: "_MP_"
#[cfg(target_os = "none")]
const MP_SIGNATURE: [u8; 4] = *b"_MP_";

/// MP Floating Pointer Structure from the Intel MP Specification.
#[repr(C, packed)]
#[derive(Copy, Clone)]
#[cfg(target_os = "none")]
struct MpFloatingPointer {
    /// Structure signature ("_MP_")
    signature: [u8; 4],
    /// Physical address of the configuration table
    config_table_ptr: u32,
    /// Length of this structure (in bytes)
    length: u8,
    /// MP specification revision
    spec_rev: u8,
    /// Checksum of this structure
    checksum: u8,
    /// MP feature byte 1
    mp_feature1: u8,
    /// MP feature byte 2
    mp_feature2: u8,
    /// MP feature byte 3
    mp_feature3: u8,
    /// MP feature byte 4
    mp_feature4: u8,
    /// MP feature byte 5
    mp_feature5: u8,
}

#[cfg(target_os = "none")]
impl MpTable {
    /// Detects the number of sockets using the Intel MP Specification.
    pub fn detect() -> MpTable {
        let mut table = MpTable { sockets: 1 };

        // MP Table lookup is only applicable to certain CPUs
        if !(super::is_intel() || super::is_vortex() || super::is_centaur()) {
            return table;
        }

        // Fallback: Scan memory ranges safely
        if let Some(mpfp) = Self::find_mpfp() {
            if mpfp.config_table_ptr != 0
                && let Some(count) = Self::parse_config_table(mpfp.config_table_ptr)
            {
                table.sockets = count;
            } else if mpfp.mp_feature1 != 0 {
                // Default configurations (1-7) all have 2 CPUs
                table.sockets = 2;
            }
        }

        table
    }

    #[inline(never)]
    fn check_sig(seg: u16, off: u16, sig: &[u8; 4]) -> bool {
        use crate::cpuid::dos::peek_u8;

        peek_u8(seg, off) == sig[0]
            && peek_u8(seg, off + 1) == sig[1]
            && peek_u8(seg, off + 2) == sig[2]
            && peek_u8(seg, off + 3) == sig[3]
    }

    #[inline(never)]
    fn parse_config_table(config_ptr: u32) -> Option<u32> {
        use crate::cpuid::dos::{peek_u8, peek_u16};

        if config_ptr == 0 || config_ptr > 0xFFF00 {
            return None;
        }

        let seg = (config_ptr >> 4) as u16;
        let off = (config_ptr & 0xF) as u16;

        if !Self::check_sig(seg, off, b"PCMP") {
            return None;
        }

        let entry_count = peek_u16(seg, off + 34);
        let mut sockets = 0;
        let mut current_off = off + 44;

        for _ in 0..entry_count {
            if current_off > 0xFFF0 {
                break;
            }
            let entry_type = peek_u8(seg, current_off);
            if entry_type == 0 {
                let flags = peek_u8(seg, current_off + 3);
                if (flags & 0x01) != 0 {
                    sockets += 1;
                }
                current_off += 20;
            } else {
                current_off += 8;
            }
        }

        if sockets > 0 { Some(sockets) } else { None }
    }

    #[inline(never)]
    fn find_mpfp() -> Option<MpFloatingPointer> {
        if let Some(ebda_seg) = Self::get_ebda_seg() {
            if let Some(fp) = Self::scan_range(ebda_seg, 0, 1024) {
                return Some(fp);
            }
        }

        if let Some(fp) = Self::scan_range(0x9FC0, 0, 1024) {
            return Some(fp);
        }

        if let Some(fp) = Self::scan_range(0xF000, 0, 0xFFFF) {
            return Some(fp);
        }

        None
    }

    #[inline(never)]
    fn scan_range(seg: u16, start_off: u16, length: u16) -> Option<MpFloatingPointer> {
        use crate::cpuid::dos::peek_u8;

        for off in (start_off..(start_off.saturating_add(length))).step_by(16) {
            if Self::check_sig(seg, off, &MP_SIGNATURE) {
                let mut bytes = [0u8; 16];
                let mut sum: u8 = 0;
                for (i, b) in bytes.iter_mut().enumerate() {
                    let val = peek_u8(seg, off + i as u16);
                    *b = val;
                    sum = sum.wrapping_add(val);
                }

                if sum == 0 {
                    return Some(MpFloatingPointer {
                        signature: MP_SIGNATURE,
                        config_table_ptr: u32::from_le_bytes([
                            bytes[4], bytes[5], bytes[6], bytes[7],
                        ]),
                        length: bytes[8],
                        spec_rev: bytes[9],
                        checksum: bytes[10],
                        mp_feature1: bytes[11],
                        mp_feature2: bytes[12],
                        mp_feature3: bytes[13],
                        mp_feature4: bytes[14],
                        mp_feature5: bytes[15],
                    });
                }
            }
        }
        None
    }

    #[inline(never)]
    fn get_ebda_seg() -> Option<u16> {
        use crate::cpuid::dos::peek_u16;
        use core::arch::asm;

        let es_val: u16;
        let flags: u16;
        unsafe {
            asm!(
                "push ds",
                "push es",
                "push esi",
                "push edi",
                "mov eax, 0xC100",
                "int 0x15",
                "pushf",
                "pop {0:x}",
                "mov {1:x}, es",
                "pop edi",
                "pop esi",
                "pop es",
                "pop ds",
                out(reg) flags,
                out(reg) es_val,
                out("eax") _,
            );
        }

        if (flags & 1) == 0 {
            Some(es_val)
        } else {
            let seg = peek_u16(0x0040, 0x000E);
            if seg != 0 { Some(seg) } else { None }
        }
    }
}
