use crate::dialogs::*;
use crate::theme::{Theme, ThemeManager};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone)]
pub struct MessageMeta {
    pub content: String,
    pub is_user: bool,
    pub token_count: Option<usize>,
    pub duration_ms: Option<u64>,
    pub tool_calls: Vec<String>,
}

impl MessageMeta {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_user: true,
            token_count: None,
            duration_ms: None,
            tool_calls: Vec::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_user: false,
            token_count: None,
            duration_ms: None,
            tool_calls: Vec::new(),
        }
    }

    pub fn with_tokens(mut self, tokens: usize) -> Self {
        self.token_count = Some(tokens);
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Chat,
    Timeline,
    ForkDialog,
    CommandPalette,
    Settings,
    ModelSelection,
    ProviderManagement,
    FileSelection,
    DirectorySelection,
    ReleaseNotes,
}

pub struct App {
    pub messages: Vec<MessageMeta>,
    pub tool_output: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub agent: String,
    pub provider: String,
    pub mode: AppMode,
    pub command_palette_input: String,
    pub scroll_offset: usize,
    pub timeline_state: ListState,
    pub fork_name_input: String,
    pub show_metadata: bool,
    pub theme_manager: ThemeManager,
    pub settings_dialog: SettingsDialog,
    pub model_selection_dialog: ModelSelectionDialog,
    pub provider_management_dialog: ProviderManagementDialog,
    pub file_selection_dialog: FileSelectionDialog,
    pub directory_selection_dialog: DirectorySelectionDialog,
    pub release_notes_dialog: ReleaseNotesDialog,
}

impl App {
    pub fn new() -> Self {
        let mut timeline_state = ListState::default();
        timeline_state.select(None);
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.current().clone();
        Self {
            messages: Vec::new(),
            tool_output: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: 0,
            agent: "build".to_string(),
            provider: "openai".to_string(),
            mode: AppMode::Chat,
            command_palette_input: String::new(),
            scroll_offset: 0,
            timeline_state,
            fork_name_input: String::new(),
            show_metadata: false,
            theme_manager,
            settings_dialog: SettingsDialog::new(theme.clone()),
            model_selection_dialog: ModelSelectionDialog::new(theme.clone()),
            provider_management_dialog: ProviderManagementDialog::new(theme.clone()),
            file_selection_dialog: FileSelectionDialog::new(theme.clone()),
            directory_selection_dialog: DirectorySelectionDialog::new(theme.clone()),
            release_notes_dialog: ReleaseNotesDialog::new(theme),
        }
    }

    pub fn add_message(&mut self, content: String, is_user: bool) {
        self.messages.push(if is_user {
            MessageMeta::user(content)
        } else {
            MessageMeta::assistant(content)
        });
    }

    pub fn add_message_with_meta(&mut self, meta: MessageMeta) {
        self.messages.push(meta);
    }

    pub fn add_tool_output(&mut self, output: String) {
        self.tool_output.push(output);
    }

    pub fn clear_tool_output(&mut self) {
        self.tool_output.clear();
    }

    pub fn load_theme(&mut self, path: &str) -> Result<(), String> {
        self.theme_manager.load_theme_file(path)
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme_manager.set_theme(theme);
    }

    fn theme(&self) -> &Theme {
        self.theme_manager.current()
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            match self.mode {
                AppMode::CommandPalette => self.handle_command_palette(&mut terminal)?,
                AppMode::Timeline => self.handle_timeline(&mut terminal)?,
                AppMode::ForkDialog => self.handle_fork_dialog(&mut terminal)?,
                AppMode::Chat => self.handle_input(&mut terminal)?,
                AppMode::Settings => self.handle_settings_dialog(&mut terminal)?,
                AppMode::ModelSelection => self.handle_model_selection_dialog(&mut terminal)?,
                AppMode::ProviderManagement => {
                    self.handle_provider_management_dialog(&mut terminal)?
                }
                AppMode::FileSelection => self.handle_file_selection_dialog(&mut terminal)?,
                AppMode::DirectorySelection => {
                    self.handle_directory_selection_dialog(&mut terminal)?
                }
                AppMode::ReleaseNotes => self.handle_release_notes_dialog(&mut terminal)?,
            }
        }
    }

    fn handle_timeline(
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
                    KeyCode::Esc | KeyCode::Char('t') => {
                        self.mode = AppMode::Chat;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = self.messages.len();
                        if len > 0 {
                            let next = self
                                .timeline_state
                                .selected()
                                .map(|i| (i + 1).min(len - 1))
                                .unwrap_or(0);
                            self.timeline_state.select(Some(next));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(i) = self.timeline_state.selected() {
                            if i > 0 {
                                self.timeline_state.select(Some(i - 1));
                            }
                        }
                    }
                    KeyCode::Char('m') => {
                        self.show_metadata = !self.show_metadata;
                    }
                    KeyCode::Char('f') => {
                        self.mode = AppMode::ForkDialog;
                        self.fork_name_input.clear();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_fork_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = AppMode::Timeline;
                        self.fork_name_input.clear();
                    }
                    KeyCode::Enter => {
                        let fork_point = self
                            .timeline_state
                            .selected()
                            .unwrap_or(self.messages.len().saturating_sub(1));
                        self.execute_fork(fork_point);
                        self.mode = AppMode::Chat;
                        self.fork_name_input.clear();
                    }
                    KeyCode::Char(c) => {
                        self.fork_name_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.fork_name_input.pop();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn execute_fork(&mut self, fork_point: usize) {
        let forked: Vec<MessageMeta> =
            self.messages[..=fork_point.min(self.messages.len().saturating_sub(1))].to_vec();
        let name = if self.fork_name_input.is_empty() {
            format!("Fork at message {}", fork_point + 1)
        } else {
            self.fork_name_input.clone()
        };
        self.messages = forked;
        self.add_message(format!("[Session forked: {}]", name), false);
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
                        self.mode = AppMode::Chat;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Enter => {
                        self.execute_command();
                        self.mode = AppMode::Chat;
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
        let cmd = self.command_palette_input.trim().to_string();
        match cmd.as_str() {
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
                    "Commands: /plan, /build, /clear, /timeline, /fork, /meta, /help".to_string(),
                    false,
                );
            }
            "/timeline" => {
                self.mode = AppMode::Timeline;
            }
            "/fork" => {
                self.mode = AppMode::ForkDialog;
                self.fork_name_input.clear();
            }
            "/meta" => {
                self.show_metadata = !self.show_metadata;
            }
            "/settings" => {
                self.mode = AppMode::Settings;
            }
            "/models" => {
                self.mode = AppMode::ModelSelection;
            }
            "/providers" => {
                self.mode = AppMode::ProviderManagement;
            }
            "/release-notes" => {
                self.mode = AppMode::ReleaseNotes;
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
                        self.mode = AppMode::CommandPalette;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.mode = AppMode::Timeline;
                        if !self.messages.is_empty() {
                            self.timeline_state.select(Some(self.messages.len() - 1));
                        }
                    }
                    KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.mode = AppMode::Settings;
                    }
                    KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.mode = AppMode::ModelSelection;
                    }
                    KeyCode::Enter => {
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

    fn draw(&mut self, f: &mut Frame) {
        match self.mode.clone() {
            AppMode::Timeline => self.draw_timeline(f),
            AppMode::ForkDialog => {
                self.draw_timeline(f);
                self.draw_fork_dialog(f);
            }
            AppMode::CommandPalette => {
                self.draw_chat(f);
                self.draw_command_palette(f);
            }
            AppMode::Chat => self.draw_chat(f),
            AppMode::Settings => {
                self.draw_chat(f);
                self.settings_dialog.draw(f, f.area());
            }
            AppMode::ModelSelection => {
                self.draw_chat(f);
                self.model_selection_dialog.draw(f, f.area());
            }
            AppMode::ProviderManagement => {
                self.draw_chat(f);
                self.provider_management_dialog.draw(f, f.area());
            }
            AppMode::FileSelection => {
                self.draw_chat(f);
                self.file_selection_dialog.draw(f, f.area());
            }
            AppMode::DirectorySelection => {
                self.draw_chat(f);
                self.directory_selection_dialog.draw(f, f.area());
            }
            AppMode::ReleaseNotes => {
                self.draw_chat(f);
                self.release_notes_dialog.draw(f, f.area());
            }
        }
    }

    fn draw_chat(&self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme();

        let (messages_height, tool_height) = if self.tool_output.is_empty() {
            (area.height.saturating_sub(3), 0)
        } else {
            let tool_height = 5.min(area.height / 3);
            (area.height.saturating_sub(tool_height + 3), tool_height)
        };

        let messages_area = Rect::new(area.x, area.y, area.width, messages_height);
        let input_area = Rect::new(area.x, messages_height, area.width, 2);
        let status_area = Rect::new(area.x, area.height - 1, area.width, 1);

        let messages: Vec<Line> = self
            .messages
            .iter()
            .skip(self.scroll_offset)
            .take(messages_height as usize)
            .flat_map(|msg| {
                let prefix = if msg.is_user { "> " } else { "  " };
                let color = if msg.is_user {
                    theme.primary_color()
                } else {
                    theme.foreground_color()
                };
                let mut lines = vec![Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(msg.content.clone()),
                ])];
                if self.show_metadata {
                    let mut meta_parts = Vec::new();
                    if let Some(tokens) = msg.token_count {
                        meta_parts.push(format!("tokens:{}", tokens));
                    }
                    if let Some(dur) = msg.duration_ms {
                        meta_parts.push(format!("{}ms", dur));
                    }
                    if !meta_parts.is_empty() {
                        lines.push(Line::from(Span::styled(
                            format!("  [{}]", meta_parts.join(" ")),
                            Style::default().fg(theme.muted_color()),
                        )));
                    }
                }
                lines
            })
            .collect();

        let messages_block = Block::default()
            .title("Messages")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color()));
        f.render_widget(
            Paragraph::new(messages).block(messages_block),
            messages_area,
        );

        if tool_height > 0 {
            let tool_area = Rect::new(area.x, messages_height, area.width, tool_height);
            let tool_output = self.tool_output.join("\n\n");
            let tool_block = Block::default()
                .title("Tool Output")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()));
            f.render_widget(Paragraph::new(tool_output).block(tool_block), tool_area);
        }

        let input_block = Block::default()
            .title("Input")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary_color()));
        f.render_widget(
            Paragraph::new(format!("> {}", self.input)).block(input_block),
            input_area,
        );

        let status = format!(
            " Agent: {} | Provider: {} | ^P: Commands | ^T: Timeline | ^,: Settings | ^M: Models | ^C: Quit",
            self.agent, self.provider
        );
        f.render_widget(
            Paragraph::new(status).style(Style::default().fg(theme.muted_color())),
            status_area,
        );
    }

    fn draw_timeline(&mut self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme_manager.current().clone();

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let role = if msg.is_user { "USER" } else { " AI " };
                let color = if msg.is_user {
                    theme.primary_color()
                } else {
                    theme.secondary_color()
                };
                let preview: String = msg.content.chars().take(area.width as usize - 20).collect();

                let mut spans = vec![
                    Span::styled(
                        format!("[{:3}] ", i + 1),
                        Style::default().fg(theme.muted_color()),
                    ),
                    Span::styled(
                        format!("[{}] ", role),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(preview),
                ];

                if self.show_metadata {
                    if let Some(tokens) = msg.token_count {
                        spans.push(Span::styled(
                            format!(" ~{}t", tokens),
                            Style::default().fg(theme.muted_color()),
                        ));
                    }
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let block = Block::default()
            .title(format!(
                "Timeline ({} messages) | ↑↓: navigate | m: metadata | f: fork | Esc: back",
                self.messages.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color()));

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(theme.primary_color()).fg(Color::Black));

        f.render_stateful_widget(list, area, &mut self.timeline_state);
    }

    fn draw_fork_dialog(&self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme();
        let dialog_width = 50.min(area.width - 4);
        let dialog_height = 6;
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let fork_point = self
            .timeline_state
            .selected()
            .unwrap_or(self.messages.len().saturating_sub(1));
        let block = Block::default()
            .title(format!("Fork Session at message {}", fork_point + 1))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_color()));

        let content = vec![
            Line::from(Span::raw(format!("Fork name: {}_", self.fork_name_input))),
            Line::from(""),
            Line::from(Span::styled(
                "Enter: confirm | Esc: cancel",
                Style::default().fg(theme.muted_color()),
            )),
        ];

        f.render_widget(Paragraph::new(content).block(block), dialog_area);
    }

    fn draw_command_palette(&self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme();
        let palette_width = 44.min(area.width - 4);
        let palette_height = 12;
        let x = (area.width - palette_width) / 2;
        let y = (area.height - palette_height) / 2;
        let palette_area = Rect::new(x, y, palette_width, palette_height);

        f.render_widget(Clear, palette_area);

        let block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary_color()));

        let commands = vec![
            Line::from(format!("> {}", self.command_palette_input)),
            Line::from(""),
            Line::from(Span::styled(
                "/plan      Switch to plan agent",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/build     Switch to build agent",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/clear     Clear messages",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/timeline  Open timeline view",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/fork      Fork at current message",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/meta      Toggle metadata display",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/settings  Open settings dialog",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/models    Open model selection",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/providers Open provider management",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/help      Show help",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "Esc        Close",
                Style::default().fg(theme.muted_color()),
            )),
        ];

        f.render_widget(Paragraph::new(commands).block(block), palette_area);
    }

    fn handle_settings_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.settings_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_model_selection_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.model_selection_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Confirm(model_id) => {
                        self.add_message(format!("Selected model: {}", model_id), false);
                        self.mode = AppMode::Chat;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_provider_management_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.provider_management_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Navigate(nav) => {
                        self.add_message(format!("Navigating to: {}", nav), false);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_file_selection_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.file_selection_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Confirm(path) => {
                        self.add_message(format!("Selected file: {}", path), false);
                        self.mode = AppMode::Chat;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_directory_selection_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.directory_selection_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Confirm(path) => {
                        self.add_message(format!("Selected directory: {}", path), false);
                        self.mode = AppMode::Chat;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_release_notes_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.release_notes_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
