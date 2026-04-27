use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

pub struct DialogSelect {
    title: String,
    message: String,
    options: Vec<String>,
    selected_index: usize,
    scroll_offset: usize,
    theme: Theme,
}

impl DialogSelect {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        options: Vec<String>,
        theme: Theme,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            options,
            selected_index: 0,
            scroll_offset: 0,
            theme,
        }
    }

    pub fn get_selected_option(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }
}

impl sealed::Sealed for DialogSelect {}

impl Dialog for DialogSelect {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = (3 + self.options.len() as u16 + 3).min(area.height.saturating_sub(4));
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

        let message = ratatui::widgets::Paragraph::new(Line::from(self.message.as_str()))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(message, inner_area);

        let list_items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let prefix = if i == self.selected_index { "> " } else { "  " };
                let suffix = if i == self.selected_index { " <" } else { "  " };
                let content = format!("{}{}{}", prefix, opt, suffix);
                ListItem::new(Line::from(content))
            })
            .collect();

        let list_area = Rect::new(
            inner_area.x,
            inner_area.y + 2,
            inner_area.width,
            dialog_height.saturating_sub(6),
        );

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(self.theme.primary_color())
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = ratatui::widgets::ListState::default();
        state = state.with_selected(Some(self.selected_index));
        state = state.with_offset(self.scroll_offset);
        f.render_stateful_widget(list, list_area, &mut state);

        let hint = Line::from(vec![
            Span::styled("Use ", Style::default()),
            Span::styled("↑", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("/", Style::default()),
            Span::styled("↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to select, ", Style::default()),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm, ", Style::default()),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default()),
        ]);
        let hint_paragraph = ratatui::widgets::Paragraph::new(hint)
            .block(Block::default().borders(Borders::NONE))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(
            hint_paragraph,
            Rect::new(
                inner_area.x,
                inner_area.y + dialog_height.saturating_sub(4),
                inner_area.width,
                1,
            ),
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        let visible_height = 10usize;

        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if let Some(selected) = self.get_selected_option() {
                    DialogAction::Confirm(selected.to_string())
                } else {
                    DialogAction::Close
                }
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_sub(if self.selected_index < self.scroll_offset { 1 } else { 0 });
                DialogAction::None
            }
            KeyCode::Down => {
                let max = self.options.len().saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                }
                if self.selected_index >= self.scroll_offset + visible_height {
                    self.scroll_offset = (self.selected_index + 1).saturating_sub(visible_height);
                }
                DialogAction::None
            }
            KeyCode::Home => {
                self.selected_index = 0;
                self.scroll_offset = 0;
                DialogAction::None
            }
            KeyCode::End => {
                self.selected_index = self.options.len().saturating_sub(1);
                if self.selected_index >= self.scroll_offset + visible_height {
                    self.scroll_offset = (self.selected_index + 1).saturating_sub(visible_height);
                }
                DialogAction::None
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
    fn test_dialog_select_new() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let select = DialogSelect::new("Title", "Message", options, theme);
        assert_eq!(select.title, "Title");
        assert_eq!(select.options.len(), 2);
        assert_eq!(select.selected_index, 0);
    }

    #[test]
    fn test_dialog_select_get_selected_option() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let select = DialogSelect::new("Title", "Message", options, theme);
        assert_eq!(select.get_selected_option(), Some("Option 1"));
    }

    #[test]
    fn test_dialog_select_handle_input_up() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        select.selected_index = 1;
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(select.handle_input(key), DialogAction::None);
        assert_eq!(select.selected_index, 0);
    }

    #[test]
    fn test_dialog_select_handle_input_down() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(select.handle_input(key), DialogAction::None);
        assert_eq!(select.selected_index, 1);
    }

    #[test]
    fn test_dialog_select_handle_input_enter() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            select.handle_input(key),
            DialogAction::Confirm("Option 1".to_string())
        );
    }

    #[test]
    fn test_dialog_select_handle_input_escape() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(select.handle_input(key), DialogAction::Close);
    }

    #[test]
    fn test_dialog_select_handle_input_home() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        select.selected_index = 1;
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(select.handle_input(key), DialogAction::None);
        assert_eq!(select.selected_index, 0);
    }

    #[test]
    fn test_dialog_select_handle_input_end() {
        let theme = Theme::default();
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let mut select = DialogSelect::new("Title", "Message", options, theme);
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(select.handle_input(key), DialogAction::None);
        assert_eq!(select.selected_index, 1);
    }
}
