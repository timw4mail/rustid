//! Pre-CPUID CPU identification methods.
//!
//! For 386/486 processors that don't support the CPUID instruction,
//! these methods use alternative techniques to identify the CPU vendor and type.
#![cfg(target_arch = "x86")]

use super::*;

/// Returns the CPU vendor for 386/486-class processors without CPUID support.
pub fn get_vendor_by_quirk() -> &'static str {
    // Cyrix quirk works for both 386 and 486 and is very reliable.
    if has_cyrix_5_2_quirk() {
        return VENDOR_CYRIX;
    }

    if is_386() {
        // Intel 386 allows toggling CR0.ET. AMD 386 has it hardwired to 1.
        if can_toggle_et() {
            return VENDOR_INTEL;
        }

        if has_amd_386_quirk() {
            return VENDOR_AMD;
        }
    }

    if is_486() {
        // Try AMD-specific quirks first.
        if has_amd_486_quirk() {
            return VENDOR_AMD;
        }

        // If it's a 486 and not AMD, and ET is hardwired to 1, it's likely Intel.
        if has_intel_486_quirk() {
            return VENDOR_INTEL;
        }
    }

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

/// Returns true if the CPU is an AMD 386 (detected via ET bit behavior).
#[inline(never)]
pub fn has_amd_386_quirk() -> bool {
    if !is_386() {
        return false;
    }
    // AMD 386 has ET (bit 4 of CR0) hardwired to 1.
    // Intel 386 allows toggling it.
    !can_toggle_et() && is_et_set()
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

/// Returns true if the CPU is an AMD processor.
#[inline(never)]
pub fn has_amd_486_quirk() -> bool {
    if !is_486() {
        return false;
    }

    // 1. FPU State check: If FPU exists, check if FNINIT fails to clear pointers.
    if has_fpu() && has_amd_fpu_quirk() {
        return true;
    }

    // 2. Known AMD-only signatures (Model 0, 1, 3, 7, 8, 9, 14, 15)
    #[cfg(target_os = "none")]
    if let Some(sig) = get_reset_signature() {
        if sig.family == 4 {
            match sig.model {
                0 | 1 | 3 | 7 | 8 | 9 | 14 | 15 => return true,
                _ => {}
            }
        }
    }

    false
}

/// Returns true if the FPU state exhibits AMD-specific behavior in reserved fields.
#[inline(never)]
pub fn has_amd_fpu_quirk() -> bool {
    let mut fpu_env = [0u16; 7]; // 14 bytes for FSTENV in 16-bit real mode
    unsafe {
        core::arch::asm!(
            "cli",
            "fld1",
            "fninit",
            "fnstenv [{0}]",
            "fwait",
            "sti",
            in(reg) &mut fpu_env,
            options(nostack)
        );
    }
    // Intel clears these on fninit, but early AMD/clones often do not.
    // Index 3: IP Offset, 4: CS Selector, 5: Data Offset, 6: Data Selector
    fpu_env[3] != 0 || fpu_env[4] != 0 || fpu_env[5] != 0 || fpu_env[6] != 0
}

/// Returns true if the CPU is definitively Intel 486.
#[inline(never)]
pub fn has_intel_486_quirk() -> bool {
    // Intel 486: Is 486, ET is hardwired to 1
    is_486() && !can_toggle_et() && is_et_set()
}

/// Returns true if the ET bit (bit 4) of CR0 can be toggled.
#[inline(never)]
fn can_toggle_et() -> bool {
    let mut original: u32;
    let mut toggled: u32;
    unsafe {
        core::arch::asm!(
            "cli",
            "mov {orig:e}, cr0",
            "mov {tmp:e}, {orig:e}",
            "xor {tmp:e}, 0x10", // Toggle ET bit
            "mov cr0, {tmp:e}",
            "xor {tmp:e}, {tmp:e}", // Clear register before reading back
            "mov {tmp:e}, cr0",
            "mov {togg:e}, {tmp:e}",
            "mov cr0, {orig:e}", // Restore
            "sti",
            orig = out(reg) original,
            tmp = out(reg) _,
            togg = out(reg) toggled,
            options(nostack)
        );
    }
    ((original ^ toggled) & 0x10) != 0
}

/// Returns true if the ET bit (bit 4) of CR0 is currently set.
#[inline(never)]
fn is_et_set() -> bool {
    let cr0: u32;
    unsafe {
        core::arch::asm!("mov {0:e}, cr0", out(reg) cr0, options(nostack));
    }
    (cr0 & 0x10) != 0
}

/// Returns true if the CPU has a Floating Point Unit (FPU).
#[inline(never)]
pub fn has_fpu() -> bool {
    let mut control: u16 = 0xFFFF;
    unsafe {
        core::arch::asm!(
            "cli",
            "fninit",
            "fnstcw [{0}]",
            "sti",
            in(reg) &mut control,
            options(nostack)
        );
    }
    (control & 0x103F) == 0x003F
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

#[cfg(feature = "debug")]
pub fn debug_quirks() {
    use crate::println;

    println!("=== Quirk Detection Debug ===");
    println!();

    println!("CPU Class:");
    println!("  is_386: {}", is_386());
    println!("  is_486: {}", is_486());
    println!();

    println!("Vendor Detection:");
    println!("  has_cyrix_5_2_quirk:  {}", has_cyrix_5_2_quirk());
    println!("  has_amd_386_quirk:    {}", has_amd_386_quirk());
    println!(
        "  has_amd_fpu_quirk:    {}",
        has_fpu() && has_amd_fpu_quirk()
    );
    println!("  has_amd_486_quirk:    {}", has_amd_486_quirk());
    println!("  has_intel_486_quirk:  {}", has_intel_486_quirk());
    println!("  can_toggle_et:        {}", can_toggle_et());
    println!();

    println!("Result: {}", get_vendor_by_quirk());
}
