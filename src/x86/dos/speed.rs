// ============================================================================
// Speed / Frequency measurement
// ============================================================================

use super::*;

use crate::x86::{constants, cpu::CpuSignature, has_tsc, is_386, vendor_str};

#[cfg(dos)]
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

                "push es",
                "mov ax, 0x40",
                "mov es, ax",
                ".align 16",
                "2:",
                "mov ax, es:[0x6C]",
                "cmp ax, {3:x}",
                "jne 2b",
                "pop es",

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
        use crate::x86::dos::peek_u16;

        // For Cyrix, only measure 486-class cpus with the fallback,
        // only the M2 chips can be measured with TSC, and only if CPUID is enabled
        if Cyrix::should_measure_speed() == false {
            return 0;
        }

        // Use BIOS timer ticks at 0040:006C
        // 1 tick = 65536 / 1193182 seconds (~54.9 ms)

        let start_ticks = peek_u16(0x0040, 0x006C);
        let mut t1 = start_ticks;

        // Wait for a fresh tick
        while t1 == start_ticks {
            t1 = peek_u16(0x0040, 0x006C);
        }

        if has_tsc() {
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

                ".align 16",
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
        let elapsed_pulses = (8u64 * 65_536) + (start_pit as i32 - end_pit as i32) as u64;

        // Calibration:
        // 486 loop: 10 cycles
        // 386 loop: 29 cycles
        // Cyrix loop: 14 cycles
        // UMC loop: 10 cycles
        // RapidCAD (486 core in 386 package): 20 cycles
        let cycles_per_loop = match &*vendor_str() {
            constants::VENDOR_CYRIX => 14,
            constants::VENDOR_UMC => 10,
            _ => {
                if is_386() {
                    let sig = CpuSignature::detect();
                    match (sig.family, sig.model) {
                        (3, 4) => 20,
                        _ => 29,
                    }
                } else {
                    10
                }
            }
        };

        let denom = elapsed_pulses * 1_000_000;
        let freq_mhz =
            (iterations as u64 * cycles_per_loop as u64 * 1_193_182 + (denom / 2)) / denom;
        freq_mhz as u32
    }
}

#[cfg(dos32a)]
impl Speed {
    #[inline(never)]
    fn measure_frequency_tsc(t1: u16) -> u32 {
        let mut tsc_values = [0u32; 4];
        let start_pit: u16;
        let end_pit: u16;

        let target_ticks = t1.wrapping_add(2);

        unsafe {
            asm!(
                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {1:x}, ax",

                "rdtsc",
                "mov [{0}], eax",
                "mov [{0} + 4], edx",

                ".align 16",
                "2:",
                "mov ax, [0x46c]",
                "cmp ax, {3:x}",
                "jne 2b",

                "rdtsc",
                "mov [{0} + 8], eax",
                "mov [{0} + 12], edx",

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

        let elapsed_pulses = (2u64 * 65_536) + (start_pit as i32 - end_pit as i32) as u64;
        let denom = elapsed_pulses * 1_000_000;
        let freq_mhz = (tsc_delta * 1_193_182 + (denom / 2)) / denom;
        freq_mhz as u32
    }

    #[inline(never)]
    pub fn measure_frequency() -> u32 {
        use crate::x86::dos::peek_u16;
        use is_386;

        if Cyrix::should_measure_speed() == false {
            return 0;
        }

        let start_ticks = peek_u16(0x46c);
        let mut t1 = start_ticks;

        while t1 == start_ticks {
            t1 = peek_u16(0x46c);
        }

        if has_tsc() {
            return Self::measure_frequency_tsc(t1);
        }

        let mut iterations: u32 = 0;
        let target_ticks = t1.wrapping_add(8);
        let mut start_pit: u16 = 0;
        let mut end_pit: u16 = 0;

        unsafe {
            core::arch::asm!(
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
                "push eax",
                "pop eax",
                "mov ax, [0x46c]",
                "cmp ax, {1:x}",
                "jne 2b",

                "xor al, al",
                "out 0x43, al",
                "in al, 0x40",
                "mov ah, al",
                "in al, 0x40",
                "xchg al, ah",
                "mov {3:x}, ax",

                inout(reg) iterations,
                in(reg) target_ticks,
                out(reg) start_pit,
                out(reg) end_pit,
                out("eax") _,
            );
        }

        let elapsed_pulses = (8u64 * 65_536) + (start_pit as i32 - end_pit as i32) as u64;

        let cycles_per_loop = match &*vendor_str() {
            constants::VENDOR_CYRIX => 14,
            constants::VENDOR_UMC => 10,
            _ => {
                if is_386() {
                    let sig = CpuSignature::detect();
                    match (sig.family, sig.model) {
                        (3, 4) => 20,
                        _ => 29,
                    }
                } else {
                    10
                }
            }
        };

        let denom = elapsed_pulses * 1_000_000;
        let freq_mhz =
            (iterations as u64 * cycles_per_loop as u64 * 1_193_182 + (denom / 2)) / denom;
        freq_mhz as u32
    }
}
