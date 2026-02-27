//! DOS/16-bit environment support for rustid.
//!
//! This module provides DOS-specific implementations including console output
//! via DOS INT 21h interrupts and a custom panic handler for bare-metal environments.

use core::arch::asm;
use core::fmt::Write;
/// Custom panic handler for no-std environments.
/// Loops indefinitely on panic to prevent undefined behavior.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Entry point for DOS executables.
/// Called by the DOS loader and invokes the main CLI function.
#[cfg(not(test))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".startup")]
pub extern "C" fn _start() -> ! {
    use crate::cli_main;

    cli_main();

    exit();
}
/// Prints a formatted string to the DOS console.
/// Supports both literal strings and format strings.
#[macro_export]
macro_rules! print {
    ($s:literal) => {
        $crate::dos::_print_str($s)
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!(&mut $crate::dos::DosWriter {}, $($arg)*);
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
fn printc(ch: u8) {
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x02_u8,
            in("dl") ch,
            out("al") _,
        );
    }
}

/// Exits the program and returns control to DOS using INT 21h, AH=4Ch.
pub fn exit() -> ! {
    // Exit to DOS via INT 21h, AH=4Ch
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x4C_u8,
            in("al") 0_u8,
            options(noreturn)
        );
    }
}
