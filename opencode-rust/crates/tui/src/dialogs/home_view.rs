use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::session::{Session, SessionManager};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HomeViewSection {
    QuickActions,
    RecentSessions,
}

pub struct HomeView {
    theme: Theme,
    selected_action: HomeAction,
    focused_section: HomeViewSection,
    selected_session_index: Option<usize>,
    recent_sessions: Vec<Session>,
    model: String,
    directory: String,
    total_sessions: usize,
    total_messages: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HomeAction {
    NewSession,
    ContinueLast,
    ViewSessions,
    Settings,
    Quit,
}

impl HomeAction {
    pub fn all() -> Vec<Self> {
        vec![
            Self::NewSession,
            Self::ContinueLast,
            Self::ViewSessions,
            Self::Settings,
            Self::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::NewSession => "New Session",
            Self::ContinueLast => "Continue Last Session",
            Self::ViewSessions => "View All Sessions",
            Self::Settings => "Settings",
            Self::Quit => "Quit",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Self::NewSession => "n",
            Self::ContinueLast => "c",
            Self::ViewSessions => "s",
            Self::Settings => ",",
            Self::Quit => "q",
        }
    }
}

impl HomeView {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            selected_action: HomeAction::NewSession,
            focused_section: HomeViewSection::QuickActions,
            selected_session_index: None,
            recent_sessions: Vec::new(),
            model: String::new(),
            directory: String::new(),
            total_sessions: 0,
            total_messages: 0,
        }
    }

    pub fn with_session_manager(mut self, session_manager: &SessionManager) -> Self {
        self.recent_sessions = session_manager
            .list()
            .into_iter()
            .take(5)
            .cloned()
            .collect();
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_directory(mut self, directory: String) -> Self {
        self.directory = directory;
        self
    }

    pub fn get_selected_action(&self) -> HomeAction {
        self.selected_action
    }

    pub fn move_selection(&mut self, delta: i32) {
        let actions = HomeAction::all();
        let current_idx = actions
            .iter()
            .position(|&a| a == self.selected_action)
            .unwrap_or(0);
        let new_idx = ((current_idx as i32) + delta).max(0) as usize;
        let new_idx = new_idx.min(actions.len() - 1);
        self.selected_action = actions[new_idx];
    }

    pub fn update_from_session_manager(&mut self, session_manager: &SessionManager) {
        self.recent_sessions = session_manager
            .list()
            .into_iter()
            .take(5)
            .cloned()
            .collect();
        self.total_sessions = session_manager.len();
        self.total_messages = session_manager.list().iter().map(|s| s.message_count).sum();
        if self.selected_session_index.is_none() && !self.recent_sessions.is_empty() {
            self.selected_session_index = Some(0);
        }
        if self
            .selected_session_index
            .map(|i| i >= self.recent_sessions.len())
            .unwrap_or(false)
        {
            self.selected_session_index = self.recent_sessions.len().checked_sub(1);
        }
    }

    pub fn get_focused_section(&self) -> HomeViewSection {
        self.focused_section
    }

    pub fn set_focused_section(&mut self, section: HomeViewSection) {
        self.focused_section = section;
    }

    pub fn switch_section(&mut self) {
        self.focused_section = match self.focused_section {
            HomeViewSection::QuickActions => HomeViewSection::RecentSessions,
            HomeViewSection::RecentSessions => HomeViewSection::QuickActions,
        };
    }

    pub fn get_selected_session_index(&self) -> Option<usize> {
        self.selected_session_index
    }

    pub fn move_session_selection(&mut self, delta: i32) {
        if self.recent_sessions.is_empty() {
            return;
        }
        let current_idx = self.selected_session_index.unwrap_or(0);
        let new_idx = ((current_idx as i32) + delta).max(0) as usize;
        let new_idx = new_idx.min(self.recent_sessions.len() - 1);
        self.selected_session_index = Some(new_idx);
    }
}

impl sealed::Sealed for HomeView {}

impl Dialog for HomeView {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let theme = &self.theme;
        let width = 60.min(area.width.saturating_sub(4));
        let height = 25.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.y + area.height / 2).saturating_sub(height / 2);
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Welcome to OpenCode")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);
        let mut y_offset = inner.y;

        let version_line = Line::from(vec![
            Span::styled("Version ", Style::default().fg(theme.muted_color())),
            Span::styled(
                env!("CARGO_PKG_VERSION"),
                Style::default()
                    .fg(theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        f.render_widget(
            Paragraph::new(vec![version_line]),
            Rect::new(inner.x, y_offset, inner.width, 1),
        );
        y_offset += 2;

        let info_line = Line::from(vec![
            Span::styled("Model: ", Style::default().fg(theme.muted_color())),
            Span::styled(
                if self.model.is_empty() {
                    "Not configured"
                } else {
                    &self.model
                },
                Style::default().fg(theme.secondary_color()),
            ),
            Span::raw("  |  "),
            Span::styled("Dir: ", Style::default().fg(theme.muted_color())),
            Span::styled(
                if self.directory.is_empty() {
                    "."
                } else {
                    &self.directory
                },
                Style::default().fg(theme.primary_color()),
            ),
        ]);
        f.render_widget(
            Paragraph::new(vec![info_line]),
            Rect::new(inner.x, y_offset, inner.width, 1),
        );
        y_offset += 2;

        let stats_line = Line::from(vec![
            Span::styled("Sessions: ", Style::default().fg(theme.muted_color())),
            Span::styled(
                self.total_sessions.to_string(),
                Style::default()
                    .fg(theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled("Messages: ", Style::default().fg(theme.muted_color())),
            Span::styled(
                self.total_messages.to_string(),
                Style::default()
                    .fg(theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        f.render_widget(
            Paragraph::new(vec![stats_line]),
            Rect::new(inner.x, y_offset, inner.width, 1),
        );
        y_offset += 2;

        let section_title = Line::from(Span::styled(
            "Quick Actions",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(
            Paragraph::new(vec![section_title]),
            Rect::new(inner.x, y_offset, inner.width, 1),
        );
        y_offset += 1;

        for action in HomeAction::all() {
            let is_selected = self.focused_section == HomeViewSection::QuickActions
                && action == self.selected_action;
            let action_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.primary_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground_color())
            };

            let action_line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", action.key()),
                    Style::default()
                        .fg(if is_selected {
                            Color::Black
                        } else {
                            theme.warning_color()
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action.label(), action_style),
            ]);
            f.render_widget(
                Paragraph::new(vec![action_line]),
                Rect::new(inner.x, y_offset, inner.width, 1),
            );
            y_offset += 1;
        }

        y_offset += 1;
        let sessions_title = Line::from(Span::styled(
            "Recent Sessions",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(
            Paragraph::new(vec![sessions_title]),
            Rect::new(inner.x, y_offset, inner.width, 1),
        );
        y_offset += 1;

        if self.recent_sessions.is_empty() {
            let no_sessions = Line::from(Span::styled(
                "No recent sessions",
                Style::default().fg(theme.muted_color()),
            ));
            f.render_widget(
                Paragraph::new(vec![no_sessions]),
                Rect::new(inner.x, y_offset, inner.width, 1),
            );
        } else {
            let session_items: Vec<ListItem> = self
                .recent_sessions
                .iter()
                .enumerate()
                .map(|(idx, s)| {
                    let time_ago = format_time_elapsed(s.time_since_active());
                    let is_selected = self.focused_section == HomeViewSection::RecentSessions
                        && self.selected_session_index == Some(idx);
                    let name_style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.foreground_color())
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(&s.name, name_style),
                        Span::styled(
                            format!(" ({})", time_ago),
                            Style::default().fg(if is_selected {
                                Color::Black
                            } else {
                                theme.muted_color()
                            }),
                        ),
                    ]))
                })
                .collect();

            let remaining_height = inner.height.saturating_sub((y_offset - inner.y) as u16);
            let sessions_height = remaining_height
                .saturating_sub(2)
                .max(1)
                .min(session_items.len().max(1) as u16);
            let sessions_area = Rect::new(inner.x, y_offset, inner.width, sessions_height);

            let list = List::new(session_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border_color())),
                )
                .highlight_style(Style::default().bg(theme.primary_color()).fg(Color::Black));

            let mut state = ratatui::widgets::ListState::default();
            state.select(self.selected_session_index);
            f.render_stateful_widget(list, sessions_area, &mut state);
        }

        let hint_line = Line::from(vec![
            Span::styled("Tab", Style::default().fg(theme.muted_color())),
            Span::raw(" Switch section "),
            Span::styled("↑↓", Style::default().fg(theme.muted_color())),
            Span::raw(" Navigate "),
            Span::styled("Enter", Style::default().fg(theme.muted_color())),
            Span::raw(" Select "),
            Span::styled("Esc", Style::default().fg(theme.muted_color())),
            Span::raw(" Start chatting"),
        ]);
        let hint_area = Rect::new(
            dialog_area.x,
            dialog_area.bottom().saturating_sub(1),
            dialog_area.width,
            1,
        );
        f.render_widget(Paragraph::new(vec![hint_line]), hint_area);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Tab => {
                self.switch_section();
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.focused_section == HomeViewSection::RecentSessions {
                    if let Some(idx) = self.selected_session_index {
                        DialogAction::Confirm(format!("load_session:{}", idx))
                    } else {
                        DialogAction::Confirm(self.selected_action.to_string())
                    }
                } else {
                    DialogAction::Confirm(self.selected_action.to_string())
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focused_section == HomeViewSection::QuickActions {
                    self.move_selection(-1);
                } else {
                    self.move_session_selection(-1);
                }
                DialogAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focused_section == HomeViewSection::QuickActions {
                    self.move_selection(1);
                } else {
                    self.move_session_selection(1);
                }
                DialogAction::None
            }
            KeyCode::Char('n') => {
                self.focused_section = HomeViewSection::QuickActions;
                self.selected_action = HomeAction::NewSession;
                DialogAction::Confirm(self.selected_action.to_string())
            }
            KeyCode::Char('c') => {
                self.focused_section = HomeViewSection::QuickActions;
                self.selected_action = HomeAction::ContinueLast;
                DialogAction::Confirm(self.selected_action.to_string())
            }
            KeyCode::Char('s') => {
                self.focused_section = HomeViewSection::QuickActions;
                self.selected_action = HomeAction::ViewSessions;
                DialogAction::Confirm(self.selected_action.to_string())
            }
            KeyCode::Char(',') => {
                self.focused_section = HomeViewSection::QuickActions;
                self.selected_action = HomeAction::Settings;
                DialogAction::Confirm(self.selected_action.to_string())
            }
            KeyCode::Char('q') => {
                self.focused_section = HomeViewSection::QuickActions;
                self.selected_action = HomeAction::Quit;
                DialogAction::Confirm(self.selected_action.to_string())
            }
            _ => DialogAction::None,
        }
    }

    fn is_modal(&self) -> bool {
        false
    }
}

fn format_time_elapsed(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

impl ToString for HomeAction {
    fn to_string(&self) -> String {
        match self {
            HomeAction::NewSession => "new_session".to_string(),
            HomeAction::ContinueLast => "continue_last".to_string(),
            HomeAction::ViewSessions => "view_sessions".to_string(),
            HomeAction::Settings => "settings".to_string(),
            HomeAction::Quit => "quit".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_action_label() {
        assert_eq!(HomeAction::NewSession.label(), "New Session");
        assert_eq!(HomeAction::Quit.label(), "Quit");
    }

    #[test]
    fn test_home_action_key() {
        assert_eq!(HomeAction::NewSession.key(), "n");
        assert_eq!(HomeAction::Quit.key(), "q");
    }

    #[test]
    fn test_home_view_new() {
        let theme = crate::theme::Theme::default();
        let view = HomeView::new(theme);
        assert_eq!(view.selected_action, HomeAction::NewSession);
    }

    #[test]
    fn test_format_time_elapsed() {
        use std::time::Duration;
        assert_eq!(format_time_elapsed(Duration::from_secs(30)), "30s ago");
        assert_eq!(format_time_elapsed(Duration::from_secs(120)), "2m ago");
        assert_eq!(format_time_elapsed(Duration::from_secs(3660)), "1h ago");
        assert_eq!(format_time_elapsed(Duration::from_secs(90000)), "1d ago");
    }

    #[test]
    fn test_home_action_to_string() {
        assert_eq!(HomeAction::NewSession.to_string(), "new_session");
        assert_eq!(HomeAction::Settings.to_string(), "settings");
    }

    #[test]
    fn test_home_view_update_from_session_manager() {
        use crate::session::SessionManager;
        let theme = crate::theme::Theme::default();
        let mut view = HomeView::new(theme);
        assert!(view.recent_sessions.is_empty());

        let mut session_manager = SessionManager::new();
        session_manager.add_session("Test Session 1");
        session_manager.add_session("Test Session 2");

        view.update_from_session_manager(&session_manager);
        assert_eq!(view.recent_sessions.len(), 2);
        assert_eq!(view.recent_sessions[0].name, "Test Session 1");
        assert_eq!(view.total_sessions, 2);
        assert_eq!(view.total_messages, 0);
    }

    #[test]
    fn test_home_view_completion_statistics() {
        use crate::session::SessionManager;
        let theme = crate::theme::Theme::default();
        let mut view = HomeView::new(theme);

        let mut session_manager = SessionManager::new();
        session_manager.add_session("Session 1");
        session_manager.add_session("Session 2");
        session_manager.add_session("Session 3");

        view.update_from_session_manager(&session_manager);
        assert_eq!(view.total_sessions, 3);
        assert_eq!(view.total_messages, 0);
    }
}
