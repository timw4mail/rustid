//! Classic Macintosh C GUI Bridge for rustid.
//! Targets Classic Mac OS (System 6 through 9) using the Macintosh Toolbox.

#include "mac_bridge.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

#include <Types.h>
#include <Quickdraw.h>
#include <Fonts.h>
#include <Events.h>
#include <Windows.h>
#include <Menus.h>
#include <TextEdit.h>
#include <Dialogs.h>
#if __has_include(<Scrap.h>)
#include <Scrap.h>
#endif
#include <StandardFile.h>
#include <Gestalt.h>
#include <ToolUtils.h>
#include <Memory.h>
#include <OSUtils.h>

#ifndef monaco
#ifdef kFontIDMonaco
#define monaco kFontIDMonaco
#else
#define monaco 4
#endif
#endif

#ifdef HiWord
#undef HiWord
#endif
#define HiWord(a) ((short)(((uint32_t)(a) >> 16) & 0xFFFF))

#ifdef LoWord
#undef LoWord
#endif
#define LoWord(a) ((short)((uint32_t)(a) & 0xFFFF))

#else
// Stubs for non-Mac host compilation / unit tests

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

    ITEM_ABOUT          = 1,

    ITEM_REFRESH        = 1,
    ITEM_QUIT           = 3,

    ITEM_COPY           = 4,

    ITEM_MODE_STD       = 1,
    ITEM_MODE_DBG       = 2,
    ITEM_MODE_ALL       = 3,
    ITEM_OPT_COLOR      = 5
};

static CmdCallback g_cmd_cb = 0;
static FileCallback g_file_cb = 0;
static QuitCallback g_quit_cb = 0;

static WindowPtr g_window = 0;
static TEHandle g_te = 0;
static bool g_running = false;
static char g_status_part1[128] = {0};
static char g_status_part2[128] = {0};
static char g_status_part3[128] = {0};

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

static void DrawStatusBar(WindowPtr win) {
    Rect bounds = win->portRect;
    Rect statusRect = bounds;
    statusRect.top = statusRect.bottom - 20;

    RGBColor bg, borderDark, textColor;
    bg.red = 0xDDDD; bg.green = 0xDDDD; bg.blue = 0xDDDD;
    borderDark.red = 0x8888; borderDark.green = 0x8888; borderDark.blue = 0x8888;
    textColor.red = 0x0000; textColor.green = 0x0000; textColor.blue = 0x0000;

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
                case ITEM_OPT_COLOR:
                    if (g_cmd_cb) g_cmd_cb(CMD_OPT_COLOR);
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
    if (!g_te || !g_window || !text) return;

    SetPort(g_window);

    // Classic Macintosh TextEdit only recognizes carriage returns ('\r' / 0x0D)
    // as line breaks. Line feeds ('\n' / 0x0A) are ignored or drawn as glyphs.
    // Replace '\n' with '\r' (1:1 byte substitution so run offsets remain aligned).
    char* mac_text = (char*)malloc(length + 1);
    if (mac_text) {
        for (uint32_t i = 0; i < length; i++) {
            mac_text[i] = (text[i] == '\n') ? '\r' : text[i];
        }
        mac_text[length] = '\0';
        TESetText((Ptr)mac_text, length, g_te);
        free(mac_text);
    } else {
        TESetText((Ptr)text, length, g_te);
    }

    // Apply color/bold runs or reset to plain text if color is disabled
    if (run_count == 0) {
        TESetSelect(0, length, g_te);
        TextStyle style;
        style.tsFont = monaco;
        style.tsSize = 9;
        style.tsFace = normal;
        style.tsColor.red = 0;
        style.tsColor.green = 0;
        style.tsColor.blue = 0;
        TESetStyle(doFont | doSize | doFace | doColor, &style, false, g_te);
    } else {
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

void mac_gui_set_menu_checks(uint32_t mode_cmd_id, bool color) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    MenuHandle hView = GetMenuHandle(MENU_VIEW);
    if (!hView) return;

    CheckItem(hView, ITEM_MODE_STD, mode_cmd_id == CMD_MODE_STANDARD);
    CheckItem(hView, ITEM_MODE_DBG, mode_cmd_id == CMD_MODE_DEBUG);
    CheckItem(hView, ITEM_MODE_ALL, mode_cmd_id == CMD_MODE_EVERYTHING);
    CheckItem(hView, ITEM_OPT_COLOR, color);
#else
    (void)mode_cmd_id; (void)color;
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
    StandardPutFile((ConstStringPtr)"\pSave CPU Report As:", pName, &reply);
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
    PutScrap(len, 'TEXT', (Ptr)text);
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
    for (size_t i = 1; i <= mLen; i++) {
        if (pMsg[i] == '\n') pMsg[i] = '\r';
    }

    ParamText(pTitle, pMsg, (ConstStringPtr)"\p", (ConstStringPtr)"\p");
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
