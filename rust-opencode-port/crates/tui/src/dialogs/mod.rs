pub mod directory_selection;
pub mod file_selection;
pub mod model_selection;
pub mod provider_management;
pub mod release_notes;
pub mod settings;

pub use directory_selection::DirectorySelectionDialog;
pub use file_selection::FileSelectionDialog;
pub use model_selection::ModelSelectionDialog;
pub use provider_management::ProviderManagementDialog;
pub use release_notes::ReleaseNotesDialog;
pub use settings::SettingsDialog;

use ratatui::Frame;

pub trait Dialog {
    fn draw(&self, f: &mut Frame, area: ratatui::layout::Rect);
    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> DialogAction;
    fn is_modal(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogAction {
    None,
    Close,
    Confirm(String),
    Navigate(String),
}
