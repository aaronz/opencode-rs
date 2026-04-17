use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::error::Error;
use std::fmt;

pub struct ApiKeyInputDialog {
    provider_id: String,
    #[allow(dead_code)]
    provider_name: String,
    title: String,
    message: String,
    input: String,
    cursor_position: usize,
    theme: Theme,
    validation_error: Option<String>,
    show_password: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiKeyValidationError {
    Empty,
    TooShort,
    ContainsWhitespace,
    InvalidFormat(String),
}

impl fmt::Display for ApiKeyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyValidationError::Empty => write!(f, "API key cannot be empty"),
            ApiKeyValidationError::TooShort => {
                write!(f, "API key is too short (minimum 10 characters)")
            }
            ApiKeyValidationError::ContainsWhitespace => {
                write!(f, "API key cannot contain whitespace")
            }
            ApiKeyValidationError::InvalidFormat(msg) => {
                write!(f, "Invalid API key format: {}", msg)
            }
        }
    }
}

impl Error for ApiKeyValidationError {}

impl ApiKeyInputDialog {
    pub fn new(theme: Theme, provider_id: String, provider_name: String) -> Self {
        let message = format!(
            "Enter your {} API key.\nYour key will be stored securely.",
            provider_name
        );
        Self {
            provider_id,
            provider_name,
            title: "Enter API Key".to_string(),
            message,
            input: String::new(),
            cursor_position: 0,
            theme,
            validation_error: None,
            show_password: false,
        }
    }

    pub fn get_api_key(&self) -> &str {
        &self.input
    }

    pub fn get_provider_id(&self) -> &str {
        &self.provider_id
    }

    fn masked_text(&self) -> String {
        if self.input.is_empty() {
            String::new()
        } else if self.show_password {
            self.input.clone()
        } else {
            "•".repeat(self.input.len())
        }
    }

    fn validate_api_key(key: &str) -> Result<(), ApiKeyValidationError> {
        if key.is_empty() {
            return Err(ApiKeyValidationError::Empty);
        }
        if key.len() < 10 {
            return Err(ApiKeyValidationError::TooShort);
        }
        if key.chars().any(|c| c.is_whitespace()) {
            return Err(ApiKeyValidationError::ContainsWhitespace);
        }
        Ok(())
    }

    fn is_placeholder(&self) -> bool {
        self.input.is_empty()
    }
}

impl sealed::Sealed for ApiKeyInputDialog {}

impl Dialog for ApiKeyInputDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = if self.validation_error.is_some() {
            12
        } else {
            10
        };
        let dialog_height = dialog_height.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let message = Paragraph::new(Line::from(self.message.as_str()))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(message, inner_area);

        let text_style = if self.is_placeholder() {
            Style::default().fg(self.theme.muted_color())
        } else {
            Style::default().fg(self.theme.foreground_color())
        };

        let cursor_char = if self.is_placeholder() {
            ' '
        } else {
            self.input.chars().nth(self.cursor_position).unwrap_or(' ')
        };

        let masked_len = self.masked_text().len();
        let before_cursor_len = self.cursor_position.min(masked_len);

        let before_cursor_text = &self.masked_text()[..before_cursor_len];
        let after_cursor_text = &self.masked_text()[before_cursor_len..];

        let input_line = Line::from(vec![
            Span::styled(before_cursor_text, text_style),
            Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(self.theme.primary_color())
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(after_cursor_text, text_style),
        ]);

        let input_paragraph = Paragraph::new(input_line)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_color())),
            )
            .alignment(Alignment::Left);
        f.render_widget(
            input_paragraph,
            Rect::new(inner_area.x, inner_area.y + 3, inner_area.width, 3),
        );

        if let Some(ref error) = self.validation_error {
            let error_line = Line::from(vec![
                Span::styled("Error: ", Style::default().fg(self.theme.error_color())),
                Span::styled(
                    error.as_str(),
                    Style::default().fg(self.theme.error_color()),
                ),
            ]);
            let error_paragraph = Paragraph::new(error_line)
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Center);
            f.render_widget(
                error_paragraph,
                Rect::new(inner_area.x, inner_area.y + 7, inner_area.width, 1),
            );
        }

        let hint = Line::from(vec![
            Span::styled("Type to input, ", Style::default()),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to save, ", Style::default()),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel, ", Style::default()),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to toggle visibility"),
        ]);
        let hint_paragraph = Paragraph::new(hint)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        f.render_widget(
            hint_paragraph,
            Rect::new(
                inner_area.x,
                inner_area.y + dialog_height.saturating_sub(2),
                inner_area.width,
                1,
            ),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                self.validation_error = None;
                match Self::validate_api_key(&self.input) {
                    Ok(()) => DialogAction::Confirm(self.input.clone()),
                    Err(e) => {
                        self.validation_error = Some(e.to_string());
                        DialogAction::None
                    }
                }
            }
            KeyCode::Tab => {
                self.show_password = !self.show_password;
                DialogAction::None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                DialogAction::None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                DialogAction::None
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                DialogAction::None
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
                DialogAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                    self.validation_error = None;
                }
                DialogAction::None
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                    self.validation_error = None;
                }
                DialogAction::None
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return DialogAction::None;
                }
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.validation_error = None;
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_dialog() -> ApiKeyInputDialog {
        ApiKeyInputDialog::new(Theme::default(), "openai".to_string(), "OpenAI".to_string())
    }

    #[test]
    fn test_dialog_renders_with_empty_input() {
        let dialog = create_dialog();
        assert!(dialog.input.is_empty());
        assert!(dialog.is_placeholder());
    }

    #[test]
    fn test_dialog_handles_char_input() {
        let mut dialog = create_dialog();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.input, "a");
        assert_eq!(dialog.cursor_position, 1);
    }

    #[test]
    fn test_dialog_handles_backspace() {
        let mut dialog = create_dialog();
        dialog.input = "test".to_string();
        dialog.cursor_position = 4;
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.input, "tes");
        assert_eq!(dialog.cursor_position, 3);
    }

    #[test]
    fn test_dialog_validates_empty_key() {
        let mut dialog = create_dialog();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.validation_error.is_some());
        assert_eq!(dialog.validation_error.unwrap(), "API key cannot be empty");
    }

    #[test]
    fn test_dialog_validates_too_short_key() {
        let mut dialog = create_dialog();
        dialog.input = "short".to_string();
        dialog.cursor_position = 5;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.validation_error.is_some());
        assert_eq!(
            dialog.validation_error.unwrap(),
            "API key is too short (minimum 10 characters)"
        );
    }

    #[test]
    fn test_dialog_validates_whitespace_in_key() {
        let mut dialog = create_dialog();
        dialog.input = "sk-test key".to_string();
        dialog.cursor_position = 10;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.validation_error.is_some());
        assert_eq!(
            dialog.validation_error.unwrap(),
            "API key cannot contain whitespace"
        );
    }

    #[test]
    fn test_dialog_accepts_valid_key() {
        let mut dialog = create_dialog();
        dialog.input = "sk-valid_api_key_12345".to_string();
        dialog.cursor_position = dialog.input.len();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            dialog.handle_input(key),
            DialogAction::Confirm("sk-valid_api_key_12345".to_string())
        );
    }

    #[test]
    fn test_dialog_closes_on_escape() {
        let mut dialog = create_dialog();
        dialog.input = "some_input".to_string();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_toggles_password_visibility() {
        let mut dialog = create_dialog();
        assert!(!dialog.show_password);
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.show_password);
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(!dialog.show_password);
    }

    #[test]
    fn test_masked_text_shows_dots() {
        let mut dialog = create_dialog();
        dialog.input = "secret".to_string();
        assert_eq!(dialog.masked_text(), "••••••");
    }

    #[test]
    fn test_masked_text_shows_plain_when_revealed() {
        let mut dialog = create_dialog();
        dialog.input = "secret".to_string();
        dialog.show_password = true;
        assert_eq!(dialog.masked_text(), "secret");
    }

    #[test]
    fn test_dialog_navigation_keys() {
        let mut dialog = create_dialog();
        dialog.input = "test".to_string();
        dialog.cursor_position = 2;

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.cursor_position, 1);

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.cursor_position, 2);

        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.cursor_position, 0);

        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert_eq!(dialog.cursor_position, 4);
    }

    #[test]
    fn test_dialog_clear_validation_error_on_input() {
        let mut dialog = create_dialog();
        dialog.input = "short".to_string();
        dialog.cursor_position = 5;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.validation_error.is_some());

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.validation_error.is_none());
    }

    #[test]
    fn test_dialog_get_provider_id() {
        let dialog = create_dialog();
        assert_eq!(dialog.get_provider_id(), "openai");
    }

    #[test]
    fn test_dialog_get_api_key() {
        let mut dialog = create_dialog();
        dialog.input = "sk-test12345678".to_string();
        assert_eq!(dialog.get_api_key(), "sk-test12345678");
    }
}
