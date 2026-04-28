#![cfg(all(target_arch = "x86", target_os = "none"))]
//! Unreal Mode (Big Real Mode) support for x86 DOS
//!
//! This module enables flat 32-bit addressing in real mode by:
//! 1. Entering protected mode to load 32-bit segment descriptors
//! 2. Exiting protected mode while keeping the 32-bit segment limits
//! 3. Result: flat 32-bit addressing in real mode

use core::arch::asm;

/// Enable unreal mode for flat 32-bit addressing
///
/// This function:
/// - Enters protected mode to establish 32-bit segment limits
/// - Exits protected mode while keeping those 32-bit limits
/// - Returns to real mode with 4GB flat address space
pub unsafe fn enable_unreal_mode() {
    unsafe {
        asm!(
            ".code16",
            "push ds",
            "push es",
            "push ss",

            // Calculate linear address of GDT
            "mov eax, cs",
            "movzx eax, ax",
            "shl eax, 4",
            "xor ebx, ebx",
            "lea bx, [4f]", // Offset of GDT entries
            "add eax, ebx",
            
            // Patch GDT pointer base
            "lea bx, [3f]", // Offset of GDT pointer base field
            ".byte 0x66, 0x89, 0x07", // mov [bx], eax

            // Load GDT
            "lea bx, [2f]", // Offset of GDT pointer
            ".byte 0x0F, 0x01, 0x17", // lgdt [bx]

            // Switch to PM
            "mov eax, cr0",
            "or al, 1",
            "mov cr0, eax",
            "jmp 5f",
            "5:",
            "mov ax, 8", // GDT selector 1 (data)
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",

            // Switch back to RM
            "and al, 0xFE",
            "mov cr0, eax",
            "jmp 6f",
            "6:",

            "pop ss",
            "pop es",
            "pop ds",
            "jmp 7f",

            ".align 4",
            "2:",                       // GDT Pointer
            ".word 15",                 // Limit
            "3:",                       // Base placeholder
            ".long 0",                  // Base
            "4:",                       // GDT Entries
            ".quad 0",                  // Null
            ".quad 0x00CF92000000FFFF", // Data
            "7:",
            options(nostack, preserves_flags),
        );
    }
}

/// Enable A20 line for memory access above 1MB
pub unsafe fn enable_a20() {
    // Basic A20 enable via Fast A20 Gate (Port 92h)
    // In a real DOS environment, we might want to use BIOS INT 15h AX=2401h
    unsafe {
        asm!(
            "in al, 0x92",
            "or al, 2",
            "out 0x92, al",
            options(preserves_flags, nostack)
        );
    }
}
