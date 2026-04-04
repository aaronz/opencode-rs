use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::command::CommandRegistry;
use crate::input::completer::{CommandCompleter, FileCompleter};
use crate::input::parser::{InputParser, InputResult, InputToken};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputBoxAction {
    None,
    Submit(InputResult),
}

#[derive(Debug, Clone)]
pub struct InputBox {
    parser: InputParser,
    file_completer: FileCompleter,
    command_completer: CommandCompleter,
    value: String,
    cursor: usize,
    completions: Vec<String>,
    selected_completion: usize,
}

impl InputBox {
    pub fn new(root: impl AsRef<Path>, command_registry: &CommandRegistry) -> Self {
        Self {
            parser: InputParser::new(),
            file_completer: FileCompleter::new(root),
            command_completer: CommandCompleter::from_registry(command_registry),
            value: String::new(),
            cursor: 0,
            completions: Vec::new(),
            selected_completion: 0,
        }
    }

    pub fn set_input(&mut self, input: String) {
        self.value = input;
        self.cursor = self.value.len();
        self.refresh_completions();
    }

    pub fn input(&self) -> &str {
        &self.value
    }

    pub fn completions(&self) -> &[String] {
        &self.completions
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputBoxAction {
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.refresh_completions();
                InputBoxAction::None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                    self.refresh_completions();
                }
                InputBoxAction::None
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                InputBoxAction::None
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.value.len());
                InputBoxAction::None
            }
            KeyCode::Tab => {
                self.apply_completion();
                InputBoxAction::None
            }
            KeyCode::Enter => {
                let parsed = self.parser.parse(&self.value);
                InputBoxAction::Submit(parsed)
            }
            _ => InputBoxAction::None,
        }
    }

    pub fn autocomplete_into_input(&mut self, input: &mut String) -> bool {
        if self.completions.is_empty() {
            return false;
        }
        self.apply_completion();
        *input = self.value.clone();
        true
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, title: &str) {
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let parsed = self.parser.parse(&self.value);
        let spans = parsed
            .tokens
            .iter()
            .map(Self::token_span)
            .collect::<Vec<_>>();

        frame.render_widget(Paragraph::new(Line::from(spans)), inner);

        if !self.completions.is_empty() && inner.height > 1 {
            let items = self
                .completions
                .iter()
                .take((inner.height - 1) as usize)
                .map(|c| ListItem::new(c.clone()))
                .collect::<Vec<_>>();
            let list_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
            frame.render_widget(List::new(items), list_area);
        }
    }

    fn refresh_completions(&mut self) {
        self.completions.clear();
        self.selected_completion = 0;

        if self.value.trim_start().starts_with('/') {
            let partial = self
                .value
                .trim_start()
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or_default();
            self.completions = self
                .command_completer
                .suggest(partial)
                .into_iter()
                .map(|item| format!("/{item}"))
                .collect();
            return;
        }

        if let Some(fragment) = current_file_fragment(&self.value) {
            self.completions = self
                .file_completer
                .suggest(&fragment)
                .into_iter()
                .map(path_display)
                .collect();
        }
    }

    fn apply_completion(&mut self) {
        let Some(selected) = self.completions.get(self.selected_completion).cloned() else {
            return;
        };

        if self.value.trim_start().starts_with('/') {
            self.value = selected;
            self.cursor = self.value.len();
            self.refresh_completions();
            return;
        }

        let mut tokens = self
            .value
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if let Some(last) = tokens.last_mut() {
            if last.starts_with('@') {
                *last = format!("@{selected}");
                self.value = tokens.join(" ");
                self.cursor = self.value.len();
                self.refresh_completions();
            }
        }
    }

    fn token_span(token: &InputToken) -> Span<'static> {
        match token {
            InputToken::Text(text) => Span::raw(text.clone()),
            InputToken::FileRef(path) => Span::styled(
                format!("@{}", path.display()),
                Style::default().fg(Color::Blue),
            ),
            InputToken::ShellCommand(cmd) => {
                Span::styled(format!("!{cmd}"), Style::default().fg(Color::Yellow))
            }
            InputToken::SlashCommand { name, args } => {
                let content = if args.is_empty() {
                    format!("/{name}")
                } else {
                    format!("/{name} {args}")
                };
                Span::styled(content, Style::default().fg(Color::Green))
            }
        }
    }
}

fn current_file_fragment(input: &str) -> Option<String> {
    input
        .split_whitespace()
        .last()
        .and_then(|token| token.strip_prefix('@'))
        .map(str::to_string)
}

fn path_display(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_special_tokens_with_expected_colors() {
        let file = InputBox::token_span(&InputToken::FileRef(PathBuf::from("src/main.rs")));
        assert_eq!(file.content, "@src/main.rs");
        assert_eq!(file.style.fg, Some(Color::Blue));

        let command = InputBox::token_span(&InputToken::SlashCommand {
            name: "help".to_string(),
            args: String::new(),
        });
        assert_eq!(command.content, "/help");
        assert_eq!(command.style.fg, Some(Color::Green));

        let shell = InputBox::token_span(&InputToken::ShellCommand("cargo check".to_string()));
        assert_eq!(shell.content, "!cargo check");
        assert_eq!(shell.style.fg, Some(Color::Yellow));
    }
}
