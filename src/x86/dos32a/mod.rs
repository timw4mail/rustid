#![cfg(dos32a)]
//! DOS/32A 32-bit protected mode support for rustid.
//!
//! This module provides DOS/32A-specific implementations including console output
//! via protected mode services and a custom panic handler for bare-metal environments.

use super::vendor::cyrix::Cyrix;
use crate::common::Speed;
use core::arch::asm;
use core::fmt::Write;

pub mod allocator;
pub use allocator::init_heap;

/// Custom panic handler for no-std environments.
/// Loops indefinitely on panic to prevent undefined behavior.
#[cfg(not(test))]
#[cold]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use crate::println;
    println!("Panic!");
    exit(1);
}

/// Prints a formatted string to the DOS console.
/// Supports both literal strings and format strings.
#[macro_export]
macro_rules! print {
    ($s:literal) => {
        $crate::x86::dos32a::_print_str($s)
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!(&mut $crate::x86::dos32a::DosWriter {}, $($arg)*);
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

/// Writes a string to the DOS console using DOS/32A protected mode services.
pub fn _print_str(s: &str) {
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

/// A writer implementation for DOS console output via the fmt::Write trait.
pub struct DosWriter;

impl Write for DosWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        _print_str(s);
        Ok(())
    }
}

/// Writes a chunk of data to stdout using INT 21h, AH=40h (protected mode supported).
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

/// Exits the program and returns control to DOS/32A using INT 21h, AH=4Ch.
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

/// Reads a byte from a 32-bit linear address.
#[inline(never)]
pub fn peek_u8(addr: u32) -> u8 {
    unsafe { *(addr as *const u8) }
}

/// Reads a 16-bit word from a 32-bit linear address.
#[inline(never)]
pub fn peek_u16(addr: u32) -> u16 {
    unsafe { *(addr as *const u16) }
}

/// Reads a 32-bit dword from a 32-bit linear address.
#[inline(never)]
pub fn peek_u32(addr: u32) -> u32 {
    unsafe { *(addr as *const u32) }
}

/// Maximum number of command-line arguments supported.
const MAX_ARGS: usize = 8;

/// A tokenized view of the DOS command-line tail.
///
/// Tokens are split on whitespace (space / tab). Each `&'static str` slice
/// points directly into the PSP region, which is valid for the program lifetime.
pub struct Args {
    tokens: [&'static str; MAX_ARGS],
    count: usize,
}

impl Args {
    /// Returns a slice of the parsed argument tokens.
    #[inline]
    pub fn as_slice(&self) -> &[&'static str] {
        &self.tokens[..self.count]
    }
}

/// Returns the 32-bit linear base address of a selector by calling
/// DPMI `INT 31h AX=0006h`. Returns `None` if the call fails.
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

static mut TAIL_BUF: [u8; 128] = [0; 128];

/// Returns the flat linear base address of the PSP.
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

/// Reads the DOS command-line tail from the PSP and tokenizes it.
///
/// Uses INT 21h AH=51h to retrieve the PSP selector, copies the command tail
/// bytes into a static buffer to ensure 'static lifetime, and splits it on whitespace.
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
    let mut raw_len: u8 = 0;

    if psp_sel < 0x0400 {
        // Protected-mode selector
        unsafe {
            asm!(
                "push es",
                "mov es, {sel:x}",
                "mov {len}, es:[0x80]",
                "pop es",
                sel = in(reg) psp_sel,
                len = out(reg_byte) raw_len,
            );
        }
        let len = (raw_len as usize).min(127);
        for i in 0..len {
            let offset = (0x81 + i) as u16;
            let mut byte: u8 = 0;
            unsafe {
                asm!(
                    "push es",
                    "mov es, {sel:x}",
                    "mov {byte}, es:[{off:e}]",
                    "pop es",
                    sel = in(reg) psp_sel,
                    off = in(reg) offset,
                    byte = out(reg_byte) byte,
                );
                TAIL_BUF[i] = byte;
            }
        }
    } else {
        // Real-mode segment
        let base = (psp_sel as u32) << 4;
        raw_len = peek_u8(base + 0x80);
        let len = (raw_len as usize).min(127);
        for i in 0..len {
            unsafe {
                TAIL_BUF[i] = peek_u8(base + 0x81 + i as u32);
            }
        }
    }

    let len = (raw_len as usize).min(127);

    // Build a &'static str over the copied static tail bytes
    let tail: &'static str = unsafe {
        let bytes = &TAIL_BUF[..len];
        // Strip any trailing CR (0x0D) DOS places at the end
        let trimmed = if bytes.last() == Some(&0x0D) {
            &bytes[..bytes.len() - 1]
        } else {
            bytes
        };
        // DOS encodes the tail as ASCII; treat as UTF-8 (all-ASCII is valid)
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

impl Speed {
    #[inline(never)]
    fn measure_frequency_tsc(t1: u16) -> u32 {
        let mut tsc_values = [0u32; 4]; // start_low, start_high, end_low, end_high
        let start_pit: u16;
        let end_pit: u16;

        // Wait for 2 ticks (~110ms)
        let target_ticks = t1.wrapping_add(2);

        unsafe {
            asm!(
                // Latch and read start PIT
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {1:x}, ax",

                // Read start TSC
                "rdtsc",
                "mov [{0}], eax",
                "mov [{0} + 4], edx",

                "push ds",
                "mov ax, 0x40",
                "mov ds, ax",
                ".align 16",
                "2:",
                "mov ax, ds:[0x6C]",
                "cmp ax, {3:x}",
                "jne 2b",
                "pop ds",

                // Read end TSC
                "rdtsc",
                "mov [{0} + 8], eax",
                "mov [{0} + 12], edx",

                // Latch and read end PIT
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {2:x}, ax",

                in(reg) tsc_values.as_mut_ptr(),
                out(reg) start_pit,
                out(reg) end_pit,
                in(reg) target_ticks,
                out("eax") _,
                out("edx") _,
                options(preserves_flags)
            );
        }

        let start_tsc = ((tsc_values[1] as u64) << 32) | (tsc_values[0] as u64);
        let end_tsc = ((tsc_values[3] as u64) << 32) | (tsc_values[2] as u64);
        let tsc_delta = end_tsc - start_tsc;

        // PIT runs at 1.193182 MHz. Each tick is 65536 PIT cycles.
        // Total pulses = (2 * 65536) + (start_pit - end_pit)
        let elapsed_pulses = (2u64 * 65_536) + (start_pit as i32 - end_pit as i32) as u64;

        // freq_hz = (tsc_delta * 1193182) / elapsed_pulses
        // freq_mhz = freq_hz / 1_000_000
        // We use rounded division: (numerator + denominator / 2) / denominator
        let denom = elapsed_pulses * 1_000_000;
        let freq_mhz = (tsc_delta * 1_193_182 + (denom / 2)) / denom;
        freq_mhz as u32
    }

    #[inline(never)]
    pub fn measure_frequency() -> u32 {
        use super::is_386;
        use crate::x86::dos32a::peek_u16;

        // For Cyrix, only measure 486-class cpus with the fallback,
        // only the M2 chips can be measured with TSC, and only if CPUID is enabled
        if Cyrix::should_measure_speed() == false {
            return 0;
        }

        // Use BIOS timer ticks at 0040:006C (linear address 0x0004006C)
        // 1 tick = 65536 / 1193182 seconds (~54.9 ms)

        let start_ticks = peek_u16(0x0004006C);
        let mut t1 = start_ticks;

        // Wait for a fresh tick
        while t1 == start_ticks {
            t1 = peek_u16(0x0004006C);
        }

        if super::has_tsc() {
            return Self::measure_frequency_tsc(t1);
        }

        // No TSC (386/486). Use a calibrated instruction loop.
        // We'll count how many times we can run a loop in 8 ticks (~440ms).
        // We also use the PIT Channel 0 for sub-tick precision.

        let mut iterations: u32 = 0;
        let target_ticks = t1.wrapping_add(8);
        let mut start_pit: u16 = 0;
        let mut end_pit: u16 = 0;

        unsafe {
            core::arch::asm!(
                "push ds",
                "mov ax, 0x40",
                "mov ds, ax",

                // Latch and read start PIT
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {2:x}, ax",

                ".align 16",
                "2:",
                "add {0:e}, 1",
                "push eax", // Extra work to slow down the loop and be more consistent
                "pop eax",
                "mov ax, ds:[0x6C]",
                "cmp ax, {1:x}",
                "jne 2b",

                // Latch and read end PIT
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {3:x}, ax",

                "pop ds",
                inout(reg) iterations,
                in(reg) target_ticks,
                out(reg) start_pit,
                out(reg) end_pit,
                out("eax") _,
            );
        }

        // PIT runs at 1.193182 MHz. Each tick is 65536 PIT cycles.
        // Total pulses = (8 * 65536) + (start_pit - end_pit)
        let elapsed_pulses = (8u64 * 65_536) + (start_pit as i32 - end_pit as i32) as u64;

        // Calibration:
        // 486 loop: 10 cycles
        // 386 loop: 29 cycles
        // Cyrix loop: 14 cycles
        // UMC loop: 10 cycles
        // RapidCAD (486 core in 386 package): 20 cycles
        let cycles_per_loop = match &*super::vendor_str() {
            super::constants::VENDOR_CYRIX => 14,
            super::constants::VENDOR_UMC => 10,
            _ => {
                if is_386() {
                    let sig = super::cpu::CpuSignature::detect();
                    match (sig.family, sig.model) {
                        // RapidCAD
                        (3, 4) => 20,
                        // 'Regular' 386 Chips
                        _ => 29,
                    }
                } else {
                    // 'Classic' 486
                    10
                }
            }
        };

        // freq_hz = (iterations * cycles_per_loop * 1193182) / elapsed_pulses
        // freq_mhz = freq_hz / 1_000_000
        // We use rounded division: (numerator + denominator / 2) / denominator
        let denom = elapsed_pulses * 1_000_000;
        let freq_mhz =
            (iterations as u64 * cycles_per_loop as u64 * 1_193_182 + (denom / 2)) / denom;
        freq_mhz as u32
    }
}
