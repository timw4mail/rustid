#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");
pub mod cpuid;

use cpuid::Cpu;

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(all(not(test), target_os = "none"))]
mod intrinsics {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        let mut i = 0;
        while i < n {
            unsafe {
                *dest.add(i) = *src.add(i);
            }

            i += 1;
        }
        dest
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
        let mut i = 0;
        while i < n {
            unsafe {
                let a = *s1.add(i);
                let b = *s2.add(i);

                if a != b {
                    return a as i32 - b as i32;
                }
            }
            i += 1;
        }
        0
    }
}

#[cfg(all(not(test), target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // VGA "heartbeat": write '!' to top-left corner
    unsafe {
        core::ptr::write_volatile(0xB8000 as *mut u16, 0x0F21);
    }

    main();

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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    Cpu::new().display();

    // println!("CPU Vendor:    {}", cpu.vendor_id());
    // println!("CPU Brand:     {}", cpu.brand_string());
    // let (stepping, model, family) = cpu.signature();
    // println!(
    //     "CPU Signature: Family {}, Model {}, Stepping {}",
    //     family, model, stepping
    // );
    // println!("Logical Cores: {}", cpu.logical_cores());

    // println!("Features:");
    // println!("  SSE:      {}", cpu.has_sse());
    // println!("  SSE2:     {}", cpu.has_sse2());
    // println!("  SSE3:     {}", cpu.has_sse3());
    // println!("  SSE4.1:   {}", cpu.has_sse41());
    // println!("  SSE4.2:   {}", cpu.has_sse42());
    // println!("  AVX:      {}", cpu.has_avx());
    // println!("  AVX2:     {}", cpu.has_avx2());
    // println!("  AVX-512F: {}", cpu.has_avx512f());
    // println!("  FMA:      {}", cpu.has_fma());
    // println!("  BMI1:     {}", cpu.has_bmi1());
    // println!("  BMI2:     {}", cpu.has_bmi2());
    // println!("  RDRAND:   {}", cpu.has_rdrand());
}
