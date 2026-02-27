//! Contains the Cpu struct for ARM.

use crate::arm::fns;

#[cfg(target_os = "none")]
use crate::println;
use core::default::Default;
use core::fmt::Debug;
#[cfg(not(target_os = "none"))]
use std::println;

use crate::arm::micro_arch::Midr;

#[derive(Debug)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub implementer: u8,
    pub variant: u8,
    pub part_number: u16,
    pub revision: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        let midr = fns::get_midr();
        Self {
            raw_midr: midr,
            midr: Midr::new(midr),
            implementer: ((midr >> 24) & 0xFF) as u8,
            variant: ((midr >> 20) & 0xF) as u8,
            part_number: ((midr >> 4) & 0xFFF) as u16,
            revision: (midr & 0xF) as u8,
        }
    }

    pub fn debug(&self) {
        println!("{:#?}", self);
    }

    pub fn display_table(&self) {
        println!();
        println!("Main ID Register (MIDR): 0x{:X}", self.raw_midr);
        println!("Implementer: 0x{:X}", self.implementer);
        println!("Variant: 0x{:X}", self.variant);
        println!("Part Number: 0x{:X}", self.part_number);
        println!("Revision: 0x{:X}", self.revision);
        println!();
    }
}
