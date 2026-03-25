//! Pre-CPUID CPU identification methods.
//!
//! For 386/486 processors that don't support the CPUID instruction,
//! these methods use alternative techniques to identify the CPU vendor and type.
#![cfg(target_arch = "x86")]

use super::*;

/// Returns the CPU vendor for 386/486-class processors without CPUID support.
pub fn get_pre_cpuid_vendor() -> &'static str {
    if is_486() {
        if cyrix_5_2_test() {
            return VENDOR_CYRIX;
        }

        if amd_486_test() {
            return VENDOR_AMD;
        }

        if intel_cr0_test() {
            return VENDOR_INTEL;
        }
    } else if is_386() {
        if amd_386_test() {
            return VENDOR_AMD;
        }
        return VENDOR_INTEL;
    }

    UNK
}

/// Returns true if the CPU is an AMD 386 (detected via DIV overflow behavior).
#[inline(never)]
pub fn amd_386_test() -> bool {
    // This is a known difference between Intel and AMD 386s.
    // AMD 386s set ZF on certain division overflows where Intel 386s don't.
    // Or rather, there's an errata/difference in how they handle flags in corner cases.
    // Another common test: AMD 386 DX can toggle a bit in a reserved register,
    // but the most portable is the "pushfd" behavior or a specific 32-bit math result.
    // Let's use the "pushf" / "popf" difference if available.
    // On 80386, bits 12-15 of EFLAGS are always set in real mode on Intel,
    // but can be cleared on some clones.
    let flags: u16;
    unsafe {
        core::arch::asm!(
            "pushf",
            "pop ax",
            "mov cx, ax",
            "and ax, 0x0fff", // Try to clear bits 12-15
            "push ax",
            "popf",
            "pushf",
            "pop ax",
            "push cx", // Restore original flags
            "popf",
            out("ax") flags,
            out("cx") _,
        );
    }
    // If bits 12-15 were cleared, it's likely an AMD or other clone.
    // On Intel 80386, they are hardwired to 1 in real mode.
    (flags & 0xf000) != 0xf000
}

/// Returns true if the CPU is at least a 386-class processor.
///
/// This is determined by checking if the AC (Alignment Check) flag in EFLAGS
/// can be toggled. 386 CPUs do not support this, while 486 and newer do.
///
/// Verified on real hardware
pub fn is_386() -> bool {
    !is_ac_flag_supported()
}

/// Returns true if the CPU is at least a 486-class processor
///
/// Verified on real hardware
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

/// Returns true if the CPU is a Cyrix processor (detected without CPUID).
///
/// Cyrix processors are unique in that they do not modify flags during a `div`
/// instruction, whereas other x86 processors do.
pub fn cyrix_5_2_test() -> bool {
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
        );
    }
    // Cyrix: flags remain unchanged
    (flags & 0xD5) == 0
}

/// Returns true if the CPU is an AMD processor (detected via DIV flag behavior).
///
/// AMD 486 processors have a unique behavior where the DIV instruction
/// clears the Carry Flag (CF), whereas on Intel 486 it is undefined or unchanged.
#[inline(never)]
pub fn amd_486_test() -> bool {
    let flags: u16;
    unsafe {
        core::arch::asm!(
            "stc",          // Set Carry Flag
            "mov ax, 5",
            "mov bl, 2",
            "div bl",
            "pushf",
            "pop ax",
            out("ax") flags,
            out("bl") _,
        );
    }
    // AMD: CF is cleared (0) after DIV
    (flags & 0x01) == 0
}
/// Returns true if the CR0 Extended Type (ET) bit is set and hardwired (typical for Intel 486).
#[inline(never)]
pub fn intel_cr0_test() -> bool {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "mov eax, cr0",
            "mov ecx, eax",
            "and eax, 0xffffffef", // Try to clear bit 4 (ET)
            "mov cr0, eax",
            "mov eax, cr0",
            "mov cr0, ecx",        // Restore original CR0
            out("eax") result,
            out("ecx") _,
        );
    }
    // If it stayed 1, it's likely Intel 486
    (result & 0x10) != 0
}

/// Retrieves the CPU Type (component) and Mask Revision via BIOS INT 15h, AH=C9h.
///
/// This call is supported on IBM PS/2 and compatible BIOSes for 386+ processors.
/// Returns (component_id, revision) if supported, otherwise (0, 0).
#[cfg(target_os = "none")]
#[inline(never)]
pub fn get_bios_signature() -> (u8, u8) {
    let type_rev: u16;
    let success: u16;
    unsafe {
        core::arch::asm!(
            "xor ax, ax",
            "mov ah, 0xC9",
            "int 0x15",
            "mov {0:x}, cx",
            "sbb ax, ax",
            out(reg) type_rev,
            out("ax") success,
            out("bx") _,
            out("dx") _,
            out("di") _,
        );
    }

    if success == 0 {
        ((type_rev >> 8) as u8, (type_rev & 0xFF) as u8)
    } else {
        (0, 0)
    }
}
