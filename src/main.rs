#![cfg_attr(all(not(test), target_os = "none"), no_std)]
#![cfg_attr(all(not(test), target_os = "none"), no_main)]

pub mod cpuid;

use cpuid::Cpu;

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(all(not(test), target_os = "none"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();
    loop {}
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only supports x86 and x86_64 architectures.");

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
