#include "common/cpu.h"
#include "common/gestalt.h"
#include "models/models.h"
#include "m68k/cpu.h"
#include "ppc/cpu.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Types.h>
#include <OSUtils.h>
#endif

void classic_mac_detect_cpu(MacCpuInfo* info) {
    memset(info, 0, sizeof(MacCpuInfo));

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    long mach_val = 0, sysv_val = 0, ram_val = 0;
    classic_mac_safe_gestalt(gestaltMachineType, &mach_val);
    classic_mac_safe_gestalt(gestaltSystemVersion, &sysv_val);
    classic_mac_safe_gestalt(gestaltPhysicalRAMSize, &ram_val);

#if defined(__m68k__) || defined(__mc68000__)
    SysEnvRec env;
    bool has_env = (SysEnvirons(curSysEnvVers, &env) == noErr);
    if (has_env) {
        if (mach_val == 0) {
            switch (env.machineType) {
                case env512KE:   mach_val = 3; break;  // Mac 512Ke
                case envMacPlus: mach_val = 4; break;  // Mac Plus
                case envSE:      mach_val = 5; break;  // Mac SE
                case envMacII:   mach_val = 6; break;  // Mac II
                case 6:          mach_val = 7; break;  // Mac IIx
                case 7:          mach_val = 8; break;  // Mac IIcx
                case 8:          mach_val = 9; break;  // Mac SE/30
                case 9:          mach_val = 10; break; // Mac IIci
                case 11:         mach_val = 13; break; // Mac IIfx
                case 13:         mach_val = 18; break; // Mac Classic
                case 14:         mach_val = 19; break; // Mac IIsi
                case 15:         mach_val = 22; break; // Mac LC
                case 17:         mach_val = 20; break; // Mac Quadra 900
                case 18:         mach_val = 23; break; // Mac Quadra 700
                case 19:         mach_val = 25; break; // Mac Classic II
                case 20:         mach_val = 24; break; // PowerBook 170
                case 21:         mach_val = 21; break; // PowerBook 100
                case 22:         mach_val = 25; break; // PowerBook 140
                default:         mach_val = (env.machineType > 0) ? env.machineType : 0; break;
            }
        }
        if (sysv_val == 0 && env.systemVersion > 0) {
            sysv_val = env.systemVersion;
        }
    }
#endif

    // 1. Probe Open Firmware / Name Registry for model identifier string (e.g. "PowerMac1,1")
    bool has_model_id = classic_mac_probe_model_identifier(info->model_id, sizeof(info->model_id));

    // 2. Query hardware specification (clocks, bus) with model_id fallback
    MacModelSpec spec = classic_mac_get_model_spec(mach_val, has_model_id ? info->model_id : NULL);

    if (ram_val > 0) {
        info->ram_mb = (uint32_t)(ram_val / (1024 * 1024));
    }

    // 3. Resolve system model name: prefer model identifier string if it exists
    const char* id_model_name = has_model_id ? classic_mac_model_from_identifier(info->model_id) : NULL;
    if (id_model_name != NULL) {
        snprintf(info->system_name, sizeof(info->system_name), "%s", id_model_name);
    } else {
        // Fall back to Gestalt machine type
        snprintf(info->system_name, sizeof(info->system_name), "%s", classic_mac_get_model_name(mach_val));

        // Query gestaltUserVisibleMachineName ('mnam') for accurate human-readable model name
        long mnam_ptr = 0;
        if (classic_mac_safe_gestalt('mnam', &mnam_ptr) && mnam_ptr > 1024) {
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

    if (sysv_val >= 0x1000) {
        long osx_maj = 0, osx_min = 0, osx_bug = 0;
        if (classic_mac_safe_gestalt('sys1', &osx_maj) && osx_maj > 0) {
            classic_mac_safe_gestalt('sys2', &osx_min);
            classic_mac_safe_gestalt('sys3', &osx_bug);
            if (osx_bug == 0) {
                snprintf(info->os_version, sizeof(info->os_version), "Mac OS X %ld.%ld", osx_maj, osx_min);
            } else {
                snprintf(info->os_version, sizeof(info->os_version), "Mac OS X %ld.%ld.%ld", osx_maj, osx_min, osx_bug);
            }
        } else {
            short minor = (short)((sysv_val >> 4) & 0x0F);
            short patch = (short)(sysv_val & 0x0F);
            if (patch == 0) {
                snprintf(info->os_version, sizeof(info->os_version), "Mac OS X 10.%d", minor);
            } else {
                snprintf(info->os_version, sizeof(info->os_version), "Mac OS X 10.%d.%d", minor, patch);
            }
        }
    } else {
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
    }

    // Determine system architecture: 68k vs PowerPC
    bool is_ppc = false;
#if defined(__powerpc__) || defined(__ppc__)
    is_ppc = true;
#endif

    long arch_val = 0;
    if (classic_mac_safe_gestalt(gestaltSysArchitecture, &arch_val)) {
        if (arch_val == gestaltPowerPC) {
            is_ppc = true;
        } else if (arch_val == gestalt68k) {
            is_ppc = false;
        }
    } else if (!is_ppc) {
        long cput_val = 0;
        if (classic_mac_safe_gestalt(gestaltNativeCPUtype, &cput_val) && cput_val >= 256) {
            is_ppc = true;
        }
    }

    info->is_powerpc = is_ppc;

    // Clock and Bus speed with model spec fallbacks (rounded to nearest MHz)
    long clk_val = 0, bclk_val = 0;
    if (classic_mac_safe_gestalt(gestaltProcClkSpeed, &clk_val) && clk_val > 0) {
        info->clock_mhz = (uint32_t)((clk_val + 500000) / 1000000);
    } else {
        info->clock_mhz = spec.clock_mhz;
    }

    if (classic_mac_safe_gestalt(gestaltBusClkSpeed, &bclk_val) && bclk_val > 0) {
        info->bus_mhz = (uint32_t)((bclk_val + 500000) / 1000000);
    } else {
        info->bus_mhz = spec.bus_mhz;
    }

    if (info->is_powerpc) {
        classic_mac_detect_ppc(info, &spec, mach_val);
    } else {
        classic_mac_detect_m68k(info, &spec);
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
        MacModelSpec spec = classic_mac_get_model_spec(0, info->model_id);
        if (spec.clock_mhz > 0) info->clock_mhz = spec.clock_mhz;
        if (spec.bus_mhz > 0) info->bus_mhz = spec.bus_mhz;
    } else {
        (void)classic_mac_get_model_name(0);
        (void)classic_mac_get_model_spec(0, NULL);
    }
#endif
}
