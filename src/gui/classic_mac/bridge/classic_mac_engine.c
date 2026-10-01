//! Standalone Classic Mac OS C CPU detection and report generator engine.
//! Used for 68k builds and Universal fat binaries where native Rust cannot compile.

#include "classic_mac_engine.h"
#include <stdio.h>
#include <string.h>

#ifndef RUSTID_VERSION
#define RUSTID_VERSION "2.3.0"
#endif

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Gestalt.h>
#include <Types.h>

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
    VIEW_EVERYTHING = 203,
    VIEW_DUMP       = 204
};

static const CRgbColor PALETTE_LIGHT_BG __attribute__((unused)) = {255, 255, 255};
static const CRgbColor PALETTE_LIGHT_LABEL     = {9, 134, 88};
static const CRgbColor PALETTE_LIGHT_SUBLABEL  = {4, 81, 165};
static const CRgbColor PALETTE_LIGHT_BODY      = {30, 30, 30};
static const CRgbColor PALETTE_LIGHT_HIGHLIGHT = {163, 21, 21};

static const CRgbColor PALETTE_DARK_BG __attribute__((unused)) = {26, 27, 38};
static const CRgbColor PALETTE_DARK_LABEL     = {115, 218, 202};
static const CRgbColor PALETTE_DARK_SUBLABEL  = {125, 207, 255};
static const CRgbColor PALETTE_DARK_BODY      = {212, 212, 212};
static const CRgbColor PALETTE_DARK_HIGHLIGHT = {255, 158, 100};

typedef struct {
    uint32_t cpu_type;  // 1=68000, 2=68010, 3=68020, 4=68030, 5=68040, 6=68060
    uint32_t clock_mhz;
    uint32_t bus_mhz;
    uint32_t fpu_type;  // 0=None, 1=68881, 2=68882, 3=68040
    uint32_t mmu_type;  // 0=None, 1=AMU, 2=68851, 3=68030, 4=68040
} MacModelSpec;

static MacModelSpec GetMacModelSpec(long mach_id) {
    MacModelSpec spec = {0, 0, 0, 0, 0};
    switch (mach_id) {
        case 1:  // 128K
        case 2:  // 512K
        case 4:  // Plus
        case 5:  // SE
        case 14: // Classic
            spec.cpu_type = 1; spec.clock_mhz = 8; spec.bus_mhz = 8; break;
        case 3:  // XL (Lisa)
            spec.cpu_type = 1; spec.clock_mhz = 5; spec.bus_mhz = 5; break;
        case 6:  // Mac II
            spec.cpu_type = 3; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 1; break;
        case 7:  // Mac IIx
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 8:  // Mac IIcx
            spec.cpu_type = 4; spec.clock_mhz = 24; spec.bus_mhz = 24; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 9:  // SE/30
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 10: // Portable
            spec.cpu_type = 1; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 11: // IIci
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 12: // IIfx
        case 13:
            spec.cpu_type = 4; spec.clock_mhz = 40; spec.bus_mhz = 40; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 15: // IIsi
        case 18:
            spec.cpu_type = 4; spec.clock_mhz = 20; spec.bus_mhz = 20; spec.mmu_type = 3; break;
        case 17: // LC
            spec.cpu_type = 3; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 19: // PB 170
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 20: // Quadra 700
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 21: // Classic II
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 22: // PB 100
            spec.cpu_type = 1; spec.clock_mhz = 16; spec.bus_mhz = 16; break;
        case 23: // PB 140
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 24: // Quadra 950
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 25: // LC III
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 26: // PB 160
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 27: // PB 180
        case 34: // PB 180c
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 28: // PB Duo 210
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 29: // PB Duo 230
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 30: // PB Duo 250
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 32: // PB 165c
        case 38: // PB 165
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 33: // Centris 650
        case 37: // Quadra 650
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 35: // PB Duo 270c
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 2; spec.mmu_type = 3; break;
        case 36: // Quadra 800
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 39: // Color Classic
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 40: // Centris 610
            spec.cpu_type = 5; spec.clock_mhz = 20; spec.bus_mhz = 20; spec.mmu_type = 4; break;
        case 41: // Quadra 610
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 42: // PB 145
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 43: // LC II
            spec.cpu_type = 4; spec.clock_mhz = 16; spec.bus_mhz = 16; spec.mmu_type = 3; break;
        case 44: // PB 520 / 540
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 45: // Quadra 605 / LC 475
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 4; break;
        case 48: // Macintosh TV
            spec.cpu_type = 4; spec.clock_mhz = 32; spec.bus_mhz = 32; spec.mmu_type = 3; break;
        case 49: // LC 520
            spec.cpu_type = 4; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.mmu_type = 3; break;
        case 50: // LC 550
            spec.cpu_type = 4; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 3; break;
        case 52: // Quadra 660AV
            spec.cpu_type = 5; spec.clock_mhz = 25; spec.bus_mhz = 25; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 53: // Quadra 840AV
            spec.cpu_type = 5; spec.clock_mhz = 40; spec.bus_mhz = 40; spec.fpu_type = 3; spec.mmu_type = 4; break;
        case 60: // LC 575
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 4; break;
        case 61: // Quadra 630 / LC 580
            spec.cpu_type = 5; spec.clock_mhz = 33; spec.bus_mhz = 33; spec.mmu_type = 4; break;
        case 70: // Power Mac 6100
            spec.clock_mhz = 60; spec.bus_mhz = 30; break;
        case 71: // Power Mac 7100
            spec.clock_mhz = 66; spec.bus_mhz = 33; break;
        case 72: // Power Mac 8100
            spec.clock_mhz = 80; spec.bus_mhz = 40; break;
        case 84: // Power Mac 7500
            spec.clock_mhz = 100; spec.bus_mhz = 50; break;
        case 85: // Power Mac 8500
            spec.clock_mhz = 120; spec.bus_mhz = 40; break;
        case 86: // Power Mac 9500
            spec.clock_mhz = 132; spec.bus_mhz = 44; break;
        case 120: // G3 Beige
            spec.clock_mhz = 266; spec.bus_mhz = 66; break;
        case 121: // G3 B&W
            spec.clock_mhz = 350; spec.bus_mhz = 100; break;
        case 406: // G4 Sawtooth
            spec.clock_mhz = 450; spec.bus_mhz = 100; break;
        case 414: // PB G4 Ti
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
        case 12: return "Macintosh IIfx";
        case 14: return "Macintosh Classic";
        case 15: return "Macintosh IIsi";
        case 17: return "Macintosh LC";
        case 18: return "Macintosh Quadra 900";
        case 19: return "PowerBook 170";
        case 20: return "Macintosh Quadra 700";
        case 21: return "Macintosh Classic II";
        case 22: return "PowerBook 100";
        case 23: return "PowerBook 140";
        case 24: return "Macintosh Quadra 950";
        case 25: return "Macintosh LC III";
        case 26: return "PowerBook 160";
        case 27: return "PowerBook 180";
        case 33: return "Macintosh Centris 650";
        case 36: return "Macintosh Quadra 800";
        case 37: return "Macintosh Quadra 650";
        case 39: return "Macintosh Color Classic";
        case 40: return "Macintosh Centris 610";
        case 41: return "Macintosh Quadra 610";
        case 45: return "Macintosh Quadra 605 / LC 475";
        case 52: return "Macintosh Quadra 660AV";
        case 53: return "Macintosh Quadra 840AV";
        case 60: return "Macintosh LC 575";
        case 61: return "Macintosh Quadra 630 / LC 580";
        case 70: return "Power Macintosh 6100";
        case 71: return "Power Macintosh 7100";
        case 72: return "Power Macintosh 8100";
        case 84: return "Power Macintosh 7500";
        case 85: return "Power Macintosh 8500";
        case 86: return "Power Macintosh 9500";
        case 120: return "Power Macintosh G3 (Beige)";
        case 121: return "Power Macintosh G3 (Blue & White)";
        case 406: return "Power Macintosh G4 (Sawtooth)";
        case 414: return "PowerBook G4 (Titanium)";
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

    MacModelSpec spec = GetMacModelSpec(mach_val);

    if (ram_val > 0) {
        info->ram_mb = (uint32_t)(ram_val / (1024 * 1024));
    }

    strncpy(info->system_name, GetMacModelName(mach_val), sizeof(info->system_name) - 1);

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

    // Clock and Bus speed with model spec fallbacks
    long clk_val = 0, bclk_val = 0;
    if (SafeGestalt(gestaltProcClkSpeed, &clk_val) && clk_val > 0) {
        info->clock_mhz = (uint32_t)(clk_val / 1000000);
    } else {
        info->clock_mhz = spec.clock_mhz;
    }

    if (SafeGestalt(gestaltBusClkSpeed, &bclk_val) && bclk_val > 0) {
        info->bus_mhz = (uint32_t)(bclk_val / 1000000);
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
                strcpy(info->model, "PowerPC 7400 (G4)");
                strcpy(info->microarch, "PowerPC 7400 (G4)");
                strcpy(info->codename, "Max");
                strcpy(info->process, "200nm");
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
#endif
}

void classic_mac_generate_report(
    const MacCpuInfo* info,
    uint32_t view_mode,
    bool color,
    bool dark_theme,
    bool verbose,
    bool compact,
    char* out_buf,
    uint32_t out_buf_size,
    CTextRun* out_runs,
    uint32_t* out_run_count,
    uint32_t max_runs
) {
    (void)verbose; (void)compact;
    CRgbColor c_label = dark_theme ? PALETTE_DARK_LABEL : PALETTE_LIGHT_LABEL;
    CRgbColor c_sublabel = dark_theme ? PALETTE_DARK_SUBLABEL : PALETTE_LIGHT_SUBLABEL;
    CRgbColor c_body = dark_theme ? PALETTE_DARK_BODY : PALETTE_LIGHT_BODY;
    CRgbColor c_highlight = dark_theme ? PALETTE_DARK_HIGHLIGHT : PALETTE_LIGHT_HIGHLIGHT;

    *out_run_count = 0;
    out_buf[0] = '\0';

    uint32_t offset = 0;

    #define APPEND_HEADER(str, clr) do { \
        size_t len = strlen(str); \
        if (offset + len < out_buf_size) { \
            strcat(out_buf, str); \
            if (color && *out_run_count < max_runs) { \
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
        snprintf(val_buf, sizeof(val_buf), "%s\r", val_str); \
        size_t val_len = strlen(val_buf); \
        if (offset + lbl_len + val_len < out_buf_size) { \
            strcat(out_buf, lbl_buf); \
            if (color && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)lbl_len; \
                out_runs[*out_run_count].color = c_label; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)lbl_len; \
            strcat(out_buf, val_buf); \
            if (color && *out_run_count < max_runs) { \
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
        if (info->microarch[0]) {
            APPEND_FIELD("MicroArch", info->microarch, c_body);
        }
        if (info->codename[0]) {
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
            APPEND_HEADER("\r--------------------\r\r", c_sublabel);
        }
        APPEND_HEADER("Debug Information:\r", c_sublabel);
        APPEND_FIELD("Arch", arch_str, c_body);
        APPEND_FIELD("Target", "Classic Macintosh Toolbox", c_body);
        APPEND_FIELD("Gestalt", "Active", c_body);
    }

    #undef APPEND_HEADER
    #undef APPEND_FIELD
}

#ifndef NO_STANDALONE_MAIN
static MacCpuInfo s_cpu_info;
static uint32_t s_view_mode = VIEW_STANDARD;
static bool s_color = true;
static bool s_dark_theme = false;
static bool s_verbose = false;
static bool s_compact = false;
static char s_report_buf[8192];
static CTextRun s_runs[256];
static uint32_t s_run_count = 0;

static void render_view(void) {
    s_run_count = 0;
    classic_mac_generate_report(
        &s_cpu_info,
        s_view_mode,
        s_color,
        s_dark_theme,
        s_verbose,
        s_compact,
        s_report_buf,
        sizeof(s_report_buf),
        s_runs,
        &s_run_count,
        256
    );

    CRgbColor bg_color = s_dark_theme ? PALETTE_DARK_BG : PALETTE_LIGHT_BG;
    mac_gui_set_text(s_report_buf, strlen(s_report_buf), s_runs, s_run_count, bg_color);
    mac_gui_set_status(s_cpu_info.system_name, s_cpu_info.model, s_cpu_info.os_version);
    mac_gui_set_menu_checks(s_view_mode, s_color, s_dark_theme, s_verbose, s_compact);
}

static void on_command(uint32_t cmd_id) {
    switch (cmd_id) {
        case 101: // CMD_FILE_OPEN
            mac_gui_open_file_dialog();
            break;
        case 102: // CMD_FILE_EXPORT
            mac_gui_save_file_dialog("CPU_Report.txt");
            break;
        case 103: // CMD_FILE_COPY
            mac_gui_copy_clipboard(s_report_buf);
            break;
        case 104: // CMD_FILE_REFRESH
            classic_mac_detect_cpu(&s_cpu_info);
            render_view();
            break;
        case 105: // CMD_FILE_EXIT
            break;
        case 201: // CMD_MODE_STANDARD
        case 202: // CMD_MODE_DEBUG
        case 203: // CMD_MODE_EVERYTHING
        case 204: // CMD_MODE_DUMP
            s_view_mode = cmd_id;
            render_view();
            break;
        case 301: // CMD_OPT_COLOR
            s_color = !s_color;
            render_view();
            break;
        case 302: // CMD_OPT_DARK_THEME
            s_dark_theme = !s_dark_theme;
            render_view();
            break;
        case 303: // CMD_OPT_VERBOSE
            s_verbose = !s_verbose;
            render_view();
            break;
        case 304: // CMD_OPT_COMPACT
            s_compact = !s_compact;
            render_view();
            break;
        case 401: // CMD_HELP_ABOUT
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

