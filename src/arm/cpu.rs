//! Contains the Cpu struct for ARM.

use super::brand::Vendor;
use crate::TCpu;
use std::println;

use crate::arm::micro_arch::Midr;

#[derive(Debug)]
pub struct Cpu {
    pub raw_midr: usize,
    pub midr: Midr,
    pub vendor: String,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        let raw_midr = super::get_midr();
        let midr = Midr::new(raw_midr);
        Self {
            raw_midr,
            midr,
            vendor: Vendor::from(midr.implementer).into(),
        }
    }
}

impl TCpu for Cpu {
    fn debug(&self) {
        println!("{:#?}", self);
    }

    fn display_table(&self) {
        println!();
        println!("Main ID Register (MIDR): 0x{:X}", self.raw_midr);
        println!("Implementer: 0x{:X}", self.midr.implementer);
        println!("Variant: 0x{:X}", self.midr.variant);
        println!("Part Number: 0x{:X}", self.midr.part);
        println!("Revision: 0x{:X}", self.midr.revision);
        println!("{:#?}", self);
        println!();
    }
}
