#![cfg(dos_os)]
//! MultiProcessor (MP) table detection for x86 systems.
//!
//! This module implements scanning and parsing of the Intel MP specification
//! tables to determine multi-processor topology (sockets, cores).

/// MultiProcessor (MP) table information for multi-socket systems.
#[derive(Debug)]
pub struct MpTable {
    /// Number of enabled processors (logical cores/threads)
    pub processors: u32,
}

impl Default for MpTable {
    fn default() -> MpTable {
        MpTable { processors: 1 }
    }
}

impl MpTable {
    /// Returns the total number of enabled logical processors (threads) found in MP Table.
    #[must_use]
    pub fn processor_count(&self) -> u32 {
        self.processors
    }

    /// Returns the detected socket count based on total logical processors and CPUID threads per package.
    #[must_use]
    pub fn socket_count(&self) -> u32 {
        let threads_per_pkg = crate::x86::cpuid_threads_per_package().max(1);
        (self.processors / threads_per_pkg).max(1)
    }

    /// Returns the total physical core count across all sockets.
    #[must_use]
    pub fn total_cores(&self) -> u32 {
        let cores_per_pkg = crate::x86::cpuid_cores_per_package().max(1);
        let sockets = self.socket_count();
        cores_per_pkg * sockets
    }

    /// Returns the total logical thread count across all sockets.
    #[must_use]
    pub fn total_threads(&self) -> u32 {
        let threads_per_pkg = crate::x86::cpuid_threads_per_package().max(1);
        self.processors.max(threads_per_pkg)
    }
}

/// MP Floating Pointer Structure signature: "_MP_"
const MP_SIGNATURE: [u8; 4] = *b"_MP_";

/// MP Floating Pointer Structure from the Intel MP Specification.
#[repr(C, packed)]
#[derive(Copy, Clone)]
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

#[cfg(dos_real)]
#[inline(always)]
fn peek_u8_so(seg: u16, off: u16) -> u8 {
    crate::x86::dos::peek_u8(seg, off)
}

#[cfg(dos_real)]
#[inline(always)]
fn peek_u16_so(seg: u16, off: u16) -> u16 {
    crate::x86::dos::peek_u16(seg, off)
}

#[cfg(dos_ext)]
#[inline(always)]
fn peek_u8_so(seg: u16, off: u16) -> u8 {
    let addr = ((seg as u32) << 4) + (off as u32);
    crate::x86::dos::peek_u8(addr)
}

#[cfg(dos_ext)]
#[inline(always)]
fn peek_u16_so(seg: u16, off: u16) -> u16 {
    let addr = ((seg as u32) << 4) + (off as u32);
    crate::x86::dos::peek_u16(addr)
}

impl MpTable {
    /// Detects the number of sockets using the Intel MP Specification.
    pub fn detect() -> MpTable {
        let mut table = MpTable { processors: 1 };

        if let Some(mpfp) = Self::find_mpfp() {
            if mpfp.config_table_ptr != 0
                && let Some(count) = Self::parse_config_table(mpfp.config_table_ptr)
            {
                table.processors = count;
            } else if mpfp.mp_feature1 != 0 {
                // Default configurations (1-7) all have 2 CPUs
                table.processors = 2;
            }
        }

        table
    }

    #[inline(never)]
    fn check_sig(seg: u16, off: u16, sig: &[u8; 4]) -> bool {
        peek_u8_so(seg, off) == sig[0]
            && peek_u8_so(seg, off + 1) == sig[1]
            && peek_u8_so(seg, off + 2) == sig[2]
            && peek_u8_so(seg, off + 3) == sig[3]
    }

    #[inline(never)]
    fn parse_config_table(config_ptr: u32) -> Option<u32> {
        if config_ptr == 0 || config_ptr > 0xFFF00 {
            return None;
        }

        let seg = (config_ptr >> 4) as u16;
        let off = (config_ptr & 0xF) as u16;

        if !Self::check_sig(seg, off, b"PCMP") {
            return None;
        }

        let mut buf = [0u8; 1024];
        for (i, b) in buf.iter_mut().enumerate() {
            if (off as usize + i) > 0xFFFF {
                break;
            }
            *b = peek_u8_so(seg, off + i as u16);
        }

        Self::parse_pcmp_slice(&buf)
    }

    /// Parses a PCMP configuration table buffer and returns the number of enabled processors.
    pub fn parse_pcmp_slice(bytes: &[u8]) -> Option<u32> {
        if bytes.len() < 44 || &bytes[0..4] != b"PCMP" {
            return None;
        }

        let entry_count = u16::from_le_bytes([bytes[34], bytes[35]]);
        let mut processors = 0;
        let mut current_off = 44;

        for _ in 0..entry_count {
            if current_off >= bytes.len() {
                break;
            }
            let entry_type = bytes[current_off];
            if entry_type == 0 {
                if current_off + 3 >= bytes.len() {
                    break;
                }
                let flags = bytes[current_off + 3];
                if (flags & 0x01) != 0 {
                    processors += 1;
                }
                current_off += 20;
            } else {
                current_off += 8;
            }
        }

        if processors > 0 {
            Some(processors)
        } else {
            None
        }
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
        for off in (start_off..(start_off.saturating_add(length))).step_by(16) {
            if Self::check_sig(seg, off, &MP_SIGNATURE) {
                let mut bytes = [0u8; 16];
                let mut sum: u8 = 0;
                for (i, b) in bytes.iter_mut().enumerate() {
                    let val = peek_u8_so(seg, off + i as u16);
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
        #[cfg(dos_real)]
        {
            let mut es_val: u16 = 0;
            let mut flags: u16 = 1; // Set carry flag to force fallback

            unsafe {
                core::arch::asm!(
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
                return Some(es_val);
            }
        }

        let seg = peek_u16_so(0x0040, 0x000E);
        if seg != 0 { Some(seg) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp_table_default() {
        let mp = MpTable::default();
        assert_eq!(mp.processor_count(), 1);
    }

    #[test]
    fn test_parse_pcmp_slice_quad_core() {
        let mut data = [0u8; 124];
        // Signature "PCMP"
        data[0..4].copy_from_slice(b"PCMP");
        // Entry count: 4
        data[34..36].copy_from_slice(&4u16.to_le_bytes());

        // 4 processor entries starting at offset 44 (20 bytes each)
        for i in 0..4 {
            let off = 44 + i * 20;
            data[off] = 0; // Entry type 0: Processor
            data[off + 1] = i as u8; // APIC ID
            data[off + 3] = 0x01; // Flags: Enabled (bit 0 = 1)
        }

        assert_eq!(MpTable::parse_pcmp_slice(&data), Some(4));
    }

    #[test]
    fn test_parse_pcmp_slice_with_disabled_core() {
        let mut data = [0u8; 124];
        data[0..4].copy_from_slice(b"PCMP");
        data[34..36].copy_from_slice(&4u16.to_le_bytes());

        for i in 0..4 {
            let off = 44 + i * 20;
            data[off] = 0; // Processor entry
            data[off + 3] = if i == 3 { 0x00 } else { 0x01 }; // 4th processor is disabled
        }

        assert_eq!(MpTable::parse_pcmp_slice(&data), Some(3));
    }

    #[test]
    fn test_parse_pcmp_slice_mixed_entries() {
        let mut data = [0u8; 92];
        data[0..4].copy_from_slice(b"PCMP");
        data[34..36].copy_from_slice(&3u16.to_le_bytes());

        // Entry 0: Processor (20 bytes)
        data[44] = 0;
        data[47] = 0x01;

        // Entry 1: Processor (20 bytes)
        data[64] = 0;
        data[67] = 0x01;

        // Entry 2: Bus entry (Type 1, 8 bytes)
        data[84] = 1;

        assert_eq!(MpTable::parse_pcmp_slice(&data), Some(2));
    }

    #[test]
    fn test_parse_pcmp_slice_invalid_signature() {
        let mut data = [0u8; 64];
        data[0..4].copy_from_slice(b"INVALID");
        assert_eq!(MpTable::parse_pcmp_slice(&data), None);
    }

    #[test]
    fn test_parse_pcmp_large_table_16_processors() {
        let mut data = [0u8; 512];
        data[0..4].copy_from_slice(b"PCMP");
        // 16 processor entries + 4 bus entries = 20 entries
        data[34..36].copy_from_slice(&20u16.to_le_bytes());

        for i in 0..16 {
            let off = 44 + i * 20;
            data[off] = 0; // Processor
            data[off + 1] = i as u8; // APIC ID
            data[off + 3] = 0x01; // Enabled
        }

        // 4 bus entries (Type 1, 8 bytes)
        for i in 0..4 {
            let off = 44 + 16 * 20 + i * 8;
            data[off] = 1;
        }

        assert_eq!(MpTable::parse_pcmp_slice(&data), Some(16));
    }

    #[test]
    fn test_mp_table_topology_calculations() {
        let mp_single = MpTable { processors: 1 };
        assert_eq!(mp_single.processor_count(), 1);
        assert_eq!(mp_single.socket_count(), 1);
        assert_eq!(mp_single.total_cores(), 1);
        assert_eq!(mp_single.total_threads(), 1);

        let mp_dual = MpTable { processors: 2 };
        assert_eq!(mp_dual.processor_count(), 2);
        assert_eq!(mp_dual.socket_count(), 2);
        assert_eq!(mp_dual.total_cores(), 2);
        assert_eq!(mp_dual.total_threads(), 2);

        let mp_quad = MpTable { processors: 4 };
        assert_eq!(mp_quad.processor_count(), 4);
        assert_eq!(mp_quad.socket_count(), 4);
        assert_eq!(mp_quad.total_cores(), 4);
        assert_eq!(mp_quad.total_threads(), 4);
    }
}
