#include "models/models.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Types.h>
#include <Multiverse.h>
#if __has_include(<CodeFragments.h>)
#include <CodeFragments.h>
#endif
#endif

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

MacModelSpec classic_mac_get_model_spec(long mach_id, const char* model_id) {
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

const char* classic_mac_get_model_name(long mach_id) {
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
