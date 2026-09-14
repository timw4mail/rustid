//! File dialogs, alert helpers, and clipboard operations for Linux GTK GUI using gtk-rs.

use gtk::prelude::*;
use gtk::{FileChooserAction, FileChooserDialog, FileFilter, MessageDialog, ResponseType, Window};

pub fn copy_to_clipboard(text: &str) {
    let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
    clipboard.set_text(text);
}

pub fn show_alert(parent: Option<&Window>, title: &str, message: &str) {
    let dialog = MessageDialog::new(
        parent,
        gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        gtk::ButtonsType::Ok,
        message,
    );
    if !title.is_empty() {
        dialog.set_title(title);
    }
    dialog.run();
    dialog.close();
}

use gtk::gdk_pixbuf::{InterpType, Pixbuf, PixbufLoader};

pub fn get_app_icon() -> Option<Pixbuf> {
    let loader = PixbufLoader::new();
    let bytes = include_bytes!("../../../assets/rustid.png");
    loader.write(bytes).ok()?;
    loader.close().ok()?;
    loader.pixbuf()
}

pub fn show_about_dialog(parent: Option<&Window>) {
    let dialog = gtk::AboutDialog::new();
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
        dialog.set_modal(true);
    }
    dialog.set_program_name("Rustid");
    dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
    dialog.set_comments(Some("Multi-architecture CPU detection tool"));
    dialog.set_website(Some("https://git.timshomepage.net/timw4mail/rustid.git"));
    dialog.set_website_label(Some("Project Repository"));
    dialog.set_authors(&["Timothy J. Warren <tim@timshome.page>"]);
    if let Some(icon) = get_app_icon() {
        if let Some(logo) = icon.scale_simple(96, 96, InterpType::Hyper) {
            dialog.set_logo(Some(&logo));
        } else {
            dialog.set_logo(Some(&icon));
        }
    }
    dialog.run();
    dialog.close();
}

#[cfg(x86_cpu)]
pub fn open_dump_file_dialog(parent: Option<&Window>) -> Option<String> {
    let dialog = FileChooserDialog::new(
        Some("Open CPUID Dump File"),
        parent,
        FileChooserAction::Open,
    );
    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Open", ResponseType::Accept);

    let filter = FileFilter::new();
    filter.set_name(Some("Text files (*.txt)"));
    filter.add_pattern("*.txt");
    dialog.add_filter(filter);

    let all_filter = FileFilter::new();
    all_filter.set_name(Some("All files (*.*)"));
    all_filter.add_pattern("*");
    dialog.add_filter(all_filter);

    let result = if dialog.run() == ResponseType::Accept {
        dialog
            .filename()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    } else {
        None
    };

    dialog.close();
    result
}

#[cfg(x86_cpu)]
pub fn export_dump_dialog(parent: Option<&Window>, default_filename: &str) -> Option<String> {
    let dialog = FileChooserDialog::new(
        Some("Export CPUID Dump File"),
        parent,
        FileChooserAction::Save,
    );
    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Save", ResponseType::Accept);

    dialog.set_do_overwrite_confirmation(true);
    if !default_filename.is_empty() {
        dialog.set_current_name(default_filename);
    }

    let filter = FileFilter::new();
    filter.set_name(Some("Text files (*.txt)"));
    filter.add_pattern("*.txt");
    dialog.add_filter(filter);

    let all_filter = FileFilter::new();
    all_filter.set_name(Some("All files (*.*)"));
    all_filter.add_pattern("*");
    dialog.add_filter(all_filter);

    let result = if dialog.run() == ResponseType::Accept {
        dialog
            .filename()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    } else {
        None
    };

    dialog.close();
    result
}

#[cfg(x86_cpu)]
pub fn read_file_to_string(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(x86_cpu)]
pub fn write_string_to_file(path: &str, content: &str) -> bool {
    std::fs::write(path, content).is_ok()
}
