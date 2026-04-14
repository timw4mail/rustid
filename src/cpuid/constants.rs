pub use crate::common::constants::*;

// ----------------------------------------------------------------------------
// ! Vendor Strings
// ----------------------------------------------------------------------------
pub const VENDOR_AMD: &str = "AuthenticAMD";
pub const VENDOR_CENTAUR: &str = "CentaurHauls";
pub const VENDOR_CYRIX: &str = "CyrixInstead";
pub const VENDOR_DMP: &str = "Vortex86 SoC";
pub const VENDOR_HYGON: &str = "HygonGenuine";
pub const VENDOR_INTEL: &str = "GenuineIntel";
pub const VENDOR_NEXGEN: &str = "NexGenDriven";
pub const VENDOR_NSC: &str = "Geode by NSC";
pub const VENDOR_RDC: &str = "Genuine  RDC";
pub const VENDOR_RISE: &str = "RiseRiseRise";
pub const VENDOR_SIS: &str = "SiS SiS SiS ";
pub const VENDOR_TRANSMETA: &str = "GenuineTMx86";
pub const VENDOR_UMC: &str = "UMC UMC UMC ";
pub const VENDOR_ZHAOXIN: &str = "  Shanghai  ";

// ----------------------------------------------------------------------------
// ! Easter Eggs
// ----------------------------------------------------------------------------

pub const AMD_EASTER_EGG_ADDR: u32 = 0x8FFF_FFFF;
#[cfg(target_arch = "x86")]
pub const RISE_EASTER_EGG_ADDR: u32 = 0x0000_5A4E;

// --------------------------------------------
// ! CPUID Leaves
// --------------------------------------------

/// CPUID leaf 0x00000000 - Maximum basic leaf
pub const LEAF_0: u32 = 0x0;

/// CPUID leaf 0x00000001 - Processor info and feature flags
pub const LEAF_1: u32 = 0x1;

/// CPUID leaf 0x00000002 - Cache descriptors
pub const LEAF_2: u32 = 0x2;

/// Intel deterministic cache parameters
pub const LEAF_4: u32 = 0x4;

/// CPUID leaf 0x00000007 - Extended feature flags
pub const LEAF_7: u32 = 0x7;

/// CPU extended topology v1
pub const LEAF_0B: u32 = 0xB;

/// CPUID leaf 0x00000016 - Intel Processor Frequency
pub const LEAF_16: u32 = 0x16;

/// Intel extended topology v2
pub const LEAF_1F: u32 = 0x1F;

/// Extended CPUID leaf 0x80000000 - Maximum extended leaf
pub const EXT_LEAF_0: u32 = 0x8000_0000;

/// Extended CPUID leaf 0x80000001 - Extended processor info
pub const EXT_LEAF_1: u32 = 0x8000_0001;

/// Cpu model string start
pub const EXT_LEAF_2: u32 = 0x8000_0002;

/// Cpu model string end
pub const EXT_LEAF_4: u32 = 0x8000_0004;

/// AMD/Transmeta L1 cache and TLB
pub const EXT_LEAF_5: u32 = 0x8000_0005;

/// AMD L2/L3 cache parameters
pub const EXT_LEAF_6: u32 = 0x8000_0006;

/// AMD address size and core count
pub const EXT_LEAF_8: u32 = 0x8000_0008;

/// AMD deterministic cache parameters
pub const EXT_LEAF_1D: u32 = 0x8000_001D;

/// AMD extended CPU topology
pub const EXT_LEAF_26: u32 = 0x8000_0026;

/// The max value of the extended CPUID leaf
pub const EXT_LEAF_MAX: u32 = 0x8000_FFFF;

/// Centaur/Zhaoxin vendor leaf base
pub const CENTAUR_LEAF_0: u32 = 0xC000_0000;

/// Centaur/Zhaoxin extended CPU features
pub const CENTAUR_LEAF_1: u32 = 0xC000_0001;

pub const CENTAUR_LEAF_2: u32 = 0xC000_0002;

/// Transmeta vendor leaf base
pub const TRANSMETA_LEAF_0: u32 = 0x8086_0000;

/// Transmeta extended CPU features
pub const TRANSMETA_LEAF_1: u32 = 0x8086_0001;

/// Transmeta CMS (Code Morphing Software)
pub const TRANSMETA_LEAF_2: u32 = 0x8086_0002;

/// Transmeta Cpu model string start
pub const TRANSMETA_LEAF_3: u32 = 0x8086_0003;

/// Transmeta Cpu model string end
pub const TRANSMETA_LEAF_6: u32 = 0x8086_0006;

/// Transmeta live CPU information
pub const TRANSMETA_LEAF_7: u32 = 0x8086_0007;
