#ifndef CLASSIC_MAC_PPC_CPU_H
#define CLASSIC_MAC_PPC_CPU_H

#include "common/cpu.h"
#include "models/models.h"

#ifdef __cplusplus
extern "C" {
#endif

void classic_mac_detect_ppc(MacCpuInfo* info, const MacModelSpec* spec, long mach_val);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_PPC_CPU_H
