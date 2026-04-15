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

pub struct DialogConfirm {
    title: String,
    message: String,
    confirm_label: String,
    cancel_label: String,
    selected_confirm: bool,
    theme: Theme,
}

impl DialogConfirm {
    pub fn new(title: impl Into<String>, message: impl Into<String>, theme: Theme) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            selected_confirm: true,
            theme,
        }
    }

    pub fn with_labels(
        mut self,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        self.confirm_label = confirm_label.into();
        self.cancel_label = cancel_label.into();
        self
    }

    pub fn is_confirmed(&self) -> bool {
        self.selected_confirm
    }
}

impl sealed::Sealed for DialogConfirm {}

impl Dialog for DialogConfirm {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 40.min(area.width.saturating_sub(4));
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

        let text = Paragraph::new(Line::from(self.message.as_str()))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(text, inner_area);

        let confirm_style = if self.selected_confirm {
            Style::default()
                .fg(Color::Black)
                .bg(self.theme.primary_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.muted_color())
        };

        let cancel_style = if !self.selected_confirm {
            Style::default()
                .fg(Color::Black)
                .bg(self.theme.secondary_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.muted_color())
        };

        let buttons = Line::from(vec![
            Span::styled("[ ", Style::default()),
            Span::styled(&self.confirm_label, confirm_style),
            Span::styled(" ]", Style::default()),
            Span::raw("   "),
            Span::styled("[ ", Style::default()),
            Span::styled(&self.cancel_label, cancel_style),
            Span::styled(" ]", Style::default()),
        ]);

        let buttons_paragraph = Paragraph::new(buttons)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        f.render_widget(
            buttons_paragraph,
            Rect::new(inner_area.x, inner_area.y + 3, inner_area.width, 1),
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
            Rect::new(inner_area.x, inner_area.y + 5, inner_area.width, 1),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Left => {
                self.selected_confirm = true;
                DialogAction::None
            }
            KeyCode::Right => {
                self.selected_confirm = false;
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.selected_confirm {
                    DialogAction::Confirm("confirm".to_string())
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

    #[test]
    fn test_dialog_confirm_new() {
        let theme = Theme::default();
        let confirm = DialogConfirm::new("Title", "Message", theme);
        assert_eq!(confirm.title, "Title");
        assert_eq!(confirm.message, "Message");
        assert!(confirm.selected_confirm);
    }

    #[test]
    fn test_dialog_confirm_with_labels() {
        let theme = Theme::default();
        let confirm = DialogConfirm::new("Title", "Message", theme).with_labels("Yes", "No");
        assert_eq!(confirm.confirm_label, "Yes");
        assert_eq!(confirm.cancel_label, "No");
    }

    #[test]
    fn test_dialog_confirm_handle_input_left() {
        let theme = Theme::default();
        let mut confirm = DialogConfirm::new("Title", "Message", theme);
        confirm.selected_confirm = false;
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(confirm.handle_input(key), DialogAction::None);
        assert!(confirm.selected_confirm);
    }

    #[test]
    fn test_dialog_confirm_handle_input_right() {
        let theme = Theme::default();
        let mut confirm = DialogConfirm::new("Title", "Message", theme);
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(confirm.handle_input(key), DialogAction::None);
        assert!(!confirm.selected_confirm);
    }

    #[test]
    fn test_dialog_confirm_handle_input_escape() {
        let theme = Theme::default();
        let mut confirm = DialogConfirm::new("Title", "Message", theme);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(confirm.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_confirm_handle_input_enter_confirm() {
        let theme = Theme::default();
        let mut confirm = DialogConfirm::new("Title", "Message", theme);
        assert!(confirm.selected_confirm);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            confirm.handle_input(key),
            DialogAction::Confirm("confirm".to_string())
        );
    }

    #[test]
    fn test_dialog_confirm_handle_input_enter_cancel() {
        let theme = Theme::default();
        let mut confirm = DialogConfirm::new("Title", "Message", theme);
        confirm.selected_confirm = false;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(confirm.handle_input(key), DialogAction::Close);
    }
}
