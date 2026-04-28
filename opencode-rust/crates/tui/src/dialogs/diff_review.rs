use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const SYNTAX_KEYWORDS: &[&str] = &[
    "fn", "let", "const", "mut", "pub", "struct", "enum", "impl", "trait", "type", "if", "else",
    "match", "for", "while", "loop", "break", "continue", "return", "use", "mod", "crate", "self",
    "super", "async", "await", "move", "class", "function", "var", "const", "let", "export",
    "import", "from", "def", "class", "import", "from", "return", "if", "elif", "else", "for",
    "while",
];

const SYNTAX_TYPES: &[&str] = &[
    "String", "str", "i32", "i64", "u32", "u64", "bool", "f32", "f64", "Vec", "Option", "Result",
    "Box", "Rc", "Arc", "Cell", "RefCell", "HashMap", "HashSet", "int", "float", "bool", "str",
    "list", "dict", "tuple",
];

#[derive(Debug, Clone, PartialEq)]
pub enum DiffState {
    Pending(String),
    Accepted(String),
    Rejected(String),
    Editing(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLayout {
    Stacked,
    SideBySide,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
    pub collapsed: bool,
    pub added_count: usize,
    pub removed_count: usize,
}

pub struct DiffReviewOverlay {
    diff_state: DiffState,
    theme: Theme,
    layout: DiffLayout,
    hunks: Vec<DiffHunk>,
    selected_hunk: usize,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Unchanged,
    Added,
    Removed,
    Context,
}

impl DiffReviewOverlay {
    pub fn new(file_path: String, theme: Theme) -> Self {
        let hunks = Self::parse_diff(file_path.clone());
        Self {
            diff_state: DiffState::Pending(file_path),
            theme,
            layout: DiffLayout::Stacked,
            hunks,
            selected_hunk: 0,
        }
    }

    pub fn toggle_layout(&mut self) {
        self.layout = match self.layout {
            DiffLayout::Stacked => DiffLayout::SideBySide,
            DiffLayout::SideBySide => DiffLayout::Stacked,
        };
    }

    fn highlight_syntax(line: &str) -> Vec<Span<'_>> {
        let mut spans = Vec::new();
        let mut remaining = line;

        while !remaining.is_empty() {
            if remaining.starts_with("//") || remaining.starts_with("#") {
                spans.push(Span::styled(
                    remaining,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ));
                break;
            } else if remaining.starts_with("\"") {
                let end = remaining[1..]
                    .find("\"")
                    .map(|i| i + 2)
                    .unwrap_or(remaining.len());
                spans.push(Span::styled(
                    &remaining[..end],
                    Style::default().fg(Color::Yellow),
                ));
                remaining = &remaining[end..];
            } else if remaining.starts_with("fn ")
                || remaining.starts_with("fn(")
                || remaining.starts_with("async fn")
            {
                let end = remaining
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(remaining.len());
                spans.push(Span::styled(
                    &remaining[..end],
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
                remaining = &remaining[end..];
            } else {
                let word_end = remaining
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(remaining.len());
                if word_end > 0 {
                    let word = &remaining[..word_end];
                    if SYNTAX_KEYWORDS.contains(&word) {
                        spans.push(Span::styled(
                            word,
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else if SYNTAX_TYPES.contains(&word) {
                        spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                    } else if word
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        spans.push(Span::styled(word, Style::default().fg(Color::Blue)));
                    } else {
                        spans.push(Span::raw(word));
                    }
                    remaining = &remaining[word_end..];
                } else {
                    spans.push(Span::raw(remaining));
                    break;
                }
            }
        }

        if spans.is_empty() {
            spans.push(Span::raw(line));
        }

        spans
    }

    fn parse_diff(file_path: String) -> Vec<DiffHunk> {
        let mut hunks = Vec::new();
        let mut current_hunk = DiffHunk {
            header: "@@ -1,20 +1,20 @@".to_string(),
            lines: Vec::new(),
            collapsed: false,
            added_count: 0,
            removed_count: 0,
        };

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            for line in content.lines().take(20) {
                if line.starts_with("+ ") {
                    current_hunk.added_count += 1;
                    current_hunk.lines.push(DiffLine {
                        content: line[2..].to_string(),
                        line_type: DiffLineType::Added,
                    });
                } else if line.starts_with("- ") {
                    current_hunk.removed_count += 1;
                    current_hunk.lines.push(DiffLine {
                        content: line[2..].to_string(),
                        line_type: DiffLineType::Removed,
                    });
                } else {
                    current_hunk.lines.push(DiffLine {
                        content: line.to_string(),
                        line_type: DiffLineType::Context,
                    });
                }
            }
            hunks.push(current_hunk);
        } else {
            current_hunk.lines.push(DiffLine {
                content: "--- a/".to_string() + &file_path,
                line_type: DiffLineType::Context,
            });
            current_hunk.lines.push(DiffLine {
                content: "+++ b/".to_string() + &file_path,
                line_type: DiffLineType::Context,
            });
            current_hunk.lines.push(DiffLine {
                content: "@@ -1,5 +1,6 @@".to_string(),
                line_type: DiffLineType::Context,
            });
            current_hunk.lines.push(DiffLine {
                content: " unchanged content".to_string(),
                line_type: DiffLineType::Unchanged,
            });
            current_hunk.removed_count += 1;
            current_hunk.lines.push(DiffLine {
                content: "- removed line".to_string(),
                line_type: DiffLineType::Removed,
            });
            current_hunk.added_count += 1;
            current_hunk.lines.push(DiffLine {
                content: "+ added line".to_string(),
                line_type: DiffLineType::Added,
            });
            hunks.push(current_hunk);
        }

        hunks
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> DiffAction {
        match key.code {
            KeyCode::Esc => DiffAction::Cancel,
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_layout();
                DiffAction::None
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                for hunk in &mut self.hunks {
                    hunk.collapsed = false;
                }
                DiffAction::None
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                for hunk in &mut self.hunks {
                    hunk.collapsed = true;
                }
                DiffAction::None
            }
            KeyCode::Char(' ') => {
                if let Some(hunk) = self.hunks.get_mut(self.selected_hunk) {
                    hunk.collapsed = !hunk.collapsed;
                }
                DiffAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_hunk > 0 {
                    self.selected_hunk -= 1;
                }
                DiffAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_hunk + 1 < self.hunks.len() {
                    self.selected_hunk += 1;
                }
                DiffAction::None
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let DiffState::Pending(path) = self.diff_state.clone() {
                    self.diff_state = DiffState::Accepted(path.clone());
                    DiffAction::Accept(path)
                } else {
                    DiffAction::None
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if let DiffState::Pending(path) = self.diff_state.clone() {
                    self.diff_state = DiffState::Rejected(path.clone());
                    DiffAction::Reject
                } else {
                    DiffAction::None
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                if let DiffState::Pending(path) = self.diff_state.clone() {
                    self.diff_state = DiffState::Editing(path.clone());
                    DiffAction::Edit(path)
                } else {
                    DiffAction::None
                }
            }
            _ => DiffAction::None,
        }
    }

    fn render_stacked_diff(&self) -> Vec<Line<'_>> {
        let mut lines = vec![Line::from("")];

        for (i, hunk) in self.hunks.iter().enumerate() {
            let is_selected = i == self.selected_hunk;
            let prefix = if is_selected { "> " } else { "  " };

            if hunk.collapsed {
                let file_name = match &self.diff_state {
                    DiffState::Pending(p)
                    | DiffState::Accepted(p)
                    | DiffState::Rejected(p)
                    | DiffState::Editing(p) => p.clone(),
                };
                let summary = format!(
                    "{}{} {} (+{} / -{})",
                    prefix, file_name, hunk.header, hunk.added_count, hunk.removed_count
                );
                let style = if is_selected {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.muted_color())
                };
                lines.push(Line::from(Span::styled(summary, style)));
            } else {
                let header_style = if is_selected {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.muted_color())
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", prefix, hunk.header),
                    header_style,
                )));

                for diff_line in &hunk.lines {
                    let (line_prefix, base_style) = match diff_line.line_type {
                        DiffLineType::Added => (
                            "+ ",
                            Style::default()
                                .fg(self.theme.success_color())
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        DiffLineType::Removed => (
                            "- ",
                            Style::default()
                                .fg(self.theme.error_color())
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        DiffLineType::Unchanged => ("  ", Style::default()),
                        DiffLineType::Context => (
                            "  ",
                            Style::default().fg(self.theme.muted_color()),
                        ),
                    };

                    let mut spans = vec![Span::raw("  "), Span::styled(line_prefix, base_style)];
                    let highlighted = Self::highlight_syntax(&diff_line.content);
                    spans.extend(highlighted);

                    lines.push(Line::from(spans));
                }
            }
        }

        lines
    }

    fn render_side_by_side_diff(&self, width: u16) -> Vec<Line<'_>> {
        let half_width = (width / 2).saturating_sub(2);
        let mut lines = vec![Line::from("")];

        for (i, hunk) in self.hunks.iter().enumerate() {
            let is_selected = i == self.selected_hunk;
            let prefix = if is_selected { "> " } else { "  " };

            if hunk.collapsed {
                let file_name = match &self.diff_state {
                    DiffState::Pending(p)
                    | DiffState::Accepted(p)
                    | DiffState::Rejected(p)
                    | DiffState::Editing(p) => p.clone(),
                };
                let summary = format!(
                    "{}{} {} (+{} / -{})",
                    prefix, file_name, hunk.header, hunk.added_count, hunk.removed_count
                );
                let style = if is_selected {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.muted_color())
                };
                lines.push(Line::from(Span::styled(summary, style)));
            } else {
                let header_style = if is_selected {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.muted_color())
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", prefix, hunk.header),
                    header_style,
                )));

                let mut left_lines: Vec<DiffLine> = Vec::new();
                let mut right_lines: Vec<DiffLine> = Vec::new();

                for diff_line in &hunk.lines {
                    match diff_line.line_type {
                        DiffLineType::Removed | DiffLineType::Unchanged | DiffLineType::Context => {
                            left_lines.push(diff_line.clone());
                        }
                        DiffLineType::Added => {
                            right_lines.push(diff_line.clone());
                        }
                    }
                }

                let max_lines = left_lines.len().max(right_lines.len());

                for j in 0..max_lines {
                    let left = left_lines.get(j);
                    let right = right_lines.get(j);

                    let left_content = left
                        .map(|l| {
                            let p = match l.line_type {
                                DiffLineType::Removed => "- ",
                                _ => "  ",
                            };
                            format!("{}{}", p, l.content)
                        })
                        .unwrap_or_default();

                    let right_content = right
                        .map(|r| {
                            let p = match r.line_type {
                                DiffLineType::Added => "+ ",
                                _ => "  ",
                            };
                            format!("{}{}", p, r.content)
                        })
                        .unwrap_or_default();

                    let truncated_left = if left_content.len() > half_width as usize {
                        left_content[..half_width as usize].to_string()
                    } else {
                        left_content
                    };

                    let truncated_right = if right_content.len() > half_width as usize {
                        right_content[..half_width as usize].to_string()
                    } else {
                        right_content
                    };

                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::raw(truncated_left),
                        Span::raw(" │ "),
                        Span::raw(truncated_right),
                    ]));
                }
            }
        }

        lines
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        f.render_widget(Clear, area);

        let dialog_width = 80.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        let (title, border_style, content_color) = match &self.diff_state {
            DiffState::Pending(_) => (
                "Review Diff - Pending",
                Style::default().fg(self.theme.warning_color()),
                self.theme.warning_color(),
            ),
            DiffState::Accepted(_) => (
                "Review Diff - Accepted",
                Style::default().fg(self.theme.success_color()),
                self.theme.success_color(),
            ),
            DiffState::Rejected(_) => (
                "Review Diff - Rejected",
                Style::default().fg(self.theme.error_color()),
                self.theme.error_color(),
            ),
            DiffState::Editing(_) => (
                "Review Diff - Editing",
                Style::default().fg(self.theme.accent_color()),
                self.theme.accent_color(),
            ),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(inner_area);

        let header_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Accept changes? ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "[Y]es",
                    Style::default()
                        .fg(self.theme.success_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[N]o",
                    Style::default()
                        .fg(self.theme.error_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[E]dit",
                    Style::default()
                        .fg(self.theme.accent_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[Esc]ancel",
                    Style::default()
                        .fg(self.theme.muted_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[T]oggle layout",
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let header_widget = Paragraph::new(header_text);
        f.render_widget(header_widget, chunks[0]);

        let diff_lines = match self.layout {
            DiffLayout::Stacked => self.render_stacked_diff(),
            DiffLayout::SideBySide => self.render_side_by_side_diff(chunks[1].width),
        };

        let diff_widget = Paragraph::new(diff_lines).alignment(Alignment::Left);
        f.render_widget(diff_widget, chunks[1]);

        let footer_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    match &self.diff_state {
                        DiffState::Pending(p)
                        | DiffState::Accepted(p)
                        | DiffState::Rejected(p)
                        | DiffState::Editing(p) => p.clone(),
                    },
                    Style::default().fg(content_color),
                ),
            ]),
        ];

        let footer_widget = Paragraph::new(footer_text);
        f.render_widget(footer_widget, chunks[2]);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffAction {
    Accept(String),
    Reject,
    Edit(String),
    Cancel,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_review_overlay_new() {
        let theme = crate::theme::Theme::default();
        let overlay = DiffReviewOverlay::new("test.rs".to_string(), theme);
        assert!(matches!(overlay.diff_state, DiffState::Pending(_)));
    }

    #[test]
    fn test_diff_review_action() {
        let theme = crate::theme::Theme::default();
        let mut overlay = DiffReviewOverlay::new("test.rs".to_string(), theme.clone());

        let action = overlay.handle_input(KeyEvent::from(KeyCode::Char('y')));
        assert!(matches!(action, DiffAction::Accept(_)));

        let mut overlay = DiffReviewOverlay::new("test.rs".to_string(), theme.clone());
        let action = overlay.handle_input(KeyEvent::from(KeyCode::Char('n')));
        assert!(matches!(action, DiffAction::Reject));

        let mut overlay = DiffReviewOverlay::new("test.rs".to_string(), theme);
        let action = overlay.handle_input(KeyEvent::from(KeyCode::Esc));
        assert!(matches!(action, DiffAction::Cancel));
    }
}
