#ifndef CLASSIC_MAC_ENGINE_H
#define CLASSIC_MAC_ENGINE_H

#include "mac_bridge.h"

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
    char system_name[64];
    char os_version[32];
    uint32_t ram_mb;
    bool is_powerpc;
} MacCpuInfo;

void classic_mac_detect_cpu(MacCpuInfo* info);
void classic_mac_generate_report(
    const MacCpuInfo* info,
    uint32_t view_mode,
    bool color,
    char* out_buf,
    uint32_t out_buf_size,
    CTextRun* out_runs,
    uint32_t* out_run_count,
    uint32_t max_runs
);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_ENGINE_H
