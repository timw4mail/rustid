//! Window management, event loop, and render pipeline for Linux GTK GUI using gtk-rs.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk::{
    AccelFlags, AccelGroup, Box, CheckMenuItem, CssProvider, Frame, Label, Menu, MenuBar, MenuItem,
    Orientation, PolicyType, ScrolledWindow, SeparatorMenuItem, ShadowType, StyleContext, TextView,
    Window, WindowType,
};

use crate::Cpu;
use crate::common::cpu::TDetect;
use crate::gui::ReportSource;
use crate::gui::common;
use crate::gui::styled_text::parse_text_runs;

use super::dialogs::*;
use super::state::{AppState, ViewMode};

struct UiWidgets {
    window: Window,
    text_view: TextView,
    status_label1: Label,
    status_label2: Label,
    status_label3: Label,
    css_provider: CssProvider,
    item_mode_standard: CheckMenuItem,
    item_mode_debug: CheckMenuItem,
    item_mode_everything: CheckMenuItem,
    #[cfg(x86_cpu)]
    item_mode_dump: CheckMenuItem,
    item_opt_color: CheckMenuItem,
    item_opt_dark_theme: CheckMenuItem,
    item_opt_verbose: CheckMenuItem,
    item_opt_compact: CheckMenuItem,
}

fn render_ui(widgets: &UiWidgets, state: &mut AppState, updating_menu: &Rc<RefCell<bool>>) {
    #[cfg(x86_cpu)]
    {
        let contents = state.loaded_file.as_deref().and_then(read_file_to_string);
        common::configure_dump_provider_from_contents(contents.as_deref());
    }

    let cpu = Cpu::detect();
    let source = ReportSource::from(state.loaded_file.is_some());

    state.current_plain_text = common::build_view_text(&cpu, state.mode, state.flags, source);

    let (bg_color, runs) =
        parse_text_runs(&state.current_plain_text, state.theme, state.flags.color);

    if let Some(buffer) = widgets.text_view.buffer() {
        buffer.set_text(&state.current_plain_text);
        let (start, end) = buffer.bounds();
        buffer.remove_all_tags(&start, &end);

        if let Some(tag_table) = buffer.tag_table() {
            for run in &runs {
                let rgba = gdk::RGBA::new(
                    run.color.r as f64 / 255.0,
                    run.color.g as f64 / 255.0,
                    run.color.b as f64 / 255.0,
                    1.0,
                );
                let tag = gtk::TextTag::new(None);
                tag.set_foreground_rgba(Some(&rgba));
                tag.set_weight(if run.bold { 700 } else { 400 });
                tag_table.add(&tag);

                let start_iter = buffer.iter_at_offset(run.offset as i32);
                let end_iter = buffer.iter_at_offset((run.offset + run.length) as i32);
                buffer.apply_tag(&tag, &start_iter, &end_iter);
            }
        }
    }

    let css = format!(
        "textview text, textview {{ background-color: rgb({},{},{}); font-family: monospace; font-size: 10pt; }}\nframe {{ border: 1px solid alpha(currentColor, 0.2); }}\n",
        bg_color.r, bg_color.g, bg_color.b
    );
    let _ = widgets.css_provider.load_from_data(css.as_bytes());

    let (part1, part2, part3) = common::format_status_parts(
        &cpu,
        state.loaded_file.as_deref(),
        state.mode,
        state.flags,
        state.theme,
    );
    widgets.status_label1.set_text(&part1);
    widgets.status_label2.set_text(&part2);
    widgets.status_label3.set_text(&part3);

    *updating_menu.borrow_mut() = true;
    widgets
        .item_mode_standard
        .set_active(state.mode == ViewMode::Standard);
    widgets
        .item_mode_debug
        .set_active(state.mode == ViewMode::Debug);
    widgets
        .item_mode_everything
        .set_active(state.mode == ViewMode::Everything);
    #[cfg(x86_cpu)]
    widgets
        .item_mode_dump
        .set_active(state.mode == ViewMode::Dump);

    widgets.item_opt_color.set_active(state.flags.color);
    widgets
        .item_opt_dark_theme
        .set_active(state.theme.is_dark());
    widgets.item_opt_verbose.set_active(state.flags.verbose);
    widgets.item_opt_compact.set_active(state.flags.compact);
    *updating_menu.borrow_mut() = false;
}

pub fn run() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    let title = format!(
        "Rustid {} ({}-{})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH,
        std::env::consts::OS
    );

    let window = Window::new(WindowType::Toplevel);
    window.set_title(&title);
    window.set_default_size(820, 640);
    window.set_size_request(580, 380);

    let accel_group = AccelGroup::new();
    window.add_accel_group(&accel_group);

    let main_vbox = Box::new(Orientation::Vertical, 0);
    window.add(&main_vbox);

    // --- Menu Bar ---
    let menubar = MenuBar::new();
    main_vbox.pack_start(&menubar, false, false, 0);

    // File Menu
    let file_menu = Menu::new();
    file_menu.set_accel_group(Some(&accel_group));
    let file_item = MenuItem::with_mnemonic("_File");
    file_item.set_submenu(Some(&file_menu));
    menubar.append(&file_item);

    #[cfg(x86_cpu)]
    let item_file_open = MenuItem::with_label("Open Dump...");
    #[cfg(x86_cpu)]
    {
        item_file_open.add_accelerator(
            "activate",
            &accel_group,
            *gdk::keys::constants::o,
            gdk::ModifierType::CONTROL_MASK,
            AccelFlags::VISIBLE,
        );
        file_menu.append(&item_file_open);
    }

    #[cfg(x86_cpu)]
    let item_file_export = MenuItem::with_label("Export Dump...");
    #[cfg(x86_cpu)]
    {
        item_file_export.add_accelerator(
            "activate",
            &accel_group,
            *gdk::keys::constants::s,
            gdk::ModifierType::CONTROL_MASK,
            AccelFlags::VISIBLE,
        );
        file_menu.append(&item_file_export);
    }

    let item_file_copy = MenuItem::with_label("Copy");
    item_file_copy.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::c,
        gdk::ModifierType::CONTROL_MASK,
        AccelFlags::VISIBLE,
    );
    file_menu.append(&item_file_copy);

    let item_file_refresh = MenuItem::with_label("Refresh");
    item_file_refresh.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::F5,
        gdk::ModifierType::empty(),
        AccelFlags::VISIBLE,
    );
    file_menu.append(&item_file_refresh);

    file_menu.append(&SeparatorMenuItem::new());

    let item_file_exit = MenuItem::with_label("Exit");
    item_file_exit.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::q,
        gdk::ModifierType::CONTROL_MASK,
        AccelFlags::VISIBLE,
    );
    file_menu.append(&item_file_exit);

    // View Menu
    let view_menu = Menu::new();
    let view_item = MenuItem::with_mnemonic("_View");
    view_item.set_submenu(Some(&view_menu));
    menubar.append(&view_item);

    let item_mode_standard = CheckMenuItem::with_label("Standard");
    view_menu.append(&item_mode_standard);

    let item_mode_debug = CheckMenuItem::with_label("Debug");
    view_menu.append(&item_mode_debug);

    let item_mode_everything = CheckMenuItem::with_label("Everything");
    view_menu.append(&item_mode_everything);

    #[cfg(x86_cpu)]
    let item_mode_dump = CheckMenuItem::with_label("CPUID Dump");
    #[cfg(x86_cpu)]
    view_menu.append(&item_mode_dump);

    // Options Menu
    let opt_menu = Menu::new();
    let opt_item = MenuItem::with_mnemonic("_Options");
    opt_item.set_submenu(Some(&opt_menu));
    menubar.append(&opt_item);

    let item_opt_color = CheckMenuItem::with_label("Colors");
    opt_menu.append(&item_opt_color);

    let item_opt_dark_theme = CheckMenuItem::with_label("Dark Theme");
    opt_menu.append(&item_opt_dark_theme);

    opt_menu.append(&SeparatorMenuItem::new());

    let item_opt_verbose = CheckMenuItem::with_label("Verbose");
    opt_menu.append(&item_opt_verbose);

    let item_opt_compact = CheckMenuItem::with_label("Compact");
    opt_menu.append(&item_opt_compact);

    // Help Menu
    let help_menu = Menu::new();
    let help_item = MenuItem::with_mnemonic("_Help");
    help_item.set_submenu(Some(&help_menu));
    menubar.append(&help_item);

    let item_help_about = MenuItem::with_label("About");
    help_menu.append(&item_help_about);

    // --- Scrolled Text View ---
    let scrolled = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scrolled.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    main_vbox.pack_start(&scrolled, true, true, 0);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_left_margin(8);
    text_view.set_right_margin(8);
    text_view.set_top_margin(6);
    text_view.set_bottom_margin(6);
    scrolled.add(&text_view);

    // --- Status Bar ---
    let status_box = Box::new(Orientation::Horizontal, 3);
    status_box.set_margin_start(2);
    status_box.set_margin_end(2);
    status_box.set_margin_top(2);
    status_box.set_margin_bottom(2);
    main_vbox.pack_start(&status_box, false, false, 0);

    let frame1 = Frame::new(None);
    frame1.set_shadow_type(ShadowType::In);
    frame1.set_size_request(320, -1);
    let status_label1 = Label::new(None);
    status_label1.set_xalign(0.0);
    status_label1.set_margin_start(4);
    status_label1.set_margin_end(4);
    frame1.add(&status_label1);
    status_box.pack_start(&frame1, false, false, 0);

    let frame2 = Frame::new(None);
    frame2.set_shadow_type(ShadowType::In);
    frame2.set_size_request(200, -1);
    let status_label2 = Label::new(None);
    status_label2.set_xalign(0.0);
    status_label2.set_margin_start(4);
    status_label2.set_margin_end(4);
    frame2.add(&status_label2);
    status_box.pack_start(&frame2, false, false, 0);

    let frame3 = Frame::new(None);
    frame3.set_shadow_type(ShadowType::In);
    let status_label3 = Label::new(None);
    status_label3.set_xalign(0.0);
    status_label3.set_margin_start(4);
    status_label3.set_margin_end(4);
    frame3.add(&status_label3);
    status_box.pack_start(&frame3, true, true, 0);

    let css_provider = CssProvider::new();
    if let Some(screen) = gdk::Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let widgets = Rc::new(UiWidgets {
        window: window.clone(),
        text_view,
        status_label1,
        status_label2,
        status_label3,
        css_provider,
        item_mode_standard: item_mode_standard.clone(),
        item_mode_debug: item_mode_debug.clone(),
        item_mode_everything: item_mode_everything.clone(),
        #[cfg(x86_cpu)]
        item_mode_dump: item_mode_dump.clone(),
        item_opt_color: item_opt_color.clone(),
        item_opt_dark_theme: item_opt_dark_theme.clone(),
        item_opt_verbose: item_opt_verbose.clone(),
        item_opt_compact: item_opt_compact.clone(),
    });

    let state = Rc::new(RefCell::new(AppState::default()));
    let updating_menu = Rc::new(RefCell::new(false));

    // Connect window events
    window.connect_destroy(|_| {
        gtk::main_quit();
    });

    // Wire actions
    #[cfg(x86_cpu)]
    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        item_file_open.connect_activate(move |_| {
            if let Some(path) = open_dump_file_dialog(Some(&w.window)) {
                s.borrow_mut().loaded_file = Some(path);
                render_ui(&w, &mut s.borrow_mut(), &u);
            }
        });
    }

    #[cfg(x86_cpu)]
    {
        let w = widgets.clone();
        item_file_export.connect_activate(move |_| {
            let cpu = Cpu::detect();
            let default_name = format!(
                "cpuid_dump_{}.txt",
                cpu.display_model_string().replace([' ', '/', '\\'], "_")
            );
            if let Some(path) = export_dump_dialog(Some(&w.window), &default_name) {
                let dump_content = crate::gui::common::generate_dump_info_plain();
                if write_string_to_file(&path, &dump_content) {
                    let msg = format!("CPUID dump successfully saved to:\n{}", path);
                    show_alert(Some(&w.window), "Export Complete", &msg);
                }
            }
        });
    }

    {
        let s = state.clone();
        item_file_copy.connect_activate(move |_| {
            copy_to_clipboard(&s.borrow().current_plain_text);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        item_file_refresh.connect_activate(move |_| {
            s.borrow_mut().loaded_file = None;
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let win = window.clone();
        item_file_exit.connect_activate(move |_| {
            win.close();
        });
    }

    // View modes
    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_mode_standard.connect_toggled(move |item| {
            if *u_clone.borrow() || !item.is_active() {
                return;
            }
            s.borrow_mut().mode = ViewMode::Standard;
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_mode_debug.connect_toggled(move |item| {
            if *u_clone.borrow() || !item.is_active() {
                return;
            }
            s.borrow_mut().mode = ViewMode::Debug;
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_mode_everything.connect_toggled(move |item| {
            if *u_clone.borrow() || !item.is_active() {
                return;
            }
            s.borrow_mut().mode = ViewMode::Everything;
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    #[cfg(x86_cpu)]
    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_mode_dump.connect_toggled(move |item| {
            if *u_clone.borrow() || !item.is_active() {
                return;
            }
            s.borrow_mut().mode = ViewMode::Dump;
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    // Options
    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_opt_color.connect_toggled(move |item| {
            if *u_clone.borrow() {
                return;
            }
            s.borrow_mut().flags.color = item.is_active();
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_opt_dark_theme.connect_toggled(move |_| {
            if *u_clone.borrow() {
                return;
            }
            s.borrow_mut().theme.toggle();
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_opt_verbose.connect_toggled(move |item| {
            if *u_clone.borrow() {
                return;
            }
            s.borrow_mut().flags.verbose = item.is_active();
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    {
        let w = widgets.clone();
        let s = state.clone();
        let u = updating_menu.clone();
        let u_clone = updating_menu.clone();
        item_opt_compact.connect_toggled(move |item| {
            if *u_clone.borrow() {
                return;
            }
            s.borrow_mut().flags.compact = item.is_active();
            render_ui(&w, &mut s.borrow_mut(), &u);
        });
    }

    // About
    {
        let w = widgets.clone();
        item_help_about.connect_activate(move |_| {
            show_about_dialog(Some(&w.window));
        });
    }

    // Initial render and show
    render_ui(&widgets, &mut state.borrow_mut(), &updating_menu);
    window.show_all();
    gtk::main();
}
