//! Contains the Cpu struct for PowerPC.

use crate::ppc::fns;
use std::println;

#[derive(Debug)]
pub struct Cpu {
    pub pvr: u32,
    pub version: u16,
    pub revision: u16,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        let pvr = fns::get_pvr();
        Self {
            pvr,
            version: (pvr >> 16) as u16,
            revision: (pvr & 0xFFFF) as u16,
        }
    }

    pub fn debug(&self) {
        println!("{:#?}", self);
    }

    pub fn display_table(&self) {
        println!();
        println!("Processor Version Register (PVR): 0x{:X}", self.pvr);
        println!("CPU Version: 0x{:X}", self.version);
        println!("CPU Revision: 0x{:X}", self.revision);
        println!();
    }
}
