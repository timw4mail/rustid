#include "m68k/cpu.h"
#include "common/gestalt.h"

#include <string.h>

#include <stdlib.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <OSUtils.h>
#endif

void classic_mac_detect_m68k(MacCpuInfo* info, const MacModelSpec* spec) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long fpu_val = 0, mmu_val = 0, cpu_val = 0;
    bool fpu_gestalt_ok = false, mmu_gestalt_ok = false, cpu_gestalt_ok = false;

#if defined(__m68k__) || defined(__mc68000__)
    SysEnvRec env;
    bool has_env = (SysEnvirons(curSysEnvVers, &env) == noErr);
#endif

    fpu_gestalt_ok = classic_mac_safe_gestalt(gestaltFPUType, &fpu_val);
    mmu_gestalt_ok = classic_mac_safe_gestalt(gestaltMMUType, &mmu_val);
    cpu_gestalt_ok = classic_mac_safe_gestalt(gestaltProcessorType, &cpu_val);

    // 1. Resolve CPU type
    if (!cpu_gestalt_ok || cpu_val == 0) {
#if defined(__m68k__) || defined(__mc68000__)
        if (has_env && env.processor >= 1 && env.processor <= 5) {
            cpu_val = env.processor;
        } else
#endif
        {
            cpu_val = spec ? spec->cpu_type : 1;
        }
    }
    if (cpu_val == 0) cpu_val = 1;

    // 2. Resolve FPU type
    if (!fpu_gestalt_ok) {
        if (cpu_val <= 2) {
            // 68000 / 68010 never have an FPU
            fpu_val = 0;
        } else {
#if defined(__m68k__) || defined(__mc68000__)
            if (has_env) {
                if (!env.hasFPU) {
                    fpu_val = 0;
                } else if (cpu_val == 4) {
                    fpu_val = (spec && spec->fpu_type > 0) ? spec->fpu_type : 2; // 68882 default on 68030
                } else if (cpu_val == 5) {
                    fpu_val = 3; // 68040 FPU
                } else {
                    fpu_val = (spec && spec->fpu_type > 0) ? spec->fpu_type : 1; // 68881 default
                }
            } else
#endif
            {
                fpu_val = spec ? spec->fpu_type : 0;
            }
        }
    }

    // 3. Resolve MMU type
    if (!mmu_gestalt_ok) {
        if (cpu_val <= 2) {
            // 68000 / 68010 never have an MMU
            mmu_val = 0;
        } else if (cpu_val == 3) {
            // 68020 has no MMU unless spec explicitly declares AMU (1) or 68851 (2)
            mmu_val = (spec && spec->mmu_type <= 2) ? spec->mmu_type : 0;
        } else if (cpu_val == 4) {
            // 68030 integrated MMU
            mmu_val = (spec && spec->mmu_type > 0) ? spec->mmu_type : 3;
        } else if (cpu_val >= 5) {
            // 68040 integrated MMU
            mmu_val = (spec && spec->mmu_type > 0) ? spec->mmu_type : 4;
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
    long fpu_val = spec ? spec->fpu_type : 0;
    long mmu_val = spec ? spec->mmu_type : 0;
    long cpu_val = spec ? spec->cpu_type : 4;
    if (cpu_val == 0) cpu_val = 4;

    const char* mock_cpu = getenv("RUSTID_MOCK_CPU");
    if (mock_cpu) cpu_val = atol(mock_cpu);
    const char* mock_fpu = getenv("RUSTID_MOCK_FPU");
    if (mock_fpu) fpu_val = atol(mock_fpu);
    const char* mock_mmu = getenv("RUSTID_MOCK_MMU");
    if (mock_mmu) mmu_val = atol(mock_mmu);

    if (cpu_val <= 2) {
        if (!mock_fpu) fpu_val = 0;
        if (!mock_mmu) mmu_val = 0;
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
#endif
}
