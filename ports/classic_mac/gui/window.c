#include "gui/gui_internal.h"
#include "common/gestalt.h"

CmdCallback g_cmd_cb = 0;
FileCallback g_file_cb = 0;
QuitCallback g_quit_cb = 0;

WindowPtr g_window = 0;
TEHandle g_te = 0;
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
ControlHandle g_scrollbar = 0;
ControlActionUPP g_scroll_action_upp = 0;
#endif
bool g_is_styled_te = false;
bool g_running = false;

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

static pascal void ScrollActionProc(ControlHandle theControl, short partCode) {
    if (partCode == 0 || !g_te || !theControl) return;

    short lineH = 12;
    short viewH = (*g_te)->viewRect.bottom - (*g_te)->viewRect.top;
    short pageLines = (viewH / lineH) - 1;
    if (pageLines < 1) pageLines = 1;

    short deltaLines = 0;
    switch (partCode) {
        case inUpButton:
            deltaLines = -1;
            break;
        case inDownButton:
            deltaLines = 1;
            break;
        case inPageUp:
            deltaLines = -pageLines;
            break;
        case inPageDown:
            deltaLines = pageLines;
            break;
    }

    if (deltaLines != 0) {
        short oldVal = GetControlValue(theControl);
        SetControlValue(theControl, oldVal + deltaLines);
        short newVal = GetControlValue(theControl);
        short targetDestTop = (*g_te)->viewRect.top - (newVal * lineH);
        short currentDestTop = (*g_te)->destRect.top;
        short scrollDelta = targetDestTop - currentDestTop;
        if (scrollDelta != 0) {
            TEScroll(0, scrollDelta, g_te);
        }
    }
}

void DoScrollLines(short deltaLines) {
    if (!g_scrollbar || !g_te || deltaLines == 0) return;
    short oldVal = GetControlValue(g_scrollbar);
    SetControlValue(g_scrollbar, oldVal + deltaLines);
    short newVal = GetControlValue(g_scrollbar);
    short lineH = 12;
    short targetDestTop = (*g_te)->viewRect.top - (newVal * lineH);
    short currentDestTop = (*g_te)->destRect.top;
    short scrollDelta = targetDestTop - currentDestTop;
    if (scrollDelta != 0) {
        TEScroll(0, scrollDelta, g_te);
    }
}

void InvalWindow(WindowPtr win) {
#if TARGET_API_MAC_CARBON
    Rect r;
    GetPortBounds(GetWindowPort(win), &r);
    InvalWindowRect(win, &r);
#else
    InvalRect(&win->portRect);
#endif
}

void UpdateScrollbar(void) {
    if (!g_scrollbar || !g_te || !g_window) return;

    short lineH = 12;
    short viewH = (*g_te)->viewRect.bottom - (*g_te)->viewRect.top;
    short visLines = viewH / lineH;
    short totLines = (*g_te)->nLines;

    short maxVal = (totLines > visLines) ? (totLines - visLines) : 0;

    SetControlMinimum(g_scrollbar, 0);
    SetControlMaximum(g_scrollbar, maxVal);

    if (maxVal == 0) {
        HiliteControl(g_scrollbar, 255);
        SetControlValue(g_scrollbar, 0);
        short scrollDelta = (*g_te)->viewRect.top - (*g_te)->destRect.top;
        if (scrollDelta != 0) {
            TEScroll(0, scrollDelta, g_te);
        }
    } else {
        HiliteControl(g_scrollbar, 0);
        short curLines = ((*g_te)->viewRect.top - (*g_te)->destRect.top) / lineH;
        if (curLines < 0) curLines = 0;
        if (curLines > maxVal) curLines = maxVal;
        SetControlValue(g_scrollbar, curLines);

        short targetDestTop = (*g_te)->viewRect.top - (curLines * lineH);
        short currentDestTop = (*g_te)->destRect.top;
        short scrollDelta = targetDestTop - currentDestTop;
        if (scrollDelta != 0) {
            TEScroll(0, scrollDelta, g_te);
        }
    }
}

void ResizeWindowContents(WindowPtr win) {
    if (!win) return;
    Rect bounds;
#if TARGET_API_MAC_CARBON
    SetPort(GetWindowPort(win));
    GetPortBounds(GetWindowPort(win), &bounds);
#else
    SetPort(win);
    bounds = win->portRect;
#endif

    short width = bounds.right - bounds.left;
    short height = bounds.bottom - bounds.top;

    if (g_scrollbar) {
        HideControl(g_scrollbar);
        MoveControl(g_scrollbar, width - 15, -1);
        SizeControl(g_scrollbar, 16, (height > 14) ? (height - 13) : 1);
        ShowControl(g_scrollbar);
    }

    if (g_te) {
        Rect teRect;
        teRect.left = 10;
        teRect.top = 10;
        teRect.right = (width > 35) ? (width - 25) : 10;
        teRect.bottom = (height > 25) ? (height - 15) : 10;

        short lineH = 12;
        short curVal = g_scrollbar ? GetControlValue(g_scrollbar) : 0;

        (*g_te)->viewRect = teRect;
        (*g_te)->destRect = teRect;
        // Keep destRect wide so word wrap is disabled and fixed-column layout is preserved
        (*g_te)->destRect.right = teRect.left + 4000;
        (*g_te)->crOnly = -1;
        (*g_te)->lineHeight = 12;
        (*g_te)->fontAscent = 9;

        OffsetRect(&(*g_te)->destRect, 0, -curVal * lineH);
        TECalText(g_te);
    }

    UpdateScrollbar();
    InvalWindow(win);
}

#endif

bool mac_gui_init(const char* title, short width, short height) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#if !TARGET_API_MAC_CARBON
    InitGraf(&qd.thePort);
    InitFonts();
    InitWindows();
    InitMenus();
    TEInit();
    InitDialogs(nil);
#endif
    InitCursor();

    // Menu Bar Setup
    Handle menuBar = GetNewMBar(128);
    if (menuBar) {
        SetMenuBar(menuBar);
#if !TARGET_API_MAC_CARBON
        AppendResMenu(GetMenuHandle(MENU_APPLE), 'DRVR');
#endif
        DrawMenuBar();
    }

    // Window Setup
    Rect bounds;
#if TARGET_API_MAC_CARBON
    BitMap screenBits;
    GetQDGlobalsScreenBits(&screenBits);
    short screenW = screenBits.bounds.right - screenBits.bounds.left;
    short screenH = screenBits.bounds.bottom - screenBits.bounds.top;
#else
    short screenW = qd.screenBits.bounds.right - qd.screenBits.bounds.left;
    short screenH = qd.screenBits.bounds.bottom - qd.screenBits.bounds.top;
#endif

    if (screenW <= 512 && width > screenW - 24) width = screenW - 24;
    if (screenH <= 342 && height > screenH - 54) height = screenH - 54;

    bounds.left = (screenW - width) / 2;
    bounds.top = (screenH <= 342) ? 26 : 40;
    bounds.right = bounds.left + width;
    bounds.bottom = bounds.top + height;

    Str255 pTitle;
    size_t len = strlen(title);
    if (len > 255) len = 255;
    pTitle[0] = (unsigned char)len;
    memcpy(&pTitle[1], title, len);

    bool has_color_qd = classic_mac_has_color_qd();
    if (has_color_qd) {
        g_window = NewCWindow(nil, &bounds, pTitle, true, zoomDocProc, (WindowPtr)-1L, true, 0);
    }
    if (!g_window) {
        g_window = NewWindow(nil, &bounds, pTitle, true, zoomDocProc, (WindowPtr)-1L, true, 0);
    }
    if (!g_window) {
        g_window = NewWindow(nil, &bounds, pTitle, true, documentProc, (WindowPtr)-1L, true, 0);
    }
    if (!g_window) return false;

#if TARGET_API_MAC_CARBON
    SetPort(GetWindowPort(g_window));
#else
    SetPort(g_window);
#endif

    TextFont(monaco);
    TextSize(9);
    TextFace(normal);
    TextMode(srcOr);

    Rect teRect;
    teRect.left = 10;
    teRect.top = 10;
    teRect.right = (width > 35) ? (width - 25) : 10;
    teRect.bottom = (height > 25) ? (height - 15) : 10;

    Rect destRect = teRect;
    destRect.right = teRect.left + 4000;

    g_is_styled_te = classic_mac_has_styled_te();
    if (g_is_styled_te) {
        g_te = TEStyleNew(&destRect, &teRect);
    }
    if (!g_te) {
        g_te = TENew(&destRect, &teRect);
        g_is_styled_te = false;
    }
    if (g_te) {
        (*g_te)->crOnly = -1;
        (*g_te)->lineHeight = 12;
        (*g_te)->fontAscent = 9;
    }

    Rect sRect;
    sRect.top = -1;
    sRect.left = width - 15;
    sRect.bottom = (height > 14) ? (height - 13) : 1;
    sRect.right = width + 1;

    g_scrollbar = NewControl(g_window, &sRect, (ConstStringPtr)"\p", true, 0, 0, 0, scrollBarProc, 0);
    g_scroll_action_upp = NewControlActionUPP(ScrollActionProc);
    UpdateScrollbar();

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

#if TARGET_API_MAC_CARBON
    SetPort(GetWindowPort(g_window));
#else
    SetPort(g_window);
#endif

    TextFont(monaco);
    TextSize(9);
    TextFace(normal);
    TextMode(srcOr);

    // 1. Reset scroll offset back to top before replacing text
    short scrollDelta = (*g_te)->viewRect.top - (*g_te)->destRect.top;
    if (scrollDelta != 0) {
        TEScroll(0, scrollDelta, g_te);
    }
    (*g_te)->destRect.top = (*g_te)->viewRect.top;
    (*g_te)->destRect.bottom = (*g_te)->viewRect.bottom;
    (*g_te)->destRect.left = (*g_te)->viewRect.left;
    (*g_te)->destRect.right = (*g_te)->viewRect.left + 4000;
    (*g_te)->crOnly = -1;
    (*g_te)->lineHeight = 12;
    (*g_te)->fontAscent = 9;

    // 2. Classic Macintosh TextEdit only recognizes carriage returns ('\r' / 0x0D)
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

    // 3. Apply style and color runs if Styled TextEdit is supported
    if (g_is_styled_te) {
        TESetSelect(0, length, g_te);
        TextStyle baseStyle;
        baseStyle.tsFont = monaco;
        baseStyle.tsSize = 9;
        baseStyle.tsFace = normal;
        baseStyle.tsColor.red = 0;
        baseStyle.tsColor.green = 0;
        baseStyle.tsColor.blue = 0;
        TESetStyle(doFont | doSize | doFace | doColor, &baseStyle, false, g_te);

        // 4. Apply color/bold runs
        if (run_count > 0) {
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
    }

    // 5. Finalize layout without word wrap
    TESetSelect(0, 0, g_te);
    (*g_te)->crOnly = -1;
    (*g_te)->destRect.right = (*g_te)->viewRect.left + 4000;
    (*g_te)->lineHeight = 12;
    (*g_te)->fontAscent = 9;
    TECalText(g_te);

    // 6. Reset scrollbar and update range
    if (g_scrollbar) {
        SetControlValue(g_scrollbar, 0);
    }
    UpdateScrollbar();
    InvalWindow(g_window);
#else
    (void)text; (void)length; (void)runs; (void)run_count; (void)bg_color;
#endif
}

void mac_gui_set_status(const char* part1, const char* part2, const char* part3) {
    (void)part1; (void)part2; (void)part3;
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
#if !TARGET_API_MAC_CARBON
                        case inSysWindow:
                            SystemClick(&event, whichWindow);
                            break;
#endif
                        case inDrag: {
                            Rect dragBounds;
#if TARGET_API_MAC_CARBON
                            BitMap sb;
                            GetQDGlobalsScreenBits(&sb);
                            dragBounds = sb.bounds;
#else
                            dragBounds = qd.screenBits.bounds;
#endif
                            DragWindow(whichWindow, event.where, &dragBounds);
                            break;
                        }
                        case inGrow: {
                            Rect sizeLimits;
                            sizeLimits.top = 200;      // Minimum height
                            sizeLimits.left = 380;     // Minimum width
#if TARGET_API_MAC_CARBON
                            BitMap sb;
                            GetQDGlobalsScreenBits(&sb);
                            sizeLimits.bottom = sb.bounds.bottom - sb.bounds.top;
                            sizeLimits.right = sb.bounds.right - sb.bounds.left;
#else
                            sizeLimits.bottom = qd.screenBits.bounds.bottom - qd.screenBits.bounds.top;
                            sizeLimits.right = qd.screenBits.bounds.right - qd.screenBits.bounds.left;
#endif
                            long newSize = GrowWindow(whichWindow, event.where, &sizeLimits);
                            if (newSize != 0) {
                                short newWidth = LoWord(newSize);
                                short newHeight = HiWord(newSize);
                                SizeWindow(whichWindow, newWidth, newHeight, true);
                                ResizeWindowContents(whichWindow);
                            }
                            break;
                        }
                        case inZoomIn:
                        case inZoomOut:
                            if (TrackBox(whichWindow, event.where, part)) {
                                ZoomWindow(whichWindow, part, (whichWindow == FrontWindow()));
                                ResizeWindowContents(whichWindow);
                            }
                            break;
                        case inContent: {
                            if (whichWindow != FrontWindow()) {
                                SelectWindow(whichWindow);
                            } else {
                                Point localPt = event.where;
                                GlobalToLocal(&localPt);
                                ControlHandle whichControl = nil;
                                short controlPart = FindControl(localPt, whichWindow, &whichControl);
                                if (controlPart != 0 && whichControl == g_scrollbar) {
                                    if (controlPart == inThumb) {
                                        short part = TrackControl(g_scrollbar, localPt, nil);
                                        if (part != 0 && g_te) {
                                            short newVal = GetControlValue(g_scrollbar);
                                            short lineH = 12;
                                            short targetDestTop = (*g_te)->viewRect.top - (newVal * lineH);
                                            short currentDestTop = (*g_te)->destRect.top;
                                            short scrollDelta = targetDestTop - currentDestTop;
                                            if (scrollDelta != 0) {
                                                TEScroll(0, scrollDelta, g_te);
                                            }
                                        }
                                    } else {
                                        TrackControl(g_scrollbar, localPt, g_scroll_action_upp);
                                    }
                                } else if (g_te) {
                                    TEClick(localPt, (event.modifiers & shiftKey) != 0, g_te);
                                }
                            }
                            break;
                        }
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
                    } else if (g_te) {
                        short lineH = 12;
                        if (key == 0x1E) { // Up Arrow
                            DoScrollLines(-1);
                        } else if (key == 0x1F) { // Down Arrow
                            DoScrollLines(1);
                        } else if (key == 0x0B) { // Page Up
                            short viewH = (*g_te)->viewRect.bottom - (*g_te)->viewRect.top;
                            short pageLines = (viewH / lineH) - 1;
                            if (pageLines < 1) pageLines = 1;
                            DoScrollLines(-pageLines);
                        } else if (key == 0x0C) { // Page Down
                            short viewH = (*g_te)->viewRect.bottom - (*g_te)->viewRect.top;
                            short pageLines = (viewH / lineH) - 1;
                            if (pageLines < 1) pageLines = 1;
                            DoScrollLines(pageLines);
                        }
                    }
                    break;
                }
                case updateEvt: {
                    WindowPtr updateWin = (WindowPtr)event.message;
                    BeginUpdate(updateWin);
                    Rect portRect;
#if TARGET_API_MAC_CARBON
                    SetPort(GetWindowPort(updateWin));
                    GetPortBounds(GetWindowPort(updateWin), &portRect);
#else
                    SetPort(updateWin);
                    portRect = updateWin->portRect;
#endif
                    EraseRect(&portRect);
                    TextFont(monaco);
                    TextSize(9);
                    TextFace(normal);
                    TextMode(srcOr);
                    if (g_te) TEUpdate(&portRect, g_te);
                    DrawControls(updateWin);
                    DrawGrowIcon(updateWin);
                    EndUpdate(updateWin);
                    break;
                }
            }
        }
    }

    if (g_quit_cb) g_quit_cb();
#endif
}
