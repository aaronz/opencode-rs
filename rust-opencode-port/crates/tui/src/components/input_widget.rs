use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct InputWidget {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub history: Vec<String>,
    pub history_index: usize,
    pub theme: Theme,
    pub multiline: bool,
}

impl InputWidget {
    pub fn new(theme: Theme) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            history: Vec::new(),
            history_index: 0,
            theme,
            multiline: false,
        }
    }

    pub fn new_multiline(theme: Theme) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            history: Vec::new(),
            history_index: 0,
            theme,
            multiline: true,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> InputAction {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => return InputAction::Cancel,
                        'a' => {
                            self.cursor_col = 0;
                            return InputAction::None;
                        }
                        'e' => {
                            self.cursor_col = self.lines[self.cursor_line].len();
                            return InputAction::None;
                        }
                        'k' => {
                            self.lines[self.cursor_line].truncate(self.cursor_col);
                            return InputAction::None;
                        }
                        _ => return InputAction::None,
                    }
                }

                let line = &mut self.lines[self.cursor_line];
                if self.cursor_col >= line.len() {
                    line.push(c);
                } else {
                    line.insert(self.cursor_col, c);
                }
                self.cursor_col += 1;
                InputAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let line = &mut self.lines[self.cursor_line];
                    line.remove(self.cursor_col - 1);
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    let line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                    self.lines[self.cursor_line].push_str(&line);
                }
                InputAction::None
            }
            KeyCode::Delete => {
                let line = &mut self.lines[self.cursor_line];
                if self.cursor_col < line.len() {
                    line.remove(self.cursor_col);
                }
                InputAction::None
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                }
                InputAction::None
            }
            KeyCode::Right => {
                if self.cursor_col < self.lines[self.cursor_line].len() {
                    self.cursor_col += 1;
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                InputAction::None
            }
            KeyCode::Up => {
                if self.multiline && self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                    InputAction::None
                } else {
                    self.history_previous()
                }
            }
            KeyCode::Down => {
                if self.multiline && self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                    InputAction::None
                } else {
                    self.history_next()
                }
            }
            KeyCode::Enter => {
                if self.multiline && key.modifiers.contains(KeyModifiers::SHIFT) {
                    let line = &mut self.lines[self.cursor_line];
                    let remaining: String = line.split_off(self.cursor_col);
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                    self.lines.insert(self.cursor_line, remaining);
                    InputAction::None
                } else {
                    self.submit()
                }
            }
            KeyCode::Esc => InputAction::Cancel,
            _ => InputAction::None,
        }
    }

    fn history_previous(&mut self) -> InputAction {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            let idx = self.history.len() - self.history_index;
            self.lines = vec![self.history[idx].clone()];
            self.cursor_line = 0;
            self.cursor_col = self.lines[0].len();
        }
        InputAction::None
    }

    fn history_next(&mut self) -> InputAction {
        if self.history_index > 0 {
            self.history_index -= 1;
            if self.history_index == 0 {
                self.lines = vec![String::new()];
            } else {
                let idx = self.history.len() - self.history_index;
                self.lines = vec![self.history[idx].clone()];
            }
            self.cursor_line = 0;
            self.cursor_col = self.lines[0].len();
        }
        InputAction::None
    }

    fn submit(&mut self) -> InputAction {
        let content = self.lines.join("\n");
        if !content.is_empty() {
            self.history.push(content.clone());
            self.history_index = 0;
        }
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        InputAction::Submit(content)
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, title: &str) {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines: Vec<Line> = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.cursor_line {
                    let mut spans = vec![];
                    if self.cursor_col < line.len() {
                        spans.push(Span::raw(&line[..self.cursor_col]));
                        spans.push(Span::styled(
                            &line[self.cursor_col..self.cursor_col + 1],
                            Style::default().add_modifier(Modifier::REVERSED),
                        ));
                        if self.cursor_col + 1 < line.len() {
                            spans.push(Span::raw(&line[self.cursor_col + 1..]));
                        }
                    } else {
                        spans.push(Span::raw(line.as_str()));
                        spans.push(Span::styled(
                            " ",
                            Style::default().add_modifier(Modifier::REVERSED),
                        ));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    None,
    Submit(String),
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_widget_new() {
        let theme = crate::theme::Theme::default();
        let input = InputWidget::new(theme);
        assert_eq!(input.lines.len(), 1);
        assert!(input.lines[0].is_empty());
    }

    #[test]
    fn test_input_widget_typing() {
        let theme = crate::theme::Theme::default();
        let mut input = InputWidget::new(theme);

        input.handle_input(KeyEvent::from(KeyCode::Char('h')));
        input.handle_input(KeyEvent::from(KeyCode::Char('i')));

        assert_eq!(input.get_content(), "hi");
    }

    #[test]
    fn test_input_widget_history() {
        let theme = crate::theme::Theme::default();
        let mut input = InputWidget::new(theme);

        input.lines[0] = "test".to_string();
        input.handle_input(KeyEvent::from(KeyCode::Enter));

        assert_eq!(input.history.len(), 1);
        assert_eq!(input.history[0], "test");
    }
}
