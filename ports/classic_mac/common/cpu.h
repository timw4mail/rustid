#ifndef CLASSIC_MAC_COMMON_CPU_H
#define CLASSIC_MAC_COMMON_CPU_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char model[64];
    char microarch[64];
    char codename[32];
    char process[32];
    char fpu[32];
    char mmu[32];
    uint32_t clock_mhz;
    uint32_t bus_mhz;
    char system_name[96];
    char model_id[32];
    char os_version[32];
    uint32_t ram_mb;
    bool is_powerpc;
} MacCpuInfo;

void classic_mac_detect_cpu(MacCpuInfo* info);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_COMMON_CPU_H
