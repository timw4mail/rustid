//! Application state and view mode definitions for the Haiku GUI.

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Standard,
    Debug,
    Everything,
    #[cfg(x86_cpu)]
    Dump,
}

pub struct AppState {
    pub mode: ViewMode,
    pub color: bool,
    pub dark_theme: bool,
    pub verbose: bool,
    pub compact: bool,
    pub loaded_file: Option<String>,
    pub current_plain_text: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: ViewMode::Standard,
            color: true,
            dark_theme: false,
            verbose: false,
            compact: false,
            loaded_file: None,
            current_plain_text: String::new(),
        }
    }
}
