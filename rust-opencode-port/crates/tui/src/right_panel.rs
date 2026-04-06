use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelContent {
    Todo,
    Diff,
    Diagnostics,
    Context,
    Permissions,
    Files,
    Messages,
    Tools,
    Sessions,
    Config,
    Debug,
}

#[derive(Debug, Clone, Default)]
pub struct RightPanelRenderData {
    pub diagnostics: Vec<String>,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
    pub files: Vec<String>,
    pub tools: Vec<String>,
    pub todos: Vec<String>,
    pub diff_content: String,
    pub context_items: Vec<String>,
    pub permission_log: Vec<String>,
    pub messages: Vec<MessageData>,
    pub sessions: Vec<SessionData>,
    pub config_data: Vec<ConfigEntry>,
    pub debug_info: Vec<DebugEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageData {
    pub role: String,
    pub content_preview: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub id: String,
    pub name: String,
    pub last_active: String,
    pub message_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct DebugEntry {
    pub category: String,
    pub content: String,
}

pub struct RightPanel {
    pub content: RightPanelContent,
    pub collapsed: bool,
    theme: Theme,
}

impl RightPanel {
    pub fn new(theme: Theme) -> Self {
        Self {
            content: RightPanelContent::Diagnostics,
            collapsed: false,
            theme,
        }
    }

    pub fn set_content(&mut self, content: RightPanelContent) {
        self.content = content;
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn cycle_tab(&mut self) {
        self.content = match self.content {
            RightPanelContent::Todo => RightPanelContent::Diff,
            RightPanelContent::Diff => RightPanelContent::Diagnostics,
            RightPanelContent::Diagnostics => RightPanelContent::Context,
            RightPanelContent::Context => RightPanelContent::Permissions,
            RightPanelContent::Permissions => RightPanelContent::Files,
            RightPanelContent::Files => RightPanelContent::Messages,
            RightPanelContent::Messages => RightPanelContent::Tools,
            RightPanelContent::Tools => RightPanelContent::Sessions,
            RightPanelContent::Sessions => RightPanelContent::Config,
            RightPanelContent::Config => RightPanelContent::Debug,
            RightPanelContent::Debug => RightPanelContent::Todo,
        };
    }

    pub fn render(&self, _area: Rect) -> std::io::Result<()> {
        Ok(())
    }

    pub fn tab_labels() -> &'static [&'static str] {
        &[
            "1:Todo",
            "2:Diff",
            "3:Diag",
            "4:Ctx",
            "5:Perm",
            "6:Files",
            "7:Msgs",
            "8:Tools",
            "9:Sess",
            "10:Config",
            "11:Debug",
        ]
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, data: &RightPanelRenderData) {
        let title = if self.collapsed {
            "Inspector (collapsed)"
        } else {
            "Inspector"
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_color()));
        let inner = block.inner(area);
        f.render_widget(block, area);

        if self.collapsed {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(inner);

        let tab_idx = match self.content {
            RightPanelContent::Todo => 0,
            RightPanelContent::Diff => 1,
            RightPanelContent::Diagnostics => 2,
            RightPanelContent::Context => 3,
            RightPanelContent::Permissions => 4,
            RightPanelContent::Files => 5,
            RightPanelContent::Messages => 6,
            RightPanelContent::Tools => 7,
            RightPanelContent::Sessions => 8,
            RightPanelContent::Config => 9,
            RightPanelContent::Debug => 10,
        };

        let tabs = Tabs::new(
            Self::tab_labels()
                .iter()
                .map(|s| Line::from(*s))
                .collect::<Vec<_>>(),
        )
        .select(tab_idx)
        .highlight_style(
            Style::default()
                .fg(self.theme.primary_color())
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
        f.render_widget(tabs, chunks[0]);

        let content_lines = self.render_content(data).join("\n");
        let content =
            Paragraph::new(content_lines).style(Style::default().fg(self.theme.foreground_color()));
        f.render_widget(content, chunks[1]);
    }

    fn render_content(&self, data: &RightPanelRenderData) -> Vec<String> {
        match self.content {
            RightPanelContent::Todo => {
                if data.todos.is_empty() {
                    vec!["No active tasks".to_string()]
                } else {
                    data.todos.iter().take(15).cloned().collect()
                }
            }
            RightPanelContent::Diff => {
                if data.diff_content.is_empty() {
                    vec!["No changes to display".to_string()]
                } else {
                    data.diff_content
                        .lines()
                        .take(50)
                        .map(|l| l.to_string())
                        .collect()
                }
            }
            RightPanelContent::Diagnostics => {
                let mut lines = vec![format!("Diagnostics: {}", data.diagnostics.len())];
                if data.diagnostics.is_empty() {
                    lines.push("No diagnostics".to_string());
                } else {
                    lines.extend(data.diagnostics.iter().take(12).cloned());
                }
                lines
            }
            RightPanelContent::Context => {
                if data.context_items.is_empty() {
                    vec!["No context items".to_string()]
                } else {
                    data.context_items.iter().take(15).cloned().collect()
                }
            }
            RightPanelContent::Permissions => {
                if data.permission_log.is_empty() {
                    vec!["No permission requests".to_string()]
                } else {
                    data.permission_log.iter().take(15).cloned().collect()
                }
            }
            RightPanelContent::Files => {
                if data.files.is_empty() {
                    vec!["No files in session".to_string()]
                } else {
                    data.files.iter().take(15).cloned().collect()
                }
            }
            RightPanelContent::Messages => {
                if data.messages.is_empty() {
                    vec!["No messages".to_string()]
                } else {
                    let mut lines = vec![format!("Total messages: {}", data.messages.len())];
                    lines.extend(
                        data.messages.iter().take(15).map(|m| {
                            format!("[{}] {}: {}", m.timestamp, m.role, m.content_preview)
                        }),
                    );
                    lines
                }
            }
            RightPanelContent::Tools => {
                if data.tools.is_empty() {
                    vec!["No tools available".to_string()]
                } else {
                    let mut lines = vec![format!("Available tools: {}", data.tools.len())];
                    lines.extend(data.tools.iter().take(15).cloned());
                    lines
                }
            }
            RightPanelContent::Sessions => {
                if data.sessions.is_empty() {
                    vec!["No saved sessions".to_string()]
                } else {
                    let mut lines = vec![format!("Saved sessions: {}", data.sessions.len())];
                    lines.extend(
                        data.sessions
                            .iter()
                            .take(10)
                            .map(|s| format!("{} - {} ({} msgs)", s.id, s.name, s.message_count)),
                    );
                    lines
                }
            }
            RightPanelContent::Config => {
                if data.config_data.is_empty() {
                    vec!["No config loaded".to_string()]
                } else {
                    let mut lines = vec!["Current configuration:".to_string()];
                    lines.extend(
                        data.config_data
                            .iter()
                            .take(15)
                            .map(|c| format!("{}: {}", c.key, c.value)),
                    );
                    lines
                }
            }
            RightPanelContent::Debug => {
                if data.debug_info.is_empty() {
                    vec!["No debug info".to_string()]
                } else {
                    let mut lines = vec!["Debug Information:".to_string()];
                    lines.push(format!(
                        "Tokens: {} | Cost: ${:.4}",
                        data.total_tokens, data.total_cost_usd
                    ));
                    lines.extend(
                        data.debug_info
                            .iter()
                            .take(15)
                            .map(|d| format!("[{}] {}", d.category, d.content)),
                    );
                    lines
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_switching_and_collapse_work() {
        let theme = Theme::default();
        let mut panel = RightPanel::new(theme);
        panel.set_content(RightPanelContent::Files);
        assert_eq!(panel.content, RightPanelContent::Files);
        panel.toggle_collapse();
        assert!(panel.collapsed);
    }

    #[test]
    fn eleven_tabs_exist() {
        let labels = RightPanel::tab_labels();
        assert_eq!(labels.len(), 11);
        assert!(labels.contains(&"1:Todo"));
        assert!(labels.contains(&"2:Diff"));
        assert!(labels.contains(&"3:Diag"));
        assert!(labels.contains(&"4:Ctx"));
        assert!(labels.contains(&"5:Perm"));
        assert!(labels.contains(&"6:Files"));
        assert!(labels.contains(&"7:Msgs"));
        assert!(labels.contains(&"8:Tools"));
        assert!(labels.contains(&"9:Sess"));
        assert!(labels.contains(&"10:Config"));
        assert!(labels.contains(&"11:Debug"));
    }

    #[test]
    fn tab_cycle_wraps_around() {
        let theme = Theme::default();
        let mut panel = RightPanel::new(theme);
        assert_eq!(panel.content, RightPanelContent::Diagnostics);

        for _ in 0..11 {
            panel.cycle_tab();
        }
        assert_eq!(panel.content, RightPanelContent::Diagnostics);
    }
}
