#![cfg(dos_os)]
//! DOS (16-bit real mode and 32-bit protected mode) environment support for rustid.

use super::vendor::cyrix::Cyrix;
use super::{cpuid_cores_per_package, cpuid_threads_per_package};
use crate::common::{DataSource, Speed, TopologyTier};
use crate::x86::cpu::Cpu;
use core::arch::asm;
use core::fmt::Write;

pub mod allocator;
pub use allocator::init_heap;

pub mod args;
pub use args::*;

#[cfg(dos_real)]
pub mod cache;

pub mod fallback;
pub use fallback::*;

pub mod mp;

pub mod speed;

/// Enriches a CPU detected via pure CPUID with live DOS hardware information
/// (MP Table multi-socket counts and calibrated PIT/TSC frequency measurement).
pub fn enrich_cpu(cpu: &mut Cpu) {
    // 1. Multi-socket detection from MP Table
    let mp_table = mp::MpTable::detect();
    let mp_sockets = mp_table.socket_count();

    if mp_sockets > 1 {
        let sockets = TopologyTier::new(mp_sockets, DataSource::MpTable);
        cpu.topology.sockets = sockets;
        let cores = cpu
            .topology
            .cores
            .count
            .max(cpuid_cores_per_package() * mp_sockets);
        let threads = cpu
            .topology
            .threads
            .count
            .max(cpuid_threads_per_package() * mp_sockets);
        cpu.topology.cores =
            TopologyTier::new(cores, DataSource::Calculated("MP Table * CPUID cores"));
        cpu.topology.threads = TopologyTier::new(
            threads,
            DataSource::Calculated("MP Table logical processors"),
        );
        if let Some(ref mut cache) = cpu.topology.cache {
            cache.resolve_share_counts(cores, threads, mp_sockets);
        }
    }

    // 2. Calibrated PIT/TSC speed measurement fallback
    if cpu.topology.speed.base == 0 {
        let s = Speed::detect();
        if s.base > 0 {
            cpu.topology.speed = s;
            if !cpu.cores.is_empty() && cpu.cores[0].speed.is_none() {
                cpu.cores[0].speed = Some(s);
            }
        }
    }
}

/// Custom panic handler for no-std environments.
/// Loops indefinitely on panic to prevent undefined behavior.
#[cfg(not(test))]
#[cold]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    #[cfg(dos_ext)]
    if let Some(location) = _info.location() {
        crate::println!(
            "Panic in file '{}' at line {}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    } else {
        crate::println!("Panic for unknown reason.");
    }

    exit(1);
}

/// Prints a formatted string to the DOS console.
/// Supports both literal strings and format strings.
#[macro_export]
macro_rules! print {
    ($s:literal) => {
        $crate::x86::dos::_print_str($s)
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!(&mut $crate::x86::dos::DosWriter {}, $($arg)*);
        }
    };
}

/// Prints a formatted string followed by a newline to the DOS console.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($s:literal) => {
        {
            $crate::print!($s);
            $crate::print!("\r\n");
        }
    };
    ($($arg:tt)*) => {
        {
            $crate::print!($($arg)*);
            $crate::print!("\r\n");
        }
    };
}

#[cfg(dos_ext)]
static mut INITIAL_ATTR: u8 = 0x07;
#[cfg(dos_ext)]
static mut CURRENT_ATTR: u8 = 0x07;
#[cfg(dos_ext)]
static mut COLOR_ENABLED: bool = true;

/// Controls whether DOS console output renders ANSI color via VRAM or plain text via DOS stdout.
pub fn set_color_mode(enabled: bool) {
    #[cfg(dos_ext)]
    unsafe {
        COLOR_ENABLED = enabled;
    }
    #[cfg(not(dos_ext))]
    let _ = enabled;
}

/// Returns whether DOS console color output is currently enabled.
pub fn is_color_enabled() -> bool {
    #[cfg(dos_ext)]
    unsafe {
        COLOR_ENABLED
    }
    #[cfg(not(dos_ext))]
    true
}

#[cfg(dos_ext)]
fn is_stdout_redirected() -> bool {
    let dev_info: u16;
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x44_u8,
            in("al") 0x00_u8,
            in("bx") 1_u16, // STDOUT handle
            lateout("dx") dev_info,
            lateout("ax") _,
            options(preserves_flags)
        );
    }
    (dev_info & 0x80) == 0
}

#[cfg(dos_ext)]
fn get_cursor_pos() -> (u8, u8) {
    let page = (peek_u8(0x0462) & 0x07) as usize;
    let bda_addr = 0x0450 + (page * 2);
    let col = peek_u8(bda_addr as u32);
    let row = peek_u8((bda_addr + 1) as u32);
    (row, col)
}

#[cfg(dos_ext)]
fn set_cursor_pos(row: u8, col: u8) {
    let page = (peek_u8(0x0462) & 0x07) as u8;
    let bda_addr = 0x0450 + (page as usize * 2);
    unsafe {
        core::ptr::write_volatile(bda_addr as *mut u8, col);
        core::ptr::write_volatile((bda_addr + 1) as *mut u8, row);
        asm!(
            "int 0x10",
            in("ah") 0x02_u8,
            in("bh") page,
            in("dh") row,
            in("dl") col,
            lateout("ax") _,
            options(preserves_flags)
        );
    }
}

#[cfg(dos_ext)]
fn dos_console_write(s: &str) {
    let video_mode = peek_u8(0x0449);
    let page_offset = peek_u16(0x044E) as usize;
    let cols = {
        let c = peek_u16(0x044A) as usize;
        if (40..=132).contains(&c) { c } else { 80 }
    };
    let rows = {
        let r = peek_u8(0x0484) as usize;
        if (23..=59).contains(&r) { r + 1 } else { 25 }
    };
    let vram_base = if video_mode == 7 {
        (0x000B0000 + page_offset) as *mut u16
    } else {
        (0x000B8000 + page_offset) as *mut u16
    };

    let (mut row, mut col) = {
        let (r, c) = get_cursor_pos();
        ((r as usize).min(rows - 1), (c as usize).min(cols - 1))
    };

    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        // Parse ANSI escape sequence \x1b[...m
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // Consume '['
            let mut code = 0u32;
            let mut has_code = false;
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    code = code * 10 + (c as u32 - '0' as u32);
                    has_code = true;
                    chars.next();
                } else if c == ';' || c == 'm' {
                    chars.next();
                    let fg = match code {
                        0 => unsafe { INITIAL_ATTR & 0x0F },
                        30 => 0x00,      // Black
                        31 => 0x04,      // Red
                        32 | 92 => 0x0A, // Light Green
                        33 | 93 => 0x0E, // Yellow
                        34 => 0x09,      // Light Blue
                        35 | 95 => 0x0D, // Light Magenta
                        36 | 96 => 0x0B, // Light Cyan
                        37 => 0x07,      // Light Gray
                        90 => 0x08,      // Dark Gray
                        91 => 0x0C,      // Light Red
                        94 => 0x0B,      // Light Cyan
                        97 => 0x0F,      // High-Intensity White
                        _ => unsafe { INITIAL_ATTR & 0x0F },
                    };
                    unsafe {
                        CURRENT_ATTR = (INITIAL_ATTR & 0xF0) | fg;
                    }
                    if c == 'm' {
                        break;
                    }
                    code = 0;
                    has_code = false;
                } else {
                    break;
                }
            }
            if !has_code && chars.peek() == Some(&'m') {
                chars.next();
                unsafe {
                    CURRENT_ATTR = INITIAL_ATTR;
                }
            }
            continue;
        }

        match ch {
            '\r' => col = 0,
            '\n' => {
                col = 0;
                row += 1;
            }
            '\t' => {
                col = (col + 8) & !7;
            }
            _ => {
                let attr = unsafe { CURRENT_ATTR };
                let cell = ((attr as u16) << 8) | (ch as u16 & 0xFF);
                let offset = row * cols + col;
                if offset < rows * cols {
                    unsafe {
                        core::ptr::write_volatile(vram_base.add(offset), cell);
                    }
                }
                col += 1;
            }
        }

        if col >= cols {
            col = 0;
            row += 1;
        }

        if row >= rows {
            let line_words = cols;
            let total_words = (rows - 1) * cols;
            unsafe {
                core::ptr::copy(vram_base.add(line_words), vram_base, total_words);
                let attr = CURRENT_ATTR;
                let blank_cell = ((attr as u16) << 8) | (' ' as u16);
                let last_line = vram_base.add(total_words);
                for i in 0..cols {
                    core::ptr::write_volatile(last_line.add(i), blank_cell);
                }
            }
            row = rows - 1;
        }
    }

    set_cursor_pos(row as u8, col as u8);
}

#[cfg(dos_ext)]
fn write_redirected_str(s: &str) {
    let mut chars = s.chars().peekable();
    let mut buf = [0u8; 256];
    let mut buf_len = 0;

    let flush_buf = |buf: &mut [u8; 256], buf_len: &mut usize| {
        if *buf_len > 0 {
            let mut offset = 0;
            while offset < *buf_len {
                let chunk_size = (*buf_len - offset).min(32767);
                write_chunk(&buf[offset..offset + chunk_size]);
                offset += chunk_size;
            }
            *buf_len = 0;
        }
    };

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // Consume '['
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == 'm' {
                    break;
                }
            }
            continue;
        }

        let mut code_units = [0u8; 4];
        let encoded = ch.encode_utf8(&mut code_units);
        for &b in encoded.as_bytes() {
            buf[buf_len] = b;
            buf_len += 1;
            if buf_len >= buf.len() {
                flush_buf(&mut buf, &mut buf_len);
            }
        }
    }

    flush_buf(&mut buf, &mut buf_len);
}

/// Writes a string to the DOS console or redirected file.
pub fn _print_str(s: &str) {
    #[cfg(dos_ext)]
    {
        if s.is_empty() {
            return;
        }
        if !is_color_enabled() || is_stdout_redirected() {
            write_redirected_str(s);
        } else {
            dos_console_write(s);
        }
    }
    #[cfg(dos_real)]
    {
        for &b in s.as_bytes() {
            printc(b);
        }
    }
}

/// A writer implementation for DOS console output via the fmt::Write trait.
pub struct DosWriter;

impl Write for DosWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        _print_str(s);
        Ok(())
    }
}

/// Outputs a single character to the DOS console using INT 21h.
#[cfg(dos_real)]
#[inline(always)]
fn printc(ch: u8) {
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x02_u8,
            in("dl") ch,
            out("al") _,
            options(preserves_flags, nostack)
        );
    }
}

/// Writes a chunk of data to stdout using INT 21h, AH=40h (protected mode supported).
#[cfg(dos_ext)]
#[inline(always)]
fn write_chunk(data: &[u8]) {
    let len = data.len() as u16;
    if len == 0 {
        return;
    }
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x40_u8,
            in("bx") 1_u16, // File handle 1 = stdout
            in("ecx") len as u32,
            in("edx") data.as_ptr() as u32,
            lateout("eax") _,
            options(preserves_flags)
        );
    }
}

/// Exits the program and returns control to DOS/extender using INT 21h, AH=4Ch.
pub fn exit(code: u8) -> ! {
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x4C_u8,
            in("al") code,
            options(noreturn)
        )
    }
}

/// Reads a byte from conventional memory (Real Mode).
#[cfg(dos_real)]
#[inline(never)]
pub fn peek_u8(seg: u16, off: u16) -> u8 {
    let val: u16;
    unsafe {
        asm!(
            "push es",
            "mov es, {0:x}",
            "mov al, es:[bx]",
            "xor ah, ah",
            "pop es",
            in(reg) seg,
            in("bx") off,
            out("ax") val,
            options(preserves_flags)
        );
    }
    val as u8
}

/// Reads a 16-bit word from conventional memory (Real Mode).
#[cfg(dos_real)]
#[inline(never)]
pub fn peek_u16(seg: u16, off: u16) -> u16 {
    let val: u16;
    unsafe {
        asm!(
            "push es",
            "mov es, {0:x}",
            "mov ax, es:[bx]",
            "pop es",
            in(reg) seg,
            in("bx") off,
            out("ax") val,
            options(preserves_flags)
        );
    }
    val
}

/// Reads a byte from a 32-bit linear address (Protected Mode).
#[cfg(dos_ext)]
#[inline(always)]
pub fn peek_u8(addr: u32) -> u8 {
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}

/// Reads a 16-bit word from a 32-bit linear address (Protected Mode).
#[cfg(dos_ext)]
#[inline(always)]
pub fn peek_u16(addr: u32) -> u16 {
    unsafe { core::ptr::read_volatile(addr as *const u16) }
}

/// Reads a 32-bit dword from a 32-bit linear address (Protected Mode).
#[cfg(dos_ext)]
#[inline(always)]
pub fn peek_u32(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
