//! MultiProcessor (MP) table detection for x86 systems.
//!
//! This module implements scanning and parsing of the Intel MP specification
//! tables to determine multi-processor topology (sockets, cores).
//! It uses BIOS interrupts where possible to avoid unsafe memory scanning
//! can hang in 16-bit real mode.

use crate::dos::{peek_u8, peek_u16};
use core::arch::asm;

/// MP Floating Pointer Structure signature: "_MP_"
const MP_SIGNATURE: [u8; 4] = *b"_MP_";

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct MpFloatingPointer {
    pub signature: [u8; 4],
    pub config_table_ptr: u32,
    pub length: u8,
    pub spec_rev: u8,
    pub checksum: u8,
    pub mp_feature1: u8,
    pub mp_feature2: u8,
    pub mp_feature3: u8,
    pub mp_feature4: u8,
    pub mp_feature5: u8,
}

#[derive(Debug, Default)]
pub struct MpTable {
    pub sockets: usize,
}

impl MpTable {
    pub fn detect() -> Self {
        #[cfg(target_os = "none")]
        {
            Self::detect_dos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::detect_linux()
        }
    }

    pub fn detect_linux() -> Self {
        unimplemented!();
    }

    pub fn detect_dos() -> Self {
        let mut table = MpTable { sockets: 1 };

        // Try BIOS interrupt first (Intel MP Spec BIOS extensions)
        if let Some((_fp_ptr, config_ptr)) = Self::get_config_via_bios() {
            if config_ptr != 0 {
                if let Some(count) = Self::parse_config_table(config_ptr) {
                    table.sockets = count;
                    return table;
                }
            }
        }

        // Fallback: Scan memory ranges safely
        if let Some(mpfp) = Self::find_mpfp() {
            if mpfp.config_table_ptr != 0 {
                if let Some(count) = Self::parse_config_table(mpfp.config_table_ptr) {
                    table.sockets = count;
                }
            } else if mpfp.mp_feature1 != 0 {
                // Default configurations (1-7) all have 2 CPUs
                table.sockets = 2;
            }
        }

        table
    }

    pub fn socket_count(&self) -> usize {
        self.sockets
    }

    /// Uses INT 15h, AX=D100h to get MP configuration pointers.
    /// This is supported by many MP-compliant BIOSes.
    #[inline(never)]
    fn get_config_via_bios() -> Option<(u32, u32)> {
        let fp_ptr: u32;
        let config_ptr: u32;
        let flags: u16;

        unsafe {
            asm!(
                "push ds",
                "push es",
                "push esi",
                "push edi",
                "mov eax, 0xD100",
                "int 0x15",
                "pushf",
                "pop {0:x}",
                "pop edi",
                "pop esi",
                "pop es",
                "pop ds",
                out(reg) flags,
                lateout("ebx") fp_ptr,
                lateout("ecx") config_ptr,
                out("eax") _,
                out("edx") _,
            );
        }

        if (flags & 1) == 0 {
            Some((fp_ptr, config_ptr))
        } else {
            None
        }
    }

    #[inline(never)]
    fn check_sig(seg: u16, off: u16, sig: &[u8; 4]) -> bool {
        peek_u8(seg, off) == sig[0]
            && peek_u8(seg, off + 1) == sig[1]
            && peek_u8(seg, off + 2) == sig[2]
            && peek_u8(seg, off + 3) == sig[3]
    }

    #[inline(never)]
    fn parse_config_table(config_ptr: u32) -> Option<usize> {
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
