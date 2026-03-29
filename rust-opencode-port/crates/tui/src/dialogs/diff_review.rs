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

pub struct DiffReviewOverlay {
    diff_state: DiffState,
    theme: Theme,
    layout: DiffLayout,
    diff_content: Vec<DiffLine>,
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
        let diff_content = Self::parse_diff(file_path.clone());
        Self {
            diff_state: DiffState::Pending(file_path),
            theme,
            layout: DiffLayout::Stacked,
            diff_content,
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
                    if SYNTAX_KEYWORDS.iter().any(|&k| k == word) {
                        spans.push(Span::styled(
                            word,
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else if SYNTAX_TYPES.iter().any(|&t| t == word) {
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

    fn parse_diff(file_path: String) -> Vec<DiffLine> {
        let mut lines = Vec::new();

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            for line in content.lines().take(20) {
                if line.starts_with("+ ") {
                    lines.push(DiffLine {
                        content: line[2..].to_string(),
                        line_type: DiffLineType::Added,
                    });
                } else if line.starts_with("- ") {
                    lines.push(DiffLine {
                        content: line[2..].to_string(),
                        line_type: DiffLineType::Removed,
                    });
                } else {
                    lines.push(DiffLine {
                        content: line.to_string(),
                        line_type: DiffLineType::Context,
                    });
                }
            }
        } else {
            lines.push(DiffLine {
                content: "--- a/".to_string() + &file_path,
                line_type: DiffLineType::Context,
            });
            lines.push(DiffLine {
                content: "+++ b/".to_string() + &file_path,
                line_type: DiffLineType::Context,
            });
            lines.push(DiffLine {
                content: "@@ -1,5 +1,6 @@".to_string(),
                line_type: DiffLineType::Context,
            });
            lines.push(DiffLine {
                content: " unchanged content".to_string(),
                line_type: DiffLineType::Unchanged,
            });
            lines.push(DiffLine {
                content: "- removed line".to_string(),
                line_type: DiffLineType::Removed,
            });
            lines.push(DiffLine {
                content: "+ added line".to_string(),
                line_type: DiffLineType::Added,
            });
        }

        lines
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> DiffAction {
        match key.code {
            KeyCode::Esc => DiffAction::Cancel,
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_layout();
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
            KeyCode::Char('e') | KeyCode::Char('E') => {
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

        for diff_line in &self.diff_content {
            let (prefix, base_style) = match diff_line.line_type {
                DiffLineType::Added => ("+ ", Style::default().fg(Color::Green)),
                DiffLineType::Removed => ("- ", Style::default().fg(Color::Red)),
                DiffLineType::Unchanged => ("  ", Style::default()),
                DiffLineType::Context => ("  ", Style::default().fg(self.theme.muted_color())),
            };

            let mut spans = vec![Span::styled(prefix, base_style)];
            let highlighted = Self::highlight_syntax(&diff_line.content);
            spans.extend(highlighted);

            lines.push(Line::from(spans));
        }

        lines
    }

    fn render_side_by_side_diff(&self, width: u16) -> Vec<Line<'_>> {
        let half_width = (width / 2).saturating_sub(2);
        let mut lines = vec![Line::from("")];

        let mut left_lines: Vec<DiffLine> = Vec::new();
        let mut right_lines: Vec<DiffLine> = Vec::new();

        for diff_line in &self.diff_content {
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

        for i in 0..max_lines {
            let left = left_lines.get(i);
            let right = right_lines.get(i);

            let left_content = left
                .map(|l| {
                    let prefix = match l.line_type {
                        DiffLineType::Removed => "- ",
                        _ => "  ",
                    };
                    format!("{}{}", prefix, l.content)
                })
                .unwrap_or_default();

            let right_content = right
                .map(|r| {
                    let prefix = match r.line_type {
                        DiffLineType::Added => "+ ",
                        _ => "  ",
                    };
                    format!("{}{}", prefix, r.content)
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
                Span::raw(truncated_left),
                Span::raw(" │ "),
                Span::raw(truncated_right),
            ]));
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
                Color::Yellow,
            ),
            DiffState::Accepted(_) => (
                "Review Diff - Accepted",
                Style::default().fg(self.theme.success_color()),
                Color::Green,
            ),
            DiffState::Rejected(_) => (
                "Review Diff - Rejected",
                Style::default().fg(self.theme.error_color()),
                Color::Red,
            ),
            DiffState::Editing(_) => (
                "Review Diff - Editing",
                Style::default().fg(self.theme.accent_color()),
                Color::Blue,
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
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[N]o",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[E]dit",
                    Style::default()
                        .fg(Color::Blue)
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
        let mut overlay = DiffReviewOverlay::new("test.rs".to_string(), theme);

        let action = overlay.handle_input(KeyEvent::from(KeyCode::Char('y')));
        assert!(matches!(action, DiffAction::Accept(_)));

        let action = overlay.handle_input(KeyEvent::from(KeyCode::Char('n')));
        assert!(matches!(action, DiffAction::Reject));

        let action = overlay.handle_input(KeyEvent::from(KeyCode::Esc));
        assert!(matches!(action, DiffAction::Cancel));
    }
}
