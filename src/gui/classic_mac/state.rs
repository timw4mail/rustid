use alloc::string::String;
use crate::common::CliFlags;
pub use crate::gui::common::{GuiTheme, ViewMode};

pub struct AppState {
    pub mode: ViewMode,
    pub flags: CliFlags,
    pub theme: GuiTheme,
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
            current_plain_text: String::new(),
        }
    }
}
