//! Credential selection dialog for picking stored credentials

use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct CredentialSelectDialog {
    provider_id: String,
    credentials: Vec<CredentialEntry>,
    selected_index: usize,
    theme: Theme,
}

pub struct CredentialEntry {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

impl CredentialSelectDialog {
    pub fn new(theme: Theme, provider_id: String) -> Self {
        Self {
            provider_id,
            credentials: Vec::new(),
            selected_index: 0,
            theme,
        }
    }

    pub fn set_credentials(&mut self, credentials: Vec<CredentialEntry>) {
        self.credentials = credentials;
        self.selected_index = 0;
    }

    pub fn credentials(&self) -> &[CredentialEntry] {
        &self.credentials
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn has_credentials(&self) -> bool {
        !self.credentials.is_empty()
    }
}

impl sealed::Sealed for CredentialSelectDialog {}

impl Dialog for CredentialSelectDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let width = 50.min(area.width.saturating_sub(4));
        let height = if self.has_credentials() {
            (self.credentials.len() + 4) as u16
        } else {
            8
        };
        let height = height.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);

        let title = if self.has_credentials() {
            "Select Credential"
        } else {
            "No Credentials"
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);

        if !self.has_credentials() {
            let msg = Paragraph::new(
                "No stored credentials for this provider.\nPress Enter to add a new credential.",
            )
            .style(Style::default().fg(self.theme.muted_color()));
            f.render_widget(msg, inner);
        } else {
            let items: Vec<ListItem> = self
                .credentials
                .iter()
                .enumerate()
                .map(|(index, cred)| {
                    let style = if index == self.selected_index {
                        Style::default()
                            .fg(self.theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(self.theme.foreground_color())
                    };
                    let text = format!("{} (added: {})", cred.name, cred.created_at);
                    ListItem::new(Line::from(Span::styled(text, style)))
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(self.selected_index));
            f.render_stateful_widget(List::new(items), inner, &mut state);
        }
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Up => {
                if self.has_credentials() {
                    if self.selected_index == 0 {
                        self.selected_index = self.credentials.len().saturating_sub(1);
                    } else {
                        self.selected_index -= 1;
                    }
                }
                DialogAction::None
            }
            KeyCode::Down => {
                if self.has_credentials() {
                    self.selected_index = (self.selected_index + 1) % self.credentials.len().max(1);
                }
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.has_credentials() {
                    let cred = &self.credentials[self.selected_index];
                    DialogAction::Confirm(cred.id.clone())
                } else {
                    DialogAction::Close
                }
            }
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_credential_dialog_renders_empty() {
        let dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        assert!(!dialog.has_credentials());
    }

    #[test]
    fn test_credential_dialog_renders_with_credentials() {
        let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        dialog.set_credentials(vec![
            CredentialEntry {
                id: "cred-1".into(),
                name: "work".into(),
                created_at: "2024-01-01".into(),
            },
            CredentialEntry {
                id: "cred-2".into(),
                name: "personal".into(),
                created_at: "2024-01-15".into(),
            },
        ]);
        assert!(dialog.has_credentials());
        assert_eq!(dialog.credentials().len(), 2);
    }

    #[test]
    fn test_credential_dialog_navigation() {
        let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        dialog.set_credentials(vec![CredentialEntry {
            id: "cred-1".into(),
            name: "work".into(),
            created_at: "2024-01-01".into(),
        }]);

        assert_eq!(dialog.selected_index, 0);
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_credential_dialog_select() {
        let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        dialog.set_credentials(vec![CredentialEntry {
            id: "my-cred-id".into(),
            name: "work".into(),
            created_at: "2024-01-01".into(),
        }]);

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("my-cred-id".into()));
    }

    #[test]
    fn test_credential_dialog_close_empty() {
        let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn test_credential_dialog_close_on_escape() {
        let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }
}
