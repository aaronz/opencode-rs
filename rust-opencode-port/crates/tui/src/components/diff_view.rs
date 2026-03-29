use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Widget},
    Frame,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DiffLineType {
    Header,
    Context,
    Addition,
    Deletion,
}

impl DiffLine {
    pub fn parse(line: &str) -> Self {
        let line_type = if line.starts_with("+++") || line.starts_with("@@") {
            DiffLineType::Header
        } else if line.starts_with('+') {
            DiffLineType::Addition
        } else if line.starts_with('-') {
            DiffLineType::Deletion
        } else {
            DiffLineType::Context
        };

        Self {
            content: line.to_string(),
            line_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffView {
    pub old_content: String,
    pub new_content: String,
    pub diff_lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub style: DiffViewStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DiffViewStyle {
    #[default]
    Auto,
    Stacked,
    SideBySide,
}

impl DiffView {
    pub fn new(old_content: &str, new_content: &str) -> Self {
        let diff_text = Self::generate_unified_diff(old_content, new_content);
        let diff_lines: Vec<DiffLine> = diff_text
            .lines()
            .filter(|l| !l.is_empty())
            .map(DiffLine::parse)
            .collect();

        Self {
            old_content: old_content.to_string(),
            new_content: new_content.to_string(),
            diff_lines,
            scroll_offset: 0,
            style: DiffViewStyle::Auto,
        }
    }

    fn generate_unified_diff(old_content: &str, new_content: &str) -> String {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut result = String::new();
        result.push_str("--- a/src/file.rs\n");
        result.push_str("+++ b/src/file.rs\n");

        let max_len = old_lines.len().max(new_lines.len());
        for i in 0..max_len {
            let old_line = old_lines.get(i).copied();
            let new_line = new_lines.get(i).copied();

            match (old_line, new_line) {
                (Some(o), Some(n)) if o == n => {
                    result.push_str(&format!(" {}\n", o));
                }
                (Some(o), Some(n)) => {
                    result.push_str(&format!("-{}\n", o));
                    result.push_str(&format!("+{}\n", n));
                }
                (Some(o), None) => {
                    result.push_str(&format!("-{}\n", o));
                }
                (None, Some(n)) => {
                    result.push_str(&format!("+{}\n", n));
                }
                _ => {}
            }
        }

        result
    }

    pub fn set_style(&mut self, style: DiffViewStyle) {
        self.style = style;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    fn get_line_style(&self, line_type: DiffLineType) -> Style {
        match line_type {
            DiffLineType::Header => Style::default().fg(Color::Cyan),
            DiffLineType::Context => Style::default().fg(Color::White),
            DiffLineType::Addition => Style::default().fg(Color::Green),
            DiffLineType::Deletion => Style::default().fg(Color::Red),
        }
    }
}

impl Widget for DiffView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .title("Diff View")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.diff_lines.is_empty() {
            return;
        }

        let max_lines = inner.height as usize;
        let visible_lines: Vec<&DiffLine> = self
            .diff_lines
            .iter()
            .skip(self.scroll_offset)
            .take(max_lines)
            .collect();

        for (i, diff_line) in visible_lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let style = self.get_line_style(diff_line.line_type);
            let span = Span::raw(&diff_line.content).style(style);
            buf.set_span(y, inner.x, &span, inner.width);
        }
    }
}

pub struct DiffRenderer;

impl DiffRenderer {
    pub fn render_simple_diff(f: &mut Frame, area: Rect, old_content: &str, new_content: &str) {
        let diff_view = DiffView::new(old_content, new_content);
        f.render_widget(diff_view, area);
    }
}
