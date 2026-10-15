#ifndef CLASSIC_MAC_COMMON_DISPLAY_H
#define CLASSIC_MAC_COMMON_DISPLAY_H

#include "common/cpu.h"
#include "gui/gui.h"

#ifdef __cplusplus
extern "C" {
#endif

enum {
    VIEW_STANDARD   = 201,
    VIEW_DEBUG      = 202,
    VIEW_EVERYTHING = 203
};

extern const CRgbColor PALETTE_BG;
extern const CRgbColor PALETTE_LABEL;
extern const CRgbColor PALETTE_SUBLABEL;
extern const CRgbColor PALETTE_BODY;
extern const CRgbColor PALETTE_HIGHLIGHT;

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

#endif // CLASSIC_MAC_COMMON_DISPLAY_H
