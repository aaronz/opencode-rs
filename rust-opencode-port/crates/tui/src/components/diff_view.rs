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
    CollapsedHunk,
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
pub struct DiffHunk {
    pub header: DiffLine,
    pub lines: Vec<DiffLine>,
    pub is_expanded: bool,
    pub collapsed_line_count: usize,
}

impl DiffHunk {
    fn new(header: DiffLine) -> Self {
        Self {
            header,
            lines: Vec::new(),
            is_expanded: true,
            collapsed_line_count: 0,
        }
    }

    fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    fn total_lines(&self) -> usize {
        if self.is_expanded {
            self.lines.len() + 1
        } else {
            2
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffView {
    pub old_content: String,
    pub new_content: String,
    pub diff_lines: Vec<DiffLine>,
    pub hunks: Vec<DiffHunk>,
    pub scroll_offset: usize,
    pub cursor_position: usize,
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

        let hunks = Self::parse_hunks(&diff_lines);

        Self {
            old_content: old_content.to_string(),
            new_content: new_content.to_string(),
            diff_lines,
            hunks,
            scroll_offset: 0,
            cursor_position: 0,
            style: DiffViewStyle::Auto,
        }
    }

    fn parse_hunks(diff_lines: &[DiffLine]) -> Vec<DiffHunk> {
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;

        for line in diff_lines {
            match line.line_type {
                DiffLineType::Header => {
                    if let Some(hunk) = current_hunk.take() {
                        hunks.push(hunk);
                    }
                    current_hunk = Some(DiffHunk::new(line.clone()));
                }
                _ => {
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(line.clone());
                    }
                }
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        hunks
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
        if self.cursor_position > self.scroll_offset {
            self.cursor_position = self.scroll_offset;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
        let max_cursor = self.scroll_offset + self.visible_lines_count().saturating_sub(1);
        if self.cursor_position < self.scroll_offset {
            self.cursor_position = self.scroll_offset;
        }
        if self.cursor_position > max_cursor {
            self.cursor_position = max_cursor;
        }
    }

    pub fn toggle_hunk_at_cursor(&mut self) {
        let visible_hunks = self.get_visible_hunks_with_offset();
        for (hunk_idx, (global_idx, is_expanded)) in visible_hunks {
            if self.cursor_position >= global_idx.0 && self.cursor_position <= global_idx.1 {
                if let Some(hunk) = self.hunks.get_mut(hunk_idx) {
                    hunk.toggle_expanded();
                    self.recalculate_cursor();
                    break;
                }
            }
        }
    }

    fn recalculate_cursor(&mut self) {
        let mut new_cursor = 0;
        let mut current_line = 0;

        for hunk in &self.hunks {
            if current_line >= self.scroll_offset {
                break;
            }
            let hunk_lines = if hunk.is_expanded {
                hunk.lines.len() + 1
            } else {
                2
            };

            if current_line + hunk_lines > self.scroll_offset {
                new_cursor = current_line;
                break;
            }
            current_line += hunk_lines;
        }

        self.cursor_position = new_cursor;
    }

    fn visible_lines_count(&self) -> usize {
        self.get_visible_lines().len()
    }

    fn get_visible_lines(&self) -> Vec<(DiffLine, Option<usize>)> {
        let mut visible = Vec::new();

        for (hunk_idx, hunk) in self.hunks.iter().enumerate() {
            visible.push((hunk.header.clone(), Some(hunk_idx)));

            if hunk.is_expanded {
                for line in &hunk.lines {
                    visible.push((line.clone(), Some(hunk_idx)));
                }
            } else {
                let collapsed_count = hunk.lines.len().saturating_sub(1);
                if collapsed_count > 0 {
                    visible.push((
                        DiffLine {
                            content: format!("  ... {} lines collapsed ...", collapsed_count),
                            line_type: DiffLineType::CollapsedHunk,
                        },
                        Some(hunk_idx),
                    ));
                }
            }
        }

        visible
    }

    fn get_visible_hunks_with_offset(&self) -> Vec<((usize, usize), (usize, bool))> {
        let mut result = Vec::new();
        let mut current_line = 0;

        for (hunk_idx, hunk) in self.hunks.iter().enumerate() {
            let start = current_line;
            let end = start + hunk.total_lines().saturating_sub(1);
            result.push(((start, end), (hunk_idx, hunk.is_expanded)));
            current_line += hunk.total_lines();
        }

        result
    }

    fn get_line_style(&self, line_type: DiffLineType) -> Style {
        match line_type {
            DiffLineType::Header => Style::default().fg(Color::Cyan),
            DiffLineType::Context => Style::default().fg(Color::White),
            DiffLineType::Addition => Style::default().fg(Color::Green),
            DiffLineType::Deletion => Style::default().fg(Color::Red),
            DiffLineType::CollapsedHunk => Style::default().fg(Color::DarkGray).add_modifier(ratatui::style::Modifier::ITALIC),
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position < self.scroll_offset {
                self.scroll_offset = self.cursor_position;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        let visible_count = self.visible_lines_count();
        if self.cursor_position < visible_count.saturating_sub(1) {
            self.cursor_position += 1;
            let max_visible = self.scroll_offset + visible_count;
            if self.cursor_position >= max_visible {
                self.scroll_offset = self.cursor_position.saturating_sub(visible_count).saturating_add(1);
            }
        }
    }
}

impl Widget for DiffView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .title("Diff View  |  Enter/Space: toggle hunk  |  ↑↓: navigate")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.hunks.is_empty() {
            return;
        }

        let visible_lines = self.get_visible_lines();
        let visible_hunks = self.get_visible_hunks_with_offset();
        let max_lines = inner.height as usize;
        let start_offset = self.scroll_offset;
        let end_offset = (start_offset + max_lines).min(visible_lines.len());

        let cursor_hunk_idx = visible_lines
            .iter()
            .enumerate()
            .skip(start_offset)
            .take(end_offset - start_offset)
            .find_map(|(idx, (_, hunk_idx))| {
                if idx == self.cursor_position {
                    Some(*hunk_idx)
                } else {
                    None
                }
            });

        for (i, (diff_line, hunk_idx)) in visible_lines
            .iter()
            .enumerate()
            .skip(start_offset)
            .take(end_offset - start_offset)
        {
            let y = inner.y + (i - start_offset) as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let is_cursor = i == self.cursor_position;
            let is_collapsed_hunk = diff_line.line_type == DiffLineType::CollapsedHunk;

            let base_style = self.get_line_style(diff_line.line_type);
            let style = if is_cursor {
                base_style.bg(Color::Rgb(50, 50, 80))
            } else if is_collapsed_hunk {
                base_style
            } else {
                base_style
            };

            let expand_indicator = if let Some(hunk_idx) = hunk_idx {
                if let Some(hunk) = self.hunks.get(hunk_idx) {
                    if is_collapsed_hunk {
                        "[+] ".to_string()
                    } else if !hunk.is_expanded {
                        "[-] ".to_string()
                    } else {
                        "   ".to_string()
                    }
                } else {
                    "   ".to_string()
                }
            } else {
                "   ".to_string()
            };

            let line_content = if matches!(diff_line.line_type, DiffLineType::CollapsedHunk) {
                diff_line.content.clone()
            } else {
                format!("{}{}", expand_indicator, diff_line.content)
            };

            let span = Span::raw(&line_content).style(style);
            buf.set_span(y, inner.x, &span, inner.width);
        }

        if let Some(hunk_idx) = cursor_hunk_idx {
            if let Some(hunk) = self.hunks.get(hunk_idx) {
                if !hunk.is_expanded && visible_lines.iter().any(|(l, _)| l.line_type == DiffLineType::CollapsedHunk) {
                }
            }
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
