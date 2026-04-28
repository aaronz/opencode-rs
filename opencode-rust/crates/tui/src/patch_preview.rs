use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchDecision {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct PatchPreview {
    expanded: bool,
    decision: PatchDecision,
    theme: Theme,
}

impl PatchPreview {
    pub fn new() -> Self {
        Self::with_theme(Theme::default())
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self {
            expanded: false,
            decision: PatchDecision::Pending,
            theme,
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn decision(&self) -> PatchDecision {
        self.decision
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(' ') | KeyCode::Char('e') | KeyCode::Char('E') => {
                self.expanded = !self.expanded;
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => self.decision = PatchDecision::Accepted,
            KeyCode::Char('n') | KeyCode::Char('N') => self.decision = PatchDecision::Rejected,
            _ => {}
        }
    }

    pub fn render_patch(&self, patch: &str, area: Rect, expanded: bool) -> std::io::Result<()> {
        let _ = (patch, area, expanded);
        Ok(())
    }

    pub fn draw(&self, f: &mut Frame, patch: &str, area: Rect) {
        let title = match self.decision {
            PatchDecision::Pending => "Patch Preview [Y:accept N:reject Space:expand]",
            PatchDecision::Accepted => "Patch Preview [Accepted]",
            PatchDecision::Rejected => "Patch Preview [Rejected]",
        };
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = patch
            .lines()
            .take(if self.expanded { usize::MAX } else { 12 })
            .map(|line| self.highlight_diff_line(line))
            .collect::<Vec<_>>();

        f.render_widget(Paragraph::new(lines), inner);
    }

    fn highlight_diff_line<'a>(&'a self, line: &'a str) -> Line<'a> {
        if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
            Line::from(Span::styled(line, Style::default().fg(self.theme.primary_color())))
        } else if line.starts_with('+') {
            Line::from(Span::styled(
                line,
                Style::default()
                    .fg(self.theme.success_color())
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ))
        } else if line.starts_with('-') {
            Line::from(Span::styled(
                line,
                Style::default()
                    .fg(self.theme.error_color())
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ))
        } else {
            Line::from(Span::raw(line))
        }
    }
}

impl Default for PatchPreview {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_toggles_expand_and_decision() {
        let mut preview = PatchPreview::new();
        preview.handle_key(KeyEvent::from(KeyCode::Char('e')));
        assert!(preview.is_expanded());

        preview.handle_key(KeyEvent::from(KeyCode::Char('y')));
        assert_eq!(preview.decision(), PatchDecision::Accepted);
    }
}
