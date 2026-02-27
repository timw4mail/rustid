pub const IMPLEMENTER_MASK: u32 = 0xFF000000;
pub const VARIANT_MASK: u32 = 0x00F00000;
pub const ARCHITECTURE_MASK: u32 = 0x000F0000;
pub const PART_MASK: u32 = 0x0000FFF0;
pub const REVISION_MASK: u32 = 0x0000000F;

pub const IMPLEMENTER_OFFSET: usize = 24;
pub const VARIANT_OFFSET: usize = 20;
pub const ARCHITECTURE_OFFSET: usize = 16;
pub const PART_OFFSET: usize = 4;
pub const REVISION_OFFSET: usize = 0;

#[derive(Debug, Default)]
pub struct Midr {
    pub implementer: u32,
    pub variant: u32,
    pub architecture: u32,
    pub part: u32,
    pub revision: u32,
}

impl Midr {
    pub fn new(midr: u32) -> Midr {
        Midr {
            implementer: midr_get_implementer(midr),
            variant: midr_get_variant(midr),
            architecture: midr_get_architecture(midr),
            part: midr_get_part(midr),
            revision: midr_get_revision(midr),
        }
    }
}

pub const fn midr_get_part(midr: u32) -> u32 {
    (midr & PART_MASK) >> PART_OFFSET
}

pub const fn midr_get_architecture(midr: u32) -> u32 {
    (midr & ARCHITECTURE_MASK) >> ARCHITECTURE_OFFSET
}

pub const fn midr_get_revision(midr: u32) -> u32 {
    midr & REVISION_MASK
}

pub const fn midr_get_variant(midr: u32) -> u32 {
    (midr & VARIANT_MASK) >> VARIANT_OFFSET
}

pub const fn midr_get_implementer(midr: u32) -> u32 {
    (midr & IMPLEMENTER_MASK) >> IMPLEMENTER_OFFSET
}
