#ifndef MAC_BRIDGE_H
#define MAC_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} CRgbColor;

typedef struct {
    uint32_t offset;
    uint32_t length;
    CRgbColor color;
    bool bold;
} CTextRun;

enum {
    CMD_FILE_REFRESH    = 101,
    CMD_FILE_EXIT       = 102,
    CMD_FILE_COPY       = 103,

    CMD_MODE_STANDARD   = 201,
    CMD_MODE_DEBUG      = 202,
    CMD_MODE_EVERYTHING = 203,

    CMD_OPT_COLOR       = 301,

    CMD_HELP_ABOUT      = 401
};

typedef void (*CmdCallback)(uint32_t cmd_id);
typedef void (*FileCallback)(const char* path, bool is_save);
typedef void (*QuitCallback)(void);

bool mac_gui_init(const char* title, short width, short height);
void mac_gui_set_callbacks(CmdCallback on_cmd, FileCallback on_file, QuitCallback on_quit);
void mac_gui_set_text(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg_color);
void mac_gui_set_status(const char* part1, const char* part2, const char* part3);
void mac_gui_set_menu_checks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact);
void mac_gui_open_file_dialog(void);
void mac_gui_save_file_dialog(const char* default_filename);
void mac_gui_copy_clipboard(const char* text);
void mac_gui_show_alert(const char* title, const char* message);
void mac_gui_run(void);

#ifdef __cplusplus
}
#endif

#endif // MAC_BRIDGE_H
