use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use opencode_llm::auth::{AuthMethod, ProviderAuth};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct ConnectMethodDialog {
    selected_index: usize,
    methods: Vec<(String, String)>,
    theme: Theme,
    is_oauth_only: bool,
    show_feedback: bool,
}

impl ConnectMethodDialog {
    pub fn new(theme: Theme, provider_id: String) -> Self {
        let auth_methods = provider_id.supported_auth_methods();
        let supports_browser = auth_methods.contains(&AuthMethod::Browser)
            || auth_methods.contains(&AuthMethod::DeviceFlow);
        let supports_api_key = auth_methods.contains(&AuthMethod::ApiKey);
        let supports_local = auth_methods.contains(&AuthMethod::Local);

        let (methods, is_oauth_only) = if provider_id == "openai" {
            (
                vec![
                    ("browser".to_string(), "Browser auth".to_string()),
                    ("api_key".to_string(), "API key".to_string()),
                ],
                false,
            )
        } else if provider_id == "google" || provider_id == "copilot" {
            (Vec::new(), true)
        } else if supports_local {
            (vec![("local".to_string(), "Local".to_string())], false)
        } else if !supports_api_key && !supports_browser {
            (Vec::new(), false)
        } else {
            let mut methods = Vec::new();
            if supports_browser {
                methods.push(("browser".to_string(), "Browser auth".to_string()));
            }
            if supports_api_key {
                methods.push(("api_key".to_string(), "API key".to_string()));
            }
            (methods, !supports_api_key && supports_browser)
        };

        Self {
            selected_index: 0,
            methods,
            theme,
            is_oauth_only,
            show_feedback: false,
        }
    }
}

impl sealed::Sealed for ConnectMethodDialog {}

impl Dialog for ConnectMethodDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let width = 60.min(area.width.saturating_sub(4));
        let height = if self.methods.is_empty() {
            8.min(area.height.saturating_sub(4))
        } else {
            10.min(area.height.saturating_sub(4))
        };
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);
        let block = Block::default()
            .title(if self.methods.is_empty() {
                "Auth Method Not Available"
            } else {
                "Select Auth Method"
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);

        if self.methods.is_empty() {
            let msg = if self.show_feedback {
                if self.is_oauth_only {
                    "OAuth authentication is not yet implemented.\nPress ESC to go back."
                } else {
                    "No authentication methods available.\nPress Enter to go back."
                }
            } else if self.is_oauth_only {
                "OAuth authentication is not yet implemented.\nPress ESC to go back."
            } else {
                "No authentication methods available.\nPress Enter to go back."
            };
            f.render_widget(
                Paragraph::new(msg).style(Style::default().fg(self.theme.muted_color())),
                inner,
            );
        } else {
            let items: Vec<ListItem> = self
                .methods
                .iter()
                .enumerate()
                .map(|(index, (_, name))| {
                    let style = if index == self.selected_index {
                        Style::default()
                            .fg(self.theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(self.theme.foreground_color())
                    };
                    ListItem::new(Line::from(Span::styled(name.clone(), style)))
                })
                .collect();

            f.render_widget(List::new(items), inner);
        }
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Up => {
                if self.selected_index == 0 {
                    self.selected_index = self.methods.len().saturating_sub(1);
                } else {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                if !self.methods.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.methods.len().max(1);
                }
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.methods.is_empty() {
                    if self.is_oauth_only {
                        self.show_feedback = true;
                        DialogAction::None
                    } else {
                        DialogAction::Close
                    }
                } else {
                    DialogAction::Confirm(self.methods[self.selected_index].0.clone())
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
    fn test_connect_method_dialog_enter_on_empty_closes() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        dialog.methods.clear();
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn test_connect_method_dialog_empty_list_up_does_not_panic() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }

    #[test]
    fn test_connect_method_dialog_single_item_down_stays_at_zero() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        assert_eq!(dialog.selected_index, 0);
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_connect_method_dialog_shows_message_when_empty() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        dialog.methods.clear();
        assert!(dialog.methods.is_empty());
        assert!(dialog.is_oauth_only);
    }

    #[test]
    fn connect_method_dialog_confirms_browser_auth_selection() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "openai".into());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("browser".into()));
    }

    #[test]
    fn api_key_providers_show_api_key_option() {
        let dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        assert_eq!(dialog.methods.len(), 1);
        assert_eq!(dialog.methods[0].0, "api_key");
        assert_eq!(dialog.methods[0].1, "API key");
    }

    #[test]
    fn openai_shows_both_auth_methods() {
        let dialog = ConnectMethodDialog::new(Theme::default(), "openai".into());
        assert_eq!(dialog.methods.len(), 2);
        assert_eq!(dialog.methods[0].0, "browser");
        assert_eq!(dialog.methods[1].0, "api_key");
    }

    #[test]
    fn oauth_only_providers_show_not_yet_implemented() {
        let dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        assert!(dialog.methods.is_empty());
        assert!(dialog.is_oauth_only);
    }

    #[test]
    fn copilot_shows_not_yet_implemented() {
        let dialog = ConnectMethodDialog::new(Theme::default(), "copilot".into());
        assert!(dialog.methods.is_empty());
        assert!(dialog.is_oauth_only);
    }

    #[test]
    fn empty_list_enter_closes_for_non_oauth() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        dialog.methods.clear();
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn empty_list_enter_does_not_close_for_oauth() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        dialog.methods.clear();
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
        assert!(dialog.show_feedback);
    }

    #[test]
    fn empty_list_navigation_does_not_panic() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        dialog.methods.clear();
        dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }

    #[test]
    fn single_item_down_stays_at_zero() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        assert_eq!(dialog.selected_index, 0);
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn api_key_selection_confirms() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("api_key".into()));
    }

    #[test]
    fn google_oauth_only_provider_shows_feedback_on_enter() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
        assert!(dialog.methods.is_empty());
        assert!(dialog.is_oauth_only);
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
        assert!(dialog.show_feedback);
    }

    #[test]
    fn copilot_oauth_only_provider_shows_feedback_on_enter() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "copilot".into());
        assert!(dialog.methods.is_empty());
        assert!(dialog.is_oauth_only);
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
        assert!(dialog.show_feedback);
    }
}
