//! macOS syntax-colored attributed string rendering.
//!
//! Mirrors the Windows `gui/rtf.rs` behaviour: plain-text reports are
//! rendered with the same line-labeling heuristics as the RTF generator,
//! but as an `NSMutableAttributedString`.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{NSColor, NSFont, NSFontAttributeName, NSForegroundColorAttributeName};
use objc2_foundation::{
    NSAttributedString, NSMutableAttributedString, NSMutableDictionary, NSString,
};

#[derive(Copy, Clone)]
enum LineStyle {
    Header,
    Label,
    Sublabel,
    Highlight,
    Divider,
    Body,
}

impl LineStyle {
    fn rgb(self, dark: bool) -> (f64, f64, f64) {
        let (label, sublabel, header, body, highlight, divider) = if dark {
            (
                (0.45, 0.85, 0.79),
                (0.49, 0.81, 1.0),
                (0.56, 0.56, 0.56),
                (0.83, 0.83, 0.83),
                (1.0, 0.62, 0.39),
                (0.34, 0.37, 0.54),
            )
        } else {
            (
                (0.04, 0.53, 0.35),
                (0.02, 0.32, 0.65),
                (0.30, 0.30, 0.30),
                (0.12, 0.12, 0.12),
                (0.64, 0.08, 0.08),
                (0.43, 0.46, 0.51),
            )
        };
        match self {
            LineStyle::Header => header,
            LineStyle::Sublabel => sublabel,
            LineStyle::Label => label,
            LineStyle::Highlight => highlight,
            LineStyle::Divider => divider,
            LineStyle::Body => body,
        }
    }
}

fn ns_color(rgb: (f64, f64, f64)) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(rgb.0, rgb.1, rgb.2, 1.0)
}

fn obj<T: objc2::Message>(o: &T) -> &AnyObject {
    // Objective-C objects can be safely reborrowed as their base `id` type.
    unsafe { &*(o as *const T as *const AnyObject) }
}

/// Build a single-colour attributed run in `font`.
fn colored_run(text: &str, font: &NSFont, rgb: (f64, f64, f64)) -> Retained<NSAttributedString> {
    let nsstr = NSString::from_str(text);
    let fg = ns_color(rgb);

    let attrs = unsafe {
        let dict = NSMutableDictionary::<NSString, AnyObject>::new();
        dict.setObject_forKey(
            obj(&*fg),
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
        dict.setObject_forKey(obj(font), ProtocolObject::from_ref(NSFontAttributeName));
        dict
    };

    unsafe {
        NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &nsstr,
            Some(attrs.as_ref()),
        )
    }
}

/// Render `plain` into an attributed string using the syntax palette.
pub fn render_report(
    plain: &str,
    dark_theme: bool,
    font: &NSFont,
) -> Retained<NSMutableAttributedString> {
    let out = NSMutableAttributedString::new();
    for line in plain.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            out.appendAttributedString(&colored_run("\n", font, LineStyle::Body.rgb(dark_theme)));
            continue;
        }
        append_line(&out, line, dark_theme, font);
        out.appendAttributedString(&colored_run("\n", font, LineStyle::Body.rgb(dark_theme)));
    }
    out
}

fn append_line(out: &Retained<NSMutableAttributedString>, line: &str, dark: bool, font: &NSFont) {
    if line.starts_with("---------------") || line.starts_with("--------------------") {
        push(out, line, dark, font, LineStyle::Header);
        return;
    }
    if line.trim_start().starts_with("Core #") {
        push(out, line, dark, font, LineStyle::Header);
        return;
    }
    if line.len() >= 16 && &line[14..16] == ": " {
        let label = &line[..14];
        let rest = &line[16..];
        push(out, label, dark, font, LineStyle::Label);
        push(out, ": ", dark, font, LineStyle::Body);
        push(out, rest, dark, font, LineStyle::Body);
        return;
    }
    if let Some(sub) = line.strip_prefix("                ") {
        if let Some(colon_idx) = sub.find(": ") {
            push(out, &line[..16], dark, font, LineStyle::Divider);
            push(out, &sub[..colon_idx], dark, font, LineStyle::Sublabel);
            push(out, ": ", dark, font, LineStyle::Body);
            push(out, &sub[colon_idx + 2..], dark, font, LineStyle::Body);
            return;
        } else if sub.starts_with('(') {
            push(out, &line[..16], dark, font, LineStyle::Divider);
            push(out, sub, dark, font, LineStyle::Highlight);
            return;
        }
    }
    push(out, line, dark, font, LineStyle::Body);
}

fn push(
    out: &Retained<NSMutableAttributedString>,
    text: &str,
    dark: bool,
    font: &NSFont,
    style: LineStyle,
) {
    out.appendAttributedString(&colored_run(text, font, style.rgb(dark)));
}
