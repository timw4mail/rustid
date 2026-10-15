#ifndef CLASSIC_MAC_GUI_INTERNAL_H
#define CLASSIC_MAC_GUI_INTERNAL_H

#include "gui/gui.h"
#include "common/gestalt.h"

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

#ifndef zoomDocProc
#define zoomDocProc 8
#endif

#ifndef inZoomIn
#define inZoomIn 7
#endif

#ifndef inZoomOut
#define inZoomOut 8
#endif

#ifndef scrollBarProc
#define scrollBarProc 16
#endif

#ifndef inUpButton
#define inUpButton 20
#endif

#ifndef inDownButton
#define inDownButton 21
#endif

#ifndef inPageUp
#define inPageUp 22
#endif

#ifndef inPageDown
#define inPageDown 23
#endif

#ifndef inThumb
#define inThumb 129
#endif

#else
// Stubs for non-Mac host compilation / unit tests
typedef void* WindowPtr;
typedef void* TEHandle;
typedef void* MenuHandle;
typedef void* ControlHandle;
typedef void* ControlActionUPP;
typedef struct { short top, left, bottom, right; } Rect;
typedef struct { unsigned short red, green, blue; } RGBColor;
#endif

enum {
    MENU_APPLE          = 128,
    MENU_FILE           = 129,
    MENU_EDIT           = 130,
    MENU_VIEW           = 131,

    ITEM_ABOUT          = 1,

    ITEM_QUIT           = 1,

    ITEM_COPY           = 1,

    ITEM_MODE_STD       = 1,
    ITEM_MODE_DBG       = 2,
    ITEM_MODE_ALL       = 3,
    ITEM_OPT_COLOR      = 5
};

// Internal GUI shared variables
extern CmdCallback g_cmd_cb;
extern FileCallback g_file_cb;
extern QuitCallback g_quit_cb;

extern WindowPtr g_window;
extern TEHandle g_te;
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
extern ControlHandle g_scrollbar;
extern ControlActionUPP g_scroll_action_upp;
#endif
extern bool g_is_styled_te;
extern bool g_running;

// Internal GUI helper functions
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
void InvalWindow(WindowPtr win);
void UpdateScrollbar(void);
void ResizeWindowContents(WindowPtr win);
void DoScrollLines(short deltaLines);
void HandleMenuCommand(long menuResult);
#endif

#endif // CLASSIC_MAC_GUI_INTERNAL_H
