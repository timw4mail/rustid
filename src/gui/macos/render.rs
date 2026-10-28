//! macOS syntax-colored attributed string rendering.
//!
//! Mirrors the Windows `gui/rtf.rs` and GTK `gui/styled_text.rs` behaviour:
//! plain-text reports are tokenized via `tokenize_report_line` and rendered
//! using the shared color palettes as an `NSMutableAttributedString`.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{NSColor, NSFont, NSFontAttributeName, NSForegroundColorAttributeName};
use objc2_foundation::{
    NSAttributedString, NSMutableAttributedString, NSMutableDictionary, NSString,
};

use crate::gui::styled_text::{
    PALETTE_DARK, PALETTE_LIGHT, Palette, ReportLineToken, RgbColor, tokenize_report_line,
};

fn ns_color(c: RgbColor) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(c.r) / 255.0,
        f64::from(c.g) / 255.0,
        f64::from(c.b) / 255.0,
        1.0,
    )
}

fn obj<T: objc2::Message>(o: &T) -> &AnyObject {
    // Objective-C objects can be safely reborrowed as their base `id` type.
    unsafe { &*(o as *const T as *const AnyObject) }
}

/// Build a single-colour attributed run in `font`.
fn colored_run(text: &str, font: &NSFont, color: RgbColor) -> Retained<NSAttributedString> {
    let nsstr = NSString::from_str(text);
    let fg = ns_color(color);

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
    let palette = if dark_theme {
        &PALETTE_DARK
    } else {
        &PALETTE_LIGHT
    };

    let out = NSMutableAttributedString::new();
    for line in plain.lines() {
        let token = tokenize_report_line(line);
        if matches!(token, ReportLineToken::Empty) {
            out.appendAttributedString(&colored_run("\n", font, palette.body));
            continue;
        }
        append_token(&out, token, palette, font);
        out.appendAttributedString(&colored_run("\n", font, palette.body));
    }
    out
}

fn append_token(
    out: &Retained<NSMutableAttributedString>,
    token: ReportLineToken<'_>,
    palette: &Palette,
    font: &NSFont,
) {
    let push = |text: &str, color: RgbColor| {
        out.appendAttributedString(&colored_run(text, font, color));
    };

    match token {
        ReportLineToken::Empty => {}
        ReportLineToken::Header(h) | ReportLineToken::CoreHeading(h) => {
            push(h, palette.sublabel);
        }
        ReportLineToken::LabeledWithSub {
            label,
            sublabel,
            value,
        } => {
            push(label, palette.label);
            push(": ", palette.body);
            push(sublabel, palette.sublabel);
            push(": ", palette.body);
            push(value, palette.body);
        }
        ReportLineToken::Labeled { label, value } => {
            push(label, palette.label);
            push(": ", palette.body);
            push(value, palette.body);
        }
        ReportLineToken::Sublabeled {
            indent,
            sublabel,
            value,
        } => {
            push(indent, palette.divider);
            push(sublabel, palette.sublabel);
            push(": ", palette.body);
            push(value, palette.body);
        }
        ReportLineToken::SublabeledHighlight { indent, text } => {
            push(indent, palette.divider);
            push(text, palette.highlight);
        }
        ReportLineToken::Plain(p) => {
            push(p, palette.body);
        }
    }
}
