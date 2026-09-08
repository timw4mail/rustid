//! Native Haiku C++ GUI implementation for rustid.

#include "haiku_bridge.h"

#include <Application.h>
#include <Window.h>
#include <View.h>
#include <MenuBar.h>
#include <Menu.h>
#include <MenuItem.h>
#include <TextView.h>
#include <ScrollView.h>
#include <FilePanel.h>
#include <Clipboard.h>
#include <Alert.h>
#include <Screen.h>
#include <Path.h>
#include <Entry.h>
#include <Font.h>
#include <String.h>
#include <cstdlib>
#include <cstring>
#include <vector>

static CmdCallback g_cmd_cb = nullptr;
static FileCallback g_file_cb = nullptr;
static QuitCallback g_quit_cb = nullptr;

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

class RustidStatusBar : public BView {
public:
    RustidStatusBar(BRect frame)
        : BView(frame, "status_bar", B_FOLLOW_LEFT_RIGHT | B_FOLLOW_BOTTOM, B_WILL_DRAW | B_FULL_UPDATE_ON_RESIZE),
          fDark(false)
    {
        SetViewColor(B_TRANSPARENT_COLOR);
    }

    void SetTheme(bool dark) {
        fDark = dark;
        Invalidate();
    }

    void SetParts(const char* p1, const char* p2, const char* p3) {
        fPart1 = p1 ? p1 : "";
        fPart2 = p2 ? p2 : "";
        fPart3 = p3 ? p3 : "";
        Invalidate();
    }

    virtual void Draw(BRect updateRect) override {
        BRect bounds = Bounds();

        rgb_color bg = fDark ? (rgb_color){26, 27, 38, 255} : ui_color(B_PANEL_BACKGROUND_COLOR);
        rgb_color borderLight = fDark ? (rgb_color){50, 52, 70, 255} : tint_color(bg, B_LIGHTEN_2_TINT);
        rgb_color borderDark = fDark ? (rgb_color){15, 16, 22, 255} : tint_color(bg, B_DARKEN_2_TINT);
        rgb_color textColor = fDark ? (rgb_color){212, 212, 212, 255} : ui_color(B_PANEL_TEXT_COLOR);

        SetLowColor(bg);
        FillRect(bounds, B_SOLID_LOW);

        // Top edge separator line
        SetHighColor(borderDark);
        StrokeLine(BPoint(bounds.left, bounds.top), BPoint(bounds.right, bounds.top));
        SetHighColor(borderLight);
        StrokeLine(BPoint(bounds.left, bounds.top + 1), BPoint(bounds.right, bounds.top + 1));

        float cellY = bounds.top + 3;
        float cellH = bounds.Height() - 4;

        float p1W = 320.0f;
        float p2W = 200.0f;

        BRect r1(bounds.left + 2, cellY, bounds.left + 2 + p1W, cellY + cellH);
        BRect r2(r1.right + 4, cellY, r1.right + 4 + p2W, cellY + cellH);
        BRect r3(r2.right + 4, cellY, bounds.right - 2, cellY + cellH);

        DrawCell(r1, fPart1.String(), borderDark, borderLight, textColor);
        DrawCell(r2, fPart2.String(), borderDark, borderLight, textColor);
        DrawCell(r3, fPart3.String(), borderDark, borderLight, textColor);
    }

private:
    void DrawCell(BRect r, const char* text, rgb_color dark, rgb_color light, rgb_color textCol) {
        if (r.left >= r.right) return;

        // Bevel border
        SetHighColor(dark);
        StrokeLine(BPoint(r.left, r.bottom), BPoint(r.left, r.top));
        StrokeLine(BPoint(r.left, r.top), BPoint(r.right, r.top));
        SetHighColor(light);
        StrokeLine(BPoint(r.right, r.top + 1), BPoint(r.right, r.bottom));
        StrokeLine(BPoint(r.left + 1, r.bottom), BPoint(r.right, r.bottom));

        // Text
        SetHighColor(textCol);
        BFont font(be_plain_font);
        font_height fh;
        font.GetHeight(&fh);
        SetFont(&font);

        float textY = r.top + (r.Height() - (fh.ascent + fh.descent)) / 2.0f + fh.ascent;
        BString truncated(text);
        font.TruncateString(&truncated, B_TRUNCATE_END, r.Width() - 10.0f);
        DrawString(truncated.String(), BPoint(r.left + 5.0f, textY));
    }

    BString fPart1;
    BString fPart2;
    BString fPart3;
    bool fDark;
};

class RustidWindow : public BWindow {
public:
    RustidWindow(BRect frame, const char* title, float min_w, float min_h)
        : BWindow(frame, title, B_TITLED_WINDOW, B_ASYNCHRONOUS_CONTROLS | B_QUIT_ON_WINDOW_CLOSE),
          fOpenPanel(nullptr),
          fSavePanel(nullptr),
          fDarkTheme(false)
    {
        SetSizeLimits(min_w, 30000.0f, min_h, 30000.0f);

        BRect bounds = Bounds();

        // Menu Bar
        fMenuBar = new BMenuBar(BRect(0, 0, bounds.Width(), 20), "menubar");

        // File Menu
        BMenu* fileMenu = new BMenu("File");
#if defined(__x86_64__) || defined(__i386__)
        fileMenu->AddItem(new BMenuItem("Open Dump…", new BMessage(CMD_FILE_OPEN), 'O'));
        fileMenu->AddItem(new BMenuItem("Export CPUID Dump…", new BMessage(CMD_FILE_EXPORT), 'S'));
        fileMenu->AddSeparatorItem();
#endif
        fileMenu->AddItem(new BMenuItem("Copy All Text", new BMessage(CMD_FILE_COPY), 'C'));
        fileMenu->AddItem(new BMenuItem("Refresh Hardware", new BMessage(CMD_FILE_REFRESH), 'R'));
        fileMenu->AddSeparatorItem();
        fileMenu->AddItem(new BMenuItem("Quit", new BMessage(CMD_FILE_EXIT), 'Q'));
        fMenuBar->AddItem(fileMenu);

        // Mode Menu
        BMenu* modeMenu = new BMenu("Mode");
        fItemStandard = new BMenuItem("Standard Summary", new BMessage(CMD_MODE_STANDARD), '1');
        fItemDebug = new BMenuItem("Debug Information (-d)", new BMessage(CMD_MODE_DEBUG), '2');
        fItemEverything = new BMenuItem("Everything (-e)", new BMessage(CMD_MODE_EVERYTHING), '3');
        modeMenu->AddItem(fItemStandard);
        modeMenu->AddItem(fItemDebug);
        modeMenu->AddItem(fItemEverything);
#if defined(__x86_64__) || defined(__i386__)
        fItemDump = new BMenuItem("Raw CPUID Dump (-r)", new BMessage(CMD_MODE_DUMP), '4');
        modeMenu->AddItem(fItemDump);
#else
        fItemDump = nullptr;
#endif
        modeMenu->AddSeparatorItem();

        fItemColor = new BMenuItem("Colorized Output", new BMessage(CMD_OPT_COLOR));
        fItemDarkTheme = new BMenuItem("Dark Theme", new BMessage(CMD_OPT_DARK_THEME));
        modeMenu->AddItem(fItemColor);
        modeMenu->AddItem(fItemDarkTheme);
        modeMenu->AddSeparatorItem();

        fItemVerbose = new BMenuItem("Verbose Mode (-v)", new BMessage(CMD_OPT_VERBOSE));
        fItemCompact = new BMenuItem("Compact Mode (-c)", new BMessage(CMD_OPT_COMPACT));
        modeMenu->AddItem(fItemVerbose);
        modeMenu->AddItem(fItemCompact);
        fMenuBar->AddItem(modeMenu);

        // Help Menu
        BMenu* helpMenu = new BMenu("Help");
        helpMenu->AddItem(new BMenuItem("About Rustid", new BMessage(CMD_HELP_ABOUT)));
        fMenuBar->AddItem(helpMenu);

        AddChild(fMenuBar);

        float menuHeight = fMenuBar->Bounds().Height();
        float statusHeight = 24.0f;
        float margin = 6.0f;

        // Text view and scroll view
        BRect textFrame(margin, menuHeight + margin, bounds.Width() - margin - B_V_SCROLL_BAR_WIDTH, bounds.Height() - statusHeight - margin);
        BRect textBounds(0, 0, textFrame.Width(), textFrame.Height());

        fTextView = new BTextView(textFrame, "report_text", textBounds, B_FOLLOW_ALL, B_WILL_DRAW | B_FRAME_EVENTS | B_NAVIGABLE);
        fTextView->SetStylable(true);
        fTextView->MakeEditable(false);
        fTextView->MakeSelectable(true);
        fTextView->SetWordWrap(false);
        fTextView->SetInsets(10.0f, 10.0f, 10.0f, 10.0f);

        BFont monoFont(be_fixed_font);
        monoFont.SetSize(12.0f);
        fTextView->SetFontAndColor(&monoFont);

        fScrollView = new BScrollView("scroll_view", fTextView, B_FOLLOW_ALL, 0, true, true, B_PLAIN_BORDER);
        AddChild(fScrollView);

        // Status bar
        BRect statusFrame(0, bounds.Height() - statusHeight, bounds.Width(), bounds.Height());
        fStatusBar = new RustidStatusBar(statusFrame);
        AddChild(fStatusBar);
    }

    virtual ~RustidWindow() {
        delete fOpenPanel;
        delete fSavePanel;
    }

    virtual void MessageReceived(BMessage* msg) override {
        switch (msg->what) {
            case CMD_FILE_OPEN:
            case CMD_FILE_EXPORT:
            case CMD_FILE_COPY:
            case CMD_FILE_REFRESH:
            case CMD_MODE_STANDARD:
            case CMD_MODE_DEBUG:
            case CMD_MODE_EVERYTHING:
            case CMD_MODE_DUMP:
            case CMD_OPT_COLOR:
            case CMD_OPT_DARK_THEME:
            case CMD_OPT_VERBOSE:
            case CMD_OPT_COMPACT:
            case CMD_HELP_ABOUT:
                if (g_cmd_cb) {
                    g_cmd_cb(msg->what);
                }
                break;

            case CMD_FILE_EXIT:
                PostMessage(B_QUIT_REQUESTED);
                break;

            case B_SIMPLE_DATA:
            case B_REFS_RECEIVED: {
                entry_ref ref;
                if (msg->FindRef("refs", &ref) == B_OK) {
                    BPath path(&ref);
                    if (path.InitCheck() == B_OK && g_file_cb) {
                        g_file_cb(path.Path(), false);
                    }
                }
                break;
            }

            case B_SAVE_REQUESTED: {
                entry_ref directory;
                const char* name = nullptr;
                if (msg->FindRef("directory", &directory) == B_OK && msg->FindString("name", &name) == B_OK) {
                    BPath path(&directory);
                    if (path.InitCheck() == B_OK) {
                        path.Append(name);
                        if (g_file_cb) {
                            g_file_cb(path.Path(), true);
                        }
                    }
                }
                break;
            }

            default:
                BWindow::MessageReceived(msg);
                break;
        }
    }

    virtual bool QuitRequested() override {
        if (g_quit_cb) {
            g_quit_cb();
        }
        be_app->PostMessage(B_QUIT_REQUESTED);
        return true;
    }

    void SetReportText(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg) {
        if (!Lock()) return;

        rgb_color bgCol = {bg.r, bg.g, bg.b, 255};
        fTextView->SetViewColor(bgCol);
        fTextView->SetLowColor(bgCol);

        if (run_count == 0 || runs == nullptr) {
            fTextView->SetText(text, length);
        } else {
            size_t alloc_size = sizeof(text_run_array) + (run_count > 1 ? (run_count - 1) * sizeof(text_run) : 0);
            text_run_array* tra = (text_run_array*)malloc(alloc_size);
            if (tra != nullptr) {
                tra->count = (int32)run_count;
                BFont baseFont(be_fixed_font);
                baseFont.SetSize(12.0f);

                for (uint32_t i = 0; i < run_count; ++i) {
                    tra->runs[i].offset = (int32)runs[i].offset;
                    tra->runs[i].font = baseFont;
                    if (runs[i].bold) {
                        tra->runs[i].font.SetFace(B_BOLD_FACE);
                    }
                    tra->runs[i].color = (rgb_color){runs[i].color.r, runs[i].color.g, runs[i].color.b, 255};
                }
                fTextView->SetText(text, length, tra);
                free(tra);
            } else {
                fTextView->SetText(text, length);
            }
        }

        fTextView->Invalidate();
        Unlock();
    }

    void SetStatus(const char* p1, const char* p2, const char* p3) {
        if (!Lock()) return;
        fStatusBar->SetParts(p1, p2, p3);
        Unlock();
    }

    void SetMenuChecks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact) {
        if (!Lock()) return;

        fItemStandard->SetMarked(mode_cmd_id == CMD_MODE_STANDARD);
        fItemDebug->SetMarked(mode_cmd_id == CMD_MODE_DEBUG);
        fItemEverything->SetMarked(mode_cmd_id == CMD_MODE_EVERYTHING);
        if (fItemDump) {
            fItemDump->SetMarked(mode_cmd_id == CMD_MODE_DUMP);
        }

        fItemColor->SetMarked(color);
        fItemDarkTheme->SetMarked(dark_theme);
        fItemVerbose->SetMarked(verbose);
        fItemCompact->SetMarked(compact);

        fDarkTheme = dark_theme;
        fStatusBar->SetTheme(dark_theme);

        Unlock();
    }

    void OpenFileDialog() {
        if (!fOpenPanel) {
            BMessenger msgr(this);
            fOpenPanel = new BFilePanel(B_OPEN_PANEL, &msgr, nullptr, B_FILE_NODE, false);
        }
        fOpenPanel->Show();
    }

    void SaveFileDialog(const char* default_filename) {
        if (!fSavePanel) {
            BMessenger msgr(this);
            fSavePanel = new BFilePanel(B_SAVE_PANEL, &msgr, nullptr, B_FILE_NODE, false);
        }
        if (default_filename) {
            fSavePanel->SetSaveText(default_filename);
        }
        fSavePanel->Show();
    }

private:
    BMenuBar* fMenuBar;
    BTextView* fTextView;
    BScrollView* fScrollView;
    RustidStatusBar* fStatusBar;
    BFilePanel* fOpenPanel;
    BFilePanel* fSavePanel;

    BMenuItem* fItemStandard;
    BMenuItem* fItemDebug;
    BMenuItem* fItemEverything;
    BMenuItem* fItemDump;
    BMenuItem* fItemColor;
    BMenuItem* fItemDarkTheme;
    BMenuItem* fItemVerbose;
    BMenuItem* fItemCompact;

    bool fDarkTheme;
};

class RustidApplication : public BApplication {
public:
    RustidApplication()
        : BApplication("application/x-vnd.rustid-gui"),
          fWindow(nullptr)
    {
    }

    void SetWindow(RustidWindow* win) {
        fWindow = win;
    }

    virtual void RefsReceived(BMessage* msg) override {
        if (fWindow) {
            fWindow->PostMessage(msg);
        }
    }

private:
    RustidWindow* fWindow;
};

static RustidApplication* g_app = nullptr;
static RustidWindow* g_window = nullptr;

extern "C" {

bool haiku_gui_init(const char* title, float width, float height, float min_w, float min_h) {
    if (!be_app) {
        g_app = new RustidApplication();
    }

    BScreen screen;
    BRect screenFrame = screen.Frame();
    float x = (screenFrame.Width() - width) / 2.0f;
    float y = (screenFrame.Height() - height) / 2.0f;

    BRect windowFrame(x, y, x + width, y + height);
    g_window = new RustidWindow(windowFrame, title, min_w, min_h);
    if (g_app) {
        g_app->SetWindow(g_window);
    }
    g_window->Show();
    return true;
}

void haiku_gui_set_callbacks(CmdCallback on_cmd, FileCallback on_file, QuitCallback on_quit) {
    g_cmd_cb = on_cmd;
    g_file_cb = on_file;
    g_quit_cb = on_quit;
}

void haiku_gui_set_text(const char* text, uint32_t length, const CTextRun* runs, uint32_t run_count, CRgbColor bg_color) {
    if (g_window) {
        g_window->SetReportText(text, length, runs, run_count, bg_color);
    }
}

void haiku_gui_set_status(const char* part1, const char* part2, const char* part3) {
    if (g_window) {
        g_window->SetStatus(part1, part2, part3);
    }
}

void haiku_gui_set_menu_checks(uint32_t mode_cmd_id, bool color, bool dark_theme, bool verbose, bool compact) {
    if (g_window) {
        g_window->SetMenuChecks(mode_cmd_id, color, dark_theme, verbose, compact);
    }
}

void haiku_gui_open_file_dialog(void) {
    if (g_window) {
        g_window->OpenFileDialog();
    }
}

void haiku_gui_save_file_dialog(const char* default_filename) {
    if (g_window) {
        g_window->SaveFileDialog(default_filename);
    }
}

void haiku_gui_copy_clipboard(const char* text) {
    if (!text || !be_clipboard) return;

    if (be_clipboard->Lock()) {
        be_clipboard->Clear();
        BMessage* clipMsg = be_clipboard->Data();
        if (clipMsg) {
            clipMsg->AddData("text/plain", B_MIME_TYPE, text, (ssize_t)strlen(text));
            be_clipboard->Commit();
        }
        be_clipboard->Unlock();
    }
}

void haiku_gui_show_alert(const char* title, const char* message) {
    BAlert* alert = new BAlert(title, message, "OK", nullptr, nullptr, B_WIDTH_AS_USUAL, B_INFO_ALERT);
    alert->Go(nullptr);
}

void haiku_gui_run(void) {
    if (be_app) {
        be_app->Run();
    }
}

} // extern "C"
