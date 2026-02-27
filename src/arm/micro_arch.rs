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

#[derive(Debug, Default, Copy, Clone)]
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
            implementer: (midr & IMPLEMENTER_MASK) >> IMPLEMENTER_OFFSET,
            variant: (midr & VARIANT_MASK) >> VARIANT_OFFSET,
            architecture: (midr & ARCHITECTURE_MASK) >> ARCHITECTURE_OFFSET,
            part: (midr & PART_MASK) >> PART_OFFSET,
            revision: midr & REVISION_MASK,
        }
    }
}
