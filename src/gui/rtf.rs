//! Plain report formatting and syntax-colored RTF generation.

use rustid::Cpu;
#[allow(unused_imports)]
use rustid::common::{CliFlags, CpuDisplay, Level1Cache, TCpuDisplay, TDetect, UNK};

pub fn generate_report_plain(
    cpu: &Cpu,
    verbose: bool,
    compact: bool,
    is_from_dump: bool,
) -> String {
    let version_header = if is_from_dump {
        rustid::format_file_version()
    } else {
        rustid::format_version()
    };
    let flags = CliFlags {
        color: false,
        compact,
        verbose,
    };
    let sep = if compact { "\r\n" } else { "\r\n\r\n" };
    let table = cpu.render_table(flags);
    let crlf_table = table.replace("\r\n", "\n").replace('\n', "\r\n");
    format!("{}{}{}", version_header, sep, crlf_table)
}

pub fn generate_debug_info_plain(cpu: &Cpu) -> String {
    cpu.render_debug()
        .replace("\r\n", "\n")
        .replace('\n', "\r\n")
}

#[cfg(x86_cpu)]
pub fn generate_dump_info_plain() -> String {
    use rustid::x86::{dump::dump_cpu, topology::Topology};
    let mut output = String::new();
    let topo = Topology::detect();
    let logical_cores = topo.threads.count as usize;
    for i in 0..logical_cores {
        dump_cpu(&mut output, i);
    }
    output.replace("\r\n", "\n").replace('\n', "\r\n")
}

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

pub fn to_rtf(plain_text: &str, dark_theme: bool, color: bool) -> String {
    let font_tbl = "{\\fonttbl{\\f0\\fmodern\\fprq1\\fcharset0 Consolas;}{\\f1\\fmodern\\fprq1\\fcharset0 Courier New;}}";

    if !color {
        let escaped = rtf_escape(plain_text);
        return if dark_theme {
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
    let color_tbl = if dark_theme {
        "{\\colortbl ;\\red115\\green218\\blue202;\\red125\\green207\\blue255;\\red212\\green212\\blue212;\\red255\\green158\\blue100;\\red86\\green95\\blue137;}"
    } else {
        "{\\colortbl ;\\red9\\green134\\blue88;\\red4\\green81\\blue165;\\red30\\green30\\blue30;\\red163\\green21\\blue21;\\red110\\green118\\blue129;}"
    };

    let mut rtf = format!(
        "{{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat{font_tbl}{color_tbl}\\viewkind4\\uc1\\f0\\f1\\fs22 "
    );

    for line in plain_text.lines() {
        if line.trim().is_empty() {
            rtf.push_str("\\par\r\n");
            continue;
        }

        // Header line (e.g. --------------- Rustid ... ---------------)
        if line.starts_with("---------------") || line.starts_with("--------------------") {
            rtf.push_str(&format!("\\cf2 {}\\par\r\n", rtf_escape(line)));
            continue;
        }

        // Core # heading line
        if line.trim_start().starts_with("Core #") {
            rtf.push_str(&format!("\\cf2\\b {}\\b0\\par\r\n", rtf_escape(line)));
            continue;
        }

        // Standard line with labels: e.g. "        Vendor: AMD (AuthenticAMD)"
        if line.len() >= 16 && &line[14..16] == ": " {
            let label_part = &line[..14];
            let rest_part = &line[16..];

            // Check for inline sublabel e.g. "Frequency: Base: "
            if let Some(colon_pos) = rest_part.find(": ")
                && colon_pos < 12
            {
                let sub_label = &rest_part[..colon_pos];
                let val = &rest_part[colon_pos + 2..];
                rtf.push_str(&format!(
                    "\\cf1 {}: \\cf2 {}: \\cf3 {}\\par\r\n",
                    rtf_escape(label_part),
                    rtf_escape(sub_label),
                    rtf_escape(val)
                ));
                continue;
            }

            rtf.push_str(&format!(
                "\\cf1 {}: \\cf3 {}\\par\r\n",
                rtf_escape(label_part),
                rtf_escape(rest_part)
            ));
            continue;
        }

        // Sublabel line e.g. "                L1i: ..." or "                (11, 15, ...)"
        if let Some(sub_rest) = line.strip_prefix("                ") {
            if let Some(colon_idx) = sub_rest.find(": ") {
                let sub_lbl = &sub_rest[..colon_idx];
                let sub_val = &sub_rest[colon_idx + 2..];
                rtf.push_str(&format!(
                    "\\cf5                 \\cf2 {}: \\cf3 {}\\par\r\n",
                    rtf_escape(sub_lbl),
                    rtf_escape(sub_val)
                ));
                continue;
            } else if sub_rest.starts_with('(') {
                rtf.push_str(&format!(
                    "\\cf5                 \\cf4 {}\\par\r\n",
                    rtf_escape(sub_rest)
                ));
                continue;
            }
        }

        // Default line
        rtf.push_str(&format!("\\cf3 {}\\par\r\n", rtf_escape(line)));
    }

    rtf.push('}');
    rtf
}
