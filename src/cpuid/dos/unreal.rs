#![cfg(all(target_arch = "x86", target_os = "none"))]
//! Unreal Mode (Big Real Mode) support for x86 DOS
use core::arch::asm;

pub static mut UNREAL_MODE_ENABLED: bool = false;

pub fn is_in_protected_mode() -> bool {
    let cr0_val: u32;
    unsafe {
        asm!("mov eax, cr0", out("eax") cr0_val, options(nostack, preserves_flags));
    }
    (cr0_val & 0x1) != 0
}

/// Enables unreal mode by briefly entering protected mode to load segment limits.
///
/// This function loads FS and GS with 4GB limits, allowing flat 32-bit
/// addressing via these registers while remaining in real mode.
/// It also attempts to set 4GB limits for DS and ES, which works on most
/// 386/486 and many Pentium CPUs.
pub unsafe fn init_unreal_mode() -> bool {
    if is_in_protected_mode() {
        unsafe {
            UNREAL_MODE_ENABLED = true;
        }
        return true;
    }

    // GDT with 3 entries: Null, Data (4GB Flat)
    // We use a simplified GDT here.
    let gdt: [u64; 3] = [
        0,                  // Null descriptor
        0x00cf9a000000ffff, // Code: Base=0, Limit=4GB, P=1, DPL=0, S=1, Type=Code EX, G=1, D=1
        0x00cf92000000ffff, // Data: Base=0, Limit=4GB, P=1, DPL=0, S=1, Type=Data RW, G=1, D=1
    ];

    let mut gdt_ptr = [0u8; 6];
    let ds: u16;
    unsafe {
        asm!("mov {0:x}, ds", out(reg) ds);
    }

    // Calculate linear address of the GDT
    let linear_base = ((ds as u32) << 4) + (&gdt as *const _ as u32);
    let limit = (core::mem::size_of_val(&gdt) - 1) as u16;

    gdt_ptr[0..2].copy_from_slice(&limit.to_le_bytes());
    gdt_ptr[2..6].copy_from_slice(&linear_base.to_le_bytes());

    unsafe {
        asm!(
            "cli",
            "push ds",
            "push es",
            "lgdt [{0}]",
            "mov eax, cr0",
            "or al, 1",
            "mov cr0, eax",
            "jmp 2f",
            "2:",
            "mov ax, 0x10", // Selector 0x10 is Entry 2 (Data)
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov eax, cr0",
            "and al, 0xFE",
            "mov cr0, eax",
            "pop es",
            "pop ds",
            "sti",
            in(reg) &gdt_ptr,
            out("eax") _,
        );
    }

    unsafe {
        UNREAL_MODE_ENABLED = true;
    }
    true
}

pub unsafe fn setup_unreal_mode() {
    unsafe {
        init_unreal_mode();
    }
}

pub unsafe fn enable_unreal_mode() {
    unsafe {
        init_unreal_mode();
    }
}

pub unsafe fn exit_unreal_mode() {
    // To "exit" unreal mode, we reload segment registers in real mode.
    // This resets their limits to 64KB on all CPUs.
    unsafe {
        asm!(
            "mov ax, ds", "mov ds, ax",
            "mov ax, es", "mov es, ax",
            "mov ax, fs", "mov fs, ax",
            "mov ax, gs", "mov gs, ax",
            out("ax") _,
        );
    }
    unsafe {
        UNREAL_MODE_ENABLED = false;
    }
}

pub unsafe fn exit_to_real_mode() {
    unsafe {
        exit_unreal_mode();
    }
}

pub fn is_unreal_mode_enabled() -> bool {
    unsafe { UNREAL_MODE_ENABLED }
}

/// Ensures the A20 address line is enabled using BIOS services.
pub unsafe fn ensure_a20_enabled() {
    let mut ax: u16;
    unsafe {
        // First check if A20 is already enabled
        asm!(
            "mov ax, 0x2402",
            "int 0x15",
            out("ax") ax,
            options(preserves_flags, nostack)
        );

        if (ax & 0xFF) == 1 {
            return; // Already enabled
        }

        // Try to enable A20 using BIOS INT 15h, AX=2401h
        asm!(
            "mov ax, 0x2401",
            "int 0x15",
            out("ax") _,
            options(preserves_flags, nostack)
        );

        // As a fallback for older machines/emulators, we could use the KBC or Fast A20 port,
        // but BIOS is generally safest and sufficient for most 386+ systems.
    }
}
