//! Plain report formatting and syntax-colored RTF generation.

pub use crate::gui::common::GuiTheme;
use crate::gui::styled_text::{ReportLineToken, tokenize_report_line};

pub fn rtf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 10);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\r' => {}
            '\n' => out.push_str("\\par\r\n"),
            c if (c as u32) < 128 => out.push(c),
            c => {
                let code = c as u32;
                if code <= 0xFFFF {
                    out.push_str(&format!("\\u{}?", code as i16));
                } else {
                    out.push('?');
                }
            }
        }
    }
    out
}

pub fn to_rtf(plain_text: &str, theme: GuiTheme, color: bool) -> String {
    let font_tbl = "{\\fonttbl{\\f0\\fmodern\\fprq1\\fcharset0 Consolas;}{\\f1\\fmodern\\fprq1\\fcharset0 Courier New;}}";

    if !color {
        let escaped = rtf_escape(plain_text);
        return if theme.is_dark() {
            format!(
                "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{font_tbl}{{\\colortbl ;\\red212\\green212\\blue212;}}\\viewkind4\\uc1\\f0\\f1\\fs22\\cf1 {}\\par}}",
                escaped
            )
        } else {
            format!(
                "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{font_tbl}{{\\colortbl ;\\red30\\green30\\blue30;}}\\viewkind4\\uc1\\f0\\f1\\fs22\\cf1 {}\\par}}",
                escaped
            )
        };
    }

    // Palette definition
    // Color 1: Green (Main labels)
    // Color 2: Blue/Cyan (Sublabels / Header banner)
    // Color 3: Body text (Off-white or Charcoal)
    // Color 4: Warm highlight (Numbers, Hex, Features)
    // Color 5: Muted gray (Dividers)
    let color_tbl = if theme.is_dark() {
        "{\\colortbl ;\\red115\\green218\\blue202;\\red125\\green207\\blue255;\\red212\\green212\\blue212;\\red255\\green158\\blue100;\\red86\\green95\\blue137;}"
    } else {
        "{\\colortbl ;\\red9\\green134\\blue88;\\red4\\green81\\blue165;\\red30\\green30\\blue30;\\red163\\green21\\blue21;\\red110\\green118\\blue129;}"
    };

    let mut rtf = format!(
        "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{font_tbl}{color_tbl}\\viewkind4\\uc1\\f0\\f1\\fs22 "
    );

    for line in plain_text.lines() {
        match tokenize_report_line(line) {
            ReportLineToken::Empty => {
                rtf.push_str("\\par\r\n");
            }
            ReportLineToken::Header(h) => {
                rtf.push_str(&format!("\\cf2 {}\\par\r\n", rtf_escape(h)));
            }
            ReportLineToken::CoreHeading(ch) => {
                rtf.push_str(&format!("\\cf2\\b {}\\b0\\par\r\n", rtf_escape(ch)));
            }
            ReportLineToken::LabeledWithSub {
                label,
                sublabel,
                value,
            } => {
                rtf.push_str(&format!(
                    "\\cf1 {}: \\cf2 {}: \\cf3 {}\\par\r\n",
                    rtf_escape(label),
                    rtf_escape(sublabel),
                    rtf_escape(value)
                ));
            }
            ReportLineToken::Labeled { label, value } => {
                rtf.push_str(&format!(
                    "\\cf1 {}: \\cf3 {}\\par\r\n",
                    rtf_escape(label),
                    rtf_escape(value)
                ));
            }
            ReportLineToken::Sublabeled {
                indent,
                sublabel,
                value,
            } => {
                rtf.push_str(&format!(
                    "\\cf5 {}\\cf2 {}: \\cf3 {}\\par\r\n",
                    indent,
                    rtf_escape(sublabel),
                    rtf_escape(value)
                ));
            }
            ReportLineToken::SublabeledHighlight { indent, text } => {
                rtf.push_str(&format!(
                    "\\cf5 {}\\cf4 {}\\par\r\n",
                    indent,
                    rtf_escape(text)
                ));
            }
            ReportLineToken::Plain(p) => {
                rtf.push_str(&format!("\\cf3 {}\\par\r\n", rtf_escape(p)));
            }
        }
    }

    rtf.push('}');
    rtf
}
