//! Shared GUI definitions, view modes, and report text generation.

use crate::Cpu;
use crate::common::{CliFlags, TCpuDisplay};

#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ViewMode {
    #[default]
    Standard,
    Debug,
    Everything,
    #[cfg(x86_cpu)]
    Dump,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ReportSource {
    #[default]
    LiveHardware,
    DumpFile,
}

impl ReportSource {
    pub fn is_dump(&self) -> bool {
        matches!(self, Self::DumpFile)
    }
}

impl From<bool> for ReportSource {
    fn from(is_dump: bool) -> Self {
        if is_dump {
            Self::DumpFile
        } else {
            Self::LiveHardware
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum GuiTheme {
    #[default]
    Light,
    Dark,
}

impl GuiTheme {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        };
    }
}

impl From<bool> for GuiTheme {
    fn from(dark: bool) -> Self {
        if dark { Self::Dark } else { Self::Light }
    }
}

#[cfg(windows)]
pub fn normalize_newlines(s: String) -> String {
    s.replace("\r\n", "\n").replace('\n', "\r\n")
}

#[cfg(not(windows))]
pub fn normalize_newlines(s: String) -> String {
    s
}

/// Formats a plain-text CPU report.
pub fn generate_report_plain(cpu: &Cpu, flags: CliFlags, source: ReportSource) -> String {
    let version_header = if source.is_dump() {
        crate::format_file_version()
    } else {
        crate::format_version()
    };
    let sep = if flags.compact {
        NEWLINE
    } else {
        #[cfg(windows)]
        {
            "\r\n\r\n"
        }
        #[cfg(not(windows))]
        {
            "\n\n"
        }
    };
    let table = normalize_newlines(cpu.render_table(flags));
    format!("{}{}{}", version_header, sep, table)
}

/// Formats debug text for a CPU.
pub fn generate_debug_info_plain(cpu: &Cpu) -> String {
    normalize_newlines(cpu.render_debug())
}

/// Formats raw CPUID dump output across all logical cores.
#[cfg(x86_cpu)]
pub fn generate_dump_info_plain() -> String {
    normalize_newlines(crate::x86::dump::dump_all_cpus())
}

/// Builds the plain text according to the specified `ViewMode`.
pub fn build_view_text(cpu: &Cpu, mode: ViewMode, flags: CliFlags, source: ReportSource) -> String {
    match mode {
        ViewMode::Standard => generate_report_plain(cpu, flags, source),
        ViewMode::Debug => generate_debug_info_plain(cpu),
        ViewMode::Everything => {
            let report = generate_report_plain(cpu, flags, source);
            let debug = generate_debug_info_plain(cpu);
            let sep = format!("{NEWLINE}--------------------{NEWLINE}{NEWLINE}");
            format!("{}{}{}", report, sep, debug)
        }
        #[cfg(x86_cpu)]
        ViewMode::Dump => generate_dump_info_plain(),
    }
}

/// Sets up or resets the CPUID dump provider depending on whether file content is provided.
pub fn configure_dump_provider_from_contents(contents: Option<&str>) {
    #[cfg(x86_cpu)]
    if let Some(c) = contents {
        let dump = crate::x86::provider::CpuDump::parse_str(c);
        crate::x86::provider::set_cpuid_provider(dump);
    } else {
        crate::x86::provider::reset_cpuid_provider();
    }
}

/// Formats status bar parts (model/arch/os, dump/hardware source, view mode / color / theme).
pub fn format_status_parts(
    cpu: &Cpu,
    loaded_file: Option<&str>,
    mode: ViewMode,
    flags: CliFlags,
    theme: GuiTheme,
) -> (String, String, String) {
    #[cfg(x86_cpu)]
    let model = cpu.display_model_string();
    #[cfg(not(x86_cpu))]
    let model = if !cpu.model.is_empty() {
        &cpu.model
    } else {
        "CPU"
    };

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let part1 = format!("{} ({}-{})", model, arch, os);
    let part2 = if let Some(path) = loaded_file {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path);
        format!("Dump File: {}", filename)
    } else {
        "Live Hardware".to_string()
    };

    let mode_str = match mode {
        ViewMode::Standard => "Standard",
        ViewMode::Debug => "Debug (-d)",
        ViewMode::Everything => "Everything (-e)",
        #[cfg(x86_cpu)]
        ViewMode::Dump => "CPUID Dump (-r)",
    };

    let part3 = format!(
        "{} | Colors: {} | Theme: {}",
        mode_str,
        if flags.color { "On" } else { "Off" },
        if theme.is_dark() { "Dark" } else { "Light" }
    );

    (part1, part2, part3)
}
