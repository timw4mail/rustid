//! Top-level macOS GUI module for rustid-gui.

pub mod render;

use std::cell::{Cell, OnceCell, RefCell};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
#[cfg(x86_cpu)]
use objc2_app_kit::NSOpenPanel;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSColor, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSMenu,
    NSMenuItem, NSModalResponseOK, NSPasteboard, NSPasteboardTypeString, NSSavePanel, NSScrollView,
    NSTextView, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
    NSUserDefaults, ns_string,
};

use render::{ViewMode, generate_debug_info_plain, generate_report_plain, render_report};
use rustid::Cpu;
#[allow(unused_imports)]
use rustid::common::CpuDisplay;
#[allow(unused_imports)]
use rustid::common::TDetect;

const WINDOW_W: f64 = 860.0;
const WINDOW_H: f64 = 620.0;

#[derive(Default)]
struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
    text_view: OnceCell<Retained<NSTextView>>,
    mode: Cell<ViewMode>,
    verbose: Cell<bool>,
    compact: Cell<bool>,
    dark_override: Cell<Option<bool>>,
    #[allow(dead_code)]
    loaded_file: RefCell<Option<String>>,
    verbose_item: OnceCell<Retained<NSMenuItem>>,
    compact_item: OnceCell<Retained<NSMenuItem>>,
    dark_item: OnceCell<Retained<NSMenuItem>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl NSApplicationDelegate for Delegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &objc2_foundation::NSNotification) {
            let mtm = self.mtm();

            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WINDOW_W, WINDOW_H)),
                    NSWindowStyleMask::Titled
                        | NSWindowStyleMask::Closable
                        | NSWindowStyleMask::Miniaturizable
                        | NSWindowStyleMask::Resizable,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            unsafe { window.setReleasedWhenClosed(false) };
            window.setTitle(ns_string!("Rustid for macOS"));
            window.setDelegate(Some(ProtocolObject::from_ref(self)));

            let text_view = NSTextView::initWithFrame(
                NSTextView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WINDOW_W, WINDOW_H)),
            );
            text_view.setEditable(false);
            text_view.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );
            self.ivars()
                .text_view
                .set(text_view)
                .expect("text view ivar set exactly once");

            let scroll = NSScrollView::initWithFrame(
                NSScrollView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WINDOW_W, WINDOW_H)),
            );
            scroll.setHasVerticalScroller(true);
            scroll.setHasHorizontalScroller(false);
            scroll.setDocumentView(Some(
                self.ivars().text_view.get().expect("text view just set"),
            ));

            let view = window.contentView().expect("window must have content view");
            view.addSubview(&scroll);
            window.center();
            window.makeKeyAndOrderFront(None);

            self.ivars()
                .window
                .set(window)
                .expect("window ivar set exactly once");

            setup_menus(self, mtm);
            self.refresh();
        }
    }

    unsafe impl NSWindowDelegate for Delegate {
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &objc2_foundation::NSNotification) {
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }
    }

    impl Delegate {
        #[unsafe(method(showStandardView:))]
        fn show_standard(&self, _sender: Option<&AnyObject>) {
            self.set_mode(ViewMode::Standard);
        }

        #[unsafe(method(showDebugView:))]
        fn show_debug(&self, _sender: Option<&AnyObject>) {
            self.set_mode(ViewMode::Debug);
        }

        #[unsafe(method(showEverythingView:))]
        fn show_everything(&self, _sender: Option<&AnyObject>) {
            self.set_mode(ViewMode::Everything);
        }

        #[cfg(x86_cpu)]
        #[unsafe(method(showDumpView:))]
        fn show_dump(&self, _sender: Option<&AnyObject>) {
            self.set_mode(ViewMode::Dump);
        }

        #[unsafe(method(toggleVerbose:))]
        fn toggle_verbose(&self, _sender: Option<&AnyObject>) {
            self.set_verbose(!self.ivars().verbose.get());
        }

        #[unsafe(method(toggleCompact:))]
        fn toggle_compact(&self, _sender: Option<&AnyObject>) {
            self.set_compact(!self.ivars().compact.get());
        }

        #[unsafe(method(toggleDark:))]
        fn toggle_dark(&self, _sender: Option<&AnyObject>) {
            let next = !self.is_dark();
            self.ivars().dark_override.set(Some(next));
            if let Some(item) = self.ivars().dark_item.get() {
                set_item_state(item, next);
            }
            self.refresh();
        }

        #[cfg(x86_cpu)]
        #[unsafe(method(openDump:))]
        #[allow(deprecated)]
        fn open_dump(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let panel = NSOpenPanel::openPanel(mtm);
            panel.setCanChooseFiles(true);
            panel.setCanChooseDirectories(false);
            panel.setAllowsMultipleSelection(false);
            let file_types = NSArray::from_retained_slice(&[NSString::from_str("txt")]);
            panel.setAllowedFileTypes(Some(&file_types));
            if panel.runModal() == NSModalResponseOK
                && let Some(url) = panel.URLs().firstObject()
                && let Some(path) = url.path()
            {
                self.ivars().loaded_file.borrow_mut().replace(path.to_string());
                self.refresh();
            }
        }

        #[unsafe(method(refreshHardware:))]
        fn refresh_hardware(&self, _sender: Option<&AnyObject>) {
            #[cfg(x86_cpu)]
            {
                rustid::x86::provider::reset_cpuid_provider();
                self.ivars().loaded_file.borrow_mut().take();
            }
            self.refresh();
        }

        #[unsafe(method(saveReport:))]
        #[allow(deprecated)]
        fn save_report(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let plain = self.current_text();
            let nsstr = NSString::from_str(&plain);

            let panel = NSSavePanel::savePanel(mtm);
            panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(&[NSString::from_str(
                "txt",
            )])));
            panel.setNameFieldStringValue(&NSString::from_str("rustid-report.txt"));
            if panel.runModal() == NSModalResponseOK
                && let Some(url) = panel.URL()
                && let Some(data) = nsstr.dataUsingEncoding(4)
            {
                data.writeToURL_atomically(&url, true);
            }
        }

        #[cfg(x86_cpu)]
        #[unsafe(method(exportDump:))]
        #[allow(deprecated)]
        fn export_dump(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let cpu = Cpu::detect();
            let model = cpu.display_model_string().replace([' ', '/', '\\'], "_");
            let dump = render::generate_dump_info_plain();
            let nsstr = NSString::from_str(&dump);

            let panel = NSSavePanel::savePanel(mtm);
            panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(&[NSString::from_str(
                "txt",
            )])));
            panel.setNameFieldStringValue(&NSString::from_str(&format!("cpuid_dump_{model}.txt")));
            if panel.runModal() == NSModalResponseOK
                && let Some(url) = panel.URL()
                && let Some(data) = nsstr.dataUsingEncoding(4)
            {
                data.writeToURL_atomically(&url, true);
            }
        }

        #[unsafe(method(copyReport:))]
        fn copy_report(&self, _sender: Option<&AnyObject>) {
            let plain = self.current_text();
            let nsstr = NSString::from_str(&plain);
            let pb = NSPasteboard::generalPasteboard();
            pb.clearContents();
            unsafe {
                pb.setString_forType(&nsstr, NSPasteboardTypeString);
            }
        }
    }
);

impl Delegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    fn text(&self) -> &NSTextView {
        self.ivars()
            .text_view
            .get()
            .expect("text view created at launch")
    }

    fn set_mode(&self, mode: ViewMode) {
        self.ivars().mode.set(mode);
        self.refresh();
    }

    fn set_verbose(&self, verbose: bool) {
        self.ivars().verbose.set(verbose);
        if let Some(item) = self.ivars().verbose_item.get() {
            set_item_state(item, verbose);
        }
        self.refresh();
    }

    fn set_compact(&self, compact: bool) {
        self.ivars().compact.set(compact);
        if let Some(item) = self.ivars().compact_item.get() {
            set_item_state(item, compact);
        }
        self.refresh();
    }

    fn is_dark(&self) -> bool {
        if let Some(override_val) = self.ivars().dark_override.get() {
            return override_val;
        }
        NSUserDefaults::standardUserDefaults()
            .stringForKey(&NSString::from_str("AppleInterfaceStyle"))
            .is_some_and(|v| v.to_string() == "Dark")
    }

    fn current_text(&self) -> String {
        #[cfg(x86_cpu)]
        if let Some(path) = self.ivars().loaded_file.borrow().clone() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                let dump = rustid::x86::provider::CpuDump::parse_str(&contents);
                rustid::x86::provider::set_cpuid_provider(dump);
            } else {
                rustid::x86::provider::reset_cpuid_provider();
            }
        }

        let cpu = Cpu::detect();
        let verbose = self.ivars().verbose.get();
        let compact = self.ivars().compact.get();
        #[cfg(x86_cpu)]
        let is_from_dump = self.ivars().loaded_file.borrow().is_some();
        #[cfg(not(x86_cpu))]
        let is_from_dump = false;

        let mode = self.ivars().mode.get();
        match mode {
            ViewMode::Standard => generate_report_plain(&cpu, verbose, compact, is_from_dump),
            ViewMode::Debug => generate_debug_info_plain(&cpu),
            ViewMode::Everything => {
                let report = generate_report_plain(&cpu, verbose, compact, is_from_dump);
                let debug = generate_debug_info_plain(&cpu);
                format!("{report}\r\n--------------------\r\n\r\n{debug}")
            }
            #[cfg(x86_cpu)]
            ViewMode::Dump => render::generate_dump_info_plain(),
        }
    }

    fn refresh(&self) {
        let plain = self.current_text();
        let dark = self.is_dark();
        let font = NSFont::monospacedSystemFontOfSize_weight(13.0, 0.0);
        let bg = if dark {
            NSColor::colorWithSRGBRed_green_blue_alpha(0.11, 0.11, 0.12, 1.0)
        } else {
            NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0)
        };

        let text_view = self.text();
        text_view.setBackgroundColor(&bg);
        let attributed = render_report(&plain, dark, &font);
        if let Some(storage) = unsafe { text_view.textStorage() } {
            storage.setAttributedString(&attributed);
        }
    }
}

fn set_item_state(item: &NSMenuItem, on: bool) {
    item.setState(if on {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    });
}

fn make_item(
    mtm: MainThreadMarker,
    title: &str,
    key: &str,
    action: objc2::runtime::Sel,
    target: Option<&AnyObject>,
) -> Retained<NSMenuItem> {
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            Some(action),
            &NSString::from_str(key),
        )
    };
    unsafe { item.setTarget(target) };
    item
}

fn setup_menus(delegate: &Delegate, mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);

    let main_menu = NSMenu::new(mtm);

    let app_menu = NSMenu::new(mtm);
    app_menu.addItem(&make_item(mtm, "Quit Rustid", "q", sel!(terminate:), None));

    let file_menu = NSMenu::new(mtm);
    file_menu.addItem(&make_item(
        mtm,
        "Copy Report",
        "c",
        sel!(copyReport:),
        Some(obj_any(delegate)),
    ));
    file_menu.addItem(&make_item(
        mtm,
        "Save Report…",
        "s",
        sel!(saveReport:),
        Some(obj_any(delegate)),
    ));
    #[cfg(x86_cpu)]
    file_menu.addItem(&make_item(
        mtm,
        "Export CPUID Dump…",
        "",
        sel!(exportDump:),
        Some(obj_any(delegate)),
    ));
    file_menu.addItem(&NSMenuItem::separatorItem(mtm));
    #[cfg(x86_cpu)]
    file_menu.addItem(&make_item(
        mtm,
        "Open CPUID Dump…",
        "o",
        sel!(openDump:),
        Some(obj_any(delegate)),
    ));
    file_menu.addItem(&make_item(
        mtm,
        "Refresh Hardware",
        "r",
        sel!(refreshHardware:),
        Some(obj_any(delegate)),
    ));

    let view_menu = NSMenu::new(mtm);
    view_menu.addItem(&make_item(
        mtm,
        "Standard View",
        "1",
        sel!(showStandardView:),
        Some(obj_any(delegate)),
    ));
    view_menu.addItem(&make_item(
        mtm,
        "Debug View",
        "2",
        sel!(showDebugView:),
        Some(obj_any(delegate)),
    ));
    view_menu.addItem(&make_item(
        mtm,
        "Everything View",
        "3",
        sel!(showEverythingView:),
        Some(obj_any(delegate)),
    ));
    #[cfg(x86_cpu)]
    view_menu.addItem(&make_item(
        mtm,
        "CPUID Dump View",
        "4",
        sel!(showDumpView:),
        Some(obj_any(delegate)),
    ));

    let options_menu = NSMenu::new(mtm);
    let verbose_item = make_item(
        mtm,
        "Verbose Output",
        "",
        sel!(toggleVerbose:),
        Some(obj_any(delegate)),
    );
    let compact_item = make_item(
        mtm,
        "Compact Mode",
        "",
        sel!(toggleCompact:),
        Some(obj_any(delegate)),
    );
    let dark_item = make_item(
        mtm,
        "Dark Mode",
        "",
        sel!(toggleDark:),
        Some(obj_any(delegate)),
    );
    options_menu.addItem(&verbose_item);
    options_menu.addItem(&compact_item);
    options_menu.addItem(&dark_item);
    delegate
        .ivars()
        .verbose_item
        .set(verbose_item)
        .expect("verbose menu item set once");
    delegate
        .ivars()
        .compact_item
        .set(compact_item)
        .expect("compact menu item set once");
    delegate
        .ivars()
        .dark_item
        .set(dark_item)
        .expect("dark menu item set once");

    let app_bar_item = make_item(mtm, "Rustid", "", sel!(noop:), None);
    let file_bar_item = make_item(mtm, "File", "", sel!(noop:), None);
    let view_bar_item = make_item(mtm, "View", "", sel!(noop:), None);
    let options_bar_item = make_item(mtm, "Options", "", sel!(noop:), None);
    app_bar_item.setSubmenu(Some(&app_menu));
    file_bar_item.setSubmenu(Some(&file_menu));
    view_bar_item.setSubmenu(Some(&view_menu));
    options_bar_item.setSubmenu(Some(&options_menu));

    main_menu.addItem(&app_bar_item);
    main_menu.addItem(&file_bar_item);
    main_menu.addItem(&view_bar_item);
    main_menu.addItem(&options_bar_item);

    app.setMainMenu(Some(&main_menu));
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
}

fn obj_any(delegate: &Delegate) -> &AnyObject {
    // The Delegate is an NSObject subclass; safe to reborrow as its base type.
    unsafe { &*(delegate as *const Delegate as *const AnyObject) }
}

pub fn run() {
    let mtm = MainThreadMarker::new().expect("macOS GUI must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let delegate = Delegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    app.run();
}
