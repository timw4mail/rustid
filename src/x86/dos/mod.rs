#![cfg(dos_os)]
//! DOS (16-bit real mode and 32-bit protected mode) environment support for rustid.

use super::vendor::cyrix::Cyrix;
use crate::common::{DataSource, Speed, TopologyTier};
use crate::x86::cpu::Cpu;
use crate::x86::{cpuid_cores_per_package, cpuid_threads_per_package};
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
        cpu.extra.topology.sockets = sockets;
        let cores = cpu
            .extra
            .topology
            .cores
            .count
            .max(cpuid_cores_per_package() * mp_sockets);
        let threads = cpu
            .extra
            .topology
            .threads
            .count
            .max(cpuid_threads_per_package() * mp_sockets);
        cpu.extra.topology.cores = TopologyTier::new(
            cores,
            DataSource::Calculated("MP Table sockets * CPUID cores"),
        );
        cpu.extra.topology.threads = TopologyTier::new(
            threads,
            DataSource::Calculated("MP Table sockets * CPUID threads"),
        );
        if let Some(ref mut cache) = cpu.extra.topology.cache {
            cache.resolve_share_counts(cores, threads, mp_sockets);
        }
    }

    // 2. Calibrated PIT/TSC speed measurement fallback
    if cpu.extra.topology.speed.base == 0 {
        let s = Speed::detect();
        if s.base > 0 {
            cpu.extra.topology.speed = s;
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
    use crate::println;

    #[cfg(dos_ext)]
    if let Some(location) = _info.location() {
        println!(
            "Panic in file '{}' at line {}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    } else {
        println!("Panic for unknown reason.");
    }

    #[cfg(dos_real)]
    println!("Panic!");
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

/// Writes a string to the DOS console.
pub fn _print_str(s: &str) {
    #[cfg(dos_ext)]
    {
        if s.is_empty() {
            return;
        }
        let bytes = s.as_bytes();
        let mut offset = 0;
        while offset < bytes.len() {
            let chunk_size = (bytes.len() - offset).min(32767);
            write_chunk(&bytes[offset..offset + chunk_size]);
            offset += chunk_size;
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
