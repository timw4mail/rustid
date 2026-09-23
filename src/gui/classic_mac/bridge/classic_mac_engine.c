//! Standalone Classic Mac OS C CPU detection and report generator engine.
//! Used for 68k builds and Universal fat binaries where native Rust cannot compile.

#include "classic_mac_engine.h"
#include <stdio.h>
#include <string.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Gestalt.h>
#include <Types.h>
#endif

enum {
    VIEW_STANDARD   = 201,
    VIEW_DEBUG      = 202,
    VIEW_EVERYTHING = 203,
    VIEW_DUMP       = 204
};

static const CRgbColor PALETTE_LIGHT_BG        = {255, 255, 255};
static const CRgbColor PALETTE_LIGHT_LABEL     = {9, 134, 88};
static const CRgbColor PALETTE_LIGHT_SUBLABEL  = {4, 81, 165};
static const CRgbColor PALETTE_LIGHT_BODY      = {30, 30, 30};
static const CRgbColor PALETTE_LIGHT_HIGHLIGHT = {163, 21, 21};

static const CRgbColor PALETTE_DARK_BG        = {26, 27, 38};
static const CRgbColor PALETTE_DARK_LABEL     = {115, 218, 202};
static const CRgbColor PALETTE_DARK_SUBLABEL  = {125, 207, 255};
static const CRgbColor PALETTE_DARK_BODY      = {212, 212, 212};
static const CRgbColor PALETTE_DARK_HIGHLIGHT = {255, 158, 100};

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

void classic_mac_detect_cpu(MacCpuInfo* info) {
    memset(info, 0, sizeof(MacCpuInfo));

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long cpu_val = 0, ppc_val = 0, fpu_val = 0, mmu_val = 0;
    long clk_val = 0, bclk_val = 0, mach_val = 0, sysv_val = 0, ram_val = 0;

    Gestalt(gestaltCPUtype, &cpu_val);
    Gestalt(gestaltPowerPCProcessorType, &ppc_val);
    Gestalt(gestaltFPUType, &fpu_val);
    Gestalt(gestaltMMUType, &mmu_val);
    Gestalt(gestaltProcClkSpeed, &clk_val);
    Gestalt(gestaltBusClkSpeed, &bclk_val);
    Gestalt(gestaltMachineType, &mach_val);
    Gestalt(gestaltSystemVersion, &sysv_val);
    Gestalt(gestaltPhysicalRAMSize, &ram_val);

    info->clock_mhz = (uint32_t)(clk_val / 1000000);
    info->bus_mhz = (uint32_t)(bclk_val / 1000000);
    info->ram_mb = (uint32_t)(ram_val / (1024 * 1024));

    strncpy(info->system_name, GetMacModelName(mach_val), sizeof(info->system_name) - 1);

    short major = (short)((sysv_val >> 8) & 0xFF);
    short minor = (short)((sysv_val >> 4) & 0x0F);
    short patch = (short)(sysv_val & 0x0F);
    const char* pfx = (major < 8) ? "System" : "Mac OS";
    if (patch == 0) {
        snprintf(info->os_version, sizeof(info->os_version), "%s %d.%d", pfx, major, minor);
    } else {
        snprintf(info->os_version, sizeof(info->os_version), "%s %d.%d.%d", pfx, major, minor, patch);
    }

    if (ppc_val > 0) {
        info->is_powerpc = true;
        strcpy(info->fpu, "Integrated FPU");
        strcpy(info->mmu, "Integrated MMU");
        switch (ppc_val) {
            case 1:
                strcpy(info->model, "PowerPC 601");
                strcpy(info->microarch, "PowerPC 601");
                strcpy(info->codename, "601");
                strcpy(info->process, "0.6μm");
                break;
            case 2:
                strcpy(info->model, "PowerPC 603");
                strcpy(info->microarch, "PowerPC 603");
                strcpy(info->codename, "603");
                strcpy(info->process, "0.5μm");
                break;
            case 3:
                strcpy(info->model, "PowerPC 604");
                strcpy(info->microarch, "PowerPC 604");
                strcpy(info->codename, "604");
                strcpy(info->process, "350nm");
                break;
            case 4:
                strcpy(info->model, "PowerPC 603e");
                strcpy(info->microarch, "PowerPC 603e");
                strcpy(info->codename, "603e");
                strcpy(info->process, "350nm");
                break;
            case 8:
                strcpy(info->model, "PowerPC 750 (G3)");
                strcpy(info->microarch, "PowerPC 750 (G3)");
                strcpy(info->codename, "Arthur");
                strcpy(info->process, "260nm");
                break;
            case 12:
                strcpy(info->model, "PowerPC 7400 (G4)");
                strcpy(info->microarch, "PowerPC 7400 (G4)");
                strcpy(info->codename, "Max");
                strcpy(info->process, "200nm");
                break;
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
        info->is_powerpc = false;
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
                strcpy(info->process, "3.5μm");
                break;
            case 2:
                strcpy(info->model, "MC68010");
                strcpy(info->microarch, "Motorola 68010");
                strcpy(info->codename, "68010");
                strcpy(info->process, "3.0μm");
                break;
            case 3:
                strcpy(info->model, "MC68020");
                strcpy(info->microarch, "Motorola 68020");
                strcpy(info->codename, "68020");
                strcpy(info->process, "1.5μm");
                break;
            case 4:
                strcpy(info->model, "MC68030");
                strcpy(info->microarch, "Motorola 68030");
                strcpy(info->codename, "68030");
                strcpy(info->process, "0.8μm");
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
                strcpy(info->process, "0.65μm");
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
    strcpy(info->process, "0.8μm");
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

    char line[256];
    uint32_t offset = 0;

    #define APPEND_LINE(str, clr, is_bold) do { \
        size_t len = strlen(str); \
        if (offset + len < out_buf_size) { \
            strcat(out_buf, str); \
            if (color && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)len; \
                out_runs[*out_run_count].color = clr; \
                out_runs[*out_run_count].bold = is_bold; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)len; \
        } \
    } while(0)

    // Header
    const char* arch_str = info->is_powerpc ? "powerpc" : "m68k";
    snprintf(line, sizeof(line), "--------------- Rustid 2.2.0 (%s-macos_classic) ---------------\n\n", arch_str);
    APPEND_LINE(line, c_sublabel, false);

    if (view_mode == VIEW_STANDARD || view_mode == VIEW_EVERYTHING) {
        snprintf(line, sizeof(line), "System:       %s\n", info->system_name);
        APPEND_LINE(line, c_label, false);

        snprintf(line, sizeof(line), "OS:           %s\n", info->os_version);
        APPEND_LINE(line, c_label, false);

        snprintf(line, sizeof(line), "Model:        %s\n", info->model);
        APPEND_LINE(line, c_highlight, true);

        snprintf(line, sizeof(line), "MicroArch:    %s\n", info->microarch);
        APPEND_LINE(line, c_body, false);

        snprintf(line, sizeof(line), "Codename:     %s\n", info->codename);
        APPEND_LINE(line, c_body, false);

        if (info->process[0]) {
            snprintf(line, sizeof(line), "Process:      %s\n", info->process);
            APPEND_LINE(line, c_body, false);
        }

        if (info->fpu[0] && strcmp(info->fpu, "None") != 0) {
            snprintf(line, sizeof(line), "FPU:          %s\n", info->fpu);
            APPEND_LINE(line, c_body, false);
        }

        if (info->mmu[0] && strcmp(info->mmu, "None") != 0) {
            snprintf(line, sizeof(line), "MMU:          %s\n", info->mmu);
            APPEND_LINE(line, c_body, false);
        }

        if (info->clock_mhz > 0) {
            snprintf(line, sizeof(line), "Frequency:    %u MHz\n", (unsigned int)info->clock_mhz);
            APPEND_LINE(line, c_highlight, false);
        }

        if (info->bus_mhz > 0) {
            snprintf(line, sizeof(line), "Bus Speed:    %u MHz\n", (unsigned int)info->bus_mhz);
            APPEND_LINE(line, c_body, false);
        }

        if (info->ram_mb > 0) {
            snprintf(line, sizeof(line), "Memory:       %u MB RAM\n", (unsigned int)info->ram_mb);
            APPEND_LINE(line, c_body, false);
        }
    }

    if (view_mode == VIEW_DEBUG || view_mode == VIEW_EVERYTHING) {
        if (view_mode == VIEW_EVERYTHING) {
            APPEND_LINE("\n--------------------\n\n", c_sublabel, false);
        }
        snprintf(line, sizeof(line), "Debug Information:\n");
        APPEND_LINE(line, c_sublabel, true);

        snprintf(line, sizeof(line), "  Arch:       %s\n", arch_str);
        APPEND_LINE(line, c_body, false);

        snprintf(line, sizeof(line), "  Target:     Classic Macintosh Toolbox\n");
        APPEND_LINE(line, c_body, false);

        snprintf(line, sizeof(line), "  Gestalt:    Active\n");
        APPEND_LINE(line, c_body, false);
    }

    #undef APPEND_LINE
}
