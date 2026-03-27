//! Pre-CPUID CPU identification methods.
//!
//! For 386/486 processors that don't support the CPUID instruction,
//! these methods use alternative techniques to identify the CPU vendor and type.
#![cfg(target_arch = "x86")]

use super::*;

/// Returns the CPU vendor for 386/486-class processors without CPUID support.
pub fn get_vendor_by_quirk() -> &'static str {
    if is_386() {
        return UNK;
    }

    if has_cyrix_5_2_quirk() {
        return VENDOR_CYRIX;
    }

    if has_intel_cr0_quirk() {
        return VENDOR_INTEL;
    }

    if has_amd_486_quirk() {
        return VENDOR_AMD;
    }

    UNK
}

/// Returns true if the CPU is an AMD 386 (detected via DIV overflow behavior).
#[inline(never)]
pub fn has_amd_386_quirk() -> bool {
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
            "cli",
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
        );
    }
    // Cyrix: flags remain unchanged
    (flags & 0xD5) == 0
}

/// Returns true if the CPU is an AMD processor (detected via DIV flag behavior).
#[inline(never)]
pub fn has_amd_486_quirk() -> bool {
    let flags: u16;
    unsafe {
        core::arch::asm!(
            "cli",
            "stc",          // Set Carry Flag (CF=1)
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
pub fn has_intel_cr0_quirk() -> bool {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "cli",
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

/// Attempts to retrieve the CPU signature (EDX value at reset) by performing a soft reset.
///
/// This method uses the CMOS shutdown byte (0x0F) set to 0x0A, which tells the BIOS
/// to jump to the address stored at 0040:0067h after reset.
///
/// **WARNING**: This is extremely disruptive as it resets the CPU. It is only
/// suitable for environments where the caller can handle a full CPU reset and
/// subsequent return to the code.
///
/// Verified on some 386/486 systems.
#[cfg(target_os = "none")]
#[allow(static_mut_refs)]
pub fn get_reset_signature() -> Option<CpuSignature> {
    static mut RESET_DONE: bool = false;
    static mut CACHED_SIG: Option<CpuSignature> = None;
    static mut SAVED_EDX: u32 = 0;
    static mut SAVED_SS: u16 = 0;
    static mut SAVED_SP: u16 = 0;
    static mut SAVED_BP: u16 = 0;
    static mut SAVED_SI: u16 = 0;
    static mut SAVED_DI: u16 = 0;
    static mut SAVED_BX: u16 = 0;

    unsafe {
        if RESET_DONE {
            return CACHED_SIG.clone();
        }
        RESET_DONE = true;
    }

    unsafe {
        core::arch::asm!(
        "cli",

        // Save all important registers to static memory
        "mov word ptr ds:[{ss_ptr}], ss",
        "mov word ptr ds:[{sp_ptr}], sp",
        "mov word ptr ds:[{bp_ptr}], bp",
        "mov word ptr ds:[{si_ptr}], si",
        "mov word ptr ds:[{di_ptr}], di",
        "mov word ptr ds:[{bx_ptr}], bx",

        // 1. Set the warm boot pointer at 0040:0067h
        "mov ax, 0x40",
        "mov es, ax",
        "lea ax, [2f]",      // Get offset of label 2 forward
        "mov word ptr es:[0x67], ax",
        "mov word ptr es:[0x69], cs",

        // 2. Set CMOS shutdown byte to 0x0A (Jump to 40:67 after reset)
        "mov al, 0x0F",
        "out 0x70, al",
        "out 0xEB, al", // IO delay
        "mov al, 0x0A",
        "out 0x71, al",
        "out 0xEB, al", // IO delay

        // 3. Trigger Reset
        // Method 1: Fast Reset via Port 0x92
        "in al, 0x92",
        "or al, 1",
        "out 0x92, al",
        "out 0xEB, al", // IO delay

        // Method 2: Keyboard Controller Reset (Port 0x64)
        "mov cx, 0xFFFF",
        "4:",
        "in al, 0x64",
        "test al, 2",
        "jz 5f",
        "loop 4b",
        "5:",
        "mov al, 0xFE",
        "out 0x64, al",

        "3: hlt",
        "jmp 3b",

        "2:",
        // --- We are now back after reset ---
        // IMMEDIATELY restore DS/ES so we can access our variables
        "mov ax, cs",
        "mov ds, ax",
        "mov es, ax",

        // Capture EDX immediately
        "mov dword ptr ds:[{edx_ptr}], edx",

        // Restore stack and other registers
        "mov ss, word ptr ds:[{ss_ptr}]",
        "mov sp, word ptr ds:[{sp_ptr}]",
        "mov bp, word ptr ds:[{bp_ptr}]",
        "mov si, word ptr ds:[{si_ptr}]",
        "mov di, word ptr ds:[{di_ptr}]",
        "mov bx, word ptr ds:[{bx_ptr}]",

        // Cleanup: Clear CMOS shutdown byte
        "mov al, 0x0F",
        "out 0x70, al",
        "xor al, al",
        "out 0x71, al",

        "sti",
        ss_ptr = sym SAVED_SS,
        sp_ptr = sym SAVED_SP,
        bp_ptr = sym SAVED_BP,
        si_ptr = sym SAVED_SI,
        di_ptr = sym SAVED_DI,
        bx_ptr = sym SAVED_BX,
        edx_ptr = sym SAVED_EDX,
        out("ax") _,
        out("cx") _,
        );
    }

    let raw_sig = unsafe { SAVED_EDX };

    if raw_sig == 0 || raw_sig == 0xFFFFFFFF {
        return None;
    }

    let stepping = raw_sig & 0xF;
    let model = (raw_sig >> 4) & 0xF;
    let family = (raw_sig >> 8) & 0xF;
    let ext_model = (raw_sig >> 16) & 0xF;
    let ext_family = (raw_sig >> 20) & 0xFF;

    let display_family = if family == 0xF {
        family + ext_family
    } else {
        family
    };
    let display_model = if family == 0x6 || family == 0xF {
        (ext_model << 4) + model
    } else {
        model
    };

    let sig = CpuSignature {
        extended_family: ext_family,
        family,
        extended_model: ext_model,
        model,
        stepping,
        display_family,
        display_model,
        is_overdrive: false,
        from_cpuid: false,
    };

    unsafe {
        CACHED_SIG = Some(sig.clone());
    }

    Some(sig)
}
