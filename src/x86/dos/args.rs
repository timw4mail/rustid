#![cfg(any(dos, dos32a))]
// ============================================================================
// Command-line arguments parsing (DOS Real Mode & Protected Mode)
// ============================================================================

use super::*;

const MAX_ARGS: usize = 8;

pub struct Args {
    tokens: [&'static str; MAX_ARGS],
    count: usize,
}

impl Args {
    #[must_use]
    pub fn as_slice(&self) -> &[&'static str] {
        &self.tokens[..self.count]
    }
}

static mut TAIL_BUF: [u8; 128] = [0; 128];

#[cfg(dos32a)]
pub fn selector_base(selector: u16) -> Option<u32> {
    let mut base_high: u16 = 0;
    let mut base_low: u16 = 0;
    let mut err_or_carry: u16 = 0x0006;
    unsafe {
        asm!(
            "int 0x31",
            "jc 2f",
            "xor ax, ax",
            "2:",
            inout("ax") err_or_carry,
            in("bx") selector,
            out("cx") base_high,
            out("dx") base_low,
        );
    }
    if err_or_carry == 0 {
        Some(((base_high as u32) << 16) | (base_low as u32))
    } else {
        None
    }
}

#[cfg(dos32a)]
pub fn psp_base() -> Option<u32> {
    let mut psp_val: u32 = 0;
    unsafe {
        asm!(
            "mov ah, 0x51",
            "int 0x21",
            out("ebx") psp_val,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }

    let psp_val = psp_val as u16;
    if psp_val < 0x0400 {
        selector_base(psp_val)
    } else {
        Some((psp_val as u32) << 4)
    }
}

#[cfg(dos)]
pub fn get_args() -> Args {
    let mut tokens = [""; MAX_ARGS];
    let mut count = 0;

    let psp_seg: u16;
    unsafe {
        asm!(
            "mov ah, 0x51",
            "int 0x21",
            out("bx") psp_seg,
            out("ax") _,
            out("cx") _,
            out("dx") _,
        );
    }

    let raw_len = peek_u8(psp_seg, 0x80);
    let len = (raw_len as usize).min(127);
    for i in 0..len {
        unsafe {
            TAIL_BUF[i] = peek_u8(psp_seg, 0x81 + i as u16);
        }
    }

    let tail: &'static str = unsafe {
        let bytes = &TAIL_BUF[..len];
        let trimmed = if bytes.last() == Some(&0x0D) {
            &bytes[..bytes.len() - 1]
        } else {
            bytes
        };
        core::str::from_utf8_unchecked(trimmed)
    };

    for token in tail.split(|c: char| c == ' ' || c == '\t') {
        let t = token.trim();
        if !t.is_empty() && count < MAX_ARGS {
            tokens[count] = t;
            count += 1;
        }
    }

    Args { tokens, count }
}

#[cfg(dos32a)]
pub fn get_args() -> Args {
    let mut tokens = [""; MAX_ARGS];
    let mut count = 0;

    let mut psp_val: u32 = 0;
    unsafe {
        asm!(
            "mov ah, 0x51",
            "int 0x21",
            out("ebx") psp_val,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }

    let psp_sel = psp_val as u16;

    let base = if psp_sel < 0x0400 {
        (psp_sel as u32) << 4
    } else {
        selector_base(psp_sel).unwrap_or((psp_sel as u32) << 4)
    };

    let raw_len = peek_u8(base + 0x80);
    let len = (raw_len as usize).min(127);
    for i in 0..len {
        unsafe {
            TAIL_BUF[i] = peek_u8(base + 0x81 + i as u32);
        }
    }

    let tail: &'static str = unsafe {
        let bytes = &TAIL_BUF[..len];
        let trimmed = if bytes.last() == Some(&0x0D) {
            &bytes[..bytes.len() - 1]
        } else {
            bytes
        };
        core::str::from_utf8_unchecked(trimmed)
    };

    for token in tail.split(|c: char| c == ' ' || c == '\t') {
        let t = token.trim();
        if !t.is_empty() && count < MAX_ARGS {
            tokens[count] = t;
            count += 1;
        }
    }

    Args { tokens, count }
}

/// Loads and executes a DOS binary via INT 21h, AH=4B00h (DOS EXEC).
/// Allocates low conventional memory (< 1MB) via DPMI AX=0100h and simulates
/// real-mode INT 21h execution via DPMI AX=0300h.
pub fn exec_dos_binary(prog: &str, cmd_tail_str: &str) -> Result<(), u16> {
    let mut rm_seg: u16 = 0;
    let mut pm_sel: u16 = 0;
    let mut carry: u8;

    unsafe {
        asm!(
            "mov ax, 0x0100",
            "mov bx, 128",
            "int 0x31",
            "setc {carry}",
            out("ax") rm_seg,
            out("bx") _,
            out("dx") pm_sel,
            carry = out(reg_byte) carry,
        );
    }

    if carry != 0 {
        return Err(8);
    }

    let base = (rm_seg as u32) << 4;

    for i in 0..64 {
        unsafe {
            core::ptr::write_volatile((base + i) as *mut u8, 0);
        }
    }

    let prog_bytes = prog.as_bytes();
    if prog_bytes.len() >= 16 {
        unsafe {
            asm!("mov ax, 0x0101", "int 0x31", in("dx") pm_sel, out("ax") _);
        }
        return Err(1);
    }
    for (i, &b) in prog_bytes.iter().enumerate() {
        unsafe {
            core::ptr::write_volatile((base + i as u32) as *mut u8, b);
        }
    }

    let tail_bytes = cmd_tail_str.as_bytes();
    let tail_len = tail_bytes.len().min(14);
    unsafe {
        core::ptr::write_volatile((base + 0x10) as *mut u8, tail_len as u8);
    }
    for (i, &b) in tail_bytes[..tail_len].iter().enumerate() {
        unsafe {
            core::ptr::write_volatile((base + 0x11 + i as u32) as *mut u8, b);
        }
    }
    unsafe {
        core::ptr::write_volatile((base + 0x11 + tail_len as u32) as *mut u8, b'\r');
    }

    unsafe {
        core::ptr::write_volatile((base + 0x22) as *mut u16, 0x0010);
        core::ptr::write_volatile((base + 0x24) as *mut u16, rm_seg);
    }

    #[repr(C, packed)]
    struct DpmiRealModeCall {
        edi: u32,
        esi: u32,
        ebp: u32,
        reserved: u32,
        ebx: u32,
        edx: u32,
        ecx: u32,
        eax: u32,
        flags: u16,
        es: u16,
        ds: u16,
        fs: u16,
        gs: u16,
        ip: u16,
        cs: u16,
        sp: u16,
        ss: u16,
    }

    let mut rms = DpmiRealModeCall {
        edi: 0,
        esi: 0,
        ebp: 0,
        reserved: 0,
        ebx: 0x0020,
        edx: 0x0000,
        ecx: 0,
        eax: 0x4B00,
        flags: 0,
        es: rm_seg,
        ds: rm_seg,
        fs: 0,
        gs: 0,
        ip: 0,
        cs: 0,
        sp: 0x0800,
        ss: rm_seg,
    };

    let mut dpmi_carry: u32;
    let rms_ptr = &mut rms as *mut DpmiRealModeCall as u32;

    unsafe {
        asm!(
            "push ds",
            "pop es",
            "push esi",
            "mov ax, 0x0300",
            "mov bl, 0x21",
            "mov bh, 0",
            "mov cx, 0",
            "int 0x31",
            "setc al",
            "movzx eax, al",
            "pop esi",
            in("edi") rms_ptr,
            out("eax") dpmi_carry,
            out("ebx") _,
            out("ecx") _,
            out("edx") _,
        );
    }

    unsafe {
        asm!("mov ax, 0x0101", "int 0x31", in("dx") pm_sel, out("ax") _);
    }

    if dpmi_carry != 0 || (rms.flags & 1) != 0 {
        Err(rms.eax as u16)
    } else {
        exit(0);
    }
}
