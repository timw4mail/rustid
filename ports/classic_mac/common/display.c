#include "common/display.h"
#include "common/gestalt.h"

#include <stdio.h>
#include <string.h>

#ifndef RUSTID_VERSION
#define RUSTID_VERSION "2.3.0"
#endif

const CRgbColor PALETTE_BG        = {255, 255, 255};
const CRgbColor PALETTE_LABEL     = {9, 134, 88};
const CRgbColor PALETTE_SUBLABEL  = {4, 81, 165};
const CRgbColor PALETTE_BODY      = {30, 30, 30};
const CRgbColor PALETTE_HIGHLIGHT = {163, 21, 21};

void classic_mac_generate_report(
    const MacCpuInfo* info,
    uint32_t view_mode,
    bool color,
    char* out_buf,
    uint32_t out_buf_size,
    CTextRun* out_runs,
    uint32_t* out_run_count,
    uint32_t max_runs
) {
    CRgbColor c_label = PALETTE_LABEL;
    CRgbColor c_sublabel = PALETTE_SUBLABEL;
    CRgbColor c_body = PALETTE_BODY;
    CRgbColor c_highlight = PALETTE_HIGHLIGHT;

    if (out_run_count) *out_run_count = 0;
    out_buf[0] = '\0';

    uint32_t offset = 0;

    #define APPEND_HEADER(str, clr) do { \
        size_t len = strlen(str); \
        if (offset + len < out_buf_size) { \
            strcat(out_buf, str); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)len; \
                out_runs[*out_run_count].color = clr; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)len; \
        } \
    } while(0)

    #define APPEND_FIELD(label, val_str, val_clr) do { \
        char lbl_buf[32]; \
        snprintf(lbl_buf, sizeof(lbl_buf), "%14s: ", label); \
        size_t lbl_len = strlen(lbl_buf); \
        char val_buf[256]; \
        snprintf(val_buf, sizeof(val_buf), "%s\r\r", val_str); \
        size_t val_len = strlen(val_buf); \
        if (offset + lbl_len + val_len < out_buf_size) { \
            strcat(out_buf, lbl_buf); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)lbl_len; \
                out_runs[*out_run_count].color = c_label; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)lbl_len; \
            strcat(out_buf, val_buf); \
            if (color && out_runs && out_run_count && *out_run_count < max_runs) { \
                out_runs[*out_run_count].offset = offset; \
                out_runs[*out_run_count].length = (uint32_t)val_len; \
                out_runs[*out_run_count].color = val_clr; \
                out_runs[*out_run_count].bold = false; \
                (*out_run_count)++; \
            } \
            offset += (uint32_t)val_len; \
        } \
    } while(0)

    // Header banner
    const char* arch_str = info->is_powerpc ? "powerpc" : "m68k";
    char banner[256];
    snprintf(banner, sizeof(banner), "--------- Rustid %s (%s-macos_classic) ---------\r\r", RUSTID_VERSION, arch_str);
    APPEND_HEADER(banner, c_sublabel);

    if (view_mode == VIEW_STANDARD || view_mode == VIEW_EVERYTHING) {
        if (info->system_name[0]) {
            APPEND_FIELD("System", info->system_name, c_body);
        }
        if (info->os_version[0]) {
            APPEND_FIELD("OS", info->os_version, c_body);
        }
        if (info->model[0]) {
            APPEND_FIELD("Model", info->model, c_highlight);
        }
        if (info->microarch[0] && !info->is_powerpc) {
            APPEND_FIELD("MicroArch", info->microarch, c_body);
        }
        if (info->codename[0] && info->is_powerpc) {
            APPEND_FIELD("Codename", info->codename, c_body);
        }
        if (info->process[0]) {
            APPEND_FIELD("Process", info->process, c_body);
        }
        if (info->fpu[0] && strcmp(info->fpu, "None") != 0) {
            APPEND_FIELD("FPU", info->fpu, c_body);
        }
        if (info->mmu[0] && strcmp(info->mmu, "None") != 0) {
            APPEND_FIELD("MMU", info->mmu, c_body);
        }
        if (info->clock_mhz > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MHz", (unsigned int)info->clock_mhz);
            APPEND_FIELD("Frequency", num_buf, c_body);
        }
        if (info->bus_mhz > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MHz", (unsigned int)info->bus_mhz);
            APPEND_FIELD("Bus Speed", num_buf, c_body);
        }
        if (info->ram_mb > 0) {
            char num_buf[32];
            snprintf(num_buf, sizeof(num_buf), "%u MB RAM", (unsigned int)info->ram_mb);
            APPEND_FIELD("Memory", num_buf, c_body);
        }
    }

    if (view_mode == VIEW_DEBUG || view_mode == VIEW_EVERYTHING) {
        if (view_mode == VIEW_EVERYTHING) {
            APPEND_HEADER("--------------------\r\r", c_sublabel);
        }
        APPEND_HEADER("Debug Information:\r\r", c_sublabel);
        APPEND_FIELD("Arch", arch_str, c_body);
        APPEND_FIELD("Target", "Classic Macintosh Toolbox", c_body);
        APPEND_FIELD("Gestalt", classic_mac_is_gestalt_available() ? "Active" : "Unavailable", c_body);
        if (info->model_id[0]) {
            APPEND_FIELD("Model ID", info->model_id, c_body);
        }
    }

    #undef APPEND_HEADER
    #undef APPEND_FIELD
}
