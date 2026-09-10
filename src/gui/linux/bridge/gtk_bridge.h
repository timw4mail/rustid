#ifndef GTK_BRIDGE_H
#define GTK_BRIDGE_H

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

typedef void (*CmdCallback)(uint32_t cmd_id);
typedef void (*FileCallback)(const char* path, bool is_save);
typedef void (*QuitCallback)(void);

bool linux_gui_init(const char* title, float width, float height, float min_w, float min_h);
void linux_gui_set_callbacks(CmdCallback on_cmd, FileCallback on_file, QuitCallback on_quit);
void linux_gui_set_text(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg_color);
void linux_gui_set_status(const char* part1, const char* part2, const char* part3);
void linux_gui_set_menu_checks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact);
void linux_gui_open_file_dialog(void);
void linux_gui_save_file_dialog(const char* default_filename);
void linux_gui_copy_clipboard(const char* text);
void linux_gui_show_alert(const char* title, const char* message);
void linux_gui_run(void);

#ifdef __cplusplus
}
#endif

#endif // GTK_BRIDGE_H
