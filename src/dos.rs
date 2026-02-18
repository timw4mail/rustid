use core::arch::asm;
use core::fmt::{self, Write};
use ufmt::uWrite;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::dos::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\r\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\r\n"), $($arg)*)
    };
}

pub fn _print(args: fmt::Arguments) {
    let mut writer = DosWriter {};
    writer.write_fmt(args).unwrap();
}

pub struct DosWriter;

impl Write for DosWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            printc(c);
        }
        Ok(())
    }
}

impl uWrite for DosWriter {
    type Error = ();
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for &b in s.as_bytes() {
            printc(b);
        }
        Ok(())
    }
}

pub fn printc(ch: u8) {
    // unsafe { asm!("int 0x21", in("ah") 0x02_u8, in("dl") ch) }

    unsafe {
        core::arch::asm!(
            "int 0x21",
            in("ah") 0x02_u8,
            in("dl") ch,
            clobber_abi("system"),
        );
    }
}

pub fn exit() -> ! {
    // Exit to DOS via INT 21h, AH=4Ch
    unsafe {
        core::arch::asm!(
        "int 0x21",
        in("ah") 0x4C_u8,
        in("al") 0_u8,
        options(noreturn)
        );
    }
}
