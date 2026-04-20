pub mod api_key_input;
pub mod connect_method;
pub mod connect_model;
pub mod connect_provider;
pub mod credential_select;
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
pub mod validation_error_dialog;

pub use api_key_input::ApiKeyInputDialog;
pub use connect_method::ConnectMethodDialog;
pub use connect_model::ConnectModelDialog;
pub use connect_provider::ConnectProviderDialog;
pub use credential_select::{CredentialEntry, CredentialSelectDialog};
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
pub use validation_error_dialog::ValidationErrorDialog;

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
    ConfirmModelWithVariant { model_id: String, variant_name: Option<String> },
    Navigate(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_action_none() {
        assert!(matches!(DialogAction::None, DialogAction::None));
    }

    #[test]
    fn test_dialog_action_close() {
        assert!(matches!(DialogAction::Close, DialogAction::Close));
    }

    #[test]
    fn test_dialog_action_confirm() {
        let action = DialogAction::Confirm("test".to_string());
        match action {
            DialogAction::Confirm(s) => assert_eq!(s, "test"),
            _ => panic!("Expected Confirm"),
        }
    }

    #[test]
    fn test_dialog_action_confirm_multiple() {
        let action = DialogAction::ConfirmMultiple(vec!["a".to_string(), "b".to_string()]);
        match action {
            DialogAction::ConfirmMultiple(v) => assert_eq!(v.len(), 2),
            _ => panic!("Expected ConfirmMultiple"),
        }
    }

    #[test]
    fn test_dialog_action_navigate() {
        let action = DialogAction::Navigate("next".to_string());
        match action {
            DialogAction::Navigate(s) => assert_eq!(s, "next"),
            _ => panic!("Expected Navigate"),
        }
    }

    #[test]
    fn test_dialog_action_equality() {
        assert_eq!(DialogAction::None, DialogAction::None);
        assert_eq!(DialogAction::Close, DialogAction::Close);
        assert_eq!(
            DialogAction::Confirm("x".to_string()),
            DialogAction::Confirm("x".to_string())
        );
        assert_ne!(DialogAction::None, DialogAction::Close);
    }

    #[test]
    fn test_dialog_action_clone() {
        let action = DialogAction::Confirm("test".to_string());
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }
}
