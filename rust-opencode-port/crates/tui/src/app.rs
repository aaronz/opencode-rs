use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct App {
    pub messages: Vec<(String, bool)>,
    pub tool_output: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub agent: String,
    pub provider: String,
    pub show_command_palette: bool,
    pub command_palette_input: String,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tool_output: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: 0,
            agent: "build".to_string(),
            provider: "openai".to_string(),
            show_command_palette: false,
            command_palette_input: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn add_message(&mut self, content: String, is_user: bool) {
        self.messages.push((content, is_user));
    }

    pub fn add_tool_output(&mut self, output: String) {
        self.tool_output.push(output);
    }

    pub fn clear_tool_output(&mut self) {
        self.tool_output.clear();
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            if self.show_command_palette {
                self.handle_command_palette(&mut terminal)?;
            } else {
                self.handle_input(&mut terminal)?;
            }
        }
    }

    fn handle_command_palette(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                    KeyCode::Esc => {
                        self.show_command_palette = false;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Char('\n') => {
                        self.execute_command();
                        self.show_command_palette = false;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Char(c) => {
                        self.command_palette_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.command_palette_input.pop();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn execute_command(&mut self) {
        let cmd = self.command_palette_input.trim();
        match cmd {
            "/plan" => {
                self.agent = "plan".to_string();
            }
            "/build" => {
                self.agent = "build".to_string();
            }
            "/clear" => {
                self.messages.clear();
                self.tool_output.clear();
            }
            "/help" => {
                self.add_message(
                    "Available commands: /plan, /build, /clear, /help".to_string(),
                    false,
                );
            }
            _ => {
                if !cmd.is_empty() {
                    self.add_message(format!("Unknown command: {}", cmd), false);
                }
            }
        }
    }

    fn handle_input(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.show_command_palette = true;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Char('\n') => {
                        let input = self.input.clone();
                        if !input.is_empty() {
                            self.history.push(input.clone());
                            self.history_index = self.history.len();
                            self.add_message(input, true);
                            self.input.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Tab => {
                        self.agent = if self.agent == "build" {
                            "plan".to_string()
                        } else {
                            "build".to_string()
                        };
                    }
                    KeyCode::Up => {
                        if !self.history.is_empty() && self.history_index > 0 {
                            self.history_index -= 1;
                            self.input = self.history[self.history_index].clone();
                        }
                    }
                    KeyCode::Down => {
                        if self.history_index < self.history.len() {
                            self.history_index += 1;
                            self.input = if self.history_index < self.history.len() {
                                self.history[self.history_index].clone()
                            } else {
                                String::new()
                            };
                        }
                    }
                    KeyCode::PageUp => {
                        self.scroll_offset = self.scroll_offset.saturating_add(10);
                    }
                    KeyCode::PageDown => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.area();

        if self.show_command_palette {
            self.draw_command_palette(f);
            return;
        }

        let (messages_height, tool_height) = if self.tool_output.is_empty() {
            (area.height.saturating_sub(2), 0)
        } else {
            let tool_height = 5.min(area.height / 3);
            (area.height.saturating_sub(tool_height + 2), tool_height)
        };

        let messages_area = Rect::new(area.x, area.y, area.width, messages_height);

        let input_area = Rect::new(area.x, messages_height, area.width, 1);

        let status_area = Rect::new(area.x, area.height - 1, area.width, 1);

        let messages: Vec<Line> = self
            .messages
            .iter()
            .skip(self.scroll_offset)
            .take(messages_height as usize)
            .map(|(content, is_user)| {
                if *is_user {
                    Line::from(format!("> {}", content))
                } else {
                    Line::from(content.clone())
                }
            })
            .collect();

        let messages_block = Block::default().title("Messages").borders(Borders::ALL);

        f.render_widget(
            Paragraph::new(messages).block(messages_block),
            messages_area,
        );

        if tool_height > 0 {
            let tool_area = Rect::new(area.x, messages_height, area.width, tool_height);

            let tool_output = self.tool_output.join("\n\n");
            let tool_block = Block::default().title("Tool Output").borders(Borders::ALL);

            f.render_widget(Paragraph::new(tool_output).block(tool_block), tool_area);
        }

        let input_block = Block::default().title("Input").borders(Borders::ALL);

        f.render_widget(
            Paragraph::new(format!("> {}", self.input)).block(input_block),
            input_area,
        );

        let status = format!(
            "Agent: {} | Provider: {} | Ctrl+P: Commands | Ctrl+C: Quit | ↑↓: History",
            self.agent, self.provider
        );
        f.render_widget(Paragraph::new(status), status_area);
    }

    fn draw_command_palette(&self, f: &mut Frame) {
        let area = f.area();
        let palette_width = 40;
        let palette_height = 10;
        let x = (area.width - palette_width) / 2;
        let y = (area.height - palette_height) / 2;

        let palette_area = Rect::new(x, y, palette_width, palette_height);

        let block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL);

        f.render_widget(block, palette_area);

        let input_area = Rect::new(x + 1, y + 1, palette_width - 2, 1);
        f.render_widget(
            Paragraph::new(format!("> {}", self.command_palette_input)),
            input_area,
        );

        let help_area = Rect::new(x + 1, y + 3, palette_width - 2, palette_height - 4);
        let help_text = vec![
            Line::from("/plan  - Switch to plan agent"),
            Line::from("/build - Switch to build agent"),
            Line::from("/clear - Clear messages"),
            Line::from("/help  - Show help"),
            Line::from("Esc   - Close palette"),
        ];
        f.render_widget(Paragraph::new(help_text), help_area);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
