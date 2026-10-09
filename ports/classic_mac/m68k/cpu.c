#include "m68k/cpu.h"
#include "common/gestalt.h"

#include <string.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <OSUtils.h>
#endif

void classic_mac_detect_m68k(MacCpuInfo* info, const MacModelSpec* spec) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long fpu_val = 0, mmu_val = 0, cpu_val = 0;

#if defined(__m68k__) || defined(__mc68000__)
    SysEnvRec env;
    bool has_env = (SysEnvirons(curSysEnvVers, &env) == noErr);
#endif

    if (!classic_mac_safe_gestalt(gestaltFPUType, &fpu_val) || fpu_val == 0) {
#if defined(__m68k__) || defined(__mc68000__)
        if (has_env && !env.hasFPU) {
            fpu_val = 0;
        } else
#endif
        {
            fpu_val = spec ? spec->fpu_type : 0;
        }
    }
    if (!classic_mac_safe_gestalt(gestaltMMUType, &mmu_val) || mmu_val == 0) {
        mmu_val = spec ? spec->mmu_type : 0;
    }
    if (!classic_mac_safe_gestalt(gestaltProcessorType, &cpu_val) || cpu_val == 0) {
#if defined(__m68k__) || defined(__mc68000__)
        if (has_env && env.processor >= 1 && env.processor <= 5) {
            cpu_val = env.processor;
        } else
#endif
        {
            cpu_val = spec ? spec->cpu_type : 0;
        }
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
#else
    (void)spec;
    strcpy(info->model, "MC68030");
    strcpy(info->microarch, "Motorola 68030");
    strcpy(info->codename, "68030");
    strcpy(info->process, "0.8\xb5m");
    strcpy(info->fpu, "Motorola 68882");
    strcpy(info->mmu, "Integrated 68030 MMU");
#endif
}
