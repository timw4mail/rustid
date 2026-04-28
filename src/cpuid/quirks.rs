//! Pre-CPUID CPU identification methods.
//!
//! For 386/486 processors that don't support the CPUID instruction,
//! these methods use alternative techniques to identify the CPU vendor and type.
#![cfg(target_arch = "x86")]

use super::*;

/// Returns the CPU vendor for 386/486-class processors without CPUID support.
pub fn get_vendor_by_quirk() -> &'static str {
    if has_cyrix_5_2_quirk() {
        return VENDOR_CYRIX;
    }

    #[cfg(target_os = "none")]
    return match get_reset_signature() {
        Some(signature) => match (signature.family, signature.model, signature.stepping) {
            // Intel RapidCAD
            (3, 4, _) => VENDOR_INTEL,
            // AMD 486DX-40
            (4, 1, 2) => VENDOR_AMD,
            // Intel 486DX-50
            (4, 1, _) => VENDOR_INTEL,
            // Intel 486SL
            (4, 4, _) => VENDOR_INTEL,

            _ => UNK,
        },
        None => UNK,
    };

    #[cfg(not(target_os = "none"))]
    UNK
}

/// Returns true if the CPU is a 386-class processor.
pub fn is_386() -> bool {
    !is_486()
}

/// Returns true if the CPU is at least a 486-class processor.
#[inline(never)]
pub fn is_486() -> bool {
    is_ac_flag_supported()
}

/// Helper to check for AC flag support in EFLAGS register, which is a shibboleth that can
/// determine if the CPU is a 386 or 486.
///
/// Verified on real hardware
fn is_ac_flag_supported() -> bool {
    let supported: u32;
    unsafe {
        core::arch::asm!(
        "pushfd",
        "pop eax",
        "mov ecx, eax",
        "xor eax, 0x40000", // Toggle AC flag (bit 18)
        "push eax",
        "popfd",
        "pushfd",
        "pop eax",
        "push ecx",
        "popfd",
        "xor eax, ecx",
        "and eax, 0x40000",
        out("eax") supported,
        out("ecx") _,
        );
    }
    supported != 0
}

/// Returns true if the CPU is a Cyrix processor.
#[inline(never)]
pub fn has_cyrix_5_2_quirk() -> bool {
    let flags: u8;
    unsafe {
        core::arch::asm!(
            "xor ax, ax",
            "sahf",
            "mov ax, 5",
            "mov bx, 2",
            "div bl",
            "lahf",
            out("ah") flags,
            out("al") _,
            out("bx") _,
            options(nostack)
        );
    }
    // Cyrix: flags remain unchanged
    (flags & 0xD5) == 0
}

/// Attempts to retrieve the CPU signature (EDX value at reset) by performing a soft reset.
///
/// This method uses the CMOS shutdown byte (0x0F) set to 0x0A, which tells the BIOS
/// to jump to the address stored at 0040:0067h after reset.
///
/// **WARNING**: This is extremely disruptive as it resets the CPU. It is only
/// suitable for environments where the caller can handle a full CPU reset and
/// subsequent return to the code.
///
/// Verified on some real 386/486 systems.
#[cfg(target_os = "none")]
#[allow(static_mut_refs)]
pub fn get_reset_signature() -> Option<CpuSignature> {
    if has_cpuid() {
        return Some(CpuSignature::detect());
    }

    // Persist register context across reset using internal scoped statics.
    // We MUST use statics for this because the CPU registers (including SS:SP)
    // are completely wiped after the reset.
    static mut SAVED_SS: u16 = 0;
    static mut SAVED_SP: u16 = 0;
    static mut SAVED_BP: u16 = 0;
    static mut SAVED_SI: u16 = 0;
    static mut SAVED_DI: u16 = 0;
    static mut SAVED_BX: u16 = 0;

    let mut raw_sig: u32 = 0;

    unsafe {
        core::arch::asm!(
        "cli",

        // Save execution context to static memory
        "mov word ptr ds:[{ss_ptr}], ss",
        "mov word ptr ds:[{sp_ptr}], sp",
        "mov word ptr ds:[{bp_ptr}], bp",
        "mov word ptr ds:[{si_ptr}], si",
        "mov word ptr ds:[{di_ptr}], di",
        "mov word ptr ds:[{bx_ptr}], bx",

        // 1. Set the warm boot pointer at 0040:0067h
        "mov ax, 0x40",
        "mov es, ax",
        "lea ax, [20f]",
        "mov word ptr es:[0x67], ax",
        "mov word ptr es:[0x69], cs",

        // 2. Set CMOS shutdown byte to 0x0A (Jump to 40:67 after reset)
        "mov al, 0x0F",
        "out 0x70, al",
        "out 0x80, al", // IO delay
        "mov al, 0x0A",
        "out 0x71, al",
        "out 0x80, al", // IO delay

        // 3. Trigger Reset
        // Method 1: Fast Reset via Port 0x92
        "in al, 0x92",
        "out 0x80, al", // IO delay
        "or al, 1",
        "out 0x92, al",
        "out 0x80, al", // IO delay

        // Method 2: Keyboard Controller Reset (Port 0x64) fallback
        "mov cx, 0x0FFF",
        "22:",
        "in al, 0x64",
        "out 0x80, al", // IO delay
        "test al, 2",
        "jz 23f",
        "loop 22b",
        "23:",
        "mov al, 0xFE",
        "out 0x64, al",

        "21: hlt",
        "jmp 21b",

        "20:",
        // --- We are now back after reset ---
        // Restore segments first so we can access statics
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",

        // Restore execution context from our code/data segment
        "mov ss, word ptr cs:[{ss_ptr}]",
        "mov sp, word ptr cs:[{sp_ptr}]",
        "mov bp, word ptr cs:[{bp_ptr}]",
        "mov si, word ptr cs:[{si_ptr}]",
        "mov di, word ptr cs:[{di_ptr}]",
        "mov bx, word ptr cs:[{bx_ptr}]",

        // Capture EDX into our local variable
        "mov {sig_out}, edx",

        // Cleanup: Clear CMOS shutdown byte
        "mov al, 0x0F",
        "out 0x70, al",
        "out 0x80, al", // IO delay
        "xor al, al",
        "out 0x71, al",

        "sti",
        ss_ptr = sym SAVED_SS,
        sp_ptr = sym SAVED_SP,
        bp_ptr = sym SAVED_BP,
        si_ptr = sym SAVED_SI,
        di_ptr = sym SAVED_DI,
        bx_ptr = sym SAVED_BX,
        sig_out = out(reg) raw_sig,
        out("ax") _,
        out("cx") _,
        );
    }

    if raw_sig == 0 || raw_sig == 0xFFFFFFFF {
        return None;
    }

    let stepping = raw_sig & 0xF;
    let model = (raw_sig >> 4) & 0xF;
    let family = (raw_sig >> 8) & 0xF;

    Some(CpuSignature::new_synth(family, model, stepping))
}

#[cfg(feature = "debug")]
pub fn debug_quirks() {
    use crate::println;

    println!("=== Quirk Detection Debug ===");

    println!("CPU Class:");
    println!("  has_cpuid: {}", has_cpuid());
    println!("  is_386: {}", is_386());
    println!("  is_486: {}", is_486());

    println!("Signature: {:?}", CpuSignature::detect());

    println!("Vendor Detection:");
    println!("  has_cyrix_5_2_quirk:  {}", has_cyrix_5_2_quirk());

    println!("Result: {}", get_vendor_by_quirk());
}
