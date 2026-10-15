#include "gui/gui_internal.h"

void mac_gui_open_file_dialog(void) {
#if !TARGET_API_MAC_CARBON && (defined(__APPLE__) || defined(__MACOS__) || defined(macintosh) || defined(__Retro68__))
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
#if !TARGET_API_MAC_CARBON && (defined(__APPLE__) || defined(__MACOS__) || defined(macintosh) || defined(__Retro68__))
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

#if TARGET_API_MAC_CARBON
typedef struct OpaqueScrapRef* ScrapRef;
int32_t ClearCurrentScrap(void);
int32_t GetCurrentScrap(ScrapRef *scrap);
int32_t PutScrapFlavor(ScrapRef scrap, uint32_t flavorType, uint32_t flavorFlags, long byteCount, const void *flavorData);
#endif

void mac_gui_copy_clipboard(const char* text) {
#if TARGET_API_MAC_CARBON
    if (!text) return;
    long len = strlen(text);
    ScrapRef scrap = NULL;
    ClearCurrentScrap();
    if (GetCurrentScrap(&scrap) == 0 && scrap) {
        PutScrapFlavor(scrap, 'TEXT', 0, len, text);
    }
#elif defined(__APPLE__) || defined(__MACOS__) || defined(macintosh) || defined(__Retro68__)
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
