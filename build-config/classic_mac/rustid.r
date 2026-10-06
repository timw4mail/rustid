/* Classic Macintosh Resource Definition for rustid */

#include "MacTypes.r"
#include "Multiverse.r"
#include "Icons.r"
#include "rustid_icons.r"

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
    };
};

/* About Alert Dialog */
resource 'ALRT' (128, "About", purgeable) {
    { 100, 100, 220, 420 },
    128,
    {
        OK, visible, silent;
        OK, visible, silent;
        OK, visible, silent;
        OK, visible, silent;
    },
    alertPositionMainScreen
};

resource 'DITL' (128, "About", purgeable) {
    {
        { 85, 230, 105, 300 }, Button { enabled, "OK" };
        { 15, 60, 35, 300 }, StaticText { disabled, "^0" };
        { 40, 60, 75, 300 }, StaticText { disabled, "^1" };
        { 15, 15, 47, 47 }, Icon { disabled, 128 };
    }
};

/* Main Window Template */
resource 'WIND' (128, "MainWindow", purgeable) {
    { 60, 40, 420, 580 },
    zoomDocProc,
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

/* Application Signature */
type 'RsId' as 'STR ';
resource 'RsId' (0, purgeable) {
    "Rustid " RUSTID_VERSION_STR
};

/* Bundle and File References */
resource 'BNDL' (128, purgeable) {
    'RsId',
    0,
    {
        'ICN#', {
            0, 128
        },
        'FREF', {
            0, 128
        }
    }
};

resource 'FREF' (128, purgeable) {
    'APPL',
    0,
    ""
};

/* Code Fragment Resource (enables native PowerPC execution for PPC, Carbon, and Fat binaries) */
#if defined(TARGET_PPC) || defined(TARGET_FAT) || defined(TARGET_CARBON)
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

/* Carbon Application Resource (identifies native Carbon app for Mac OS X LaunchServices & CarbonLib) */
#if defined(TARGET_CARBON)
type 'carb' {
};

resource 'carb' (0) {
};
#endif
