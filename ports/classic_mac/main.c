#include "classic_mac_engine.h"

#include <stdio.h>
#include <string.h>

#ifndef RUSTID_VERSION
#define RUSTID_VERSION "2.3.0"
#endif

#ifndef NO_STANDALONE_MAIN
static MacCpuInfo s_cpu_info;
static uint32_t s_view_mode = VIEW_STANDARD;
static bool s_color = true;
static char s_report_buf[8192];
static CTextRun s_runs[256];
static uint32_t s_run_count = 0;

static void render_view(void) {
    s_run_count = 0;
    classic_mac_generate_report(
        &s_cpu_info,
        s_view_mode,
        s_color,
        s_report_buf,
        sizeof(s_report_buf),
        s_runs,
        &s_run_count,
        256
    );

    mac_gui_set_text(s_report_buf, strlen(s_report_buf), s_runs, s_run_count, PALETTE_BG);
    mac_gui_set_status(s_cpu_info.system_name, s_cpu_info.model, s_cpu_info.os_version);
    mac_gui_set_menu_checks(s_view_mode, s_color);
}

static void on_command(uint32_t cmd_id) {
    switch (cmd_id) {
        case CMD_FILE_REFRESH: // 101
            classic_mac_detect_cpu(&s_cpu_info);
            render_view();
            break;
        case CMD_FILE_EXIT: // 102
            break;
        case CMD_FILE_COPY: // 103
            mac_gui_copy_clipboard(s_report_buf);
            break;
        case CMD_MODE_STANDARD: // 201
        case CMD_MODE_DEBUG: // 202
        case CMD_MODE_EVERYTHING: // 203
            s_view_mode = cmd_id;
            render_view();
            break;
        case CMD_OPT_COLOR: // 301
            s_color = !s_color;
            render_view();
            break;
        case CMD_HELP_ABOUT: // 401
            mac_gui_show_alert("About rustid", "rustid " RUSTID_VERSION " for Classic Macintosh\rCPU Identification Tool");
            break;
        default:
            break;
    }
}

static void on_file(const char* path, bool is_save) {
    if (is_save && path && path[0]) {
        FILE* f = fopen(path, "w");
        if (f) {
            fputs(s_report_buf, f);
            fclose(f);
        }
    }
}

static void on_quit(void) {
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    if (!mac_gui_init("rustid", 480, 320)) {
        return 1;
    }
    mac_gui_set_callbacks(on_command, on_file, on_quit);
    classic_mac_detect_cpu(&s_cpu_info);
    render_view();
    mac_gui_run();
    return 0;
}
#endif
