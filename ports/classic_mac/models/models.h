#ifndef CLASSIC_MAC_MODELS_H
#define CLASSIC_MAC_MODELS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint32_t cpu_type;  // 1=68000, 2=68010, 3=68020, 4=68030, 5=68040, 6=68060
    uint32_t clock_mhz;
    uint32_t bus_mhz;
    uint32_t fpu_type;  // 0=None, 1=68881, 2=68882, 3=68040
    uint32_t mmu_type;  // 0=None, 1=AMU, 2=68851, 3=68030, 4=68040
} MacModelSpec;

const char* classic_mac_model_from_identifier(const char* identifier);
bool classic_mac_probe_model_identifier(char* out_buf, size_t out_buf_size);
const char* classic_mac_get_model_name(long mach_id);
MacModelSpec classic_mac_get_model_spec(long mach_id, const char* model_id);

#ifdef __cplusplus
}
#endif

#endif // CLASSIC_MAC_MODELS_H
