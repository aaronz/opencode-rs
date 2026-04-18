use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct ValidationErrorDialog {
    title: String,
    message: String,
    selected_try_again: bool,
    theme: Theme,
}

impl ValidationErrorDialog {
    pub fn new(title: String, message: String, theme: Theme) -> Self {
        Self {
            title,
            message,
            selected_try_again: true,
            theme,
        }
    }

    pub fn from_validation_error(error_message: &str, provider_name: &str, theme: Theme) -> Self {
        let title = "API Key Validation Failed".to_string();
        let message = format!(
            "{}\n\nYour API key for {} was not accepted.\nPlease check your key and try again.",
            error_message, provider_name
        );
        Self::new(title, message, theme)
    }
}

impl sealed::Sealed for ValidationErrorDialog {}

impl Dialog for ValidationErrorDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 12.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.error_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let error_style = Style::default().fg(self.theme.error_color());
        let message_lines: Vec<Line> = self
            .message
            .lines()
            .map(|line| Line::from(Span::styled(line, error_style)))
            .collect();
        let text = Paragraph::new(message_lines)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        f.render_widget(
            text,
            Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 4),
        );

        let try_again_style = if self.selected_try_again {
            Style::default()
                .fg(Color::Black)
                .bg(self.theme.primary_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.muted_color())
        };

        let cancel_style = if !self.selected_try_again {
            Style::default()
                .fg(Color::Black)
                .bg(self.theme.secondary_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.muted_color())
        };

        let buttons = Line::from(vec![
            Span::styled("[ ", Style::default()),
            Span::styled("Try Again", try_again_style),
            Span::styled(" ]", Style::default()),
            Span::raw("   "),
            Span::styled("[ ", Style::default()),
            Span::styled("Cancel", cancel_style),
            Span::styled(" ]", Style::default()),
        ]);

        let buttons_paragraph = Paragraph::new(buttons)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        f.render_widget(
            buttons_paragraph,
            Rect::new(inner_area.x, inner_area.y + 6, inner_area.width, 1),
        );

        let hint = Line::from(vec![
            Span::styled("Use ", Style::default()),
            Span::styled("←", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("/", Style::default()),
            Span::styled("→", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to select, ", Style::default()),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm, ", Style::default()),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default()),
        ]);
        let hint_paragraph = Paragraph::new(hint)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        f.render_widget(
            hint_paragraph,
            Rect::new(inner_area.x, inner_area.y + 8, inner_area.width, 1),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Left => {
                self.selected_try_again = true;
                DialogAction::None
            }
            KeyCode::Right => {
                self.selected_try_again = false;
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.selected_try_again {
                    DialogAction::Confirm("retry".to_string())
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
    use crossterm::event::KeyModifiers;

    fn create_dialog() -> ValidationErrorDialog {
        ValidationErrorDialog::new(
            "Test Title".to_string(),
            "Test error message".to_string(),
            Theme::default(),
        )
    }

    #[test]
    fn test_dialog_new() {
        let dialog = create_dialog();
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test error message");
        assert!(dialog.selected_try_again);
    }

    #[test]
    fn test_dialog_from_validation_error() {
        let theme = Theme::default();
        let dialog =
            ValidationErrorDialog::from_validation_error("Invalid API key", "OpenAI", theme);
        assert_eq!(dialog.title, "API Key Validation Failed");
        assert!(dialog.message.contains("Invalid API key"));
        assert!(dialog.message.contains("OpenAI"));
    }

    #[test]
    fn test_dialog_handles_left_arrow() {
        let mut dialog = create_dialog();
        dialog.selected_try_again = false;
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(dialog.selected_try_again);
    }

    #[test]
    fn test_dialog_handles_right_arrow() {
        let mut dialog = create_dialog();
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
        assert!(!dialog.selected_try_again);
    }

    #[test]
    fn test_dialog_handles_escape() {
        let mut dialog = create_dialog();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_handles_enter_try_again() {
        let mut dialog = create_dialog();
        assert!(dialog.selected_try_again);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            dialog.handle_input(key),
            DialogAction::Confirm("retry".to_string())
        );
    }

    #[test]
    fn test_dialog_handles_enter_cancel() {
        let mut dialog = create_dialog();
        dialog.selected_try_again = false;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_handles_other_keys() {
        let mut dialog = create_dialog();
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(dialog.handle_input(key), DialogAction::None);
    }
}
