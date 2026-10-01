/* Classic Macintosh Resource Definition for rustid */

#include "MacTypes.r"
#include "Multiverse.r"

/* Menu Bar */
resource 'MBAR' (128, "MenuBar", purgeable) {
    { 128, 129, 130, 131 };
};

/* Apple Menu */
resource 'MENU' (128, "Apple", preload) {
    128, textMenuProc,
    0b1111111111111111111111111111111,
    enabled,
    apple,
    {
        "About Rustid...", noIcon, noKey, noMark, plain;
        "-", noIcon, noKey, noMark, plain;
    };
};

/* File Menu */
resource 'MENU' (129, "File", preload) {
    129, textMenuProc,
    0b1111111111111111111111111111111,
    enabled,
    "File",
    {
        "Export Report...", noIcon, "E", noMark, plain;
        "-", noIcon, noKey, noMark, plain;
        "Refresh", noIcon, "R", noMark, plain;
        "-", noIcon, noKey, noMark, plain;
        "Quit", noIcon, "Q", noMark, plain;
    };
};

/* Edit Menu */
resource 'MENU' (130, "Edit", preload) {
    130, textMenuProc,
    0b1111111111111111111111111111111,
    enabled,
    "Edit",
    {
        "Undo", noIcon, "Z", noMark, plain;
        "-", noIcon, noKey, noMark, plain;
        "Cut", noIcon, "X", noMark, plain;
        "Copy", noIcon, "C", noMark, plain;
        "Paste", noIcon, "V", noMark, plain;
        "Clear", noIcon, noKey, noMark, plain;
    };
};

/* View Menu */
resource 'MENU' (131, "View", preload) {
    131, textMenuProc,
    0b1111111111111111111111111111111,
    enabled,
    "View",
    {
        "Standard", noIcon, "1", noMark, plain;
        "Debug", noIcon, "2", noMark, plain;
        "Everything", noIcon, "3", noMark, plain;
        "-", noIcon, noKey, noMark, plain;
        "Colors", noIcon, noKey, check, plain;
        "Dark Theme", noIcon, noKey, noMark, plain;
        "Verbose", noIcon, noKey, noMark, plain;
        "Compact", noIcon, noKey, noMark, plain;
    };
};

/* Main Window Template */
resource 'WIND' (128, "MainWindow", purgeable) {
    { 60, 40, 420, 580 },
    documentProc,
    invisible,
    goAway,
    0x0,
    "Rustid",
    centerParentWindowScreen
};

/* MultiFinder / Memory Configuration */
resource 'SIZE' (-1) {
    reserved,
    acceptSuspendResumeEvents,
    reserved,
    canBackground,
    multiFinderAware,
    backgroundAndForeground,
    dontGetFrontClicks,
    ignoreChildDiedEvents,
    is32BitCompatible,
    isHighLevelEventAware,
    onlyLocalHLEvents,
    notStationeryAware,
    dontUseTextEditServices,
    notDisplayManagerAware,
    reserved,
    reserved,
    2048 * 1024,      /* Preferred Size: 2048 KB */
    1024 * 1024       /* Minimum Size: 1024 KB */
};

/* Application Version */
#ifndef VERSION_MAJOR_BCD
#define VERSION_MAJOR_BCD 0x02
#endif
#ifndef VERSION_MINOR_BCD
#define VERSION_MINOR_BCD 0x30
#endif
#ifndef RUSTID_VERSION_STR
#define RUSTID_VERSION_STR "2.3.0"
#endif

resource 'vers' (1) {
    VERSION_MAJOR_BCD, VERSION_MINOR_BCD, release, 0x00,
    verUS,
    RUSTID_VERSION_STR,
    "Rustid " RUSTID_VERSION_STR " (Classic Mac OS 68k & PPC)"
};

resource 'vers' (2) {
    VERSION_MAJOR_BCD, VERSION_MINOR_BCD, release, 0x00,
    verUS,
    RUSTID_VERSION_STR,
    "Rustid CPU Identification Utility"
};

/* Bundle and File References */
resource 'BNDL' (128) {
    'RSID',
    0,
    {
        'FREF', { 0, 128 };
        'ICN#', { 0, 128 };
    };
};

resource 'FREF' (128) {
    'APPL',
    0,
    ""
};

/* Code Fragment Resource (enables native PowerPC execution for PPC and Fat binaries) */
#if defined(TARGET_PPC) || defined(TARGET_FAT)
#include "CodeFragments.r"

resource 'cfrg' (0) {
    {
        kPowerPCCFragArch, kIsCompleteCFrag, kNoVersionNum, kNoVersionNum,
        kDefaultStackSize, kNoAppSubFolder,
        kApplicationCFrag, kDataForkCFragLocator, kZeroOffset, kCFragGoesToEOF,
        "rustid"
    }
};
#endif
