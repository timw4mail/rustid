use core::arch::asm;
use ufmt::uWrite;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use ufmt::uWrite;
            let _ = ufmt::uwrite!(&mut $crate::dos::DosWriter {}, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($($arg:tt)*) => {
        {
            $crate::print!($($arg)*);
            $crate::print!("\r\n");
        }
    };
}

pub struct DosWriter;

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
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x02_u8,
            in("dl") ch,
            out("al") _,
        );
    }
}

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
