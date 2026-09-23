//! Classic Macintosh Gestalt selectors, machine types, and trap interfaces.

pub const GESTALT_MACH_TYPE: u32 = 0x6d61_6368; // 'mach'
pub const GESTALT_CPU_TYPE: u32 = 0x6370_7574; // 'cput' or 'mach' in early System
pub const GESTALT_FPU_TYPE: u32 = 0x6670_7520; // 'fpu '
pub const GESTALT_MMU_TYPE: u32 = 0x6d6d_7520; // 'mmu '
pub const GESTALT_PROC_CLK: u32 = 0x7063_6c6b; // 'pclk'
pub const GESTALT_BUS_CLK: u32 = 0x6263_6c6b; // 'bclk'
pub const GESTALT_SYS_VERSION: u32 = 0x7379_7376; // 'sysv'
pub const GESTALT_PHYS_RAM: u32 = 0x7261_6d20; // 'ram '
pub const GESTALT_ROM_VERSION: u32 = 0x726f_6d20; // 'rom '

/// Identifies a classic Macintosh model from its Gestalt machine code.
pub fn get_mac_model_name(mach_id: u32) -> &'static str {
    match mach_id {
        1 => "Macintosh 128K",
        2 => "Macintosh 512K",
        3 => "Macintosh XL",
        4 => "Macintosh Plus",
        5 => "Macintosh SE",
        6 => "Macintosh II",
        7 => "Macintosh IIx",
        8 => "Macintosh IIcx",
        9 => "Macintosh SE/30",
        10 => "Macintosh Portable",
        11 => "Macintosh IIci",
        12 => "Macintosh IIfx",
        14 => "Macintosh Classic",
        15 => "Macintosh IIsi",
        17 => "Macintosh LC",
        18 => "Macintosh Quadra 900",
        19 => "PowerBook 170",
        20 => "Macintosh Quadra 700",
        21 => "Macintosh Classic II",
        22 => "PowerBook 100",
        23 => "PowerBook 140",
        24 => "Macintosh Quadra 950",
        25 => "Macintosh LC III",
        26 => "PowerBook 160",
        27 => "PowerBook 180",
        28 => "PowerBook Duo 210",
        29 => "PowerBook Duo 230",
        30 => "PowerBook Duo 250",
        32 => "PowerBook 165c",
        33 => "Macintosh Centris 650",
        34 => "PowerBook 180c",
        35 => "PowerBook Duo 270c",
        36 => "Macintosh Quadra 800",
        37 => "Macintosh Quadra 650",
        38 => "PowerBook 165",
        39 => "Macintosh Color Classic",
        40 => "Macintosh Centris 610",
        41 => "Macintosh Quadra 610",
        42 => "PowerBook 145",
        43 => "Macintosh LC II",
        44 => "PowerBook 520 / 540",
        45 => "Macintosh Quadra 605 / LC 475",
        48 => "Macintosh TV",
        49 => "Macintosh LC 520",
        50 => "Macintosh LC 550",
        52 => "Macintosh Quadra 660AV",
        53 => "Macintosh Quadra 840AV",
        60 => "Macintosh LC 575",
        61 => "Macintosh Quadra 630 / LC 580",
        70 => "Power Macintosh 6100",
        71 => "Power Macintosh 7100",
        72 => "Power Macintosh 8100",
        75 => "Power Macintosh 5200",
        76 => "Power Macintosh 6200",
        84 => "Power Macintosh 7500",
        85 => "Power Macintosh 8500",
        86 => "Power Macintosh 9500",
        98 => "Power Macintosh 7200",
        108 => "PowerBook 3400c",
        110 => "PowerBook G3",
        120 => "Power Macintosh G3 (Beige)",
        121 => "Power Macintosh G3 (Blue & White)",
        126 => "PowerBook G3 (Wallstreet)",
        127 => "PowerBook G3 (Lombard)",
        130 => "PowerBook G3 (Pismo)",
        131 => "iMac G3",
        406 => "Power Macintosh G4 (Sawtooth)",
        407 => "Power Macintosh G4 (Gigabit)",
        414 => "PowerBook G4 (Titanium)",
        _ => "Macintosh (Generic)",
    }
}

/// Formats a Classic Mac OS system version (e.g. 0x0755 -> "System 7.5.5").
pub fn format_mac_os_version(version_bcd: u32) -> alloc::string::String {
    let major = (version_bcd >> 8) & 0xFF;
    let minor = (version_bcd >> 4) & 0x0F;
    let patch = version_bcd & 0x0F;

    let prefix = if major < 8 {
        "System"
    } else {
        "Mac OS"
    };

    if patch == 0 {
        alloc::format!("{} {}.{}", prefix, major, minor)
    } else {
        alloc::format!("{} {}.{}.{}", prefix, major, minor, patch)
    }
}
