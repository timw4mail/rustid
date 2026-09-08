//! Plain report formatting and styled text run generation for Haiku's BTextView.

use crate::Cpu;
#[allow(unused_imports)]
use crate::common::{CliFlags, CpuDisplay, Level1Cache, TCpuDisplay, TDetect, UNK};

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

pub fn generate_report_plain(
    cpu: &Cpu,
    verbose: bool,
    compact: bool,
    is_from_dump: bool,
) -> String {
    let version_header = if is_from_dump {
        crate::format_file_version()
    } else {
        crate::format_version()
    };
    let flags = CliFlags {
        color: false,
        compact,
        verbose,
    };
    let sep = if compact { "\n" } else { "\n\n" };
    let table = cpu.render_table(flags);
    format!("{}{}{}", version_header, sep, table)
}

pub fn generate_debug_info_plain(cpu: &Cpu) -> String {
    cpu.render_debug()
}

#[cfg(x86_cpu)]
pub fn generate_dump_info_plain() -> String {
    use crate::x86::{dump::dump_cpu, topology::Topology};
    let mut output = String::new();
    let topo = Topology::detect();
    let logical_cores = topo.threads.count as usize;
    for i in 0..logical_cores {
        dump_cpu(&mut output, i);
    }
    output
}

pub fn parse_text_runs(text: &str, dark_theme: bool, color: bool) -> (RgbColor, Vec<TextRun>) {
    let palette = if dark_theme {
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
        let line_trimmed = line.trim_end_matches(['\r', '\n']);

        if line_trimmed.trim().is_empty() {
            current_offset += line_len;
            continue;
        }

        // Header line (e.g. --------------- Rustid ... ---------------)
        if line_trimmed.starts_with("---------------")
            || line_trimmed.starts_with("--------------------")
        {
            runs.push(TextRun {
                offset: current_offset,
                length: line_trimmed.len() as u32,
                color: palette.sublabel,
                bold: false,
            });
            current_offset += line_len;
            continue;
        }

        // Core # heading line
        if line_trimmed.trim_start().starts_with("Core #") {
            runs.push(TextRun {
                offset: current_offset,
                length: line_trimmed.len() as u32,
                color: palette.sublabel,
                bold: true,
            });
            current_offset += line_len;
            continue;
        }

        // Standard line with labels: e.g. "        Vendor: AMD (AuthenticAMD)"
        if line_trimmed.len() >= 16 && &line_trimmed[14..16] == ": " {
            let label_len = 14u32;
            let colon_len = 2u32;
            let rest_part = &line_trimmed[16..];

            // Label part (Green)
            runs.push(TextRun {
                offset: current_offset,
                length: label_len + colon_len,
                color: palette.label,
                bold: false,
            });

            // Check for inline sublabel e.g. "Frequency: Base: 4.29 GHz"
            if let Some(colon_pos) = rest_part.find(": ")
                && colon_pos < 12
            {
                let sub_label_len = (colon_pos + 2) as u32;
                runs.push(TextRun {
                    offset: current_offset + label_len + colon_len,
                    length: sub_label_len,
                    color: palette.sublabel,
                    bold: false,
                });
                let val_len = (rest_part.len() - colon_pos - 2) as u32;
                if val_len > 0 {
                    runs.push(TextRun {
                        offset: current_offset + label_len + colon_len + sub_label_len,
                        length: val_len,
                        color: palette.body,
                        bold: false,
                    });
                }
            } else {
                let rest_len = rest_part.len() as u32;
                if rest_len > 0 {
                    runs.push(TextRun {
                        offset: current_offset + label_len + colon_len,
                        length: rest_len,
                        color: palette.body,
                        bold: false,
                    });
                }
            }

            current_offset += line_len;
            continue;
        }

        // Sublabel line e.g. "                L1i: ..." or "                (11, 15, ...)"
        if let Some(sub_rest) = line_trimmed.strip_prefix("                ") {
            let indent_len = 16u32;

            if let Some(colon_idx) = sub_rest.find(": ") {
                let sub_lbl_len = (colon_idx + 2) as u32;
                let sub_val_len = (sub_rest.len() - colon_idx - 2) as u32;

                runs.push(TextRun {
                    offset: current_offset,
                    length: indent_len,
                    color: palette.divider,
                    bold: false,
                });
                runs.push(TextRun {
                    offset: current_offset + indent_len,
                    length: sub_lbl_len,
                    color: palette.sublabel,
                    bold: false,
                });
                if sub_val_len > 0 {
                    runs.push(TextRun {
                        offset: current_offset + indent_len + sub_lbl_len,
                        length: sub_val_len,
                        color: palette.body,
                        bold: false,
                    });
                }
            } else if sub_rest.starts_with('(') {
                runs.push(TextRun {
                    offset: current_offset,
                    length: indent_len,
                    color: palette.divider,
                    bold: false,
                });
                runs.push(TextRun {
                    offset: current_offset + indent_len,
                    length: sub_rest.len() as u32,
                    color: palette.highlight,
                    bold: false,
                });
            } else {
                runs.push(TextRun {
                    offset: current_offset,
                    length: line_trimmed.len() as u32,
                    color: palette.body,
                    bold: false,
                });
            }

            current_offset += line_len;
            continue;
        }

        // Default line
        runs.push(TextRun {
            offset: current_offset,
            length: line_trimmed.len() as u32,
            color: palette.body,
            bold: false,
        });

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
        let (bg, runs) = parse_text_runs(sample, false, false);
        assert_eq!(bg, PALETTE_LIGHT.background);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].offset, 0);
        assert_eq!(runs[0].length, sample.len() as u32);
        assert_eq!(runs[0].color, PALETTE_LIGHT.body);
    }

    #[test]
    fn test_parse_text_runs_colored_light_and_dark() {
        let sample = "--------------- Rustid 2.1.1 ---------------\n        Vendor: AuthenticAMD\n";
        let (bg_light, runs_light) = parse_text_runs(sample, false, true);
        assert_eq!(bg_light, PALETTE_LIGHT.background);
        assert!(runs_light.len() >= 2);

        let (bg_dark, runs_dark) = parse_text_runs(sample, true, true);
        assert_eq!(bg_dark, PALETTE_DARK.background);
        assert_eq!(runs_dark.len(), runs_light.len());
        assert_eq!(runs_dark[0].color, PALETTE_DARK.sublabel);
    }

    #[test]
    fn test_parse_text_runs_sublabel_and_signature() {
        let sample = "     Signature: Family 1Ah, Model 44h\n                (11, 15, 4, 4, 0)\n";
        let (_bg, runs) = parse_text_runs(sample, false, true);
        assert!(runs.len() >= 3);
        // Signature tuple should be highlighted
        let has_highlight = runs.iter().any(|r| r.color == PALETTE_LIGHT.highlight);
        assert!(has_highlight);
    }
}
