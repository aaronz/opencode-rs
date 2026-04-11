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

pub struct DialogPrompt {
    title: String,
    message: String,
    placeholder: String,
    input: String,
    cursor_position: usize,
    theme: Theme,
}

impl DialogPrompt {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        placeholder: impl Into<String>,
        theme: Theme,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            placeholder: placeholder.into(),
            input: String::new(),
            cursor_position: 0,
            theme,
        }
    }

    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.input = value.into();
        self.cursor_position = self.input.len();
        self
    }

    pub fn get_value(&self) -> &str {
        &self.input
    }

    fn display_text(&self) -> String {
        if self.input.is_empty() {
            self.placeholder.clone()
        } else {
            self.input.clone()
        }
    }

    fn is_placeholder(&self) -> bool {
        self.input.is_empty()
    }
}

impl Dialog for DialogPrompt {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(4));
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

        let display_text = self.display_text();
        let placeholder_style = if self.is_placeholder() {
            Style::default().fg(self.theme.muted_color())
        } else {
            Style::default().fg(self.theme.foreground_color())
        };

        let cursor_char = if self.is_placeholder() {
            ' '
        } else {
            self.input.chars().nth(self.cursor_position).unwrap_or(' ')
        };

        let before_cursor = &display_text[..self.cursor_position.min(display_text.len())];
        let after_cursor = &display_text[self.cursor_position.min(display_text.len())..];

        let input_line = Line::from(vec![
            Span::styled(before_cursor, placeholder_style),
            Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(self.theme.primary_color())
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(after_cursor, placeholder_style),
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
            Rect::new(inner_area.x, inner_area.y + 2, inner_area.width, 3),
        );

        let hint = Line::from(vec![
            Span::styled("Type to input, ", Style::default()),
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
            Rect::new(inner_area.x, inner_area.y + 6, inner_area.width, 1),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if self.input.is_empty() {
                    DialogAction::Close
                } else {
                    DialogAction::Confirm(self.input.clone())
                }
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
                }
                DialogAction::None
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                }
                DialogAction::None
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return DialogAction::None;
                }
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_prompt_new() {
        let theme = Theme::default();
        let prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        assert_eq!(prompt.title, "Title");
        assert_eq!(prompt.message, "Message");
        assert!(prompt.input.is_empty());
        assert_eq!(prompt.cursor_position, 0);
    }

    #[test]
    fn test_dialog_prompt_with_default_value() {
        let theme = Theme::default();
        let prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme)
            .with_default_value("default");
        assert_eq!(prompt.input, "default");
        assert_eq!(prompt.cursor_position, 7);
    }

    #[test]
    fn test_dialog_prompt_handle_input_char() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::None);
        assert_eq!(prompt.input, "a");
        assert_eq!(prompt.cursor_position, 1);
    }

    #[test]
    fn test_dialog_prompt_handle_input_backspace() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        prompt.input = "test".to_string();
        prompt.cursor_position = 4;
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::None);
        assert_eq!(prompt.input, "tes");
        assert_eq!(prompt.cursor_position, 3);
    }

    #[test]
    fn test_dialog_prompt_handle_input_enter() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        prompt.input = "test".to_string();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            prompt.handle_input(key),
            DialogAction::Confirm("test".to_string())
        );
    }

    #[test]
    fn test_dialog_prompt_handle_input_enter_empty() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        assert!(prompt.input.is_empty());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_prompt_handle_input_escape() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        prompt.input = "test".to_string();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_prompt_handle_input_left() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        prompt.input = "test".to_string();
        prompt.cursor_position = 3;
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::None);
        assert_eq!(prompt.cursor_position, 2);
    }

    #[test]
    fn test_dialog_prompt_handle_input_right() {
        let theme = Theme::default();
        let mut prompt = DialogPrompt::new("Title", "Message", "Placeholder", theme);
        prompt.input = "test".to_string();
        prompt.cursor_position = 2;
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(prompt.handle_input(key), DialogAction::None);
        assert_eq!(prompt.cursor_position, 3);
    }
}
