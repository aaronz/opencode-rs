pub mod connect_method;
pub mod connect_model;
pub mod connect_provider;
pub mod dialog_alert;
pub mod dialog_confirm;
pub mod dialog_prompt;
pub mod dialog_select;
pub mod diff_review;
pub mod directory_selection;
pub mod file_selection;
pub mod home_view;
pub use home_view::{HomeAction, HomeView, HomeViewSection};
pub mod model_selection;
pub mod provider_management;
pub mod release_notes;
pub mod settings;
pub mod slash_command;

pub use connect_method::ConnectMethodDialog;
pub use connect_model::ConnectModelDialog;
pub use connect_provider::ConnectProviderDialog;
pub use dialog_alert::DialogAlert;
pub use dialog_confirm::DialogConfirm;
pub use dialog_prompt::DialogPrompt;
pub use dialog_select::DialogSelect;
pub use diff_review::{DiffAction, DiffReviewOverlay, DiffState};
pub use directory_selection::DirectorySelectionDialog;
pub use file_selection::FileSelectionDialog;
pub use model_selection::{ModelInfo, ModelSelectionDialog};
pub use provider_management::{ProviderInfo, ProviderManagementDialog, ProviderStatus};
pub use release_notes::ReleaseNotesDialog;
pub use settings::SettingsDialog;
pub use slash_command::SlashCommandOverlay;

use ratatui::Frame;

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub trait Dialog: sealed::Sealed {
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
    ConfirmMultiple(Vec<String>),
    Navigate(String),
}
