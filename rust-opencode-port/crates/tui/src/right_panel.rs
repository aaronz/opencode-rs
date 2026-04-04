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
    Diagnostics,
    Tokens,
    Files,
    Tools,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct RightPanelRenderData {
    pub diagnostics: Vec<String>,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
    pub files: Vec<String>,
    pub tools: Vec<String>,
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

    pub fn render(&self, _area: Rect) -> std::io::Result<()> {
        Ok(())
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, data: &RightPanelRenderData) {
        let title = if self.collapsed {
            "Right Panel (collapsed)"
        } else {
            "Right Panel"
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

        let tabs = Tabs::new(
            vec!["1:Diag", "2:Tokens", "3:Files", "4:Tools", "0:None"]
                .into_iter()
                .map(Line::from)
                .collect::<Vec<_>>(),
        )
        .select(match self.content {
            RightPanelContent::Diagnostics => 0,
            RightPanelContent::Tokens => 1,
            RightPanelContent::Files => 2,
            RightPanelContent::Tools => 3,
            RightPanelContent::None => 4,
        })
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
            RightPanelContent::Diagnostics => {
                let mut lines = vec![format!("Diagnostics: {}", data.diagnostics.len())];
                if data.diagnostics.is_empty() {
                    lines.push("No diagnostics".to_string());
                } else {
                    lines.extend(data.diagnostics.iter().take(8).cloned());
                }
                lines
            }
            RightPanelContent::Tokens => vec![
                format!("Total tokens: {}", data.total_tokens),
                format!("Estimated cost: ${:.6}", data.total_cost_usd),
            ],
            RightPanelContent::Files => {
                if data.files.is_empty() {
                    vec!["No referenced files".to_string()]
                } else {
                    data.files.iter().take(10).cloned().collect()
                }
            }
            RightPanelContent::Tools => {
                if data.tools.is_empty() {
                    vec!["No tools registered".to_string()]
                } else {
                    data.tools.iter().take(12).cloned().collect()
                }
            }
            RightPanelContent::None => vec!["Panel content disabled".to_string()],
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
}
