use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum InputElement {
    Text(String),
    Chip {
        display: String,
        value: String,
        color: Color,
    },
}

impl InputElement {
    pub fn text(content: impl Into<String>) -> Self {
        InputElement::Text(content.into())
    }

    pub fn chip(display: impl Into<String>, value: impl Into<String>, color: Color) -> Self {
        InputElement::Chip {
            display: display.into(),
            value: value.into(),
            color,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InputElement::Text(s) => s.len(),
            InputElement::Chip { display, .. } => display.len() + 2,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            InputElement::Text(s) => s.is_empty(),
            InputElement::Chip { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum OverflowMode {
    #[default]
    Truncate,
    Scroll,
    Wrap,
}

#[derive(Debug, Clone)]
pub struct TypewriterState {
    pub full_content: String,
    pub displayed_length: usize,
    pub delay_ms: u64,
    pub is_streaming: bool,
}

impl TypewriterState {
    pub fn new(content: &str, delay_ms: u64) -> Self {
        Self {
            full_content: content.to_string(),
            displayed_length: 0,
            delay_ms,
            is_streaming: true,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.displayed_length < self.full_content.len() {
            self.displayed_length += 1;
            true
        } else {
            self.is_streaming = false;
            false
        }
    }

    pub fn skip(&mut self) {
        self.displayed_length = self.full_content.len();
        self.is_streaming = false;
    }

    pub fn current_display(&self) -> String {
        self.full_content
            .chars()
            .take(self.displayed_length)
            .collect()
    }
}

pub struct InputWidget {
    pub elements: Vec<InputElement>,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_index: usize,
    pub theme: Theme,
    pub multiline: bool,
    pub leader_active: bool,
    pub scroll_x: usize,
    pub overflow_mode: OverflowMode,
    pub typewriter_state: Option<TypewriterState>,
    pub typewriter_speed_ms: u64,
}

impl InputWidget {
    pub fn new(theme: Theme) -> Self {
        Self {
            elements: vec![InputElement::text("")],
            cursor_pos: 0,
            history: Vec::new(),
            history_index: 0,
            theme,
            multiline: false,
            leader_active: false,
            scroll_x: 0,
            overflow_mode: OverflowMode::Truncate,
            typewriter_state: None,
            typewriter_speed_ms: 20,
        }
    }

    pub fn new_multiline(theme: Theme) -> Self {
        Self {
            elements: vec![InputElement::text("")],
            cursor_pos: 0,
            history: Vec::new(),
            history_index: 0,
            theme,
            multiline: true,
            leader_active: false,
            scroll_x: 0,
            overflow_mode: OverflowMode::Wrap,
            typewriter_state: None,
            typewriter_speed_ms: 20,
        }
    }

    pub fn start_typewriter(&mut self, content: &str) {
        self.typewriter_state = Some(TypewriterState::new(content, self.typewriter_speed_ms));
    }

    pub fn tick_typewriter(&mut self) -> bool {
        if let Some(ref mut state) = self.typewriter_state {
            state.tick()
        } else {
            false
        }
    }

    pub fn skip_typewriter(&mut self) {
        if let Some(ref mut state) = self.typewriter_state {
            state.skip();
            self.elements = vec![InputElement::Text(state.full_content.clone())];
        }
    }

    pub fn is_typewriter_active(&self) -> bool {
        self.typewriter_state
            .as_ref()
            .map(|s| s.is_streaming)
            .unwrap_or(false)
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> InputAction {
        if self.leader_active {
            return InputAction::None;
        }

        if self.is_typewriter_active() {
            self.skip_typewriter();
            return InputAction::None;
        }

        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => return InputAction::Cancel,
                        'a' => {
                            self.cursor_pos = 0;
                            return InputAction::None;
                        }
                        'e' => {
                            self.cursor_pos = self.get_content().len();
                            return InputAction::None;
                        }
                        'k' => {
                            self.truncate_at_cursor();
                            return InputAction::None;
                        }
                        _ => return InputAction::None,
                    }
                }

                self.insert_char(c);
                InputAction::None
            }
            KeyCode::Backspace => {
                self.delete_backward();
                InputAction::None
            }
            KeyCode::Delete => {
                self.delete_forward();
                InputAction::None
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_backward();
                } else if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                InputAction::None
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_forward();
                } else {
                    let content_len = self.get_content().len();
                    if self.cursor_pos < content_len {
                        self.cursor_pos += 1;
                    }
                }
                InputAction::None
            }
            KeyCode::Up => self.history_previous(),
            KeyCode::Down => self.history_next(),
            KeyCode::Enter => {
                if self.multiline && key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.insert_char('\n');
                    InputAction::None
                } else {
                    self.submit()
                }
            }
            KeyCode::Esc => InputAction::Cancel,
            _ => InputAction::None,
        }
    }

    fn insert_char(&mut self, c: char) {
        let content = self.get_content();
        let mut chars: Vec<char> = content.chars().collect();
        if self.cursor_pos > chars.len() {
            self.cursor_pos = chars.len();
        }
        chars.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.elements = vec![InputElement::Text(chars.into_iter().collect())];
    }

    fn delete_backward(&mut self) {
        if self.cursor_pos > 0 {
            let content = self.get_content();
            let mut chars: Vec<char> = content.chars().collect();

            let element_at_pos = self.get_element_at_pos(self.cursor_pos - 1);
            if let Some(InputElement::Chip { .. }) = element_at_pos {
                self.delete_chip_at_pos(self.cursor_pos - 1);
            } else {
                chars.remove(self.cursor_pos - 1);
                self.cursor_pos -= 1;
                self.elements = vec![InputElement::Text(chars.into_iter().collect())];
            }
        }
    }

    fn delete_forward(&mut self) {
        let content = self.get_content();
        let chars: Vec<char> = content.chars().collect();
        if self.cursor_pos < chars.len() {
            let element_at_pos = self.get_element_at_pos(self.cursor_pos);
            if let Some(InputElement::Chip { .. }) = element_at_pos {
                self.delete_chip_at_pos(self.cursor_pos);
            } else {
                let mut chars = chars;
                chars.remove(self.cursor_pos);
                self.elements = vec![InputElement::Text(chars.into_iter().collect())];
            }
        }
    }

    fn get_element_at_pos(&self, pos: usize) -> Option<&InputElement> {
        let mut current_pos = 0;
        for element in &self.elements {
            let element_len = match element {
                InputElement::Text(s) => s.len(),
                InputElement::Chip { display, .. } => display.len() + 4,
            };
            if pos >= current_pos && pos < current_pos + element_len {
                return Some(element);
            }
            current_pos += element_len;
        }
        None
    }

    fn delete_chip_at_pos(&mut self, pos: usize) {
        let mut current_pos = 0;
        let mut index_to_remove = None;

        for (i, element) in self.elements.iter().enumerate() {
            let element_len = match element {
                InputElement::Text(s) => s.len(),
                InputElement::Chip { display, .. } => display.len() + 4,
            };
            if pos >= current_pos && pos < current_pos + element_len {
                if matches!(element, InputElement::Chip { .. }) {
                    index_to_remove = Some(i);
                    self.cursor_pos = current_pos;
                }
                break;
            }
            current_pos += element_len;
        }

        if let Some(idx) = index_to_remove {
            self.elements.remove(idx);
            if self.elements.is_empty() {
                self.elements.push(InputElement::text(""));
            }
        }
    }

    fn truncate_at_cursor(&mut self) {
        let content = self.get_content();
        let chars: Vec<char> = content.chars().collect();
        let truncated: String = chars[..self.cursor_pos].iter().collect();
        self.elements = vec![InputElement::Text(truncated)];
    }

    fn move_word_backward(&mut self) {
        let content = self.get_content();
        let chars: Vec<char> = content.chars().collect();
        while self.cursor_pos > 0 && chars[self.cursor_pos - 1].is_whitespace() {
            self.cursor_pos -= 1;
        }
        while self.cursor_pos > 0 && !chars[self.cursor_pos - 1].is_whitespace() {
            self.cursor_pos -= 1;
        }
    }

    fn move_word_forward(&mut self) {
        let content = self.get_content();
        let chars: Vec<char> = content.chars().collect();
        while self.cursor_pos < chars.len() && !chars[self.cursor_pos].is_whitespace() {
            self.cursor_pos += 1;
        }
        while self.cursor_pos < chars.len() && chars[self.cursor_pos].is_whitespace() {
            self.cursor_pos += 1;
        }
    }

    fn history_previous(&mut self) -> InputAction {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            let idx = self.history.len() - self.history_index;
            let content = self.history[idx].clone();
            self.elements = vec![InputElement::Text(content)];
            self.cursor_pos = self.get_content().len();
        }
        InputAction::None
    }

    fn history_next(&mut self) -> InputAction {
        if self.history_index > 0 {
            self.history_index -= 1;
            if self.history_index == 0 {
                self.elements = vec![InputElement::text("")];
            } else {
                let idx = self.history.len() - self.history_index;
                let content = self.history[idx].clone();
                self.elements = vec![InputElement::Text(content)];
            }
            self.cursor_pos = self.get_content().len();
        }
        InputAction::None
    }

    fn submit(&mut self) -> InputAction {
        let content = self.get_content();
        if !content.is_empty() {
            self.history.push(content.clone());
            self.history_index = 0;
        }
        self.elements = vec![InputElement::text("")];
        self.cursor_pos = 0;
        InputAction::Submit(content)
    }

    pub fn get_content(&self) -> String {
        self.elements
            .iter()
            .map(|e| match e {
                InputElement::Text(s) => s.clone(),
                InputElement::Chip { value, .. } => format!("@{}", value),
            })
            .collect()
    }

    pub fn get_chips(&self) -> Vec<(String, String)> {
        self.elements
            .iter()
            .filter_map(|e| match e {
                InputElement::Chip { display, value, .. } => Some((display.clone(), value.clone())),
                _ => None,
            })
            .collect()
    }

    pub fn insert_chip(&mut self, display: String, value: String, color: Color) {
        let chip = InputElement::chip(display, value, color);
        let chip_len = chip.len();

        if self.elements.len() == 1 {
            if let InputElement::Text(ref s) = self.elements[0] {
                if s.is_empty() {
                    self.elements = vec![chip, InputElement::text("")];
                    self.cursor_pos = chip_len;
                    return;
                }
            }
        }

        if self.elements.len() >= 20 {
            return;
        }

        self.elements.push(InputElement::text(""));
        self.elements.push(chip);
        self.cursor_pos = self.get_content().len();
    }

    pub fn clear(&mut self) {
        self.elements = vec![InputElement::text("")];
        self.cursor_pos = 0;
    }

    pub fn set_leader_active(&mut self, active: bool) {
        self.leader_active = active;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, title: &str) {
        let border_color = if self.leader_active {
            self.theme.warning_color()
        } else {
            self.theme.primary_color()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let visible_width = inner.width as usize;
        let content = self.get_content();

        let (display_start, display_end) = match self.overflow_mode {
            OverflowMode::Scroll => {
                let start = self.scroll_x.min(content.len());
                let end = (start + visible_width).min(content.len());
                (start, end)
            }
            _ => (0, content.len().min(visible_width)),
        };

        let display_content = if display_start < display_end {
            content
                .chars()
                .skip(display_start)
                .take(display_end - display_start)
                .collect()
        } else {
            String::new()
        };

        let mut spans: Vec<Span> = Vec::new();
        let mut char_pos = 0;

        let is_shell_command = content.starts_with('!');

        for c in display_content.chars() {
            let style = if is_shell_command && char_pos == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_shell_command {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let actual_cursor_pos =
                if self.cursor_pos > display_start && self.cursor_pos <= display_end {
                    self.cursor_pos - display_start
                } else {
                    usize::MAX
                };

            if char_pos == actual_cursor_pos {
                spans.push(Span::styled(
                    c.to_string(),
                    style.add_modifier(Modifier::REVERSED),
                ));
            } else {
                spans.push(Span::styled(c.to_string(), style));
            }
            char_pos += 1;
        }

        if self.cursor_pos >= display_end && char_pos < visible_width {
            spans.push(Span::styled(
                " ",
                Style::default().add_modifier(Modifier::REVERSED),
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans));
        f.render_widget(paragraph, inner);

        let display_len = display_content.len();
        let cursor_display_pos = if self.cursor_pos > display_start {
            (self.cursor_pos - display_start).min(display_len)
        } else {
            0
        };
        let cursor_x = inner.x + cursor_display_pos as u16;
        let cursor_y = inner.y;
        #[allow(deprecated)]
        f.set_cursor(cursor_x, cursor_y);
    }

    pub fn scroll_left(&mut self) {
        if self.scroll_x > 0 {
            self.scroll_x -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        let content_len = self.get_content().len();
        let visible_width = 78;
        if self.scroll_x + visible_width < content_len {
            self.scroll_x += 1;
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    None,
    Submit(String),
    Cancel,
    ChipSelected(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_widget_new() {
        let theme = crate::theme::Theme::default();
        let input = InputWidget::new(theme);
        assert_eq!(input.elements.len(), 1);
        assert!(input.elements[0].is_empty());
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
    fn test_input_widget_chip() {
        let theme = crate::theme::Theme::default();
        let mut input = InputWidget::new(theme);

        input.insert_chip(
            "main.rs".to_string(),
            "src/main.rs".to_string(),
            Color::Blue,
        );

        assert_eq!(input.get_content(), "@src/main.rs");
        assert_eq!(input.get_chips().len(), 1);
    }

    #[test]
    fn test_input_widget_chip_deletion() {
        let theme = crate::theme::Theme::default();
        let mut input = InputWidget::new(theme);

        input.insert_chip(
            "main.rs".to_string(),
            "src/main.rs".to_string(),
            Color::Blue,
        );
        input.handle_input(KeyEvent::from(KeyCode::Backspace));

        assert_eq!(input.get_chips().len(), 0);
    }
}
