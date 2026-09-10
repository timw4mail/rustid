//! Menu command constants for the Linux GTK GUI.

#[cfg(x86_cpu)]
pub const IDM_FILE_OPEN: u32 = 101;
#[cfg(x86_cpu)]
pub const IDM_FILE_EXPORT: u32 = 102;
pub const IDM_FILE_COPY: u32 = 103;
pub const IDM_FILE_REFRESH: u32 = 104;
pub const IDM_FILE_EXIT: u32 = 105;

pub const IDM_MODE_STANDARD: u32 = 201;
pub const IDM_MODE_DEBUG: u32 = 202;
pub const IDM_MODE_EVERYTHING: u32 = 203;
#[cfg(x86_cpu)]
pub const IDM_MODE_DUMP: u32 = 204;

pub const IDM_OPT_COLOR: u32 = 301;
pub const IDM_OPT_DARK_THEME: u32 = 302;
pub const IDM_OPT_VERBOSE: u32 = 303;
pub const IDM_OPT_COMPACT: u32 = 304;

pub const IDM_HELP_ABOUT: u32 = 401;
