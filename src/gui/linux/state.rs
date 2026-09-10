use crate::common::CliFlags;
pub use crate::gui::common::{GuiTheme, ReportSource, ViewMode};

pub struct AppState {
    pub mode: ViewMode,
    pub flags: CliFlags,
    pub theme: GuiTheme,
    pub loaded_file: Option<String>,
    pub current_plain_text: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: ViewMode::Standard,
            flags: CliFlags {
                compact: false,
                color: true,
                verbose: false,
            },
            theme: GuiTheme::Light,
            loaded_file: None,
            current_plain_text: String::new(),
        }
    }
}
