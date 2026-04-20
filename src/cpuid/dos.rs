#![cfg(all(target_arch = "x86", target_os = "none"))]
//! DOS/16-bit environment support for rustid.
//!
//! This module provides DOS-specific implementations including console output
//! via DOS INT 21h interrupts and a custom panic handler for bare-metal environments.

use crate::common::Speed;
use core::arch::asm;
use core::fmt::Write;

/// Custom panic handler for no-std environments.
/// Loops indefinitely on panic to prevent undefined behavior.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Prints a formatted string to the DOS console.
/// Supports both literal strings and format strings.
#[macro_export]
macro_rules! print {
    ($s:literal) => {
        $crate::cpuid::dos::_print_str($s)
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!(&mut $crate::cpuid::dos::DosWriter {}, $($arg)*);
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

#[macro_export]
macro_rules! sfmt {
    ($($arg:tt)*) => {
        {
            use $crate::cpuid::type_wrappers::String;
            use $crate::cpuid::type_wrappers::Str;
            use core::fmt::Write;
            let mut buf = String::<{ $crate::cpuid::type_wrappers::MAX_FMT_LEN }>::new();
            let _ = buf.write_fmt(core::format_args!($($arg)*));
            let mut result = Str::<_>::new();
            result.push_str(buf.as_str());
            result
        }
    };
}

/// Writes a string to the DOS console character by character.
pub fn _print_str(s: &str) {
    for &b in s.as_bytes() {
        printc(b);
    }
}

/// A writer implementation for DOS console output via the fmt::Write trait.
pub struct DosWriter;

impl Write for DosWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for &b in s.as_bytes() {
            printc(b);
        }
        Ok(())
    }
}

/// Outputs a single character to the DOS console using INT 21h.
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

/// Exits the program and returns control to DOS using INT 21h, AH=4Ch.
pub fn exit() -> ! {
    // Exit to DOS via INT 21h, AH=4Ch
    unsafe {
        asm!(
            "int 0x21",
            in("ax") 0x4C00_u16,
            options(noreturn)
        );
    }
}

/// Reads a byte from a segmented memory address.
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

/// Reads a 16-bit word from a segmented memory address.
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

impl Speed {
    fn measure_frequency_tsc(t1: u16) -> u32 {
        use crate::cpuid::dos::peek_u16;

        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdtsc as rdtsc;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdtsc as rdtsc;

        let start_tsc = unsafe { rdtsc() };

        // Wait for 2 ticks (~110ms)
        let target_ticks = t1.wrapping_add(2);
        while peek_u16(0x0040, 0x006C) != target_ticks {
            core::hint::spin_loop();
        }

        let end_tsc = unsafe { rdtsc() };
        let tsc_delta = end_tsc - start_tsc;

        // freq_mhz = (tsc_delta * 1193182) / (2 * 65536 * 1_000_000)
        let freq_mhz = (tsc_delta * 1193182) / 131072000000u64;
        freq_mhz as u32
    }

    pub fn measure_frequency() -> u32 {
        use super::is_386;
        use crate::cpuid::dos::peek_u16;

        // Use BIOS timer ticks at 0040:006C
        // 1 tick = 65536 / 1193182 seconds (~54.9 ms)

        let start_ticks = peek_u16(0x0040, 0x006C);
        let mut t1 = start_ticks;

        // Wait for a fresh tick
        while t1 == start_ticks {
            t1 = peek_u16(0x0040, 0x006C);
        }

        if super::has_tsc() {
            return Self::measure_frequency_tsc(t1);
        }

        // If the Cryix cpu supports enabling cpuid, this
        // fallback method is going to be wildly inaccurate,
        // so just skip it
        if super::vendor::cyrix::Cyrix::can_enable_cpuid() {
            return 0;
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
                "push es",
                "mov ax, 0x40",
                "mov es, ax",

                // Latch and read start PIT
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {2:x}, ax",

                "2:",
                "add {0:e}, 1",
                "push ax", // Extra work to slow down the loop and be more consistent
                "pop ax",
                "mov ax, es:[0x6C]",
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

                "pop es",
                inout(reg) iterations,
                in(reg) target_ticks,
                out(reg) start_pit,
                out(reg) end_pit,
                out("ax") _,
            );
        }

        // PIT runs at 1.193182 MHz. Each tick is 65536 PIT cycles.
        // Total pulses = (8 * 65536) + (start_pit - end_pit)
        let elapsed_pulses = (8u64 * 65536) + (start_pit as i32 - end_pit as i32) as u64;

        // Calibration:
        // 486 loop: add(2) + push(1) + pop(1) + mov mem(3) + cmp(1) + jne(3) = 11 cycles
        // 386 loop: add(4) + push(2) + pop(4) + mov mem(6) + cmp(2) + jne(7) = 25 cycles
        // RapidCAD (486 core in 386 package): ~20 cycles
        let cycles_per_loop = match &*super::vendor_str() {
            super::constants::VENDOR_CYRIX => 14,
            super::constants::VENDOR_UMC => 12,
            _ => {
                if is_386() {
                    let sig = super::cpu::CpuSignature::detect();
                    match (sig.family, sig.model) {
                        // RapidCAD
                        (3, 4) => 20,
                        // 'Regular' 386 Chips
                        _ => 25,
                    }
                } else {
                    // 'Classic' 486
                    11
                }
            }
        };

        // freq_hz = (iterations * cycles_per_loop * 1193182) / elapsed_pulses
        // freq_mhz = freq_hz / 1_000_000
        // We use rounded division: (numerator + denominator / 2) / denominator
        let denom = elapsed_pulses * 1000000;
        let freq_mhz = (iterations as u64 * cycles_per_loop as u64 * 1193182 + (denom / 2)) / denom;
        freq_mhz as u32
    }
}
