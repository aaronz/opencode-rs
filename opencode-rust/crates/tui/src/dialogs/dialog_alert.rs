use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct DialogAlert {
    title: String,
    message: String,
    theme: Theme,
}

impl DialogAlert {
    pub fn new(title: impl Into<String>, message: impl Into<String>, theme: Theme) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            theme,
        }
    }
}

impl sealed::Sealed for DialogAlert {}

impl Dialog for DialogAlert {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 40.min(area.width.saturating_sub(4));
        let dialog_height = 8.min(area.height.saturating_sub(4));
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

        let ok_text = Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" or ", Style::default()),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to dismiss", Style::default()),
        ]);
        let ok_paragraph = Paragraph::new(ok_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(
            ok_paragraph,
            Rect::new(inner_area.x, inner_area.y + 3, inner_area.width, 1),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => DialogAction::Close,
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_dialog_alert_new() {
        let theme = Theme::default();
        let alert = DialogAlert::new("Title", "Message", theme);
        assert_eq!(alert.title, "Title");
        assert_eq!(alert.message, "Message");
    }

    #[test]
    fn test_dialog_alert_handle_input_escape() {
        let theme = Theme::default();
        let mut alert = DialogAlert::new("Title", "Message", theme);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(alert.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_alert_handle_input_enter() {
        let theme = Theme::default();
        let mut alert = DialogAlert::new("Title", "Message", theme);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(alert.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_alert_handle_input_other() {
        let theme = Theme::default();
        let mut alert = DialogAlert::new("Title", "Message", theme);
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(alert.handle_input(key), DialogAction::None);
    }
}
