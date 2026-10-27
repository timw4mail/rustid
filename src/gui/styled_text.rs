//! Plain report formatting and styled text run generation for GUI frontends.

pub use crate::gui::common::GuiTheme;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextRun {
    pub offset: u32,
    pub length: u32,
    pub color: RgbColor,
    pub bold: bool,
}

pub struct Palette {
    pub background: RgbColor,
    pub label: RgbColor,
    pub sublabel: RgbColor,
    pub body: RgbColor,
    pub highlight: RgbColor,
    pub divider: RgbColor,
}

pub const PALETTE_LIGHT: Palette = Palette {
    background: RgbColor::new(255, 255, 255),
    label: RgbColor::new(9, 134, 88),
    sublabel: RgbColor::new(4, 81, 165),
    body: RgbColor::new(30, 30, 30),
    highlight: RgbColor::new(163, 21, 21),
    divider: RgbColor::new(110, 118, 129),
};

pub const PALETTE_DARK: Palette = Palette {
    background: RgbColor::new(26, 27, 38),
    label: RgbColor::new(115, 218, 202),
    sublabel: RgbColor::new(125, 207, 255),
    body: RgbColor::new(212, 212, 212),
    highlight: RgbColor::new(255, 158, 100),
    divider: RgbColor::new(86, 95, 137),
};

#[derive(Debug, PartialEq, Eq)]
pub enum ReportLineToken<'a> {
    Empty,
    Header(&'a str),
    CoreHeading(&'a str),
    Labeled {
        label: &'a str,
        value: &'a str,
    },
    LabeledWithSub {
        label: &'a str,
        sublabel: &'a str,
        value: &'a str,
    },
    Sublabeled {
        indent: &'a str,
        sublabel: &'a str,
        value: &'a str,
    },
    SublabeledHighlight {
        indent: &'a str,
        text: &'a str,
    },
    Plain(&'a str),
}

/// Tokenizes a single line from a formatted CPU report into semantic styling components.
pub fn tokenize_report_line(line: &str) -> ReportLineToken<'_> {
    let line_trimmed = line.trim_end_matches(['\r', '\n']);
    if line_trimmed.trim().is_empty() {
        return ReportLineToken::Empty;
    }

    if line_trimmed.starts_with("---------------")
        || line_trimmed.starts_with("--------------------")
    {
        return ReportLineToken::Header(line_trimmed);
    }

    if line_trimmed.trim_start().starts_with("Core #") {
        return ReportLineToken::CoreHeading(line_trimmed);
    }

    if line_trimmed.len() >= 16 && &line_trimmed[14..16] == ": " {
        let label = &line_trimmed[..14];
        let rest = &line_trimmed[16..];
        if let Some(pos) = rest.find(": ")
            && pos < 12
        {
            return ReportLineToken::LabeledWithSub {
                label,
                sublabel: &rest[..pos],
                value: &rest[pos + 2..],
            };
        } else {
            return ReportLineToken::Labeled { label, value: rest };
        }
    }

    if let Some(sub_rest) = line_trimmed.strip_prefix("                ") {
        let indent = &line_trimmed[..16];
        if let Some(pos) = sub_rest.find(": ") {
            return ReportLineToken::Sublabeled {
                indent,
                sublabel: &sub_rest[..pos],
                value: &sub_rest[pos + 2..],
            };
        } else if sub_rest.starts_with('(') {
            return ReportLineToken::SublabeledHighlight {
                indent,
                text: sub_rest,
            };
        }
    }

    ReportLineToken::Plain(line_trimmed)
}

pub fn parse_text_runs(text: &str, theme: GuiTheme, color: bool) -> (RgbColor, Vec<TextRun>) {
    let palette = if theme.is_dark() {
        &PALETTE_DARK
    } else {
        &PALETTE_LIGHT
    };

    if !color {
        let runs = if text.is_empty() {
            Vec::new()
        } else {
            vec![TextRun {
                offset: 0,
                length: text.len() as u32,
                color: palette.body,
                bold: false,
            }]
        };
        return (palette.background, runs);
    }

    let mut runs = Vec::new();
    let mut current_offset: u32 = 0;

    for line in text.split_inclusive('\n') {
        let line_len = line.len() as u32;

        match tokenize_report_line(line) {
            ReportLineToken::Empty => {}
            ReportLineToken::Header(h) => {
                runs.push(TextRun {
                    offset: current_offset,
                    length: h.len() as u32,
                    color: palette.sublabel,
                    bold: false,
                });
            }
            ReportLineToken::CoreHeading(ch) => {
                runs.push(TextRun {
                    offset: current_offset,
                    length: ch.len() as u32,
                    color: palette.sublabel,
                    bold: true,
                });
            }
            ReportLineToken::Labeled { label, value } => {
                let label_full_len = label.len() as u32 + 2;
                runs.push(TextRun {
                    offset: current_offset,
                    length: label_full_len,
                    color: palette.label,
                    bold: false,
                });
                if !value.is_empty() {
                    runs.push(TextRun {
                        offset: current_offset + label_full_len,
                        length: value.len() as u32,
                        color: palette.body,
                        bold: false,
                    });
                }
            }
            ReportLineToken::LabeledWithSub {
                label,
                sublabel,
                value,
            } => {
                let label_full_len = label.len() as u32 + 2;
                let sub_full_len = sublabel.len() as u32 + 2;
                runs.push(TextRun {
                    offset: current_offset,
                    length: label_full_len,
                    color: palette.label,
                    bold: false,
                });
                runs.push(TextRun {
                    offset: current_offset + label_full_len,
                    length: sub_full_len,
                    color: palette.sublabel,
                    bold: false,
                });
                if !value.is_empty() {
                    runs.push(TextRun {
                        offset: current_offset + label_full_len + sub_full_len,
                        length: value.len() as u32,
                        color: palette.body,
                        bold: false,
                    });
                }
            }
            ReportLineToken::Sublabeled {
                indent,
                sublabel,
                value,
            } => {
                let indent_len = indent.len() as u32;
                let sub_full_len = sublabel.len() as u32 + 2;
                runs.push(TextRun {
                    offset: current_offset,
                    length: indent_len,
                    color: palette.divider,
                    bold: false,
                });
                runs.push(TextRun {
                    offset: current_offset + indent_len,
                    length: sub_full_len,
                    color: palette.sublabel,
                    bold: false,
                });
                if !value.is_empty() {
                    runs.push(TextRun {
                        offset: current_offset + indent_len + sub_full_len,
                        length: value.len() as u32,
                        color: palette.body,
                        bold: false,
                    });
                }
            }
            ReportLineToken::SublabeledHighlight { indent, text: hl_text } => {
                let indent_len = indent.len() as u32;
                runs.push(TextRun {
                    offset: current_offset,
                    length: indent_len,
                    color: palette.divider,
                    bold: false,
                });
                runs.push(TextRun {
                    offset: current_offset + indent_len,
                    length: hl_text.len() as u32,
                    color: palette.highlight,
                    bold: false,
                });
            }
            ReportLineToken::Plain(p) => {
                runs.push(TextRun {
                    offset: current_offset,
                    length: p.len() as u32,
                    color: palette.body,
                    bold: false,
                });
            }
        }

        current_offset += line_len;
    }

    (palette.background, runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_runs_plain() {
        let sample = "--------------- Rustid 2.1.1 ---------------\n        Vendor: AuthenticAMD\n";
        let (bg, runs) = parse_text_runs(sample, GuiTheme::Light, false);
        assert_eq!(bg, PALETTE_LIGHT.background);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].offset, 0);
        assert_eq!(runs[0].length, sample.len() as u32);
        assert_eq!(runs[0].color, PALETTE_LIGHT.body);
    }

    #[test]
    fn test_parse_text_runs_colored_light_and_dark() {
        let sample = "--------------- Rustid 2.1.1 ---------------\n        Vendor: AuthenticAMD\n";
        let (bg_light, runs_light) = parse_text_runs(sample, GuiTheme::Light, true);
        assert_eq!(bg_light, PALETTE_LIGHT.background);
        assert!(runs_light.len() >= 2);

        let (bg_dark, runs_dark) = parse_text_runs(sample, GuiTheme::Dark, true);
        assert_eq!(bg_dark, PALETTE_DARK.background);
        assert_eq!(runs_dark.len(), runs_light.len());
        assert_eq!(runs_dark[0].color, PALETTE_DARK.sublabel);
    }

    #[test]
    fn test_parse_text_runs_sublabel_and_signature() {
        let sample = "     Signature: Family 1Ah, Model 44h\n                (11, 15, 4, 4, 0)\n";
        let (_bg, runs) = parse_text_runs(sample, GuiTheme::Light, true);
        assert!(runs.len() >= 3);
        // Signature tuple should be highlighted
        let has_highlight = runs.iter().any(|r| r.color == PALETTE_LIGHT.highlight);
        assert!(has_highlight);
    }
}
