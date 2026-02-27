pub const IMPLEMENTER_MASK: usize = 0xFF000000;
pub const VARIANT_MASK: usize = 0x00F00000;
pub const ARCHITECTURE_MASK: usize = 0x000F0000;
pub const PART_MASK: usize = 0x0000FFF0;
pub const REVISION_MASK: usize = 0x0000000F;

pub const IMPLEMENTER_OFFSET: usize = 24;
pub const VARIANT_OFFSET: usize = 20;
pub const ARCHITECTURE_OFFSET: usize = 16;
pub const PART_OFFSET: usize = 4;
pub const REVISION_OFFSET: usize = 0;

#[derive(Debug, Default)]
pub struct Midr {
    pub implementer: usize,
    pub variant: usize,
    pub architecture: usize,
    pub part: usize,
    pub revision: usize,
}

impl Midr {
    pub fn new(midr: usize) -> Midr {
        Midr {
            implementer: midr_get_implementer(midr),
            variant: midr_get_variant(midr),
            architecture: midr_get_architecture(midr),
            part: midr_get_part(midr),
            revision: midr_get_revision(midr),
        }
    }
}

pub const fn midr_get_part(midr: usize) -> usize {
    (midr & PART_MASK) >> PART_OFFSET
}

pub const fn midr_get_architecture(midr: usize) -> usize {
    (midr & ARCHITECTURE_MASK) >> ARCHITECTURE_OFFSET
}

pub const fn midr_get_revision(midr: usize) -> usize {
    midr & REVISION_MASK
}

pub const fn midr_get_variant(midr: usize) -> usize {
    (midr & VARIANT_MASK) >> VARIANT_OFFSET
}

pub const fn midr_get_implementer(midr: usize) -> usize {
    (midr & IMPLEMENTER_MASK) >> IMPLEMENTER_OFFSET
}
