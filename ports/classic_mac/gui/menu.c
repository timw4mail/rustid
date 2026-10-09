#include "gui/gui_internal.h"
#include "common/gestalt.h"

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)

void HandleMenuCommand(long menuResult) {
    short menuID = HiWord(menuResult);
    short menuItem = LoWord(menuResult);

    if (menuID == 0) return;

    switch (menuID) {
        case MENU_APPLE:
            if (menuItem == ITEM_ABOUT) {
                if (g_cmd_cb) g_cmd_cb(CMD_HELP_ABOUT);
            }
#if !TARGET_API_MAC_CARBON
            else {
                Str255 deskName;
                GetMenuItemText(GetMenuHandle(MENU_APPLE), menuItem, deskName);
                OpenDeskAcc(deskName);
            }
#endif
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

void mac_gui_set_menu_checks(uint32_t mode_cmd_id, bool color) {
#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
    MenuHandle hView = GetMenuHandle(MENU_VIEW);
    if (!hView) return;

#if TARGET_API_MAC_CARBON
    CheckMenuItem(hView, ITEM_MODE_STD, mode_cmd_id == CMD_MODE_STANDARD);
    CheckMenuItem(hView, ITEM_MODE_DBG, mode_cmd_id == CMD_MODE_DEBUG);
    CheckMenuItem(hView, ITEM_MODE_ALL, mode_cmd_id == CMD_MODE_EVERYTHING);
    CheckMenuItem(hView, ITEM_OPT_COLOR, color);
#else
    CheckItem(hView, ITEM_MODE_STD, mode_cmd_id == CMD_MODE_STANDARD);
    CheckItem(hView, ITEM_MODE_DBG, mode_cmd_id == CMD_MODE_DEBUG);
    CheckItem(hView, ITEM_MODE_ALL, mode_cmd_id == CMD_MODE_EVERYTHING);
    CheckItem(hView, ITEM_OPT_COLOR, color);
    if (!classic_mac_has_color_qd() || !g_is_styled_te) {
        DisableItem(hView, ITEM_OPT_COLOR);
    }
#endif
#else
    (void)mode_cmd_id; (void)color;
#endif
}
