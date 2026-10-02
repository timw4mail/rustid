//! Standalone Classic Mac OS C CPU detection and report generator engine.
//! Used for 68k builds and Universal fat binaries where native Rust cannot compile.

#include "classic_mac_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef RUSTID_VERSION
#define RUSTID_VERSION "2.3.0"
#endif

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Gestalt.h>
#include <Types.h>
#if defined(__powerpc__) || defined(__ppc__)
#include <Multiverse.h>
#endif

#ifndef gestaltProcessorType
#define gestaltProcessorType 'proc'
#endif

#ifndef gestaltNativeCPUtype
#define gestaltNativeCPUtype 'cput'
#endif

#ifndef gestaltSysArchitecture
#define gestaltSysArchitecture 'sysa'
#endif

#ifndef gestalt68k
#define gestalt68k 1
#endif

#ifndef gestaltPowerPC
#define gestaltPowerPC 2
#endif

#ifndef gestaltProcClkSpeed
#define gestaltProcClkSpeed 'pclk'
#endif

#ifndef gestaltBusClkSpeed
#define gestaltBusClkSpeed 'bclk'
#endif
#endif

enum {
    VIEW_STANDARD   = 201,
    VIEW_DEBUG      = 202,
    VIEW_EVERYTHING = 203
};

static const CRgbColor PALETTE_BG __attribute__((unused)) = {255, 255, 255};
static const CRgbColor PALETTE_LABEL     = {9, 134, 88};
static const CRgbColor PALETTE_SUBLABEL  = {4, 81, 165};
static const CRgbColor PALETTE_BODY      = {30, 30, 30};
static const CRgbColor PALETTE_HIGHLIGHT = {163, 21, 21};

typedef struct {
    uint32_t cpu_type;  // 1=68000, 2=68010, 3=68020, 4=68030, 5=68040, 6=68060
    uint32_t clock_mhz;
    uint32_t bus_mhz;
    uint32_t fpu_type;  // 0=None, 1=68881, 2=68882, 3=68040
    uint32_t mmu_type;  // 0=None, 1=AMU, 2=68851, 3=68030, 4=68040
} MacModelSpec;

typedef struct {
    const char* id;
    const char* name;
} MacModelIdMap;

static const MacModelIdMap kMacModelIdTable[] = {
    // Power Macintosh / Power Mac (G3, G4, G5)
    {"PowerMac1,1", "Power Macintosh G3 (Blue and White)"},
    {"PowerMac1,2", "Power Mac G4 (PCI Graphics)"},
    {"PowerMac2,1", "iMac (Slot Loading)"},
    {"PowerMac2,2", "iMac (Summer 2000)"},
    {"PowerMac3,1", "Power Mac G4 (AGP Graphics)"},
    {"PowerMac3,2", "Power Mac G4 (Uni-N)"},
    {"PowerMac3,3", "Power Mac G4 (Gigabit Ethernet)"},
    {"PowerMac3,4", "Power Mac G4 (Digital Audio)"},
    {"PowerMac3,5", "Power Mac G4 (Quicksilver)"},
    {"PowerMac3,6", "Power Mac G4 (FW 800 | Mirrored Drive Doors)"},
    {"PowerMac4,1", "iMac (Early/Summer 2001)"},
    {"PowerMac4,2", "iMac (15-inch Flat Panel)"},
    {"PowerMac4,4", "eMac"},
    {"PowerMac4,5", "iMac (17-inch Flat Panel)"},
    {"PowerMac5,1", "Power Mac G4 Cube"},
    {"PowerMac6,1", "iMac (15/17-inch Flat Panel, 1GHz/USB 2.0)"},
    {"PowerMac6,3", "iMac (15/17/20-inch USB 2.0)"},
    {"PowerMac6,4", "eMac (USB 2.0 | 2005)"},
    {"PowerMac7,2", "Power Mac G5 (June 2003 | Early 2005)"},
    {"PowerMac7,3", "Power Mac G5 (June 2004 | Early 2005)"},
    {"PowerMac8,1", "iMac G5 (17/20-inch)"},
    {"PowerMac8,2", "iMac G5 (Ambient Light Sensor)"},
    {"PowerMac9,1", "Power Mac G5 (Late 2004)"},
    {"PowerMac10,1", "Mac mini"},
    {"PowerMac10,2", "Mac mini (Late 2005)"},
    {"PowerMac11,2", "Power Mac G5 (Late 2005)"},
    {"PowerMac12,1", "iMac G5 (17/20-inch iSight)"},

    // PowerBook & iBook (G3, G4)
    {"PowerBook1,1", "PowerBook G3 (Bronze Keyboard)"},
    {"PowerBook2,1", "iBook"},
    {"PowerBook2,2", "iBook (FireWire)"},
    {"PowerBook3,1", "PowerBook (Firewire)"},
    {"PowerBook3,2", "PowerBook G4 (Titanium)"},
    {"PowerBook3,3", "PowerBook G4 (Gigabit Ethernet)"},
    {"PowerBook3,4", "PowerBook G4 (DVI)"},
    {"PowerBook3,5", "PowerBook G4 (1GHz/867MHz)"},
    {"PowerBook4,1", "iBook (Dual USB | late 2001)"},
    {"PowerBook4,2", "iBook (14.1 LCD)"},
    {"PowerBook4,3", "iBook (14.1 LCD 16 VRAM | Opaque 16 VRAM | 32 VRAM)"},
    {"PowerBook5,1", "PowerBook G4 (17-inch)"},
    {"PowerBook5,2", "PowerBook G4 (15-inch FW 800)"},
    {"PowerBook5,3", "PowerBook G4 (17-inch 1.33GHz)"},
    {"PowerBook5,4", "PowerBook G4 (15-inch 1.5/1.33GHz)"},
    {"PowerBook5,5", "PowerBook G4 (17-inch 1.5GHz)"},
    {"PowerBook5,6", "PowerBook G4 (15-inch 1.66/1.5GHz)"},
    {"PowerBook5,7", "PowerBook G4 (17-inch 1.67GHz)"},
    {"PowerBook5,8", "PowerBook G4 (15-inch Double-Layer SD)"},
    {"PowerBook5,9", "PowerBook G4 (17-inch Double-Layer SD)"},
    {"PowerBook6,1", "PowerBook G4 (12-inch)"},
    {"PowerBook6,2", "PowerBook G4 (12-inch DVI)"},
    {"PowerBook6,3", "iBook G4"},
    {"PowerBook6,4", "PowerBook G4 (12-inch 1.33GHz)"},
    {"PowerBook6,5", "iBook G4 (2004)"},
    {"PowerBook6,7", "iBook G4 (Mid 2005)"},
    {"PowerBook6,8", "PowerBook G4 (12-inch 1.5GHz)"},

    // iMac (Original) & Xserve
    {"iMac,1", "iMac (Original, 5 Flavors)"},
    {"RackMac1,1", "XServe"},
    {"RackMac1,2", "XServe (Slot Load | Cluster Node)"},
    {"RackMac3,1", "XServe G5"},

    // OldWorld Open Firmware PCI models
    {"AAPL,PowerMac G3", "Power Macintosh G3 (Beige)"},
    {"AAPL,Gossamer", "Power Macintosh G3 (Beige)"},
    {"AAPL,7200", "Power Macintosh 7200"},
    {"AAPL,7300", "Power Macintosh 7300"},
    {"AAPL,7500", "Power Macintosh 7500"},
    {"AAPL,7600", "Power Macintosh 7600"},
    {"AAPL,8500", "Power Macintosh 8500"},
    {"AAPL,8600", "Power Macintosh 8600"},
    {"AAPL,9500", "Power Macintosh 9500"},
    {"AAPL,9600", "Power Macintosh 9600"},
    {"AAPL,3400/240", "PowerBook 3400c"},
    {"AAPL,e411", "PowerBook 3400c"},
    {"AAPL,3500", "PowerBook G3"},

    // Early Intel models
    {"MacBook1,1", "MacBook (13-inch)"},
    {"MacBookPro1,1", "MacBook Pro"},
    {"MacBookPro1,2", "MacBook Pro (17-inch)"},
    {"Macmini1,1", "Mac mini (Early/Late 2006)"},
    {"MacPro1,1", "Mac Pro"},
    {"iMac4,1", "iMac (Early 2006)"},
    {"iMac4,2", "iMac (Mid 2006 17-inch)"},
    {"Xserve1,1", "XServe (Late 2006)"}
};

const char* classic_mac_model_from_identifier(const char* identifier) {
    if (!identifier || !identifier[0]) {
        return NULL;
    }
    while (*identifier == ' ') identifier++;
    if (strncmp(identifier, "Apple ", 6) == 0) {
        identifier += 6;
        while (*identifier == ' ') identifier++;
    }

    size_t count = sizeof(kMacModelIdTable) / sizeof(kMacModelIdTable[0]);
    for (size_t i = 0; i < count; i++) {
        if (strcmp(identifier, kMacModelIdTable[i].id) == 0) {
            return kMacModelIdTable[i].name;
        }
    }
    return NULL;
}

static __attribute__((unused)) bool IsAppleModelIdentifier(const char* s) {
    if (!s || !s[0]) return false;
    if (classic_mac_model_from_identifier(s) != NULL) {
        return true;
    }

    static const char* const prefixes[] = {
        "PowerMac",
        "PowerBook",
        "iMac",
        "RackMac",
        "Macmini",
        "MacPro",
        "MacBook",
        "Mac"
    };
    for (size_t i = 0; i < sizeof(prefixes) / sizeof(prefixes[0]); i++) {
        size_t plen = strlen(prefixes[i]);
        if (strncmp(s, prefixes[i], plen) == 0) {
            const char* rest = s + plen;
            const char* comma = strchr(rest, ',');
            if (comma && comma > rest && *(comma + 1) != '\0') {
                return true;
            }
            if (strcmp(prefixes[i], "iMac") == 0 && *rest == ',' && *(rest + 1) != '\0') {
                return true;
            }
        }
    }
    return false;
}

#if (defined(__powerpc__) || defined(__ppc__)) && (defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__))
typedef struct RegEntryID {
    uint8_t opaque[16];
} RegEntryID;

typedef uint32_t RegPropertyValueSize;
typedef int32_t (*RegistryEntryIDInitProc)(RegEntryID *id);
typedef int32_t (*RegistryCStrEntryLookupProc)(const RegEntryID *searchPointID, const char *pathName, RegEntryID *foundEntry);
typedef int32_t (*RegistryPropertyGetProc)(const RegEntryID *entryID, const char *propertyName, void *propertyValue, RegPropertyValueSize *propertySize);
typedef int32_t (*RegistryEntryIDDisposeProc)(RegEntryID *id);

static void MakePStr(const char* cstr, unsigned char* pstr) {
    size_t len = strlen(cstr);
    if (len > 255) len = 255;
    pstr[0] = (unsigned char)len;
    memcpy(&pstr[1], cstr, len);
}
#endif

bool classic_mac_probe_model_identifier(char* out_buf, size_t out_buf_size) {
    if (!out_buf || out_buf_size == 0) return false;
    out_buf[0] = '\0';

#if (defined(__powerpc__) || defined(__ppc__)) && (defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__))
    ConnectionID conn = NULL;
    Ptr mainAddr = NULL;
    Str255 errName;
    unsigned char pLib[64];
    MakePStr("NameRegistryLib", pLib);

    OSErr err = GetSharedLibrary(pLib, kPowerPCArch, kReferenceCFrag, &conn, &mainAddr, errName);
    if (err == noErr && conn != NULL) {
        Ptr symLookup = NULL, symPropGet = NULL, symDispose = NULL, symInit = NULL;
        SymClass symClass = 0;
        unsigned char pSym[64];

        MakePStr("RegistryCStrEntryLookup", pSym);
        OSErr errLookup = FindSymbol(conn, pSym, &symLookup, &symClass);
        MakePStr("RegistryPropertyGet", pSym);
        OSErr errPropGet = FindSymbol(conn, pSym, &symPropGet, &symClass);
        MakePStr("RegistryEntryIDDispose", pSym);
        FindSymbol(conn, pSym, &symDispose, &symClass);
        MakePStr("RegistryEntryIDInit", pSym);
        FindSymbol(conn, pSym, &symInit, &symClass);

        if (errLookup == noErr && errPropGet == noErr && symLookup && symPropGet) {
            RegistryCStrEntryLookupProc pLookup = (RegistryCStrEntryLookupProc)symLookup;
            RegistryPropertyGetProc pPropGet = (RegistryPropertyGetProc)symPropGet;
            RegistryEntryIDDisposeProc pDispose = (RegistryEntryIDDisposeProc)symDispose;
            RegistryEntryIDInitProc pInit = (RegistryEntryIDInitProc)symInit;

            RegEntryID entry;
            if (pInit) pInit(&entry);

            static const char* const tree_paths[] = {
                "Devices:device-tree",
                ":Devices:device-tree",
                "device-tree",
                ":device-tree"
            };

            bool found_entry = false;
            for (size_t i = 0; i < sizeof(tree_paths)/sizeof(tree_paths[0]); i++) {
                if (pLookup(NULL, tree_paths[i], &entry) == 0) {
                    found_entry = true;
                    break;
                }
            }

            if (found_entry) {
                // First check "compatible" property: list of null-terminated strings
                char comp_buf[512];
                RegPropertyValueSize sz = sizeof(comp_buf) - 2;
                if (pPropGet(&entry, "compatible", comp_buf, &sz) == 0 && sz > 0) {
                    comp_buf[sz] = '\0';
                    comp_buf[sz + 1] = '\0';
                    uint32_t idx = 0;
                    while (idx < sz) {
                        const char* s = &comp_buf[idx];
                        size_t slen = strlen(s);
                        if (slen > 0) {
                            if (classic_mac_model_from_identifier(s) != NULL) {
                                strncpy(out_buf, s, out_buf_size - 1);
                                out_buf[out_buf_size - 1] = '\0';
                                break;
                            }
                            if (out_buf[0] == '\0' && IsAppleModelIdentifier(s)) {
                                strncpy(out_buf, s, out_buf_size - 1);
                                out_buf[out_buf_size - 1] = '\0';
                            }
                        }
                        idx += (uint32_t)(slen + 1);
                    }
                }

                // If not found in "compatible" or not in table, check "model" property
                if (out_buf[0] == '\0' || classic_mac_model_from_identifier(out_buf) == NULL) {
                    char model_buf[256];
                    sz = sizeof(model_buf) - 1;
                    if (pPropGet(&entry, "model", model_buf, &sz) == 0 && sz > 0) {
                        model_buf[sz] = '\0';
                        if (classic_mac_model_from_identifier(model_buf) != NULL) {
                            strncpy(out_buf, model_buf, out_buf_size - 1);
                            out_buf[out_buf_size - 1] = '\0';
                        } else if (out_buf[0] == '\0' && IsAppleModelIdentifier(model_buf)) {
                            strncpy(out_buf, model_buf, out_buf_size - 1);
                            out_buf[out_buf_size - 1] = '\0';
                        }
                    }
                }

                if (pDispose) pDispose(&entry);
            }
        }
        CloseConnection(&conn);
    }
#endif

    // Fallback: check environment variable for testing/emulation
    if (out_buf[0] == '\0') {
        const char* env_id = getenv("RUSTID_MAC_MODEL_ID");
        if (env_id && env_id[0]) {
            strncpy(out_buf, env_id, out_buf_size - 1);
            out_buf[out_buf_size - 1] = '\0';
        }
    }

    return (out_buf[0] != '\0');
}

static MacModelSpec GetMacModelSpec(long mach_id, const char* model_id) {
    MacModelSpec spec = {0, 0, 0, 0, 0};

    if (model_id && model_id[0]) {
        if (strcmp(model_id, "PowerMac1,1") == 0) { spec.clock_mhz = 350; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac1,2") == 0) { spec.clock_mhz = 400; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac2,1") == 0) { spec.clock_mhz = 350; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac2,2") == 0) { spec.clock_mhz = 400; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac3,1") == 0) { spec.clock_mhz = 450; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac3,2") == 0) { spec.clock_mhz = 450; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac3,3") == 0) { spec.clock_mhz = 500; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac3,4") == 0) { spec.clock_mhz = 667; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerMac3,5") == 0) { spec.clock_mhz = 800; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerMac3,6") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac4,1") == 0) { spec.clock_mhz = 500; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac4,2") == 0) { spec.clock_mhz = 700; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac4,4") == 0) { spec.clock_mhz = 700; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac4,5") == 0) { spec.clock_mhz = 800; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac5,1") == 0) { spec.clock_mhz = 450; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerMac6,1") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac6,3") == 0) { spec.clock_mhz = 1250; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac6,4") == 0) { spec.clock_mhz = 1250; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac7,2") == 0) { spec.clock_mhz = 1600; spec.bus_mhz = 800; return spec; }
        if (strcmp(model_id, "PowerMac7,3") == 0) { spec.clock_mhz = 1800; spec.bus_mhz = 900; return spec; }
        if (strcmp(model_id, "PowerMac8,1") == 0) { spec.clock_mhz = 1600; spec.bus_mhz = 533; return spec; }
        if (strcmp(model_id, "PowerMac8,2") == 0) { spec.clock_mhz = 1800; spec.bus_mhz = 600; return spec; }
        if (strcmp(model_id, "PowerMac9,1") == 0) { spec.clock_mhz = 1800; spec.bus_mhz = 900; return spec; }
        if (strcmp(model_id, "PowerMac10,1") == 0) { spec.clock_mhz = 1250; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac10,2") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerMac11,2") == 0) { spec.clock_mhz = 2000; spec.bus_mhz = 1000; return spec; }
        if (strcmp(model_id, "PowerMac12,1") == 0) { spec.clock_mhz = 1900; spec.bus_mhz = 633; return spec; }
        if (strcmp(model_id, "PowerBook1,1") == 0) { spec.clock_mhz = 333; spec.bus_mhz = 66; return spec; }
        if (strcmp(model_id, "PowerBook2,1") == 0) { spec.clock_mhz = 300; spec.bus_mhz = 66; return spec; }
        if (strcmp(model_id, "PowerBook2,2") == 0) { spec.clock_mhz = 366; spec.bus_mhz = 66; return spec; }
        if (strcmp(model_id, "PowerBook3,1") == 0) { spec.clock_mhz = 400; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerBook3,2") == 0) { spec.clock_mhz = 400; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerBook3,3") == 0) { spec.clock_mhz = 550; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerBook3,4") == 0) { spec.clock_mhz = 667; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook3,5") == 0) { spec.clock_mhz = 867; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook4,1") == 0) { spec.clock_mhz = 500; spec.bus_mhz = 66; return spec; }
        if (strcmp(model_id, "PowerBook4,2") == 0) { spec.clock_mhz = 600; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerBook4,3") == 0) { spec.clock_mhz = 800; spec.bus_mhz = 100; return spec; }
        if (strcmp(model_id, "PowerBook5,1") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,2") == 0) { spec.clock_mhz = 1250; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,3") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,4") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,5") == 0) { spec.clock_mhz = 1500; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,6") == 0) { spec.clock_mhz = 1500; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,7") == 0) { spec.clock_mhz = 1670; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,8") == 0) { spec.clock_mhz = 1670; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook5,9") == 0) { spec.clock_mhz = 1670; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook6,1") == 0) { spec.clock_mhz = 867; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook6,2") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook6,3") == 0) { spec.clock_mhz = 800; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook6,4") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "PowerBook6,5") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook6,7") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "PowerBook6,8") == 0) { spec.clock_mhz = 1500; spec.bus_mhz = 167; return spec; }
        if (strcmp(model_id, "iMac,1") == 0) { spec.clock_mhz = 233; spec.bus_mhz = 66; return spec; }
        if (strcmp(model_id, "RackMac1,1") == 0) { spec.clock_mhz = 1000; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "RackMac1,2") == 0) { spec.clock_mhz = 1330; spec.bus_mhz = 133; return spec; }
        if (strcmp(model_id, "RackMac3,1") == 0) { spec.clock_mhz = 2000; spec.bus_mhz = 1000; return spec; }
    }

    switch (mach_id) {
        case 1:  // 128K
        case 2:  // 512K / 512Ke
        case 4:  // Plus
        case 5:  // SE
        case 17: // Classic
            spec.cpu_type = 1; spec.clock_mhz = 8; spec.bus_mhz = 8; break;
        case 3:  // XL (Lisa)
            spec.cpu_type = 1; spec.clock_mhz = 5; spec.bus_mhz = 5; break;
        case 6:  // Mac II
            spec.cpu_type = 3; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 1; break;
        case 7:  // Mac IIx
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 8:  // Mac IIcx
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 9:  // SE/30
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 10: // Portable
            spec.cpu_type = 1; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 11: // IIci
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 13: // IIfx
            spec.cpu_type = 4; spec.clock_mhz = 40; spec.bus_mhz = 40; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 18: // IIsi
            spec.cpu_type = 4; spec.clock_mhz = 20; spec.bus_mhz = 20; spec.mmu_type = 3; break;
        case 19: // LC
            spec.cpu_type = 3; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 20: // Quadra 900
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 21: // PB 170
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 22: // Quadra 700
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 23: // Classic II
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 24: // PB 100
            spec.cpu_type = 1; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 25: // PB 140
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 26: // Quadra 950
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 27: // LC III / Performa 450
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 29: // PB Duo 210
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 30: // Centris 650
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 32: // PB Duo 230
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 33: // PB 180
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 34: // PB 160
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 35: // Quadra 800
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 36: // Quadra 650
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 37: // LC II
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 38: // PB Duo 250
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 44: // IIvi
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 45: // IIvm / Performa 600
            spec.cpu_type = 4; spec.clock_mhz = 32; spec.bus_mhz = 32; spec.mmu_type = 3; break;
        case 48: // IIvx
            spec.cpu_type = 4; spec.clock_mhz = 32; spec.bus_mhz = 32; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 49: // Color Classic
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 50: // PB 165c
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 52: // Centris 610
            spec.cpu_type = 5; spec.clock_mhz = 20; spec.bus_mhz = 20; spec.mmu_type = 4; break;
        case 53: // Quadra 610
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 54: // PB 145
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 56: // LC 520
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 60: // Centris 660AV / Quadra 660AV
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 71: // PB 180c
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 72: // PB 520 / 540
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 77: // PB Duo 270c
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 78: // Quadra 840AV
            spec.cpu_type = 5; spec.clock_mhz = 40; spec.bus_mhz = 40; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 80: // Performa 550 / LC 550
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 84: // PB 165
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 85: // PB 190
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 4; break;
        case 88: // Macintosh TV
            spec.cpu_type = 4; spec.clock_mhz = 32; spec.bus_mhz = 32; spec.mmu_type = 3; break;
        case 89: // LC 475
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 92: // LC 575
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 94: // Quadra 605
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 98: // Quadra 630
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 4; break;
        case 99: // LC 580
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 4; break;
        case 75: // Power Mac 6100/60
            spec.clock_mhz = 60; spec.bus_mhz = 30; break;
        case 100: // Power Mac 6100/66
            spec.clock_mhz = 66; spec.bus_mhz = 33; break;
        case 112: // Power Mac 7100/66
            spec.clock_mhz = 66; spec.bus_mhz = 33; break;
        case 47:  // Power Mac 7100/80
            spec.clock_mhz = 80; spec.bus_mhz = 40; break;
        case 65:  // Power Mac 8100/80
            spec.clock_mhz = 80; spec.bus_mhz = 40; break;
        case 55:  // Power Mac 8100/100
            spec.clock_mhz = 100; spec.bus_mhz = 33; break;
        case 40:  // Power Mac 8100/110
            spec.clock_mhz = 110; spec.bus_mhz = 37; break;
        case 68:  // Power Mac 7500
            spec.clock_mhz = 100; spec.bus_mhz = 50; break;
        case 69:  // Power Mac 8500
            spec.clock_mhz = 120; spec.bus_mhz = 40; break;
        case 67:  // Power Mac 9500
            spec.clock_mhz = 132; spec.bus_mhz = 44; break;
        case 510: // Power Mac G3
            spec.clock_mhz = 266; spec.bus_mhz = 66; break;
        case 406: // NewWorld Mac (PowerBook G4, iMac, etc.)
            spec.clock_mhz = 500; spec.bus_mhz = 100; break;
        default:
            break;
    }
    return spec;
}

static const char* GetMacModelName(long mach_id) {
    switch (mach_id) {
        case 1: return "Macintosh 128K";
        case 2: return "Macintosh 512K";
        case 3: return "Macintosh XL";
        case 4: return "Macintosh Plus";
        case 5: return "Macintosh SE";
        case 6: return "Macintosh II";
        case 7: return "Macintosh IIx";
        case 8: return "Macintosh IIcx";
        case 9: return "Macintosh SE/30";
        case 10: return "Macintosh Portable";
        case 11: return "Macintosh IIci";
        case 13: return "Macintosh IIfx";
        case 17: return "Macintosh Classic";
        case 18: return "Macintosh IIsi";
        case 19: return "Macintosh LC";
        case 20: return "Macintosh Quadra 900";
        case 21: return "PowerBook 170";
        case 22: return "Macintosh Quadra 700";
        case 23: return "Macintosh Classic II";
        case 24: return "PowerBook 100";
        case 25: return "PowerBook 140";
        case 26: return "Macintosh Quadra 950";
        case 27: return "Macintosh LC III";
        case 29: return "PowerBook Duo 210";
        case 30: return "Macintosh Centris 650";
        case 32: return "PowerBook Duo 230";
        case 33: return "PowerBook 180";
        case 34: return "PowerBook 160";
        case 35: return "Macintosh Quadra 800";
        case 36: return "Macintosh Quadra 650";
        case 37: return "Macintosh LC II";
        case 38: return "PowerBook Duo 250";
        case 44: return "Macintosh IIvi";
        case 45: return "Macintosh IIvm";
        case 48: return "Macintosh IIvx";
        case 49: return "Macintosh Color Classic";
        case 50: return "PowerBook 165c";
        case 52: return "Macintosh Centris 610";
        case 53: return "Macintosh Quadra 610";
        case 54: return "PowerBook 145";
        case 56: return "Macintosh LC 520";
        case 60: return "Macintosh Quadra 660AV";
        case 71: return "PowerBook 180c";
        case 72: return "PowerBook 520/540";
        case 75: return "Power Macintosh 6100/60";
        case 77: return "PowerBook Duo 270c";
        case 78: return "Macintosh Quadra 840AV";
        case 80: return "Macintosh LC 550";
        case 84: return "PowerBook 165";
        case 85: return "PowerBook 190";
        case 88: return "Macintosh TV";
        case 89: return "Macintosh LC 475";
        case 92: return "Macintosh LC 575";
        case 94: return "Macintosh Quadra 605";
        case 98: return "Macintosh Quadra 630";
        case 99: return "Macintosh LC 580";
        case 100: return "Power Macintosh 6100/66";
        case 112: return "Power Macintosh 7100/66";
        case 47:  return "Power Macintosh 7100/80";
        case 65:  return "Power Macintosh 8100/80";
        case 55:  return "Power Macintosh 8100/100";
        case 40:  return "Power Macintosh 8100/110";
        case 68:  return "Power Macintosh 7500";
        case 69:  return "Power Macintosh 8500";
        case 67:  return "Power Macintosh 9500";
        case 510: return "Power Macintosh G3";
        case 406: return "Power Macintosh G4";
        default: return "Macintosh (Generic)";
    }
}

#if defined(__m68k__) || defined(__mc68000__)
static bool SafeGestalt(OSType selector, long* response) {
    if (response) *response = 0;
    register unsigned long reg_d0 __asm__("d0") = selector;
    register long reg_a0 __asm__("a0") = 0;
    register short err __asm__("d0");
    __asm__ volatile(
        "dc.w 0xa1ad"
        : "=d"(err), "=a"(reg_a0)
        : "0"(reg_d0)
        : "d1", "d2", "a1", "memory"
    );
    if (err == 0) {
        if (response) *response = reg_a0;
        return true;
    }
    return false;
}
#elif defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
static bool SafeGestalt(OSType selector, long* response) {
    if (response) *response = 0;
    long val = 0;
    OSErr err = Gestalt(selector, &val);
    if (err == 0) {
        if (response) *response = val;
        return true;
    }
    return false;
}
#endif



void classic_mac_detect_cpu(MacCpuInfo* info) {
    memset(info, 0, sizeof(MacCpuInfo));

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long mach_val = 0, sysv_val = 0, ram_val = 0;
    SafeGestalt(gestaltMachineType, &mach_val);
    SafeGestalt(gestaltSystemVersion, &sysv_val);
    SafeGestalt(gestaltPhysicalRAMSize, &ram_val);

    // 1. Probe Open Firmware / Name Registry for model identifier string (e.g. "PowerMac1,1")
    bool has_model_id = classic_mac_probe_model_identifier(info->model_id, sizeof(info->model_id));

    // 2. Query hardware specification (clocks, bus) with model_id fallback
    MacModelSpec spec = GetMacModelSpec(mach_val, has_model_id ? info->model_id : NULL);

    if (ram_val > 0) {
        info->ram_mb = (uint32_t)(ram_val / (1024 * 1024));
    }

    // 3. Resolve system model name: prefer model identifier string if it exists
    const char* id_model_name = has_model_id ? classic_mac_model_from_identifier(info->model_id) : NULL;
    if (id_model_name != NULL) {
        snprintf(info->system_name, sizeof(info->system_name), "%s", id_model_name);
    } else {
        // Fall back to Gestalt machine type
        snprintf(info->system_name, sizeof(info->system_name), "%s", GetMacModelName(mach_val));

        // Query gestaltUserVisibleMachineName ('mnam') for accurate human-readable model name
        long mnam_ptr = 0;
        if (SafeGestalt('mnam', &mnam_ptr) && mnam_ptr > 1024) {
            const unsigned char* pstr = (const unsigned char*)mnam_ptr;
            uint8_t len = pstr[0];
            if (len > 0 && len < sizeof(info->system_name)) {
                bool valid = true;
                for (uint8_t i = 1; i <= len; i++) {
                    if (pstr[i] < 32 || pstr[i] > 254) {
                        valid = false;
                        break;
                    }
                }
                if (valid) {
                    char mnam_buf[64];
                    memcpy(mnam_buf, &pstr[1], len);
                    mnam_buf[len] = '\0';
                    const char* from_mnam = classic_mac_model_from_identifier(mnam_buf);
                    if (from_mnam) {
                        snprintf(info->system_name, sizeof(info->system_name), "%s", from_mnam);
                        if (!has_model_id) {
                            snprintf(info->model_id, sizeof(info->model_id), "%.*s", (int)sizeof(info->model_id) - 1, mnam_buf);
                            has_model_id = true;
                        }
                    } else {
                        snprintf(info->system_name, sizeof(info->system_name), "%s", mnam_buf);
                    }
                }
            }
        } else if (has_model_id && (info->system_name[0] == '\0' || strcmp(info->system_name, "Macintosh (Generic)") == 0)) {
            // Identifier string exists but is not in our dictionary and Gestalt was generic
            snprintf(info->system_name, sizeof(info->system_name), "%s", info->model_id);
        }
    }

    short major = (short)((sysv_val >> 8) & 0xFF);
    short minor = (short)((sysv_val >> 4) & 0x0F);
    short patch = (short)(sysv_val & 0x0F);
    const char* pfx = (major < 8) ? "System" : "Mac OS";
    if (major > 0) {
        if (patch == 0) {
            snprintf(info->os_version, sizeof(info->os_version), "%s %d.%d", pfx, major, minor);
        } else {
            snprintf(info->os_version, sizeof(info->os_version), "%s %d.%d.%d", pfx, major, minor, patch);
        }
    } else {
        strcpy(info->os_version, "System Software");
    }

    // Determine system architecture: 68k vs PowerPC
    bool is_ppc = false;
#if defined(__powerpc__) || defined(__ppc__)
    is_ppc = true;
#endif

    long arch_val = 0;
    if (SafeGestalt(gestaltSysArchitecture, &arch_val)) {
        if (arch_val == gestaltPowerPC) {
            is_ppc = true;
        } else if (arch_val == gestalt68k) {
            is_ppc = false;
        }
    } else if (!is_ppc) {
        long cput_val = 0;
        if (SafeGestalt(gestaltNativeCPUtype, &cput_val) && cput_val >= 256) {
            is_ppc = true;
        }
    }

    info->is_powerpc = is_ppc;

    // Clock and Bus speed with model spec fallbacks (rounded to nearest MHz)
    long clk_val = 0, bclk_val = 0;
    if (SafeGestalt(gestaltProcClkSpeed, &clk_val) && clk_val > 0) {
        info->clock_mhz = (uint32_t)((clk_val + 500000) / 1000000);
    } else {
        info->clock_mhz = spec.clock_mhz;
    }

    if (SafeGestalt(gestaltBusClkSpeed, &bclk_val) && bclk_val > 0) {
        info->bus_mhz = (uint32_t)((bclk_val + 500000) / 1000000);
    } else {
        info->bus_mhz = spec.bus_mhz;
    }

    if (info->is_powerpc) {
        long ppc_val = 0;
        SafeGestalt(gestaltNativeCPUtype, &ppc_val);
        strcpy(info->fpu, "Integrated FPU");
        strcpy(info->mmu, "Integrated MMU");
        switch (ppc_val) {
            case 257:
            case 1:
                strcpy(info->model, "PowerPC 601");
                strcpy(info->microarch, "PowerPC 601");
                strcpy(info->codename, "601");
                strcpy(info->process, "0.6\xb5m");
                break;
            case 259:
            case 2:
            case 3:
                strcpy(info->model, "PowerPC 603");
                strcpy(info->microarch, "PowerPC 603");
                strcpy(info->codename, "603");
                strcpy(info->process, "0.5\xb5m");
                break;
            case 260:
            case 4:
                strcpy(info->model, "PowerPC 604");
                strcpy(info->microarch, "PowerPC 604");
                strcpy(info->codename, "604");
                strcpy(info->process, "350nm");
                break;
            case 262:
            case 6:
                strcpy(info->model, "PowerPC 603e");
                strcpy(info->microarch, "PowerPC 603e");
                strcpy(info->codename, "603e");
                strcpy(info->process, "350nm");
                break;
            case 263:
            case 7:
                strcpy(info->model, "PowerPC 603ev");
                strcpy(info->microarch, "PowerPC 603ev");
                strcpy(info->codename, "603ev");
                strcpy(info->process, "290nm");
                break;
            case 264:
            case 267:
            case 8:
                strcpy(info->model, "PowerPC 750 (G3)");
                strcpy(info->microarch, "PowerPC 750 (G3)");
                strcpy(info->codename, "Arthur");
                strcpy(info->process, "260nm");
                break;
            case 265:
            case 9:
                strcpy(info->model, "PowerPC 604e");
                strcpy(info->microarch, "PowerPC 604e");
                strcpy(info->codename, "604e");
                strcpy(info->process, "250nm");
                break;
            case 268:
            case 12:
                if (strstr(info->system_name, "PowerBook G4") != NULL ||
                    (info->model_id[0] && strncmp(info->model_id, "PowerBook", 9) == 0) ||
                    mach_val == 414) {
                    if (info->clock_mhz > 500) {
                        strcpy(info->model, "PowerPC 7455 (G4)");
                        strcpy(info->microarch, "PowerPC 7455 (G4)");
                        strcpy(info->codename, "Apollo 6");
                        strcpy(info->process, "150nm");
                    } else {
                        strcpy(info->model, "PowerPC 7410 (G4)");
                        strcpy(info->microarch, "PowerPC 7410 (G4)");
                        strcpy(info->codename, "Nitro");
                        strcpy(info->process, "180nm");
                    }
                } else if (info->model_id[0] && (strcmp(info->model_id, "PowerMac3,5") == 0 ||
                                                strcmp(info->model_id, "PowerMac3,6") == 0 ||
                                                info->clock_mhz > 700)) {
                    strcpy(info->model, "PowerPC 7455 (G4)");
                    strcpy(info->microarch, "PowerPC 7455 (G4)");
                    strcpy(info->codename, "Apollo 6");
                    strcpy(info->process, "150nm");
                } else if (info->model_id[0] && strcmp(info->model_id, "PowerMac3,4") == 0) {
                    strcpy(info->model, "PowerPC 7450 (G4)");
                    strcpy(info->microarch, "PowerPC 7450 (G4)");
                    strcpy(info->codename, "Vger");
                    strcpy(info->process, "180nm");
                } else {
                    strcpy(info->model, "PowerPC 7400 (G4)");
                    strcpy(info->microarch, "PowerPC 7400 (G4)");
                    strcpy(info->codename, "Max");
                    strcpy(info->process, "200nm");
                }
                break;
            case 269:
            case 16:
                strcpy(info->model, "PowerPC 7450 (G4)");
                strcpy(info->microarch, "PowerPC 7450 (G4)");
                strcpy(info->codename, "Vger");
                strcpy(info->process, "180nm");
                break;
            default:
                strcpy(info->model, "PowerPC (Generic)");
                strcpy(info->microarch, "PowerPC");
                strcpy(info->codename, "PPC");
                break;
        }
    } else {
        long fpu_val = 0, mmu_val = 0, cpu_val = 0;
        if (!SafeGestalt(gestaltFPUType, &fpu_val) || fpu_val == 0) {
            fpu_val = spec.fpu_type;
        }
        if (!SafeGestalt(gestaltMMUType, &mmu_val) || mmu_val == 0) {
            mmu_val = spec.mmu_type;
        }
        if (!SafeGestalt(gestaltProcessorType, &cpu_val) || cpu_val == 0) {
            cpu_val = spec.cpu_type;
        }

        switch (fpu_val) {
            case 1: strcpy(info->fpu, "Motorola 68881"); break;
            case 2: strcpy(info->fpu, "Motorola 68882"); break;
            case 3: strcpy(info->fpu, "Integrated 68040 FPU"); break;
            default: strcpy(info->fpu, "None"); break;
        }

        switch (mmu_val) {
            case 1: strcpy(info->mmu, "Apple AMU"); break;
            case 2: strcpy(info->mmu, "Motorola 68851 PMMU"); break;
            case 3: strcpy(info->mmu, "Integrated 68030 MMU"); break;
            case 4: strcpy(info->mmu, "Integrated 68040 MMU"); break;
            default: strcpy(info->mmu, "None"); break;
        }

        switch (cpu_val) {
            case 1:
                strcpy(info->model, "MC68000");
                strcpy(info->microarch, "Motorola 68000");
                strcpy(info->codename, "68000");
                strcpy(info->process, "3.5\xb5m");
                break;
            case 2:
                strcpy(info->model, "MC68010");
                strcpy(info->microarch, "Motorola 68010");
                strcpy(info->codename, "68010");
                strcpy(info->process, "3.0\xb5m");
                break;
            case 3:
                strcpy(info->model, "MC68020");
                strcpy(info->microarch, "Motorola 68020");
                strcpy(info->codename, "68020");
                strcpy(info->process, "1.5\xb5m");
                break;
            case 4:
                strcpy(info->model, "MC68030");
                strcpy(info->microarch, "Motorola 68030");
                strcpy(info->codename, "68030");
                strcpy(info->process, "0.8\xb5m");
                break;
            case 5:
                if (fpu_val == 0) {
                    strcpy(info->model, "MC68LC040");
                    strcpy(info->microarch, "Motorola 68LC040");
                    strcpy(info->codename, "68LC040");
                } else {
                    strcpy(info->model, "MC68040");
                    strcpy(info->microarch, "Motorola 68040");
                    strcpy(info->codename, "68040");
                }
                strcpy(info->process, "0.65\xb5m");
                break;
            case 6:
                strcpy(info->model, "MC68060");
                strcpy(info->microarch, "Motorola 68060");
                strcpy(info->codename, "68060");
                strcpy(info->process, "350nm");
                break;
            default:
                strcpy(info->model, "Motorola 68k (Generic)");
                strcpy(info->microarch, "68k");
                break;
        }
    }
#else
    // Fallback for host builds / tests
    strcpy(info->model, "MC68030");
    strcpy(info->microarch, "Motorola 68030");
    strcpy(info->codename, "68030");
    strcpy(info->process, "0.8\xb5m");
    strcpy(info->fpu, "Motorola 68882");
    strcpy(info->mmu, "Integrated 68030 MMU");
    info->clock_mhz = 25;
    info->bus_mhz = 25;
    info->ram_mb = 8;
    strcpy(info->system_name, "Macintosh SE/30");
    strcpy(info->os_version, "System 7.5.5");
    info->is_powerpc = false;

    // Allow testing model_id resolution via RUSTID_MAC_MODEL_ID environment variable
    if (classic_mac_probe_model_identifier(info->model_id, sizeof(info->model_id))) {
        const char* id_name = classic_mac_model_from_identifier(info->model_id);
        if (id_name) {
            strncpy(info->system_name, id_name, sizeof(info->system_name) - 1);
        } else {
            strncpy(info->system_name, info->model_id, sizeof(info->system_name) - 1);
        }
        MacModelSpec spec = GetMacModelSpec(0, info->model_id);
        if (spec.clock_mhz > 0) info->clock_mhz = spec.clock_mhz;
        if (spec.bus_mhz > 0) info->bus_mhz = spec.bus_mhz;
    } else {
        (void)GetMacModelName(0);
        (void)GetMacModelSpec(0, NULL);
    }
#endif
}

void classic_mac_generate_report(
    const MacCpuInfo* info,
    uint32_t view_mode,
    bool color,
    char* out_buf,
    uint32_t out_buf_size,
    CTextRun* out_runs,
    uint32_t* out_run_count,
    uint32_t max_runs
) {
    CRgbColor c_label = PALETTE_LABEL;
    CRgbColor c_sublabel = PALETTE_SUBLABEL;
    CRgbColor c_body = PALETTE_BODY;
    CRgbColor c_highlight = PALETTE_HIGHLIGHT;

    if (out_run_count) *out_run_count = 0;
    out_buf[0] = '\0';

    uint32_t offset = 0;

    #define APPEND_HEADER(str, clr) do { \
        size_t len = strlen(str); \
        if (offset + len < out_buf_size) { \
            strcat(out_buf, str); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)len; \
                out_runs[*out_run_count].color = clr; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)len; \
        } \
    } while(0)

    #define APPEND_FIELD(label, val_str, val_clr) do { \
        char lbl_buf[32]; \
        snprintf(lbl_buf, sizeof(lbl_buf), "%14s: ", label); \
        size_t lbl_len = strlen(lbl_buf); \
        char val_buf[256]; \
        snprintf(val_buf, sizeof(val_buf), "%s\r\r", val_str); \
        size_t val_len = strlen(val_buf); \
        if (offset + lbl_len + val_len < out_buf_size) { \
            strcat(out_buf, lbl_buf); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)lbl_len; \
                out_runs[*out_run_count].color = c_label; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)lbl_len; \
            strcat(out_buf, val_buf); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)val_len; \
                out_runs[*out_run_count].color = val_clr; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)val_len; \
        } \
    } while(0)

    // Header banner
    const char* arch_str = info->is_powerpc ? "powerpc" : "m68k";
    char banner[256];
    snprintf(banner, sizeof(banner), "--------------- Rustid %s (%s-macos_classic) ---------------\r\r", RUSTID_VERSION, arch_str);
    APPEND_HEADER(banner, c_sublabel);

    if (view_mode == VIEW_STANDARD || view_mode == VIEW_EVERYTHING) {
        if (info->system_name[0]) {
            APPEND_FIELD("System", info->system_name, c_body);
        }
        if (info->os_version[0]) {
            APPEND_FIELD("OS", info->os_version, c_body);
        }
        if (info->model[0]) {
            APPEND_FIELD("Model", info->model, c_highlight);
        }
        if (info->microarch[0] && !info->is_powerpc) {
            APPEND_FIELD("MicroArch", info->microarch, c_body);
        }
        if (info->codename[0] && info->is_powerpc) {
            APPEND_FIELD("Codename", info->codename, c_body);
        }
        if (info->process[0]) {
            APPEND_FIELD("Process", info->process, c_body);
        }
        if (info->fpu[0] && strcmp(info->fpu, "None") != 0) {
            APPEND_FIELD("FPU", info->fpu, c_body);
        }
        if (info->mmu[0] && strcmp(info->mmu, "None") != 0) {
            APPEND_FIELD("MMU", info->mmu, c_body);
        }
        if (info->clock_mhz > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MHz", (unsigned int)info->clock_mhz);
            APPEND_FIELD("Frequency", num_buf, c_body);
        }
        if (info->bus_mhz > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MHz", (unsigned int)info->bus_mhz);
            APPEND_FIELD("Bus Speed", num_buf, c_body);
        }
        if (info->ram_mb > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MB RAM", (unsigned int)info->ram_mb);
            APPEND_FIELD("Memory", num_buf, c_body);
        }
    }

    if (view_mode == VIEW_DEBUG || view_mode == VIEW_EVERYTHING) {
        if (view_mode == VIEW_EVERYTHING) {
            APPEND_HEADER("--------------------\r\r", c_sublabel);
        }
        APPEND_HEADER("Debug Information:\r\r", c_sublabel);
        APPEND_FIELD("Arch", arch_str, c_body);
        APPEND_FIELD("Target", "Classic Macintosh Toolbox", c_body);
        APPEND_FIELD("Gestalt", "Active", c_body);
        if (info->model_id[0]) {
            APPEND_FIELD("Model ID", info->model_id, c_body);
        }
    }

    #undef APPEND_HEADER
    #undef APPEND_FIELD
}

#ifndef NO_STANDALONE_MAIN
static MacCpuInfo s_cpu_info;
static uint32_t s_view_mode = VIEW_STANDARD;
static bool s_color = true;
static char s_report_buf[8192];
static CTextRun s_runs[256];
static uint32_t s_run_count = 0;

static void render_view(void) {
    s_run_count = 0;
    classic_mac_generate_report(
        &s_cpu_info,
        s_view_mode,
        s_color,
        s_report_buf,
        sizeof(s_report_buf),
        s_runs,
        &s_run_count,
        256
    );

    mac_gui_set_text(s_report_buf, strlen(s_report_buf), s_runs, s_run_count, PALETTE_BG);
    mac_gui_set_status(s_cpu_info.system_name, s_cpu_info.model, s_cpu_info.os_version);
    mac_gui_set_menu_checks(s_view_mode, s_color);
}

static void on_command(uint32_t cmd_id) {
    switch (cmd_id) {
        case CMD_FILE_REFRESH: // 101
            classic_mac_detect_cpu(&s_cpu_info);
            render_view();
            break;
        case CMD_FILE_EXIT: // 102
            break;
        case CMD_FILE_COPY: // 103
            mac_gui_copy_clipboard(s_report_buf);
            break;
        case CMD_MODE_STANDARD: // 201
        case CMD_MODE_DEBUG: // 202
        case CMD_MODE_EVERYTHING: // 203
            s_view_mode = cmd_id;
            render_view();
            break;
        case CMD_OPT_COLOR: // 301
            s_color = !s_color;
            render_view();
            break;
        case CMD_HELP_ABOUT: // 401
            mac_gui_show_alert("About rustid", "rustid " RUSTID_VERSION " for Classic Macintosh\rCPU Identification Tool");
            break;
        default:
            break;
    }
}

static void on_file(const char* path, bool is_save) {
    if (is_save && path && path[0]) {
        FILE* f = fopen(path, "w");
        if (f) {
            fputs(s_report_buf, f);
            fclose(f);
        }
    }
}

static void on_quit(void) {
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    if (!mac_gui_init("rustid", 480, 320)) {
        return 1;
    }
    mac_gui_set_callbacks(on_command, on_file, on_quit);
    classic_mac_detect_cpu(&s_cpu_info);
    render_view();
    mac_gui_run();
    return 0;
}
#endif

