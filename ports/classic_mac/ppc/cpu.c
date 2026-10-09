#include "ppc/cpu.h"
#include "common/gestalt.h"

#include <string.h>

void classic_mac_detect_ppc(MacCpuInfo* info, const MacModelSpec* spec, long mach_val) {
    (void)spec;
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long ppc_val = 0;
    classic_mac_safe_gestalt(gestaltNativeCPUtype, &ppc_val);
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
        case 270:
            strcpy(info->model, "PowerPC 970 (G5)");
            strcpy(info->microarch, "PowerPC 970 (G5)");
            strcpy(info->codename, "GP");
            strcpy(info->process, "130nm");
            break;
        case 275:
            strcpy(info->model, "PowerPC 970FX (G5)");
            strcpy(info->microarch, "PowerPC 970FX (G5)");
            strcpy(info->codename, "GP-Plus");
            strcpy(info->process, "90nm");
            break;
        case 276:
            strcpy(info->model, "PowerPC 970MP (G5)");
            strcpy(info->microarch, "PowerPC 970MP (G5)");
            strcpy(info->codename, "Antares");
            strcpy(info->process, "90nm");
            break;
        default:
            strcpy(info->model, "PowerPC (Generic)");
            strcpy(info->microarch, "PowerPC");
            strcpy(info->codename, "PPC");
            break;
    }
#else
    (void)mach_val;
    strcpy(info->model, "PowerPC 750 (G3)");
    strcpy(info->microarch, "PowerPC 750 (G3)");
    strcpy(info->codename, "Arthur");
    strcpy(info->process, "260nm");
    strcpy(info->fpu, "Integrated FPU");
    strcpy(info->mmu, "Integrated MMU");
#endif
}
