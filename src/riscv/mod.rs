#![cfg(target_arch = "riscv64")]
//! RISC-V CPU detection.

pub mod brand;
pub mod cpu;
pub mod display;
pub mod features;
pub mod micro_arch;
pub mod os;
use crate::common::{CliFlags, CpuDisplay, UNK};
pub use cpu::*;
pub use micro_arch::{CpuArch, CpuCore};
pub use os::*;

pub(crate) trait TRiscvCpu {
    /// Returns the CPU model name, if available
    #[allow(unused)]
    fn model(&self) -> Option<&str> {
        None
    }

    #[allow(unused)]
    fn vendor(&self) -> &str;
}
