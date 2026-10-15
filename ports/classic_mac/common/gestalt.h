#ifndef CLASSIC_MAC_COMMON_GESTALT_H
#define CLASSIC_MAC_COMMON_GESTALT_H

#include <stdint.h>
#include <stdbool.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Gestalt.h>
#endif

#ifdef __cplusplus
extern "C" {
#endif

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

#ifndef gestaltMachineType
#define gestaltMachineType 'mach'
#endif

#ifndef gestaltSystemVersion
#define gestaltSystemVersion 'sysv'
#endif

#ifndef gestaltPhysicalRAMSize
#define gestaltPhysicalRAMSize 'ram '
#endif

#ifndef gestaltFPUType
#define gestaltFPUType 'fpu '
#endif

#ifndef gestaltMMUType
#define gestaltMMUType 'mmu '
#endif

bool classic_mac_safe_gestalt(uint32_t selector, long* response);
bool classic_mac_is_gestalt_available(void);
bool classic_mac_has_color_qd(void);
bool classic_mac_has_styled_te(void);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_COMMON_GESTALT_H
