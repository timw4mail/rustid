#ifndef CLASSIC_MAC_M68K_CPU_H
#define CLASSIC_MAC_M68K_CPU_H

#include "common/cpu.h"
#include "models/models.h"

#ifdef __cplusplus
extern "C" {
#endif

void classic_mac_detect_m68k(MacCpuInfo* info, const MacModelSpec* spec);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_M68K_CPU_H
