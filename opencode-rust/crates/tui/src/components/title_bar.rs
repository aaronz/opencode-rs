use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub message_count: usize,
    pub created_at: String,
}

pub struct TitleBar {
    pub current_session: String,
    sessions: Vec<SessionInfo>,
    pub show_dropdown: bool,
    selected_index: usize,
    theme: Theme,
}

impl TitleBar {
    pub fn new(theme: Theme) -> Self {
        Self {
            current_session: "New Session".to_string(),
            sessions: vec![
                SessionInfo {
                    id: "1".to_string(),
                    name: "Current Session".to_string(),
                    message_count: 12,
                    created_at: "2024-01-15".to_string(),
                },
                SessionInfo {
                    id: "2".to_string(),
                    name: "Project Planning".to_string(),
                    message_count: 45,
                    created_at: "2024-01-14".to_string(),
                },
                SessionInfo {
                    id: "3".to_string(),
                    name: "Code Review".to_string(),
                    message_count: 23,
                    created_at: "2024-01-13".to_string(),
                },
            ],
            show_dropdown: false,
            selected_index: 0,
            theme,
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.show_dropdown = !self.show_dropdown;
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.sessions.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn get_selected_session(&self) -> Option<&SessionInfo> {
        self.sessions.get(self.selected_index)
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let title_text = format!("▾ {}", self.current_session);
        let paragraph = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::BOTTOM))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);

        if self.show_dropdown {
            let dropdown_height = (self.sessions.len() + 2).min(10) as u16;
            let dropdown_area = Rect::new(area.x, area.y + 1, area.width, dropdown_height);
            f.render_widget(Clear, dropdown_area);

            let items: Vec<ListItem> = self
                .sessions
                .iter()
                .map(|session| {
                    ListItem::new(Line::from(vec![
                        Span::raw(format!("{} ", session.name)),
                        Span::styled(
                            format!("({} messages)", session.message_count),
                            Style::default().fg(self.theme.muted_color()),
                        ),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Recent Sessions"),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            let mut state = ratatui::widgets::ListState::default();
            state.select(Some(self.selected_index));
            f.render_stateful_widget(list, dropdown_area, &mut state);
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> TitleBarAction {
        if !self.show_dropdown {
            return TitleBarAction::None;
        }

        match key.code {
            KeyCode::Esc => {
                self.show_dropdown = false;
                TitleBarAction::Close
            }
            KeyCode::Enter => {
                self.show_dropdown = false;
                let session_id = self.get_selected_session().map(|s| s.id.clone());
                let session_name = self.get_selected_session().map(|s| s.name.clone());
                if let (Some(id), Some(name)) = (session_id, session_name) {
                    self.current_session = name;
                    TitleBarAction::Select(id)
                } else {
                    TitleBarAction::Close
                }
            }
            KeyCode::Up => {
                self.select_previous();
                TitleBarAction::None
            }
            KeyCode::Down => {
                self.select_next();
                TitleBarAction::None
            }
            _ => TitleBarAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TitleBarAction {
    None,
    Close,
    Select(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_bar_new() {
        let theme = crate::theme::Theme::default();
        let bar = TitleBar::new(theme);
        assert_eq!(bar.current_session, "New Session");
        assert!(!bar.sessions.is_empty());
    }

    #[test]
    fn test_title_bar_navigation() {
        let theme = crate::theme::Theme::default();
        let mut bar = TitleBar::new(theme);
        bar.select_next();
        assert_eq!(bar.selected_index, 1);
        bar.select_previous();
        assert_eq!(bar.selected_index, 0);
    }
}
