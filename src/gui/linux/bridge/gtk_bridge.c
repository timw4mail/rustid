//! Native Linux GTK3 GUI implementation for rustid.

#include "gtk_bridge.h"

#include <gtk/gtk.h>
#include <gdk/gdk.h>
#include <pango/pango.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

static CmdCallback g_cmd_cb = NULL;
static FileCallback g_file_cb = NULL;
static QuitCallback g_quit_cb = NULL;

static GtkWidget *g_window = NULL;
static GtkWidget *g_text_view = NULL;
static GtkTextBuffer *g_text_buffer = NULL;

static GtkWidget *g_status_label1 = NULL;
static GtkWidget *g_status_label2 = NULL;
static GtkWidget *g_status_label3 = NULL;

static GtkCheckMenuItem *g_item_mode_standard = NULL;
static GtkCheckMenuItem *g_item_mode_debug = NULL;
static GtkCheckMenuItem *g_item_mode_everything = NULL;
static GtkCheckMenuItem *g_item_mode_dump = NULL;

static GtkCheckMenuItem *g_item_opt_color = NULL;
static GtkCheckMenuItem *g_item_opt_dark_theme = NULL;
static GtkCheckMenuItem *g_item_opt_verbose = NULL;
static GtkCheckMenuItem *g_item_opt_compact = NULL;

static GtkCssProvider *g_css_provider = NULL;
static bool g_updating_menu = false;

enum {
    CMD_FILE_OPEN       = 101,
    CMD_FILE_EXPORT     = 102,
    CMD_FILE_COPY       = 103,
    CMD_FILE_REFRESH    = 104,
    CMD_FILE_EXIT       = 105,

    CMD_MODE_STANDARD   = 201,
    CMD_MODE_DEBUG      = 202,
    CMD_MODE_EVERYTHING = 203,
    CMD_MODE_DUMP       = 204,

    CMD_OPT_COLOR       = 301,
    CMD_OPT_DARK_THEME  = 302,
    CMD_OPT_VERBOSE     = 303,
    CMD_OPT_COMPACT     = 304,

    CMD_HELP_ABOUT      = 401
};

static void on_menu_item_activate(GtkMenuItem *item, gpointer user_data) {
    (void)item;
    if (g_updating_menu) {
        return;
    }
    uint32_t cmd_id = (uint32_t)(uintptr_t)user_data;
    if (cmd_id == CMD_FILE_EXIT) {
        if (g_window) {
            gtk_widget_destroy(g_window);
        }
        return;
    }
    if (g_cmd_cb) {
        g_cmd_cb(cmd_id);
    }
}

static void on_window_destroy(GtkWidget *widget, gpointer user_data) {
    (void)widget;
    (void)user_data;
    if (g_quit_cb) {
        g_quit_cb();
    }
    gtk_main_quit();
}

static GtkWidget* create_menu_item(GtkWidget *menu, const char *label, uint32_t cmd_id,
                                   GtkAccelGroup *accel_group, guint key, GdkModifierType mods) {
    GtkWidget *item = gtk_menu_item_new_with_label(label);
    g_signal_connect(G_OBJECT(item), "activate", G_CALLBACK(on_menu_item_activate), (gpointer)(uintptr_t)cmd_id);
    if (accel_group && key != 0) {
        gtk_widget_add_accelerator(item, "activate", accel_group, key, mods, GTK_ACCEL_VISIBLE);
    }
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), item);
    return item;
}

static GtkCheckMenuItem* create_check_menu_item(GtkWidget *menu, const char *label, uint32_t cmd_id) {
    GtkWidget *item = gtk_check_menu_item_new_with_label(label);
    g_signal_connect(G_OBJECT(item), "activate", G_CALLBACK(on_menu_item_activate), (gpointer)(uintptr_t)cmd_id);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), item);
    return GTK_CHECK_MENU_ITEM(item);
}

bool linux_gui_init(const char* title, float width, float height, float min_w, float min_h) {
    int argc = 0;
    char **argv = NULL;
    if (!gtk_init_check(&argc, &argv)) {
        return false;
    }

    g_window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    if (!g_window) {
        return false;
    }

    gtk_window_set_title(GTK_WINDOW(g_window), title);
    gtk_window_set_default_size(GTK_WINDOW(g_window), (gint)width, (gint)height);
    GdkGeometry geom;
    geom.min_width = (gint)min_w;
    geom.min_height = (gint)min_h;
    gtk_window_set_geometry_hints(GTK_WINDOW(g_window), NULL, &geom, GDK_HINT_MIN_SIZE);

    g_signal_connect(G_OBJECT(g_window), "destroy", G_CALLBACK(on_window_destroy), NULL);

    GtkAccelGroup *accel_group = gtk_accel_group_new();
    gtk_window_add_accel_group(GTK_WINDOW(g_window), accel_group);

    GtkWidget *main_vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_container_add(GTK_CONTAINER(g_window), main_vbox);

    // Menu bar
    GtkWidget *menubar = gtk_menu_bar_new();
    gtk_box_pack_start(GTK_BOX(main_vbox), menubar, FALSE, FALSE, 0);

    // --- File Menu ---
    GtkWidget *file_menu = gtk_menu_new();
    gtk_menu_set_accel_group(GTK_MENU(file_menu), accel_group);
    GtkWidget *file_item = gtk_menu_item_new_with_mnemonic("_File");
    gtk_menu_item_set_submenu(GTK_MENU_ITEM(file_item), file_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menubar), file_item);

    create_menu_item(file_menu, "Open Dump...", CMD_FILE_OPEN, accel_group, GDK_KEY_o, GDK_CONTROL_MASK);
    create_menu_item(file_menu, "Export Dump...", CMD_FILE_EXPORT, accel_group, GDK_KEY_s, GDK_CONTROL_MASK);
    create_menu_item(file_menu, "Copy", CMD_FILE_COPY, accel_group, GDK_KEY_c, GDK_CONTROL_MASK);
    create_menu_item(file_menu, "Refresh", CMD_FILE_REFRESH, accel_group, GDK_KEY_F5, 0);
    gtk_menu_shell_append(GTK_MENU_SHELL(file_menu), gtk_separator_menu_item_new());
    create_menu_item(file_menu, "Exit", CMD_FILE_EXIT, accel_group, GDK_KEY_q, GDK_CONTROL_MASK);

    // --- View Menu ---
    GtkWidget *view_menu = gtk_menu_new();
    GtkWidget *view_item = gtk_menu_item_new_with_mnemonic("_View");
    gtk_menu_item_set_submenu(GTK_MENU_ITEM(view_item), view_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menubar), view_item);

    g_item_mode_standard = create_check_menu_item(view_menu, "Standard", CMD_MODE_STANDARD);
    g_item_mode_debug = create_check_menu_item(view_menu, "Debug", CMD_MODE_DEBUG);
    g_item_mode_everything = create_check_menu_item(view_menu, "Everything", CMD_MODE_EVERYTHING);
    g_item_mode_dump = create_check_menu_item(view_menu, "CPUID Dump", CMD_MODE_DUMP);

    // --- Options Menu ---
    GtkWidget *opt_menu = gtk_menu_new();
    GtkWidget *opt_item = gtk_menu_item_new_with_mnemonic("_Options");
    gtk_menu_item_set_submenu(GTK_MENU_ITEM(opt_item), opt_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menubar), opt_item);

    g_item_opt_color = create_check_menu_item(opt_menu, "Colors", CMD_OPT_COLOR);
    g_item_opt_dark_theme = create_check_menu_item(opt_menu, "Dark Theme", CMD_OPT_DARK_THEME);
    gtk_menu_shell_append(GTK_MENU_SHELL(opt_menu), gtk_separator_menu_item_new());
    g_item_opt_verbose = create_check_menu_item(opt_menu, "Verbose", CMD_OPT_VERBOSE);
    g_item_opt_compact = create_check_menu_item(opt_menu, "Compact", CMD_OPT_COMPACT);

    // --- Help Menu ---
    GtkWidget *help_menu = gtk_menu_new();
    GtkWidget *help_item = gtk_menu_item_new_with_mnemonic("_Help");
    gtk_menu_item_set_submenu(GTK_MENU_ITEM(help_item), help_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menubar), help_item);

    create_menu_item(help_menu, "About", CMD_HELP_ABOUT, NULL, 0, 0);

    // Text View with Scrolled Window
    GtkWidget *scrolled = gtk_scrolled_window_new(NULL, NULL);
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(scrolled), GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
    gtk_box_pack_start(GTK_BOX(main_vbox), scrolled, TRUE, TRUE, 0);

    g_text_view = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(g_text_view), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(g_text_view), FALSE);
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(g_text_view), TRUE);
    gtk_text_view_set_left_margin(GTK_TEXT_VIEW(g_text_view), 8);
    gtk_text_view_set_right_margin(GTK_TEXT_VIEW(g_text_view), 8);
    gtk_text_view_set_top_margin(GTK_TEXT_VIEW(g_text_view), 6);
    gtk_text_view_set_bottom_margin(GTK_TEXT_VIEW(g_text_view), 6);

    g_text_buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(g_text_view));
    gtk_container_add(GTK_CONTAINER(scrolled), g_text_view);

    // Status bar (3 frames)
    GtkWidget *status_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3);
    gtk_widget_set_margin_start(status_box, 2);
    gtk_widget_set_margin_end(status_box, 2);
    gtk_widget_set_margin_top(status_box, 2);
    gtk_widget_set_margin_bottom(status_box, 2);
    gtk_box_pack_start(GTK_BOX(main_vbox), status_box, FALSE, FALSE, 0);

    GtkWidget *frame1 = gtk_frame_new(NULL);
    gtk_frame_set_shadow_type(GTK_FRAME(frame1), GTK_SHADOW_IN);
    g_status_label1 = gtk_label_new("");
    gtk_label_set_xalign(GTK_LABEL(g_status_label1), 0.0f);
    gtk_widget_set_margin_start(g_status_label1, 4);
    gtk_widget_set_margin_end(g_status_label1, 4);
    gtk_widget_set_size_request(frame1, 320, -1);
    gtk_container_add(GTK_CONTAINER(frame1), g_status_label1);
    gtk_box_pack_start(GTK_BOX(status_box), frame1, FALSE, FALSE, 0);

    GtkWidget *frame2 = gtk_frame_new(NULL);
    gtk_frame_set_shadow_type(GTK_FRAME(frame2), GTK_SHADOW_IN);
    g_status_label2 = gtk_label_new("");
    gtk_label_set_xalign(GTK_LABEL(g_status_label2), 0.0f);
    gtk_widget_set_margin_start(g_status_label2, 4);
    gtk_widget_set_margin_end(g_status_label2, 4);
    gtk_widget_set_size_request(frame2, 200, -1);
    gtk_container_add(GTK_CONTAINER(frame2), g_status_label2);
    gtk_box_pack_start(GTK_BOX(status_box), frame2, FALSE, FALSE, 0);

    GtkWidget *frame3 = gtk_frame_new(NULL);
    gtk_frame_set_shadow_type(GTK_FRAME(frame3), GTK_SHADOW_IN);
    g_status_label3 = gtk_label_new("");
    gtk_label_set_xalign(GTK_LABEL(g_status_label3), 0.0f);
    gtk_widget_set_margin_start(g_status_label3, 4);
    gtk_widget_set_margin_end(g_status_label3, 4);
    gtk_container_add(GTK_CONTAINER(frame3), g_status_label3);
    gtk_box_pack_start(GTK_BOX(status_box), frame3, TRUE, TRUE, 0);

    // CSS styling provider
    g_css_provider = gtk_css_provider_new();
    GdkScreen *screen = gdk_screen_get_default();
    if (screen) {
        gtk_style_context_add_provider_for_screen(screen, GTK_STYLE_PROVIDER(g_css_provider),
                                                  GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    return true;
}

void linux_gui_set_callbacks(CmdCallback on_cmd, FileCallback on_file, QuitCallback on_quit) {
    g_cmd_cb = on_cmd;
    g_file_cb = on_file;
    g_quit_cb = on_quit;
}

void linux_gui_set_text(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg_color) {
    if (!g_text_buffer) {
        return;
    }

    gtk_text_buffer_set_text(g_text_buffer, text ? text : "", (gint)length);

    GtkTextIter buf_start, buf_end;
    gtk_text_buffer_get_start_iter(g_text_buffer, &buf_start);
    gtk_text_buffer_get_end_iter(g_text_buffer, &buf_end);
    gtk_text_buffer_remove_all_tags(g_text_buffer, &buf_start, &buf_end);

    for (uint32_t i = 0; i < run_count; i++) {
        const CTextRun *run = &runs[i];
        GdkRGBA rgba;
        rgba.red = (double)run->color.r / 255.0;
        rgba.green = (double)run->color.g / 255.0;
        rgba.blue = (double)run->color.b / 255.0;
        rgba.alpha = 1.0;

        GtkTextTag *tag = gtk_text_buffer_create_tag(g_text_buffer, NULL,
            "foreground-rgba", &rgba,
            "weight", run->bold ? PANGO_WEIGHT_BOLD : PANGO_WEIGHT_NORMAL,
            NULL);

        GtkTextIter start_iter, end_iter;
        gtk_text_buffer_get_iter_at_offset(g_text_buffer, &start_iter, (gint)run->offset);
        gtk_text_buffer_get_iter_at_offset(g_text_buffer, &end_iter, (gint)(run->offset + run->length));
        gtk_text_buffer_apply_tag(g_text_buffer, tag, &start_iter, &end_iter);
    }

    if (g_css_provider) {
        char css[512];
        snprintf(css, sizeof(css),
            "textview text, textview { background-color: rgb(%u,%u,%u); font-family: monospace; font-size: 10pt; }\n"
            "frame { border: 1px solid alpha(currentColor, 0.2); }\n",
            bg_color.r, bg_color.g, bg_color.b);
        gtk_css_provider_load_from_data(g_css_provider, css, -1, NULL);
    }
}

void linux_gui_set_status(const char* part1, const char* part2, const char* part3) {
    if (g_status_label1) gtk_label_set_text(GTK_LABEL(g_status_label1), part1 ? part1 : "");
    if (g_status_label2) gtk_label_set_text(GTK_LABEL(g_status_label2), part2 ? part2 : "");
    if (g_status_label3) gtk_label_set_text(GTK_LABEL(g_status_label3), part3 ? part3 : "");
}

void linux_gui_set_menu_checks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact) {
    g_updating_menu = true;

    if (g_item_mode_standard) gtk_check_menu_item_set_active(g_item_mode_standard, mode_cmd_id == CMD_MODE_STANDARD);
    if (g_item_mode_debug) gtk_check_menu_item_set_active(g_item_mode_debug, mode_cmd_id == CMD_MODE_DEBUG);
    if (g_item_mode_everything) gtk_check_menu_item_set_active(g_item_mode_everything, mode_cmd_id == CMD_MODE_EVERYTHING);
    if (g_item_mode_dump) gtk_check_menu_item_set_active(g_item_mode_dump, mode_cmd_id == CMD_MODE_DUMP);

    if (g_item_opt_color) gtk_check_menu_item_set_active(g_item_opt_color, color);
    if (g_item_opt_dark_theme) gtk_check_menu_item_set_active(g_item_opt_dark_theme, dark_theme);
    if (g_item_opt_verbose) gtk_check_menu_item_set_active(g_item_opt_verbose, verbose);
    if (g_item_opt_compact) gtk_check_menu_item_set_active(g_item_opt_compact, compact);

    g_updating_menu = false;
}

void linux_gui_open_file_dialog(void) {
    if (!g_window) return;

    GtkWidget *dialog = gtk_file_chooser_dialog_new(
        "Open CPUID Dump File",
        GTK_WINDOW(g_window),
        GTK_FILE_CHOOSER_ACTION_OPEN,
        "_Cancel", GTK_RESPONSE_CANCEL,
        "_Open", GTK_RESPONSE_ACCEPT,
        NULL
    );

    GtkFileFilter *filter = gtk_file_filter_new();
    gtk_file_filter_set_name(filter, "Text files (*.txt)");
    gtk_file_filter_add_pattern(filter, "*.txt");
    gtk_file_chooser_add_filter(GTK_FILE_CHOOSER(dialog), filter);

    GtkFileFilter *all_filter = gtk_file_filter_new();
    gtk_file_filter_set_name(all_filter, "All files (*.*)");
    gtk_file_filter_add_pattern(all_filter, "*");
    gtk_file_chooser_add_filter(GTK_FILE_CHOOSER(dialog), all_filter);

    if (gtk_dialog_run(GTK_DIALOG(dialog)) == GTK_RESPONSE_ACCEPT) {
        char *filename = gtk_file_chooser_get_filename(GTK_FILE_CHOOSER(dialog));
        if (filename) {
            if (g_file_cb) {
                g_file_cb(filename, false);
            }
            g_free(filename);
        }
    }
    gtk_widget_destroy(dialog);
}

void linux_gui_save_file_dialog(const char* default_filename) {
    if (!g_window) return;

    GtkWidget *dialog = gtk_file_chooser_dialog_new(
        "Export CPUID Dump File",
        GTK_WINDOW(g_window),
        GTK_FILE_CHOOSER_ACTION_SAVE,
        "_Cancel", GTK_RESPONSE_CANCEL,
        "_Save", GTK_RESPONSE_ACCEPT,
        NULL
    );

    gtk_file_chooser_set_do_overwrite_confirmation(GTK_FILE_CHOOSER(dialog), TRUE);
    if (default_filename && *default_filename) {
        gtk_file_chooser_set_current_name(GTK_FILE_CHOOSER(dialog), default_filename);
    }

    GtkFileFilter *filter = gtk_file_filter_new();
    gtk_file_filter_set_name(filter, "Text files (*.txt)");
    gtk_file_filter_add_pattern(filter, "*.txt");
    gtk_file_chooser_add_filter(GTK_FILE_CHOOSER(dialog), filter);

    if (gtk_dialog_run(GTK_DIALOG(dialog)) == GTK_RESPONSE_ACCEPT) {
        char *filename = gtk_file_chooser_get_filename(GTK_FILE_CHOOSER(dialog));
        if (filename) {
            if (g_file_cb) {
                g_file_cb(filename, true);
            }
            g_free(filename);
        }
    }
    gtk_widget_destroy(dialog);
}

void linux_gui_copy_clipboard(const char* text) {
    if (!text) return;
    GtkClipboard *clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
    if (clipboard) {
        gtk_clipboard_set_text(clipboard, text, -1);
    }
}

void linux_gui_show_alert(const char* title, const char* message) {
    GtkWidget *dialog = gtk_message_dialog_new(
        g_window ? GTK_WINDOW(g_window) : NULL,
        GTK_DIALOG_DESTROY_WITH_PARENT,
        GTK_MESSAGE_INFO,
        GTK_BUTTONS_OK,
        "%s", message ? message : ""
    );
    if (title && *title) {
        gtk_window_set_title(GTK_WINDOW(dialog), title);
    }
    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}

void linux_gui_run(void) {
    if (g_window) {
        gtk_widget_show_all(g_window);
        gtk_main();
    }
}
