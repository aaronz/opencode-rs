use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RightPanelTab {
    Diagnostics,
    Todo,
    PermissionQueue,
}

impl RightPanelTab {
    pub fn next(&self) -> Self {
        match self {
            RightPanelTab::Diagnostics => RightPanelTab::Todo,
            RightPanelTab::Todo => RightPanelTab::PermissionQueue,
            RightPanelTab::PermissionQueue => RightPanelTab::Diagnostics,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            RightPanelTab::Diagnostics => RightPanelTab::PermissionQueue,
            RightPanelTab::Todo => RightPanelTab::Diagnostics,
            RightPanelTab::PermissionQueue => RightPanelTab::Todo,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            RightPanelTab::Diagnostics => 0,
            RightPanelTab::Todo => 1,
            RightPanelTab::PermissionQueue => 2,
        }
    }
}

pub struct RightPanel {
    theme: Theme,
}

impl RightPanel {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, active_tab: &RightPanelTab) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_color()));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(inner_area);

        let titles: Vec<Line> = vec!["1:Diag", "2:Todo", "3:Perms"]
            .into_iter()
            .map(|t| Line::from(t))
            .collect();

        let tabs = Tabs::new(titles)
            .select(active_tab.index())
            .highlight_style(
                Style::default()
                    .fg(self.theme.primary_color())
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ");

        f.render_widget(tabs, chunks[0]);

        let content = match active_tab {
            RightPanelTab::Diagnostics => "No diagnostics available.",
            RightPanelTab::Todo => "No pending todos.",
            RightPanelTab::PermissionQueue => "No pending permissions.",
        };

        let content_widget =
            Paragraph::new(content).style(Style::default().fg(self.theme.foreground_color()));
        f.render_widget(content_widget, chunks[1]);
    }
}
