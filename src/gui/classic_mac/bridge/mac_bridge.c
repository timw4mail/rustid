//! Classic Macintosh C GUI Bridge for rustid.
//! Targets Classic Mac OS (System 6 through 9) using the Macintosh Toolbox.

#include "mac_bridge.h"

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

#include <Types.h>
#include <Quickdraw.h>
#include <Fonts.h>
#include <Events.h>
#include <Windows.h>
#include <Menus.h>
#include <TextEdit.h>
#include <Dialogs.h>
#include <Scrap.h>
#include <StandardFile.h>
#include <Gestalt.h>
#include <ToolUtils.h>
#include <Memory.h>
#include <OSUtils.h>

#else
// Stubs for non-Mac host compilation / unit tests
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void* WindowPtr;
typedef void* TEHandle;
typedef void* MenuHandle;
typedef struct { short top, left, bottom, right; } Rect;
typedef struct { unsigned short red, green, blue; } RGBColor;
#endif

enum {
    MENU_APPLE          = 128,
    MENU_FILE           = 129,
    MENU_EDIT           = 130,
    MENU_VIEW           = 131,
    MENU_HELP           = 132,

    ITEM_ABOUT          = 1,

    ITEM_OPEN           = 1,
    ITEM_EXPORT         = 2,
    ITEM_REFRESH        = 4,
    ITEM_QUIT           = 6,

    ITEM_COPY           = 4,

    ITEM_MODE_STD       = 1,
    ITEM_MODE_DBG       = 2,
    ITEM_MODE_ALL       = 3,
    ITEM_MODE_DUMP      = 4,
    ITEM_OPT_COLOR      = 6,
    ITEM_OPT_DARK       = 7,
    ITEM_OPT_VERBOSE    = 8,
    ITEM_OPT_COMPACT    = 9,

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

static CmdCallback g_cmd_cb = 0;
static FileCallback g_file_cb = 0;
static QuitCallback g_quit_cb = 0;

static WindowPtr g_window = 0;
static TEHandle g_te = 0;
static bool g_running = false;
static bool g_dark_theme = false;
static char g_status_part1[128] = {0};
static char g_status_part2[128] = {0};
static char g_status_part3[128] = {0};

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

static void DrawStatusBar(WindowPtr win) {
    Rect bounds = win->portRect;
    Rect statusRect = bounds;
    statusRect.top = statusRect.bottom - 20;

    RGBColor bg, borderDark, borderLight, textColor;
    if (g_dark_theme) {
        bg.red = 0x1A1A; bg.green = 0x1B1B; bg.blue = 0x2626;
        borderDark.red = 0x0F0F; borderDark.green = 0x1010; borderDark.blue = 0x1616;
        borderLight.red = 0x3232; borderLight.green = 0x3434; borderLight.blue = 0x4646;
        textColor.red = 0xD4D4; textColor.green = 0xD4D4; textColor.blue = 0xD4D4;
    } else {
        bg.red = 0xDDDD; bg.green = 0xDDDD; bg.blue = 0xDDDD;
        borderDark.red = 0x8888; borderDark.green = 0x8888; borderDark.blue = 0x8888;
        borderLight.red = 0xFFFF; borderLight.green = 0xFFFF; borderLight.blue = 0xFFFF;
        textColor.red = 0x0000; textColor.green = 0x0000; textColor.blue = 0x0000;
    }

    RGBForeColor(&bg);
    PaintRect(&statusRect);

    // Separator line
    MoveTo(statusRect.left, statusRect.top);
    RGBForeColor(&borderDark);
    LineTo(statusRect.right, statusRect.top);

    RGBForeColor(&textColor);
    TextFont(monaco);
    TextSize(9);

    // Status text parts
    MoveTo(statusRect.left + 5, statusRect.bottom - 5);
    DrawText(g_status_part1, 0, strlen(g_status_part1));

    MoveTo(statusRect.left + 260, statusRect.bottom - 5);
    DrawText(g_status_part2, 0, strlen(g_status_part2));

    MoveTo(statusRect.left + 420, statusRect.bottom - 5);
    DrawText(g_status_part3, 0, strlen(g_status_part3));
}

static void HandleMenuCommand(long menuResult) {
    short menuID = HiWord(menuResult);
    short menuItem = LoWord(menuResult);

    if (menuID == 0) return;

    switch (menuID) {
        case MENU_APPLE:
            if (menuItem == ITEM_ABOUT) {
                if (g_cmd_cb) g_cmd_cb(CMD_HELP_ABOUT);
            } else {
                Str255 deskName;
                GetMenuItemText(GetMenuHandle(MENU_APPLE), menuItem, deskName);
                OpenDeskAcc(deskName);
            }
            break;

        case MENU_FILE:
            switch (menuItem) {
                case ITEM_OPEN:
                    if (g_cmd_cb) g_cmd_cb(CMD_FILE_OPEN);
                    break;
                case ITEM_EXPORT:
                    if (g_cmd_cb) g_cmd_cb(CMD_FILE_EXPORT);
                    break;
                case ITEM_REFRESH:
                    if (g_cmd_cb) g_cmd_cb(CMD_FILE_REFRESH);
                    break;
                case ITEM_QUIT:
                    if (g_cmd_cb) g_cmd_cb(CMD_FILE_EXIT);
                    g_running = false;
                    break;
            }
            break;

        case MENU_EDIT:
            if (menuItem == ITEM_COPY) {
                if (g_cmd_cb) g_cmd_cb(CMD_FILE_COPY);
            }
            break;

        case MENU_VIEW:
            switch (menuItem) {
                case ITEM_MODE_STD:
                    if (g_cmd_cb) g_cmd_cb(CMD_MODE_STANDARD);
                    break;
                case ITEM_MODE_DBG:
                    if (g_cmd_cb) g_cmd_cb(CMD_MODE_DEBUG);
                    break;
                case ITEM_MODE_ALL:
                    if (g_cmd_cb) g_cmd_cb(CMD_MODE_EVERYTHING);
                    break;
                case ITEM_MODE_DUMP:
                    if (g_cmd_cb) g_cmd_cb(CMD_MODE_DUMP);
                    break;
                case ITEM_OPT_COLOR:
                    if (g_cmd_cb) g_cmd_cb(CMD_OPT_COLOR);
                    break;
                case ITEM_OPT_DARK:
                    if (g_cmd_cb) g_cmd_cb(CMD_OPT_DARK_THEME);
                    break;
                case ITEM_OPT_VERBOSE:
                    if (g_cmd_cb) g_cmd_cb(CMD_OPT_VERBOSE);
                    break;
                case ITEM_OPT_COMPACT:
                    if (g_cmd_cb) g_cmd_cb(CMD_OPT_COMPACT);
                    break;
            }
            break;
    }

    HiliteMenu(0);
}

#endif

bool mac_gui_init(const char* title, short width, short height) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    InitGraf(&qd.thePort);
    InitFonts();
    InitWindows();
    InitMenus();
    TEInit();
    InitDialogs(nil);
    InitCursor();

    // Menu Bar Setup
    Handle menuBar = GetNewMBar(128);
    if (menuBar) {
        SetMenuBar(menuBar);
        AppendResMenu(GetMenuHandle(MENU_APPLE), 'DRVR');
        DrawMenuBar();
    }

    // Window Setup
    Rect bounds;
    short screenW = qd.screenBits.bounds.right - qd.screenBits.bounds.left;
    short screenH = qd.screenBits.bounds.bottom - qd.screenBits.bounds.top;

    bounds.left = (screenW - width) / 2;
    bounds.top = (screenH - height) / 2;
    if (bounds.top < 40) bounds.top = 40;
    bounds.right = bounds.left + width;
    bounds.bottom = bounds.top + height;

    Str255 pTitle;
    size_t len = strlen(title);
    if (len > 255) len = 255;
    pTitle[0] = (unsigned char)len;
    memcpy(&pTitle[1], title, len);

    g_window = NewCWindow(nil, &bounds, pTitle, true, documentProc, (WindowPtr)-1L, true, 0);
    if (!g_window) {
        g_window = NewWindow(nil, &bounds, pTitle, true, documentProc, (WindowPtr)-1L, true, 0);
    }
    if (!g_window) return false;

    SetPort(g_window);

    Rect teRect = bounds;
    teRect.left = 10;
    teRect.top = 10;
    teRect.right = width - 10;
    teRect.bottom = height - 30;

    g_te = TEStyleNew(&teRect, &teRect);
    if (!g_te) {
        g_te = TENew(&teRect, &teRect);
    }

    return true;
#else
    (void)title; (void)width; (void)height;
    return true;
#endif
}

void mac_gui_set_callbacks(CmdCallback on_cmd, FileCallback on_file, QuitCallback on_quit) {
    g_cmd_cb = on_cmd;
    g_file_cb = on_file;
    g_quit_cb = on_quit;
}

void mac_gui_set_text(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg_color) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    if (!g_te || !g_window) return;

    SetPort(g_window);
    TESetText(text, length, g_te);

    // Apply color/bold runs
    for (uint32_t i = 0; i < run_count; i++) {
        const CTextRun* r = &runs[i];
        TESetSelect(r->offset, r->offset + r->length, g_te);

        TextStyle style;
        style.tsFont = monaco;
        style.tsSize = 9;
        style.tsFace = r->bold ? bold : normal;
        style.tsColor.red = ((unsigned short)r->color.r) << 8;
        style.tsColor.green = ((unsigned short)r->color.g) << 8;
        style.tsColor.blue = ((unsigned short)r->color.b) << 8;

        TESetStyle(doFont | doSize | doFace | doColor, &style, false, g_te);
    }

    TESetSelect(0, 0, g_te);
    InvalRect(&g_window->portRect);
#else
    (void)text; (void)length; (void)runs; (void)run_count; (void)bg_color;
#endif
}

void mac_gui_set_status(const char* part1, const char* part2, const char* part3) {
    if (part1) strncpy(g_status_part1, part1, sizeof(g_status_part1) - 1);
    if (part2) strncpy(g_status_part2, part2, sizeof(g_status_part2) - 1);
    if (part3) strncpy(g_status_part3, part3, sizeof(g_status_part3) - 1);

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    if (g_window) InvalRect(&g_window->portRect);
#endif
}

void mac_gui_set_menu_checks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact) {
    g_dark_theme = dark_theme;

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    MenuHandle hView = GetMenuHandle(MENU_VIEW);
    if (!hView) return;

    CheckItem(hView, ITEM_MODE_STD, mode_cmd_id == CMD_MODE_STANDARD);
    CheckItem(hView, ITEM_MODE_DBG, mode_cmd_id == CMD_MODE_DEBUG);
    CheckItem(hView, ITEM_MODE_ALL, mode_cmd_id == CMD_MODE_EVERYTHING);
    CheckItem(hView, ITEM_MODE_DUMP, mode_cmd_id == CMD_MODE_DUMP);

    CheckItem(hView, ITEM_OPT_COLOR, color);
    CheckItem(hView, ITEM_OPT_DARK, dark_theme);
    CheckItem(hView, ITEM_OPT_VERBOSE, verbose);
    CheckItem(hView, ITEM_OPT_COMPACT, compact);
#else
    (void)mode_cmd_id; (void)color; (void)verbose; (void)compact;
#endif
}

void mac_gui_open_file_dialog(void) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    SFTypeList types = {'TEXT', 'ttxt', 0, 0};
    StandardFileReply reply;
    StandardGetFile(nil, 2, types, &reply);
    if (reply.sfGood && g_file_cb) {
        char path[256] = {0};
        memcpy(path, &reply.sfFile.name[1], reply.sfFile.name[0]);
        g_file_cb(path, false);
    }
#endif
}

void mac_gui_save_file_dialog(const char* default_filename) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    Str255 pName;
    size_t len = default_filename ? strlen(default_filename) : 0;
    if (len > 255) len = 255;
    pName[0] = (unsigned char)len;
    if (len > 0) memcpy(&pName[1], default_filename, len);

    StandardFileReply reply;
    StandardPutFile("\pSave CPU Report As:", pName, &reply);
    if (reply.sfGood && g_file_cb) {
        char path[256] = {0};
        memcpy(path, &reply.sfFile.name[1], reply.sfFile.name[0]);
        g_file_cb(path, true);
    }
#else
    (void)default_filename;
#endif
}

void mac_gui_copy_clipboard(const char* text) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    if (!text) return;
    long len = strlen(text);
    ZeroScrap();
    PutScrap(len, 'TEXT', text);
#else
    (void)text;
#endif
}

void mac_gui_show_alert(const char* title, const char* message) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    Str255 pTitle, pMsg;
    size_t tLen = strlen(title); if (tLen > 255) tLen = 255;
    pTitle[0] = (unsigned char)tLen; memcpy(&pTitle[1], title, tLen);

    size_t mLen = strlen(message); if (mLen > 255) mLen = 255;
    pMsg[0] = (unsigned char)mLen; memcpy(&pMsg[1], message, mLen);

    ParamText(pTitle, pMsg, "\p", "\p");
    Alert(128, nil);
#else
    (void)title; (void)message;
#endif
}

void mac_gui_run(void) {
    g_running = true;

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    EventRecord event;
    while (g_running) {
        if (WaitNextEvent(everyEvent, &event, 6, nil)) {
            switch (event.what) {
                case mouseDown: {
                    WindowPtr whichWindow;
                    short part = FindWindow(event.where, &whichWindow);
                    switch (part) {
                        case inMenuBar:
                            HandleMenuCommand(MenuSelect(event.where));
                            break;
                        case inSysWindow:
                            SystemClick(&event, whichWindow);
                            break;
                        case inDrag:
                            DragWindow(whichWindow, event.where, &qd.screenBits.bounds);
                            break;
                        case inGoAway:
                            if (TrackGoAway(whichWindow, event.where)) {
                                g_running = false;
                            }
                            break;
                    }
                    break;
                }
                case keyDown:
                case autoKey: {
                    char key = event.message & charCodeMask;
                    if (event.modifiers & cmdKey) {
                        HandleMenuCommand(MenuKey(key));
                    }
                    break;
                }
                case updateEvt: {
                    WindowPtr updateWin = (WindowPtr)event.message;
                    BeginUpdate(updateWin);
                    SetPort(updateWin);
                    EraseRect(&updateWin->portRect);
                    if (g_te) TEUpdate(&updateWin->portRect, g_te);
                    DrawStatusBar(updateWin);
                    EndUpdate(updateWin);
                    break;
                }
            }
        }
    }

    if (g_quit_cb) g_quit_cb();
#endif
}
