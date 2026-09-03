//! Top-level macOS GUI module for rustid-gui.

pub mod render;

use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSColor, NSFont, NSMenu, NSMenuItem, NSModalResponseOK, NSPasteboard,
    NSPasteboardTypeString, NSSavePanel, NSScrollView, NSTextView, NSWindow, NSWindowDelegate,
    NSWindowStyleMask,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString, NSUserDefaults,
};

use rustid::Cpu;
#[allow(unused_imports)]
use rustid::common::CpuDisplay;
use render::{ViewMode, generate_debug_info_plain, generate_report_plain, render_report};
#[allow(unused_imports)]
use rustid::common::TDetect;

const WINDOW_W: f64 = 860.0;
const WINDOW_H: f64 = 620.0;

#[derive(Debug, Default)]
struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
    text_view: OnceCell<Retained<NSTextView>>,
    mode: OnceCell<ViewMode>,
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
            self.ivars().mode.set(ViewMode::Standard).unwrap();

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
            self.ivars().text_view.set(text_view).unwrap();

            let scroll = NSScrollView::initWithFrame(
                NSScrollView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WINDOW_W, WINDOW_H)),
            );
            scroll.setHasVerticalScroller(true);
            scroll.setHasHorizontalScroller(false);
            scroll.setDocumentView(Some(self.ivars().text_view.get().unwrap()));

            let view = window.contentView().expect("window must have content view");
            view.addSubview(&scroll);
            window.center();
            window.makeKeyAndOrderFront(None);

            self.ivars().window.set(window).unwrap();

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

        #[unsafe(method(saveReport:))]
        #[allow(deprecated)]
        fn save_report(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let cpu = Cpu::detect();
            let plain = generate_report_plain(&cpu, false, false, false);
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

        #[unsafe(method(copyReport:))]
        fn copy_report(&self, _sender: Option<&AnyObject>) {
            let cpu = Cpu::detect();
            let plain = generate_report_plain(&cpu, false, false, false);
            let nsstr = NSString::from_str(&plain);
            let pb = NSPasteboard::generalPasteboard();
            pb.clearContents();
            unsafe {
                pb.setString_forType(&nsstr, &NSPasteboardTypeString);
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
        self.ivars().text_view.get().unwrap()
    }

    fn set_mode(&self, mode: ViewMode) {
        self.ivars().mode.set(mode).unwrap();
        self.refresh();
    }

    fn is_dark(&self) -> bool {
        NSUserDefaults::standardUserDefaults()
            .stringForKey(&NSString::from_str("AppleInterfaceStyle"))
            .is_some_and(|v| v.to_string() == "Dark")
    }

    fn refresh(&self) {
        let cpu = Cpu::detect();
        let mode = *self.ivars().mode.get().unwrap();
        let plain = match mode {
            ViewMode::Standard => generate_report_plain(&cpu, false, false, false),
            ViewMode::Debug => generate_debug_info_plain(&cpu),
            ViewMode::Everything => generate_report_plain(&cpu, true, false, false),
            #[cfg(x86_cpu)]
            ViewMode::Dump => render::generate_dump_info_plain(),
        };

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
        unsafe {
            let _: () = msg_send![text_view, setAttributedString: &*attributed];
        }
    }
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

    let app_bar_item = make_item(mtm, "Rustid", "", sel!(noop:), None);
    let file_bar_item = make_item(mtm, "File", "", sel!(noop:), None);
    let view_bar_item = make_item(mtm, "View", "", sel!(noop:), None);
    app_bar_item.setSubmenu(Some(&app_menu));
    file_bar_item.setSubmenu(Some(&file_menu));
    view_bar_item.setSubmenu(Some(&view_menu));

    main_menu.addItem(&app_bar_item);
    main_menu.addItem(&file_bar_item);
    main_menu.addItem(&view_bar_item);

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
