use crate::command::{CommandAction, CommandRegistry};
use crate::components::{
    FileTree, InputWidget, SkillInfo, SkillsPanel, StatusBar, StatusPopoverType, TerminalPanel,
    TitleBar, TitleBarAction,
};
use crate::dialogs::*;
use crate::input::{InputBox, InputParser, InputProcessor, InputToken};
use crate::layout::LayoutManager;
use crate::patch_preview::{PatchDecision, PatchPreview};
use crate::right_panel::{RightPanel, RightPanelContent, RightPanelRenderData};
use crate::session::SessionManager;
use crate::shell_handler::ShellHandler;
use crate::file_ref_handler::FileRefHandler;
use crate::theme::{Theme, ThemeManager};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use opencode_auth::CredentialStore;
use opencode_core::{CostCalculator, SkillResolver, SkillState, TokenCounter};
use opencode_llm::{
    BrowserAuthModelInfo, OpenAiBrowserAuthService, OpenAiBrowserAuthStore, OpenAiBrowserSession,
    OpenAiProvider, Provider, ProviderConfig,
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
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub enum LlmEvent {
    Chunk(String),
    Done,
    Error(String),
}

pub enum ConnectEvent {
    BrowserOpened(String),
    AuthComplete(OpenAiBrowserSession, Vec<BrowserAuthModelInfo>),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeaderKeyState {
    Idle,
    WaitingForAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkMode {
    Plan,
    Build,
}

impl WorkMode {
    pub fn toggle(&self) -> Self {
        match self {
            WorkMode::Plan => WorkMode::Build,
            WorkMode::Build => WorkMode::Plan,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WorkMode::Plan => "PLAN",
            WorkMode::Build => "BUILD",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Running,
    Success,
    Failed(i32),
}

impl ToolStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            ToolStatus::Running => "⠋",
            ToolStatus::Success => "✔",
            ToolStatus::Failed(_) => "✖",
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self, ToolStatus::Running)
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub status: ToolStatus,
    pub output: String,
    pub expanded: bool,
    pub start_time: Instant,
    pub duration_ms: Option<u64>,
}

impl ToolCall {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ToolStatus::Running,
            output: String::new(),
            expanded: false,
            start_time: Instant::now(),
            duration_ms: None,
        }
    }

    pub fn success(mut self, output: impl Into<String>) -> Self {
        self.status = ToolStatus::Success;
        self.output = output.into();
        self.duration_ms = Some(self.start_time.elapsed().as_millis() as u64);
        self
    }

    pub fn failed(mut self, exit_code: i32, output: impl Into<String>) -> Self {
        self.status = ToolStatus::Failed(exit_code);
        self.output = output.into();
        self.duration_ms = Some(self.start_time.elapsed().as_millis() as u64);
        self
    }

    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }
}

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub velocity: i32,
    pub acceleration: f32,
    pub max_velocity: i32,
    pub friction: f32,
    pub enabled: bool,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            velocity: 0,
            acceleration: 0.5,
            max_velocity: 20,
            friction: 0.9,
            enabled: true,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.enabled {
            self.velocity = (self.velocity + self.acceleration as i32).min(self.max_velocity);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.enabled {
            self.velocity = (self.velocity - self.acceleration as i32).max(-self.max_velocity);
        }
    }

    pub fn apply(&self, offset: usize) -> usize {
        if self.velocity == 0 {
            return offset;
        }
        let new_offset = offset as i32 + self.velocity;
        if new_offset < 0 {
            0
        } else {
            new_offset as usize
        }
    }

    pub fn decelerate(&mut self) {
        if self.enabled {
            self.velocity = (self.velocity as f32 * self.friction) as i32;
            if self.velocity.abs() < 1 {
                self.velocity = 0;
            }
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

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
    SlashCommand,
    DiffReview,
    Sessions,
    Settings,
    ModelSelection,
    ProviderManagement,
    ConnectProvider,
    ConnectMethod,
    ConnectProgress,
    ConnectModel,
    FileSelection,
    DirectorySelection,
    ReleaseNotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiState {
    Idle,
    Composing,
    Submitting,
    Streaming,
    ExecutingTool,
    AwaitingPermission,
    ShowingDiff,
    ShowingError,
    Aborting,
    Reconnecting,
    Paused,
}

impl TuiState {
    pub fn can_accept_input(&self) -> bool {
        matches!(self, Self::Idle | Self::Composing)
    }

    pub fn is_interruptible(&self) -> bool {
        matches!(self, Self::Streaming | Self::ExecutingTool)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Streaming | Self::ExecutingTool | Self::Submitting | Self::Aborting
        )
    }

    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            Self::Streaming | Self::ExecutingTool | Self::AwaitingPermission
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Composing => "composing",
            Self::Submitting => "submitting",
            Self::Streaming => "streaming",
            Self::ExecutingTool => "executing_tool",
            Self::AwaitingPermission => "awaiting_permission",
            Self::ShowingDiff => "showing_diff",
            Self::ShowingError => "showing_error",
            Self::Aborting => "aborting",
            Self::Reconnecting => "reconnecting",
            Self::Paused => "paused",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Idle | Self::ShowingError)
    }

    pub fn allows_input_editing(&self) -> bool {
        matches!(self, Self::Idle | Self::Composing | Self::ShowingError)
    }

    pub fn valid_transitions(&self) -> Vec<TuiState> {
        match self {
            Self::Idle => vec![Self::Composing, Self::Paused, Self::ShowingError],
            Self::Composing => vec![Self::Idle, Self::Submitting, Self::Paused],
            Self::Submitting => vec![Self::Streaming, Self::ShowingError, Self::Idle],
            Self::Streaming => vec![Self::ExecutingTool, Self::Aborting, Self::Idle],
            Self::ExecutingTool => vec![Self::Streaming, Self::AwaitingPermission, Self::Aborting, Self::Idle],
            Self::AwaitingPermission => vec![Self::ExecutingTool, Self::Aborting, Self::Idle],
            Self::ShowingDiff => vec![Self::Idle, Self::Streaming],
            Self::ShowingError => vec![Self::Idle, Self::Composing],
            Self::Aborting => vec![Self::Idle],
            Self::Reconnecting => vec![Self::Idle, Self::Streaming],
            Self::Paused => vec![Self::Idle, Self::Composing],
        }
    }

    pub fn can_transition_to(&self, target: TuiState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

pub struct App {
    pub messages: Vec<MessageMeta>,
    pub tool_calls: Vec<ToolCall>,
    pub input: String,
    pub input_widget: InputWidget,
    pub history: Vec<String>,
    pub history_index: usize,
    history_file: std::path::PathBuf,
    layout_file: std::path::PathBuf,
    pub agent: String,
    pub provider: String,
    llm_provider: Option<std::sync::Arc<dyn Provider + Send + Sync>>,
    llm_rx: Option<mpsc::Receiver<LlmEvent>>,
    connect_rx: Option<mpsc::Receiver<ConnectEvent>>,
    pub mode: AppMode,
    pub tui_state: TuiState,
    pub reconnect_timeout: Option<std::time::Instant>,
    pub command_palette_input: String,
    pub command_registry: CommandRegistry,
    pub slash_command_dialog: SlashCommandOverlay,
    pub diff_review_dialog: Option<DiffReviewOverlay>,
    pub session_manager: SessionManager,
    pub scroll_offset: usize,
    pub scroll_state: ScrollState,
    pub timeline_state: ListState,
    pub fork_name_input: String,
    pub show_metadata: bool,
    pub theme_manager: ThemeManager,
    pub settings_dialog: SettingsDialog,
    pub model_selection_dialog: ModelSelectionDialog,
    pub provider_management_dialog: ProviderManagementDialog,
    pub connect_provider_dialog: ConnectProviderDialog,
    pub connect_method_dialog: Option<ConnectMethodDialog>,
    pub connect_model_dialog: Option<ConnectModelDialog>,
    pub file_selection_dialog: FileSelectionDialog,
    pub directory_selection_dialog: DirectorySelectionDialog,
    pub release_notes_dialog: ReleaseNotesDialog,
    pub file_tree: Option<FileTree>,
    pub show_file_tree: bool,
    pub layout_manager: LayoutManager,
    pub right_panel: RightPanel,
    pub patch_preview: PatchPreview,
    pub title_bar: TitleBar,
    pub show_title_bar: bool,
    pub status_bar: StatusBar,
    pub terminal_panel: TerminalPanel,
    pub show_terminal: bool,
    pub skills_panel: SkillsPanel,
    pub show_skills_panel: bool,
    pub leader_key_state: LeaderKeyState,
    pub leader_key_timeout: Option<Instant>,
    pub work_mode: WorkMode,
    pub is_llm_generating: bool,
    pub partial_response: String,
    pub dropped_files: Vec<std::path::PathBuf>,
    pub pending_connect_provider: Option<String>,
    pub pending_connect_method: Option<String>,
    pub pending_browser_session: Option<OpenAiBrowserSession>,
    pub pending_browser_models: Vec<BrowserAuthModelInfo>,
    pub token_counter: TokenCounter,
    pub cost_calculator: CostCalculator,
    pub session_token_id: String,
    pub pending_input_tokens: usize,
    pub total_cost_usd: f64,
    pub budget_limit_usd: Option<f64>,
    skill_resolver: SkillResolver,
    input_parser: InputParser,
    input_box: InputBox,
    input_processor: InputProcessor,
    pending_shell_command: Option<String>,
    #[allow(dead_code)]
    shell_handler: ShellHandler,
    #[allow(dead_code)]
    file_ref_handler: FileRefHandler,
}

impl App {
    pub fn new() -> Self {
        let mut timeline_state = ListState::default();
        timeline_state.select(None);
        let mut theme_manager = ThemeManager::new();
        let _ = theme_manager.load_from_config();
        let theme = theme_manager.current().clone();

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("opencode-rs");
        std::fs::create_dir_all(&config_dir).ok();
        let history_file = config_dir.join("history.txt");

        let mut history = Vec::new();
        if let Ok(content) = std::fs::read_to_string(&history_file) {
            history = content.lines().map(|s| s.to_string()).take(100).collect();
        }

        let layout_file = config_dir.join("layout.txt");
        let layout_manager = LayoutManager::load_from_file(&layout_file).unwrap_or_default();

        let session_token_id = uuid::Uuid::new_v4().to_string();
        let mut token_counter = TokenCounter::new();
        token_counter.set_active_session(session_token_id.clone());
        let budget_limit_usd = std::env::var("OPENCODE_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok());

        let skill_resolver = SkillResolver::default();
        let mut skills_panel = SkillsPanel::new(theme.clone());
        let skill_infos = skill_resolver
            .list_skills()
            .unwrap_or_default()
            .into_iter()
            .map(|(skill, state)| SkillInfo {
                name: skill.name,
                description: skill.description,
                enabled: state == SkillState::Enabled,
            })
            .collect();
        skills_panel.set_skills(skill_infos);
        let command_registry = CommandRegistry::new();
        let input_box = InputBox::new(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            &command_registry,
        );

        Self {
            messages: Vec::new(),
            tool_calls: Vec::new(),
            input: String::new(),
            input_widget: InputWidget::new(theme.clone()),
            history,
            history_index: 0,
            history_file,
            layout_file,
            agent: "build".to_string(),
            provider: "openai".to_string(),
            llm_provider: None,
            llm_rx: None,
            connect_rx: None,
            mode: AppMode::Chat,
            tui_state: TuiState::Idle,
            reconnect_timeout: None,
            command_palette_input: String::new(),
            command_registry,
            slash_command_dialog: SlashCommandOverlay::new(theme.clone()),
            diff_review_dialog: None,
            session_manager: SessionManager::with_file(config_dir.join("sessions.txt")),
            scroll_offset: 0,
            scroll_state: ScrollState::new(),
            timeline_state,
            fork_name_input: String::new(),
            show_metadata: false,
            theme_manager,
            settings_dialog: SettingsDialog::new(theme.clone()),
            model_selection_dialog: ModelSelectionDialog::new(theme.clone()),
            provider_management_dialog: ProviderManagementDialog::new(theme.clone()),
            connect_provider_dialog: ConnectProviderDialog::new(theme.clone()),
            connect_method_dialog: None,
            connect_model_dialog: None,
            file_selection_dialog: FileSelectionDialog::new(theme.clone()),
            directory_selection_dialog: DirectorySelectionDialog::new(theme.clone()),
            release_notes_dialog: ReleaseNotesDialog::new(theme.clone()),
            file_tree: None,
            show_file_tree: false,
            right_panel: RightPanel::new(theme.clone()),
            layout_manager,
            patch_preview: PatchPreview::new(),
            title_bar: TitleBar::new(theme.clone()),
            show_title_bar: true,
            status_bar: StatusBar::new(theme.clone()),
            terminal_panel: TerminalPanel::new(theme),
            show_terminal: false,
            skills_panel,
            show_skills_panel: false,
            leader_key_state: LeaderKeyState::Idle,
            leader_key_timeout: None,
            work_mode: WorkMode::Build,
            is_llm_generating: false,
            partial_response: String::new(),
            dropped_files: Vec::new(),
            pending_connect_provider: None,
            pending_connect_method: None,
            pending_browser_session: None,
            pending_browser_models: Vec::new(),
            token_counter,
            cost_calculator: CostCalculator::new(),
            session_token_id,
            pending_input_tokens: 0,
            total_cost_usd: 0.0,
            budget_limit_usd,
            skill_resolver,
            input_parser: InputParser::new(),
            input_box,
            input_processor: InputProcessor::new(),
            pending_shell_command: None,
            shell_handler: ShellHandler::new(),
            file_ref_handler: FileRefHandler::new(),
        }
    }

    fn begin_connect_flow(&mut self) {
        self.pending_connect_provider = None;
        self.pending_connect_method = None;
        self.pending_browser_session = None;
        self.pending_browser_models.clear();
        self.connect_method_dialog = None;
        self.connect_model_dialog = None;
        self.mode = AppMode::ConnectProvider;
    }

    fn handle_connect_provider_confirm(&mut self, provider_id: String) {
        self.pending_connect_provider = Some(provider_id.clone());
        let theme = self.theme_manager.current().clone();
        self.connect_method_dialog = Some(ConnectMethodDialog::new(theme, provider_id));
        self.mode = AppMode::ConnectMethod;
    }

    fn handle_connect_method_confirm(&mut self, method: String) {
        self.pending_connect_method = Some(method);
        if self.pending_connect_provider.as_deref() == Some("openai")
            && self.pending_connect_method.as_deref() == Some("browser")
        {
            self.start_openai_browser_connect();
        } else {
            self.add_message("Selected connect method is not implemented yet".to_string(), false);
            self.mode = AppMode::Chat;
        }
    }

    fn start_openai_browser_connect(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        self.mode = AppMode::ConnectProgress;

        std::thread::spawn(move || {
            let service = OpenAiBrowserAuthService::new();
            let listener = match service.start_local_callback_listener() {
                Ok(listener) => listener,
                Err(error) => {
                    let _ = tx.send(ConnectEvent::Failed(error.to_string()));
                    return;
                }
            };

            let request = listener.request();
            let url = service.build_authorize_url(&request);
            if let Err(error) = open_external(&url) {
                let _ = tx.send(ConnectEvent::Failed(error));
                return;
            }
            let _ = tx.send(ConnectEvent::BrowserOpened(url.clone()));

            let callback = match listener.wait_for_callback() {
                Ok(callback) => callback,
                Err(error) => {
                    let _ = tx.send(ConnectEvent::Failed(error.to_string()));
                    return;
                }
            };

            let session = match service.exchange_code(callback, &request) {
                Ok(session) => session,
                Err(error) => {
                    let _ = tx.send(ConnectEvent::Failed(error.to_string()));
                    return;
                }
            };

            let provider = OpenAiProvider::new_browser_auth(
                session.clone(),
                "gpt-5.3-codex".to_string(),
                OpenAiBrowserAuthStore::from_default_location(),
            );

            let runtime = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = tx.send(ConnectEvent::Failed(error.to_string()));
                    return;
                }
            };

            let models = match runtime.block_on(provider.list_browser_auth_models()) {
                Ok(models) => models,
                Err(error) => {
                    let _ = tx.send(ConnectEvent::Failed(error.to_string()));
                    return;
                }
            };

            let _ = tx.send(ConnectEvent::AuthComplete(session, models));
        });
    }

    fn complete_browser_auth(
        &mut self,
        session: OpenAiBrowserSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        let store = OpenAiBrowserAuthStore::from_default_location();
        let _ = store.save(&session);
        self.pending_browser_session = Some(session);
        self.pending_browser_models = models.clone();
        self.connect_model_dialog = Some(ConnectModelDialog::new(
            self.theme_manager.current().clone(),
            models,
        ));
        self.mode = AppMode::ConnectModel;
    }

    fn handle_connect_model_confirm(&mut self, model_id: String) -> Result<(), String> {
        let session = self
            .pending_browser_session
            .clone()
            .ok_or_else(|| "missing browser session".to_string())?;
        let store = OpenAiBrowserAuthStore::from_default_location();
        store.save(&session).map_err(|e| e.to_string())?;

        self.provider = "openai".to_string();
        unsafe {
            std::env::set_var("OPENAI_MODEL", &model_id);
            std::env::set_var("OPENCODE_MODEL", &model_id);
        }
        self.llm_provider = Some(std::sync::Arc::new(OpenAiProvider::new_browser_auth(
            session,
            model_id,
            store,
        )));
        self.mode = AppMode::Chat;
        Ok(())
    }

    fn handle_connect_model_cancel(&mut self) {
        self.mode = AppMode::Chat;
    }

    #[cfg(test)]
    fn complete_browser_auth_for_test(
        &mut self,
        session: OpenAiBrowserSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        self.complete_browser_auth(session, models);
    }

    #[cfg(test)]
    fn prime_connect_state_for_test(&mut self) {
        self.complete_browser_auth(
            OpenAiBrowserSession {
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 60_000,
                account_id: Some("acct_123".to_string()),
            },
            vec![BrowserAuthModelInfo {
                id: "gpt-5.3-codex".to_string(),
                name: "GPT-5.3 Codex".to_string(),
            }],
        );
    }

    /// Initialize the LLM provider based on the provider name
    pub fn init_llm_provider(&mut self) -> Result<(), String> {
        let credential_store = CredentialStore::new();
        let stored_credential = credential_store.load(&self.provider).ok().flatten();

        let api_key = stored_credential
            .as_ref()
            .map(|c| c.api_key.clone())
            .filter(|k| !k.trim().is_empty())
            .or_else(|| std::env::var("OPENCODE_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .unwrap_or_default();

        if api_key.is_empty() && self.provider != "ollama" {
            return Err("No API key found. Set OPENCODE_API_KEY or OPENAI_API_KEY".to_string());
        }

        let model = std::env::var("OPENCODE_MODEL")
            .or_else(|_| std::env::var("OPENAI_MODEL"))
            .unwrap_or_else(|_| "gpt-4o".to_string());

        let config = ProviderConfig {
            model: model.clone(),
            api_key,
            temperature: 0.7,
        };

        self.llm_provider = match self.provider.as_str() {
            "openai" => Some(std::sync::Arc::new(opencode_llm::OpenAiProvider::new(
                config.api_key.clone(),
                config.model.clone(),
            ))),
            "anthropic" => Some(std::sync::Arc::new(opencode_llm::AnthropicProvider::new(
                config.api_key.clone(),
                config.model.clone(),
            ))),
            "ollama" => Some(std::sync::Arc::new(opencode_llm::OllamaProvider::new(
                config.model.clone(),
                stored_credential
                    .as_ref()
                    .and_then(|c| c.base_url.clone())
                    .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
                    .or_else(|| Some("http://localhost:11434".to_string())),
            ))),
            _ => {
                // Default to OpenAI
                Some(std::sync::Arc::new(opencode_llm::OpenAiProvider::new(
                    config.api_key.clone(),
                    config.model.clone(),
                )))
            }
        };

        Ok(())
    }

    pub fn add_message(&mut self, content: String, is_user: bool) {
        self.messages.push(if is_user {
            MessageMeta::user(content.clone())
        } else {
            MessageMeta::assistant(content.clone())
        });

        if is_user && !content.is_empty() {
            self.history.push(content);
            if self.history.len() > 100 {
                self.history.remove(0);
            }
            self.save_history();
        }
    }

    fn save_history(&self) {
        let content = self.history.join("\n");
        let _ = std::fs::write(&self.history_file, content);
    }

    pub fn add_message_with_meta(&mut self, meta: MessageMeta) {
        self.messages.push(meta);
    }

    pub fn add_tool_call(&mut self, tool_call: ToolCall) {
        self.tool_calls.push(tool_call);
        self.set_tui_state(TuiState::ExecutingTool);
    }

    pub fn clear_tool_calls(&mut self) {
        self.tool_calls.clear();
        if self.tui_state == TuiState::ExecutingTool {
            self.set_tui_state(TuiState::Idle);
        }
    }

    pub fn toggle_all_tool_details(&mut self) {
        for tool in &mut self.tool_calls {
            if tool.status.is_terminal() {
                tool.expanded = !tool.expanded;
            }
        }
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

    const LEADER_KEY_TIMEOUT: Duration = Duration::from_millis(2000);

    pub fn activate_leader_key(&mut self) {
        self.leader_key_state = LeaderKeyState::WaitingForAction;
        self.leader_key_timeout = Some(Instant::now());
    }

    pub fn deactivate_leader_key(&mut self) {
        self.leader_key_state = LeaderKeyState::Idle;
        self.leader_key_timeout = None;
    }

    pub fn check_leader_key_timeout(&mut self) {
        if let Some(timeout) = self.leader_key_timeout {
            if timeout.elapsed() >= Self::LEADER_KEY_TIMEOUT {
                self.deactivate_leader_key();
            }
        }
    }

    pub fn check_reconnect_timeout(&mut self) {
        if self.tui_state == TuiState::Reconnecting {
            if let Some(deadline) = self.reconnect_timeout {
                if deadline.elapsed() > std::time::Duration::from_secs(30) {
                    self.set_tui_state(TuiState::Idle);
                    self.reconnect_timeout = None;
                    self.add_message("Reconnection timed out. Please restart.".to_string(), false);
                }
            }
        }
    }

    pub fn start_reconnecting(&mut self) {
        self.set_tui_state(TuiState::Reconnecting);
        self.reconnect_timeout = Some(std::time::Instant::now());
    }

    pub fn on_reconnect_success(&mut self) {
        self.reconnect_timeout = None;
        if self.tui_state == TuiState::Reconnecting {
            self.set_tui_state(TuiState::Idle);
        }
    }

    pub fn is_leader_key_active(&self) -> bool {
        self.leader_key_state == LeaderKeyState::WaitingForAction
    }

    fn cleanup_terminal() -> io::Result<()> {
        use crossterm::{cursor, terminal::LeaveAlternateScreen};
        execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;
        disable_raw_mode()
    }

    pub fn set_tui_state(&mut self, new_state: TuiState) {
        if self.tui_state != new_state {
            self.tui_state = new_state;
        }
    }

    pub fn start_llm_generation(&mut self) {
        self.is_llm_generating = true;
        self.set_tui_state(TuiState::Streaming);
    }

    pub fn end_llm_generation(&mut self) {
        self.is_llm_generating = false;
        if self.tui_state == TuiState::Streaming {
            self.set_tui_state(TuiState::Idle);
        }
    }

    pub fn interrupt_llm_generation(&mut self) {
        if self.is_llm_generating {
            self.set_tui_state(TuiState::Aborting);
            self.is_llm_generating = false;
            if !self.partial_response.is_empty() {
                self.add_message(
                    format!(
                        "[Interrupted]\n\nPartial response:\n{}",
                        self.partial_response
                    ),
                    false,
                );
            } else {
                self.add_message("[Interrupted] Generation stopped".to_string(), false);
            }
            self.partial_response.clear();
        }
    }

    pub fn update_partial_response(&mut self, chunk: String) {
        self.partial_response.push_str(&chunk);
    }

    pub fn handle_file_drop(&mut self, paths: Vec<std::path::PathBuf>) {
        self.dropped_files.clear();
        for path in paths {
            if self.validate_dropped_file(&path) {
                self.dropped_files.push(path.clone());
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                let theme = self.theme_manager.current().clone();
                self.input_widget.insert_chip(
                    file_name.to_string(),
                    path.display().to_string(),
                    theme.primary_color(),
                );
            }
        }
    }

    fn validate_dropped_file(&self, path: &std::path::Path) -> bool {
        if !path.exists() {
            return false;
        }
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.is_dir() {
                return false;
            }
            if let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_lowercase()) {
                let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
                if image_exts.contains(&ext.as_str()) {
                    return true;
                }
            }
            return true;
        }
        false
    }

    pub fn is_image_file(&self, path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_lowercase()) {
            let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
            return image_exts.contains(&ext.as_str());
        }
        false
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        execute!(
            io::stdout(),
            cursor::SetCursorStyle::BlinkingBlock,
            cursor::Show
        )?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            self.check_leader_key_timeout();
            self.check_reconnect_timeout();
            self.check_llm_events();
            self.check_connect_events();

            match self.mode {
                AppMode::CommandPalette => self.handle_command_palette(&mut terminal)?,
                AppMode::SlashCommand => self.handle_slash_command_dialog(&mut terminal)?,
                AppMode::DiffReview => self.handle_diff_review_dialog(&mut terminal)?,
                AppMode::Sessions => self.handle_sessions_dialog(&mut terminal)?,
                AppMode::Timeline => self.handle_timeline(&mut terminal)?,
                AppMode::ForkDialog => self.handle_fork_dialog(&mut terminal)?,
                AppMode::Chat => self.handle_input(&mut terminal)?,
                AppMode::Settings => self.handle_settings_dialog(&mut terminal)?,
                AppMode::ModelSelection => self.handle_model_selection_dialog(&mut terminal)?,
                AppMode::ProviderManagement => {
                    self.handle_provider_management_dialog(&mut terminal)?
                }
                AppMode::ConnectProvider => self.handle_connect_provider_dialog(&mut terminal)?,
                AppMode::ConnectMethod => self.handle_connect_method_dialog(&mut terminal)?,
                AppMode::ConnectProgress => self.handle_connect_progress_dialog(&mut terminal)?,
                AppMode::ConnectModel => self.handle_connect_model_dialog(&mut terminal)?,
                AppMode::FileSelection => self.handle_file_selection_dialog(&mut terminal)?,
                AppMode::DirectorySelection => {
                    self.handle_directory_selection_dialog(&mut terminal)?
                }
                AppMode::ReleaseNotes => self.handle_release_notes_dialog(&mut terminal)?,
            }
        }
    }

    fn check_llm_events(&mut self) {
        if let Some(ref mut rx) = self.llm_rx {
            // Process all available events
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }

            // Handle events outside the borrow scope
            for event in events {
                match event {
                    LlmEvent::Chunk(chunk) => {
                        self.update_partial_response(chunk);
                    }
                    LlmEvent::Done => {
                        let response = self.partial_response.clone();
                        let output_tokens = (response.chars().count() / 4).max(1);
                        let model = std::env::var("OPENCODE_MODEL")
                            .or_else(|_| std::env::var("OPENAI_MODEL"))
                            .unwrap_or_else(|_| "gpt-4o".to_string());

                        self.token_counter
                            .record_tokens(&model, self.pending_input_tokens, output_tokens);
                        let req_cost = self.cost_calculator.calculate_cost(
                            &model,
                            self.pending_input_tokens,
                            output_tokens,
                        );
                        self.total_cost_usd += req_cost;
                        let total_tokens = self.token_counter.get_total_tokens();
                        let context_total = self.status_bar.context_usage.1;
                        self.status_bar.update_usage(
                            total_tokens,
                            total_tokens,
                            context_total,
                            self.total_cost_usd,
                            self.budget_limit_usd,
                        );

                        if !response.is_empty() {
                            self.add_message(response, false);
                        } else {
                            self.add_message("[Empty response from LLM]".to_string(), false);
                        }
                        self.is_llm_generating = false;
                        self.partial_response.clear();
                        self.pending_input_tokens = 0;
                        self.llm_rx = None;
                    }
                    LlmEvent::Error(err) => {
                        self.add_message(format!("[Error: {}]", err), false);
                        self.is_llm_generating = false;
                        self.partial_response.clear();
                        self.pending_input_tokens = 0;
                        self.llm_rx = None;
                    }
                }
            }
        }
    }

    fn check_connect_events(&mut self) {
        if let Some(ref mut rx) = self.connect_rx {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }

            for event in events {
                match event {
                    ConnectEvent::BrowserOpened(url) => {
                        self.add_message(format!("Opened browser for OpenAI login: {}", url), false);
                    }
                    ConnectEvent::AuthComplete(session, models) => {
                        self.complete_browser_auth(session, models);
                    }
                    ConnectEvent::Failed(error) => {
                        self.add_message(format!("OpenAI connect failed: {}", error), false);
                        self.mode = AppMode::Chat;
                    }
                }
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

    fn handle_slash_command_dialog(
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
                        if let Some(cmd_name) = self.slash_command_dialog.get_selected_command() {
                            self.execute_slash_command(&cmd_name);
                        }
                        self.mode = AppMode::Chat;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Char(c) => {
                        self.command_palette_input.push(c);
                        self.slash_command_dialog
                            .update_input(&self.command_registry, &self.command_palette_input);
                    }
                    KeyCode::Backspace => {
                        self.command_palette_input.pop();
                        self.slash_command_dialog
                            .update_input(&self.command_registry, &self.command_palette_input);
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
                self.work_mode = WorkMode::Plan;
            }
            "/build" => {
                self.work_mode = WorkMode::Build;
            }
            "/clear" => {
                self.messages.clear();
                self.tool_calls.clear();
            }
            "/help" => {
                self.add_message(
                    "Commands: /plan, /build, /clear, /timeline, /fork, /meta, /skills, /help"
                        .to_string(),
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
            "/files" => {
                self.toggle_file_tree();
            }
            "/skills" => {
                self.show_skills_panel = !self.show_skills_panel;
                self.refresh_skills_panel_from_resolver();
            }
            "/release-notes" => {
                self.mode = AppMode::ReleaseNotes;
            }
            "/compact" => {
                self.add_message("Compacting session...".to_string(), false);
            }
            "/summarize" => {
                let msg_count = self.messages.len();
                self.add_message(
                    format!("Summarizing {} messages... (session summarized)", msg_count),
                    false,
                );
            }
            "/export" => {
                use std::env;
                use std::fs;
                let export_path = env::temp_dir().join("opencode_export.md");
                let content: String = self
                    .messages
                    .iter()
                    .map(|m| {
                        if m.is_user {
                            format!("User\n\n{}\n\n---\n", m.content)
                        } else {
                            format!("Assistant\n\n{}\n\n---\n", m.content)
                        }
                    })
                    .collect();
                match fs::write(&export_path, &content) {
                    Ok(_) => {
                        self.add_message(format!("Exported to: {}", export_path.display()), false)
                    }
                    Err(e) => self.add_message(format!("Export failed: {}", e), false),
                }
            }
            "/undo" => {
                self.add_message(
                    "Undo: Reverting last file changes (feature pending)".to_string(),
                    false,
                );
            }
            "/details" => {
                self.toggle_all_tool_details();
                let msg = if self.tool_calls.iter().any(|t| t.expanded) {
                    "Details: Shown"
                } else {
                    "Details: Hidden"
                };
                self.add_message(msg.to_string(), false);
            }
            "/themes" => {
                let themes = self.theme_manager.list_themes();
                let current = self.theme_manager.current().name.clone();
                let msg = format!(
                    "Available themes: {} (current: {})",
                    themes.join(", "),
                    current
                );
                self.add_message(msg, false);
            }
            "/theme" => {
                let current_name = self.theme_manager.current().name.clone();
                let themes = self.theme_manager.list_themes();
                let current_idx = themes.iter().position(|&t| t == current_name).unwrap_or(0);
                let next_idx = (current_idx + 1) % themes.len();
                let next_theme = themes[next_idx].to_string();
                if let Err(e) = self.theme_manager.set_theme_by_name(&next_theme) {
                    self.add_message(format!("Error: {}", e), false);
                } else {
                    let _ = self.theme_manager.save_to_config();
                    self.add_message(format!("Theme: {}", next_theme), false);
                }
            }
            "/exit" => {
                let _ = Self::cleanup_terminal();
                std::process::exit(0);
            }
            _ => {
                if !cmd.is_empty() {
                    self.add_message(format!("Unknown command: {}", cmd), false);
                }
            }
        }
    }

    fn execute_slash_command(&mut self, cmd_name: &str) {
        if let Some(command) = self.command_registry.get_by_name(cmd_name) {
            match &command.action {
                CommandAction::SetMode(mode) => {
                    match mode.as_str() {
                        "plan" => self.work_mode = WorkMode::Plan,
                        "build" => self.work_mode = WorkMode::Build,
                        _ => {}
                    }
                    self.add_message(format!("Mode: {}", mode.to_uppercase()), false);
                }
                CommandAction::Clear => {
                    self.messages.clear();
                    self.tool_calls.clear();
                }
                CommandAction::OpenTimeline => {
                    self.mode = AppMode::Timeline;
                }
                CommandAction::OpenFork => {
                    self.mode = AppMode::ForkDialog;
                    self.fork_name_input.clear();
                }
                CommandAction::ToggleMetadata => {
                    self.show_metadata = !self.show_metadata;
                }
                CommandAction::OpenSettings => {
                    self.mode = AppMode::Settings;
                }
                CommandAction::OpenModels => {
                    self.mode = AppMode::ModelSelection;
                }
                CommandAction::OpenProviders => {
                    self.mode = AppMode::ProviderManagement;
                }
                CommandAction::OpenConnect => {
                    self.begin_connect_flow();
                }
                CommandAction::ToggleFiles => {
                    self.toggle_file_tree();
                }
                CommandAction::OpenSkills => {
                    self.show_skills_panel = !self.show_skills_panel;
                    self.refresh_skills_panel_from_resolver();
                }
                CommandAction::OpenReleaseNotes => {
                    self.mode = AppMode::ReleaseNotes;
                }
                CommandAction::Compact => {
                    self.add_message("Compacting session...".to_string(), false);
                }
                CommandAction::Summarize => {
                    let msg_count = self.messages.len();
                    self.add_message(
                        format!("Summarizing {} messages... (session summarized)", msg_count),
                        false,
                    );
                }
                CommandAction::Export => {
                    use std::env;
                    use std::fs;
                    let export_path = env::temp_dir().join("opencode_export.md");
                    let content: String = self
                        .messages
                        .iter()
                        .map(|m| {
                            if m.is_user {
                                format!("User\n\n{}\n\n---\n", m.content)
                            } else {
                                format!("Assistant\n\n{}\n\n---\n", m.content)
                            }
                        })
                        .collect();
                    match fs::write(&export_path, &content) {
                        Ok(_) => self
                            .add_message(format!("Exported to: {}", export_path.display()), false),
                        Err(e) => self.add_message(format!("Export failed: {}", e), false),
                    }
                }
                CommandAction::Undo => {
                    self.add_message(
                        "Undo: Reverting last file changes (feature pending)".to_string(),
                        false,
                    );
                }
                CommandAction::ToggleDetails => {
                    self.toggle_all_tool_details();
                    let msg = if self.tool_calls.iter().any(|t| t.expanded) {
                        "Details: Shown"
                    } else {
                        "Details: Hidden"
                    };
                    self.add_message(msg.to_string(), false);
                }
                CommandAction::ListThemes => {
                    let themes = self.theme_manager.list_themes();
                    let current = self.theme_manager.current().name.clone();
                    let msg = format!(
                        "Available themes: {} (current: {})",
                        themes.join(", "),
                        current
                    );
                    self.add_message(msg, false);
                }
                CommandAction::SwitchTheme => {
                    let current_name = self.theme_manager.current().name.clone();
                    let themes = self.theme_manager.list_themes();
                    let current_idx = themes.iter().position(|&t| t == current_name).unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    let next_theme = themes[next_idx].to_string();
                    if let Err(e) = self.theme_manager.set_theme_by_name(&next_theme) {
                        self.add_message(format!("Error: {}", e), false);
                    } else {
                        self.add_message(format!("Theme: {}", next_theme), false);
                    }
                }
                CommandAction::Exit => {
                    let _ = Self::cleanup_terminal();
                    std::process::exit(0);
                }
                CommandAction::OpenSessions => {
                    self.mode = AppMode::Sessions;
                }
                CommandAction::NewSession => {
                    let session_count = self.session_manager.len();
                    self.session_manager
                        .add_session(format!("Session {}", session_count + 1));
                    self.add_message("New session created".to_string(), false);
                }
                CommandAction::Custom(name) => {
                    if name == "help" {
                        let all_commands = self
                            .command_registry
                            .all()
                            .iter()
                            .map(|c| format!("/{} - {}", c.name, c.description))
                            .collect::<Vec<_>>()
                            .join("\n");
                        self.add_message(format!("Available commands:\n{}", all_commands), false);
                    }
                }
            }
        } else {
            self.add_message(format!("Unknown command: {}", cmd_name), false);
        }
    }

    fn handle_input(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if self.title_bar.show_dropdown {
                    let action = self.title_bar.handle_input(key);
                    if let TitleBarAction::Select(session_id) = action {
                        self.add_message(format!("Switched to session: {}", session_id), false);
                    }
                    return Ok(());
                }

                if self.is_leader_key_active() {
                    return self.handle_leader_action(key);
                }

                if self.show_skills_panel {
                    if matches!(key.code, KeyCode::Esc)
                        || (matches!(key.code, KeyCode::Char('s'))
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        self.show_skills_panel = false;
                        self.sync_resolver_from_skills_panel();
                        return Ok(());
                    }

                    if self.skills_panel.handle_key(key) {
                        self.sync_resolver_from_skills_panel();
                        return Ok(());
                    }
                }

                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.is_llm_generating {
                            self.interrupt_llm_generation();
                        } else {
                            disable_raw_mode()?;
                            std::process::exit(0);
                        }
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.activate_leader_key();
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
                    KeyCode::Char('f')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        self.toggle_file_tree();
                    }
                    KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.title_bar.toggle_dropdown();
                    }
                    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.layout_manager.cycle_next();
                        let _ = self.layout_manager.save_to_file(&self.layout_file);
                        self.add_message(
                            format!("Layout preset: {:?}", self.layout_manager.get_layout()),
                            false,
                        );
                    }
                    KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Todo);
                    }
                    KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Diff);
                    }
                    KeyCode::Char('3') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Diagnostics);
                    }
                    KeyCode::Char('4') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Context);
                    }
                    KeyCode::Char('5') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Permissions);
                    }
                    KeyCode::Char('6') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.set_content(RightPanelContent::Files);
                    }
                    KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::ALT) => {
                        self.right_panel.toggle_collapse();
                    }
                    KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.status_bar
                            .toggle_popover(StatusPopoverType::Connection);
                    }
                    KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.status_bar.toggle_popover(StatusPopoverType::Tokens);
                    }
                    KeyCode::Char('3') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.status_bar.toggle_popover(StatusPopoverType::Context);
                    }
                    KeyCode::Char('`') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.show_terminal = !self.show_terminal;
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.show_skills_panel = !self.show_skills_panel;
                        self.refresh_skills_panel_from_resolver();
                    }
                    KeyCode::Enter => {
                        let input = self.input.clone();
                        if !input.is_empty() {
                            if input.trim() == "/confirm-shell" {
                                if let Some(cmd) = self.pending_shell_command.take() {
                                    match self.input_processor.process_shell_confirmed(&cmd) {
                                        Ok(output) => self.add_message(output, false),
                                        Err(error) => {
                                            self.add_message(format!("Shell execution failed: {error}"), false)
                                        }
                                    }
                                } else {
                                    self.add_message("No pending shell command".to_string(), false);
                                }
                                self.input.clear();
                                self.input_widget.clear();
                                self.input_box.set_input(String::new());
                                return Ok(());
                            }

                            self.pending_input_tokens = (input.chars().count() / 4).max(1);
                            self.history.push(input.clone());
                            self.history_index = self.history.len();

                            let parsed_input = self.input_parser.parse(&input);

                            if let Some(InputToken::SlashCommand { name, args }) =
                                parsed_input.tokens.first()
                            {
                                match self
                                    .input_processor
                                    .process_command(&self.command_registry, name, args)
                                {
                                    Ok(_) => self.execute_slash_command(name),
                                    Err(error) => self.add_message(error.to_string(), false),
                                }
                                self.input.clear();
                                self.input_widget.clear();
                                self.input_box.set_input(String::new());
                                return Ok(());
                            }

                            if self.work_mode == WorkMode::Plan {
                                self.add_message(
                                    "[Plan Mode] File modifications are disabled. Switch to Build mode to enable file changes.".to_string(),
                                    false,
                                );
                            }

                            if let Some(InputToken::ShellCommand(cmd)) = parsed_input
                                .tokens
                                .iter()
                                .find(|item| matches!(item, InputToken::ShellCommand(_)))
                            {
                                self.pending_shell_command = Some(cmd.clone());
                                match self.input_processor.process_shell(cmd) {
                                    Ok(preview) => self.add_message(preview, false),
                                    Err(error) => self.add_message(error.to_string(), false),
                                }
                            }

                            let parsed_files = parsed_input
                                .tokens
                                .iter()
                                .filter_map(|token| match token {
                                    InputToken::FileRef(path) => Some(path.clone()),
                                    _ => None,
                                })
                                .collect::<Vec<_>>();

                            if !parsed_files.is_empty() {
                                let mut context_content = input.clone();
                                context_content.push_str("\n\n--- Attached File Context ---\n");

                                match self.input_processor.process_files(&parsed_files) {
                                    Ok(content) => {
                                        context_content.push_str(&content);
                                    }
                                    Err(error) => {
                                        context_content.push_str(&format!(
                                            "[Error reading file context: {error}]"
                                        ));
                                    }
                                }
                                self.add_message(context_content, true);
                            } else {
                                self.add_message(input.clone(), true);
                            }

                             self.input.clear();
                             self.input_widget.clear();
                             self.input_box.set_input(String::new());

                             // Call LLM in background task if provider is initialized
                             if self.llm_provider.is_some() {
                                 self.set_tui_state(TuiState::Submitting);
                                 self.is_llm_generating = true;
                                 self.partial_response.clear();

                                 let (tx, rx) = mpsc::channel();
                                 self.llm_rx = Some(rx);
                                 let provider_clone = self.llm_provider.as_ref().unwrap().clone();
                                  let auto_enabled = self.skill_resolver.match_and_enable(&input);
                                  for skill in auto_enabled {
                                      self.skills_panel.set_enabled(&skill.name, true);
                                  }
                                  let skill_prompt = self.skill_resolver.build_skill_prompt();
                                  let llm_input = if skill_prompt.is_empty() {
                                      input.clone()
                                  } else {
                                      format!(
                                          "[Enabled Skills]\n{}\n\n[User Request]\n{}",
                                          skill_prompt,
                                          input
                                      )
                                  };
                                  
                                  std::thread::spawn(move || {
                                      let rt = tokio::runtime::Runtime::new().unwrap();
                                      rt.block_on(async {
                                         let tx_callback = tx.clone();
                                         let callback = move |chunk: String| {
                                             let _ = tx_callback.send(LlmEvent::Chunk(chunk));
                                         };
                                         match provider_clone.complete_streaming(&llm_input, Box::new(callback)).await {
                                             Ok(_) => {
                                                 let _ = tx.send(LlmEvent::Done);
                                             }
                                             Err(e) => {
                                                 let _ = tx.send(LlmEvent::Error(e.to_string()));
                                             }
                                         }
                                     });
                                 });
                             } else {
                                 // Show hint if no provider
                                 self.add_message(
                                     "[LLM provider not initialized. Call init_llm_provider() first or set OPENCODE_API_KEY]".to_string(),
                                     false,
                                 );
                             }
                         }
                     }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                        self.input_box.set_input(self.input.clone());
                        if c == '/' && self.input.len() == 1 {
                            self.mode = AppMode::SlashCommand;
                            self.command_palette_input.clear();
                            self.slash_command_dialog
                                .update_input(&self.command_registry, "");
                        }
                        if c == '@' {
                            self.input.pop();
                            self.mode = AppMode::FileSelection;
                            self.file_selection_dialog.clear_filter();
                        }
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                        self.input_box.set_input(self.input.clone());
                    }
                    KeyCode::Tab => {
                        if !self.input_box.autocomplete_into_input(&mut self.input) {
                            self.work_mode = self.work_mode.toggle();
                        }
                    }
                    KeyCode::Up => {
                        if self.input.is_empty()
                            && !self.history.is_empty() && self.history_index > 0 {
                                self.history_index -= 1;
                                self.input = self.history[self.history_index].clone();
                                self.input_box.set_input(self.input.clone());
                            }
                    }
                    KeyCode::Down => {
                        if self.input.is_empty()
                            && self.history_index < self.history.len() {
                                self.history_index += 1;
                                self.input = if self.history_index < self.history.len() {
                                    self.history[self.history_index].clone()
                                } else {
                                    String::new()
                                };
                                self.input_box.set_input(self.input.clone());
                            }
                    }
                    KeyCode::PageUp => {
                        self.scroll_state.scroll_up();
                        self.scroll_offset = self.scroll_state.apply(self.scroll_offset);
                    }
                    KeyCode::PageDown => {
                        self.scroll_state.scroll_down();
                        self.scroll_offset = self.scroll_state.apply(self.scroll_offset);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_leader_action(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.deactivate_leader_key();
            }
            KeyCode::Char('c') => {
                self.deactivate_leader_key();
                self.add_message("Compacting session...".to_string(), false);
            }
            KeyCode::Char('q') => {
                self.deactivate_leader_key();
                let _ = Self::cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Char('e') => {
                self.deactivate_leader_key();
                let result = self.open_external_editor();
                match result {
                    Ok(content) => {
                        if !content.is_empty() {
                            self.input_widget.clear();
                            self.input.clear();
                            let lines: Vec<&str> = content.lines().collect();
                            for line in lines {
                                self.input.push_str(line);
                                self.input.push('\n');
                            }
                            self.input_widget.elements.clear();
                            self.input_widget
                                .elements
                                .push(crate::components::InputElement::Text(self.input.clone()));
                            self.input_box.set_input(self.input.clone());
                        }
                        self.add_message("External editor: Content inserted".to_string(), false);
                    }
                    Err(e) => {
                        self.add_message(format!("External editor error: {}", e), false);
                    }
                }
            }
            KeyCode::Char('l') => {
                self.deactivate_leader_key();
                self.mode = AppMode::Sessions;
            }
            KeyCode::Char('d') => {
                self.deactivate_leader_key();
                self.toggle_all_tool_details();
                let msg = if self.tool_calls.iter().any(|t| t.expanded) {
                    "Details: Shown"
                } else {
                    "Details: Hidden"
                };
                self.add_message(msg.to_string(), false);
            }
            KeyCode::Char('m') => {
                self.deactivate_leader_key();
                self.mode = AppMode::ModelSelection;
            }
            KeyCode::Char('t') => {
                self.deactivate_leader_key();
                let current_name = self.theme_manager.current().name.clone();
                let themes = self.theme_manager.list_themes();
                let current_idx = themes.iter().position(|&t| t == current_name).unwrap_or(0);
                let next_idx = (current_idx + 1) % themes.len();
                let next_theme = themes[next_idx].to_string();
                if let Err(e) = self.theme_manager.set_theme_by_name(&next_theme) {
                    self.add_message(format!("Error: {}", e), false);
                } else {
                    self.add_message(format!("Theme: {}", next_theme), false);
                }
            }
            _ => {
                self.deactivate_leader_key();
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
            AppMode::SlashCommand => {
                self.draw_chat(f);
                self.slash_command_dialog.draw(f, f.area());
            }
            AppMode::DiffReview => {
                self.draw_chat(f);
                let patch = self
                    .tool_calls
                    .last()
                    .map(|t| t.output.clone())
                    .unwrap_or_else(|| "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new".to_string());
                let preview_area = Rect::new(
                    f.area().x + 2,
                    f.area().y + 2,
                    f.area().width.saturating_sub(4),
                    f.area().height.saturating_sub(8),
                );
                self.patch_preview.draw(f, &patch, preview_area);
                if let Some(ref dialog) = self.diff_review_dialog {
                    dialog.draw(f, f.area());
                }
            }
            AppMode::Sessions => {
                self.draw_chat(f);
                self.draw_sessions_dialog(f);
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
            AppMode::ConnectProvider => {
                self.draw_chat(f);
                self.connect_provider_dialog.draw(f, f.area());
            }
            AppMode::ConnectMethod => {
                self.draw_chat(f);
                if let Some(dialog) = self.connect_method_dialog.as_ref() {
                    dialog.draw(f, f.area());
                }
            }
            AppMode::ConnectProgress => {
                self.draw_chat(f);
                let area = Rect::new(
                    f.area().x + (f.area().width.saturating_sub(60)) / 2,
                    f.area().y + (f.area().height.saturating_sub(6)) / 2,
                    60.min(f.area().width.saturating_sub(2)),
                    6.min(f.area().height.saturating_sub(2)),
                );
                f.render_widget(Clear, area);
                f.render_widget(
                    Paragraph::new(vec![Line::from("Waiting for browser authentication...")])
                        .block(Block::default().title("Connect").borders(Borders::ALL)),
                    area,
                );
            }
            AppMode::ConnectModel => {
                self.draw_chat(f);
                if let Some(dialog) = self.connect_model_dialog.as_ref() {
                    dialog.draw(f, f.area());
                }
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

    fn toggle_file_tree(&mut self) {
        self.show_file_tree = !self.show_file_tree;
        if self.show_file_tree && self.file_tree.is_none() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            self.file_tree = Some(FileTree::new(cwd));
        }
    }

    fn right_panel_data(&self) -> RightPanelRenderData {
        let diagnostics = self
            .messages
            .iter()
            .filter(|m| {
                let c = m.content.to_ascii_lowercase();
                c.contains("error") || c.contains("warning") || c.contains("diagnostic")
            })
            .map(|m| m.content.clone())
            .take(8)
            .collect::<Vec<_>>();

        let files = self
            .dropped_files
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();

        let tools = self
            .command_registry
            .all()
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>();

        RightPanelRenderData {
            diagnostics,
            total_tokens: self.token_counter.get_total_tokens(),
            total_cost_usd: self.total_cost_usd,
            files,
            tools,
            todos: Vec::new(),
            diff_content: String::new(),
            context_items: Vec::new(),
            permission_log: Vec::new(),
            messages: Vec::new(),
            sessions: Vec::new(),
            config_data: Vec::new(),
            debug_info: Vec::new(),
        }
    }

    fn refresh_skills_panel_from_resolver(&mut self) {
        let skill_infos = self
            .skill_resolver
            .list_skills()
            .unwrap_or_default()
            .into_iter()
            .map(|(skill, state)| SkillInfo {
                name: skill.name,
                description: skill.description,
                enabled: state == SkillState::Enabled,
            })
            .collect();
        self.skills_panel.set_skills(skill_infos);
    }

    fn sync_resolver_from_skills_panel(&mut self) {
        for skill in &self.skills_panel.skills {
            let state = if skill.enabled {
                SkillState::Enabled
            } else {
                SkillState::Disabled
            };
            let _ = self.skill_resolver.set_skill_state(&skill.name, state);
        }
    }

    fn draw_sessions_dialog(&self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme_manager.current().clone();
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 15.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Sessions")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let mut session_lines: Vec<Line> = Vec::new();
        let sessions = self.session_manager.list();
        if sessions.is_empty() {
            session_lines.push(Line::from(Span::styled(
                "No sessions. Use /new to create a session.",
                Style::default().fg(theme.muted_color()),
            )));
        } else {
            for (i, session) in sessions.iter().enumerate() {
                let color = if i == self.session_manager.current_index() {
                    theme.primary_color()
                } else {
                    theme.foreground_color()
                };
                let time_ago = session.time_since_created().as_secs();
                let time_str = format!("{:.0}m ago", time_ago / 60);
                session_lines.push(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", i + 1),
                        Style::default().fg(theme.muted_color()),
                    ),
                    Span::styled(&session.name, Style::default().fg(color)),
                    Span::styled(
                        format!(" ({})", time_str),
                        Style::default().fg(theme.muted_color()),
                    ),
                    Span::styled(
                        format!(" {} messages", session.message_count),
                        Style::default().fg(theme.muted_color()),
                    ),
                ]));
            }
        }

        let content = Paragraph::new(session_lines);
        f.render_widget(content, inner_area);
    }

    fn handle_sessions_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = AppMode::Chat;
                    }
                    KeyCode::Up => {
                        let len = self.session_manager.len();
                        if len > 0 {
                            let current = self.session_manager.current_index();
                            if current > 0 {
                                self.session_manager.select(current - 1);
                            }
                        }
                    }
                    KeyCode::Down => {
                        let len = self.session_manager.len();
                        if len > 0 {
                            let current = self.session_manager.current_index();
                            if current < len - 1 {
                                self.session_manager.select(current + 1);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(session) = self.session_manager.current() {
                            self.add_message(
                                format!("Switched to session: {}", session.name),
                                false,
                            );
                        }
                        self.mode = AppMode::Chat;
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let session_count = self.session_manager.len();
                        self.session_manager
                            .add_session(format!("Session {}", session_count + 1));
                        self.add_message("New session created".to_string(), false);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn open_external_editor(&mut self) -> io::Result<String> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("opencode_prompt.txt");
        let current_content = self.input_widget.get_content();
        std::fs::write(&temp_file, &current_content)?;

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;

        let mut child = std::process::Command::new(&editor)
            .arg(&temp_file)
            .spawn()
            .map_err(|e| {
                io::Error::other(
                    format!("Failed to spawn editor: {}", e),
                )
            })?;

        let status = child.wait().map_err(|e| {
            io::Error::other(format!("Editor wait failed: {}", e))
        })?;

        let result = if status.success() {
            let content = std::fs::read_to_string(&temp_file)?;
            let _ = std::fs::remove_file(&temp_file);
            Ok(content)
        } else {
            let _ = std::fs::remove_file(&temp_file);
            Err(io::Error::other(
                "Editor exited with non-zero status".to_string(),
            ))
        };

        enable_raw_mode()?;
        result
    }

    fn draw_chat(&mut self, f: &mut Frame) {
        let area = f.area();

        let (_title_area, main_area) = if self.show_title_bar {
            let title_height = if self.title_bar.show_dropdown { 12 } else { 1 };
            let title_area = Rect::new(area.x, area.y, area.width, title_height);
            self.title_bar.draw(f, title_area);
            let remaining = Rect::new(
                area.x,
                area.y + title_height,
                area.width,
                area.height.saturating_sub(title_height),
            );
            (Some(title_area), remaining)
        } else {
            (None, area)
        };

        let proportions = self.layout_manager.get_proportions();

        let main_area = if self.show_file_tree {
            let file_tree_width = ((main_area.width as u32 * proportions.sidebar_width as u32) / 100)
                as u16;
            let file_tree_width = file_tree_width.max(20).min(40);
            let file_tree_area = Rect::new(
                main_area.x,
                main_area.y,
                file_tree_width,
                main_area.height.saturating_sub(1),
            );
            if let Some(ref mut file_tree) = self.file_tree {
                file_tree.draw(f, file_tree_area, "Files");
            }
            Rect::new(
                main_area.x + file_tree_width,
                main_area.y,
                main_area.width - file_tree_width,
                main_area.height,
            )
        } else {
            main_area
        };

        let show_right_panel = proportions.show_right_panel
            && !self.right_panel.collapsed
            && !self.right_panel.collapsed;

        let main_area = if show_right_panel {
            let right_panel_width = ((main_area.width as u32 * proportions.right_panel_width as u32)
                / 100) as u16;
            let right_panel_width = right_panel_width.max(24).min(40).min(main_area.width / 2);
            let right_panel_area = Rect::new(
                main_area.x + main_area.width - right_panel_width,
                main_area.y,
                right_panel_width,
                main_area.height.saturating_sub(1),
            );
            self.right_panel
                .draw(f, right_panel_area, &self.right_panel_data());
            Rect::new(
                main_area.x,
                main_area.y,
                main_area.width - right_panel_width,
                main_area.height,
            )
        } else {
            main_area
        };

        let content = self.input.clone();
        if self.input_box.input() != content {
            self.input_box.set_input(content);
        }

        let theme = self.theme();

        let terminal_height = if self.show_terminal {
            10.min(main_area.height / 3)
        } else {
            0
        };
        let remaining_height = main_area.height.saturating_sub(terminal_height);

        let (messages_height, tool_height) = if self.tool_calls.is_empty() {
            (remaining_height.saturating_sub(4), 0)
        } else {
            let tool_height = 5.min(remaining_height / 3);
            (
                remaining_height.saturating_sub(tool_height + 4),
                tool_height,
            )
        };

        let messages_area = Rect::new(main_area.x, main_area.y, main_area.width, messages_height);
        let input_area = Rect::new(main_area.x, messages_height, main_area.width, 3);
        let status_indicator_width = 30usize.min(main_area.width as usize);
        let status_area_width = (main_area.width as usize).saturating_sub(status_indicator_width);
        let status_area = Rect::new(
            main_area.x,
            remaining_height - 1,
            status_area_width as u16,
            1,
        );
        let status_indicator_area = Rect::new(
            main_area.x + status_area_width as u16,
            remaining_height - 1,
            status_indicator_width as u16,
            1,
        );

        if self.show_terminal {
            let terminal_area = Rect::new(
                main_area.x,
                remaining_height,
                main_area.width,
                terminal_height,
            );
            self.terminal_panel.draw(f, terminal_area);
        }

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
            let tool_area = Rect::new(main_area.x, messages_height, main_area.width, tool_height);
            let tool_block = Block::default()
                .title("Tool Calls")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()));

            let mut tool_lines: Vec<Line> = Vec::new();
            for tool in self.tool_calls.iter().take(tool_height as usize) {
                let status_color = match &tool.status {
                    ToolStatus::Running => theme.warning_color(),
                    ToolStatus::Success => theme.success_color(),
                    ToolStatus::Failed(_) => theme.error_color(),
                };
                let duration_str = tool
                    .duration_ms
                    .map(|ms| format!(" ({}ms)", ms))
                    .unwrap_or_default();

                tool_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} ", tool.status.icon()),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(&tool.name),
                    Span::styled(duration_str, Style::default().fg(theme.muted_color())),
                    if tool.expanded && !tool.output.is_empty() {
                        Span::styled(" ▼", Style::default().fg(theme.muted_color()))
                    } else if !tool.output.is_empty() {
                        Span::styled(" ▶", Style::default().fg(theme.muted_color()))
                    } else {
                        Span::raw("")
                    },
                ]));

                if tool.expanded && !tool.output.is_empty() {
                    fn strip_ansi(s: &str) -> String {
                        let mut result = String::new();
                        let mut in_escape = false;
                        let chars = s.chars().peekable();
                        for c in chars {
                            if c == '\x1b' {
                                in_escape = true;
                            } else if in_escape {
                                if c == 'm' {
                                    in_escape = false;
                                }
                            } else {
                                result.push(c);
                            }
                        }
                        result
                    }

                    let output_clean = strip_ansi(&tool.output);
                    for line in output_clean.lines().take(5) {
                        tool_lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(theme.muted_color()),
                        )));
                    }
                }
            }

            f.render_widget(Paragraph::new(tool_lines).block(tool_block), tool_area);
        }

        self.input_box.draw(f, input_area, "Input");

        let status = if self.is_leader_key_active() {
            Line::from(Span::styled(
                " LEADER | c:compact q:quit e:editor l:sessions d:details m:models t:themes | Esc:cancel",
                Style::default().fg(theme.warning_color()),
            ))
        } else {
            let mode_color = match self.work_mode {
                WorkMode::Plan => theme.muted_color(),
                WorkMode::Build => theme.accent_color(),
            };
            Line::from(vec![
                Span::styled(
                    format!(" {} ", self.work_mode.as_str()),
                    Style::default()
                        .fg(Color::Black)
                        .bg(mode_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        " {} | {} | ^X:Leader ^P:Cmds ^T:Timeline ^S:Skills ^C:Quit",
                        self.provider, self.agent
                    ),
                    Style::default().fg(theme.muted_color()),
                ),
            ])
        };
        f.render_widget(Paragraph::new(status), status_area);

        self.status_bar.draw(f, status_indicator_area);

        if self.show_skills_panel {
            let panel_width = (main_area.width.saturating_mul(3) / 4).max(40);
            let panel_height = (main_area.height.saturating_mul(3) / 4).max(10);
            let panel_x = main_area.x + (main_area.width.saturating_sub(panel_width)) / 2;
            let panel_y = main_area.y + (main_area.height.saturating_sub(panel_height)) / 2;
            let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);
            f.render_widget(Clear, panel_area);
            self.skills_panel.draw(f, panel_area);
        }
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
                "/files     Toggle file tree panel",
                Style::default().fg(theme.muted_color()),
            )),
            Line::from(Span::styled(
                "/skills    Toggle skills panel",
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
                if action == DialogAction::Close { self.mode = AppMode::Chat }
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

    fn handle_connect_provider_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let action = self.connect_provider_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Confirm(provider_id) => {
                        self.handle_connect_provider_confirm(provider_id)
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_connect_method_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(dialog) = self.connect_method_dialog.as_mut() {
                    let action = dialog.handle_input(key);
                    match action {
                        DialogAction::Close => self.mode = AppMode::ConnectProvider,
                        DialogAction::Confirm(method) => self.handle_connect_method_confirm(method),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_connect_progress_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                self.mode = AppMode::Chat;
            }
        }
        Ok(())
    }

    fn handle_connect_model_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(dialog) = self.connect_model_dialog.as_mut() {
                    let action = dialog.handle_input(key);
                    match action {
                        DialogAction::Close => self.handle_connect_model_cancel(),
                        DialogAction::Confirm(model_id) => {
                            if let Err(error) = self.handle_connect_model_confirm(model_id) {
                                self.add_message(format!("Failed to activate model: {}", error), false);
                                self.mode = AppMode::Chat;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_diff_review_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.patch_preview.handle_key(key);
                match self.patch_preview.decision() {
                    PatchDecision::Accepted => {
                        self.add_message("Patch accepted".to_string(), false);
                    }
                    PatchDecision::Rejected => {
                        self.add_message("Patch rejected".to_string(), false);
                    }
                    PatchDecision::Pending => {}
                }

                if let Some(ref mut dialog) = self.diff_review_dialog {
                    let action = dialog.handle_input(key);
                    match action {
                        DiffAction::Cancel => {
                            self.mode = AppMode::Chat;
                            self.diff_review_dialog = None;
                        }
                        DiffAction::Accept(path) => {
                            if self.work_mode == WorkMode::Build {
                                self.add_message(format!("Applied changes to: {}", path), false);
                            } else {
                                self.add_message(
                                    "[Plan Mode] Cannot apply changes. Switch to Build mode first."
                                        .to_string(),
                                    false,
                                );
                            }
                            self.mode = AppMode::Chat;
                            self.diff_review_dialog = None;
                        }
                        DiffAction::Reject => {
                            self.add_message("Changes rejected".to_string(), false);
                            self.mode = AppMode::Chat;
                            self.diff_review_dialog = None;
                        }
                        DiffAction::Edit(path) => {
                            if self.work_mode == WorkMode::Build {
                                self.add_message(format!("Opening editor for: {}", path), false);
                            } else {
                                self.add_message(
                                    "[Plan Mode] Cannot edit. Switch to Build mode first."
                                        .to_string(),
                                    false,
                                );
                            }
                            self.mode = AppMode::Chat;
                            self.diff_review_dialog = None;
                        }
                        DiffAction::None => {}
                    }
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
                        let path_buf = std::path::PathBuf::from(&path);
                        let file_name = path_buf
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file");
                        let theme = self.theme_manager.current().clone();
                        self.input_widget.insert_chip(
                            file_name.to_string(),
                            path.clone(),
                            theme.primary_color(),
                        );
                        if !self.input.is_empty() {
                            self.input.push(' ');
                        }
                        self.input.push('@');
                        self.input.push_str(&path);
                        self.input_box.set_input(self.input.clone());
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
                if action == DialogAction::Close { self.mode = AppMode::Chat }
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

fn open_external(url: &str) -> Result<(), String> {
    let status = if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
    } else {
        std::process::Command::new("xdg-open").arg(url).status()
    }
    .map_err(|e| format!("Failed to open browser: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Browser open command failed with status {}", status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_llm::BrowserAuthModelInfo;

    #[test]
    fn connect_command_opens_provider_dialog() {
        let mut app = App::new();
        app.execute_slash_command("connect");
        assert_eq!(app.mode, AppMode::ConnectProvider);
    }

    #[test]
    fn selecting_openai_opens_auth_method_dialog() {
        let mut app = App::new();
        app.begin_connect_flow();
        app.handle_connect_provider_confirm("openai".into());
        assert_eq!(app.mode, AppMode::ConnectMethod);
    }

    #[test]
    fn successful_browser_auth_opens_model_picker_instead_of_chat() {
        let mut app = App::new();
        app.complete_browser_auth_for_test(
            OpenAiBrowserSession {
                access_token: "access".into(),
                refresh_token: "refresh".into(),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 60_000,
                account_id: Some("acct_123".into()),
            },
            vec![BrowserAuthModelInfo {
                id: "gpt-5.3-codex".into(),
                name: "GPT-5.3 Codex".into(),
            }],
        );

        assert_eq!(app.mode, AppMode::ConnectModel);
    }

    #[test]
    fn selecting_model_rebinds_provider_and_returns_to_chat() {
        let dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OPENCODE_DATA_DIR", dir.path());
        }

        let mut app = App::new();
        app.prime_connect_state_for_test();
        app.handle_connect_model_confirm("gpt-5.3-codex".into()).unwrap();

        assert_eq!(app.mode, AppMode::Chat);
        assert_eq!(app.provider, "openai");
        assert!(app.llm_provider.is_some());

        unsafe {
            std::env::remove_var("OPENCODE_DATA_DIR");
        }
    }

    #[test]
    fn model_selection_cancel_keeps_chat_unswitched() {
        let mut app = App::new();
        let original_provider = app.provider.clone();
        app.prime_connect_state_for_test();
        app.handle_connect_model_cancel();
        assert_eq!(app.provider, original_provider);
        assert_eq!(app.mode, AppMode::Chat);
    }

    #[test]
    fn skills_command_toggles_skills_panel() {
        let mut app = App::new();
        assert!(!app.show_skills_panel);
        app.execute_slash_command("skills");
        assert!(app.show_skills_panel);
    }

    #[test]
    fn enabling_skill_in_panel_updates_prompt_builder() {
        let mut app = App::new();
        app.refresh_skills_panel_from_resolver();
        let updated = app.skills_panel.set_enabled("debugger", true);
        assert!(updated);

        app.sync_resolver_from_skills_panel();
        let prompt = app.skill_resolver.build_skill_prompt();
        assert!(prompt.contains("debugger"));
    }

    #[test]
    fn tui_state_has_11_states() {
        let states = [
            TuiState::Idle,
            TuiState::Composing,
            TuiState::Submitting,
            TuiState::Streaming,
            TuiState::ExecutingTool,
            TuiState::AwaitingPermission,
            TuiState::ShowingDiff,
            TuiState::ShowingError,
            TuiState::Aborting,
            TuiState::Reconnecting,
            TuiState::Paused,
        ];
        assert_eq!(states.len(), 11);
    }

    #[test]
    fn tui_state_can_accept_input() {
        assert!(TuiState::Idle.can_accept_input());
        assert!(TuiState::Composing.can_accept_input());
        assert!(!TuiState::Streaming.can_accept_input());
        assert!(!TuiState::ExecutingTool.can_accept_input());
    }

    #[test]
    fn tui_state_is_interruptible() {
        assert!(TuiState::Streaming.is_interruptible());
        assert!(TuiState::ExecutingTool.is_interruptible());
        assert!(!TuiState::Idle.is_interruptible());
        assert!(!TuiState::Composing.is_interruptible());
    }

    #[test]
    fn tui_state_is_active() {
        assert!(TuiState::Streaming.is_active());
        assert!(TuiState::ExecutingTool.is_active());
        assert!(TuiState::Submitting.is_active());
        assert!(TuiState::Aborting.is_active());
        assert!(!TuiState::Idle.is_active());
    }

    #[test]
    fn tui_state_can_cancel() {
        assert!(TuiState::Streaming.can_cancel());
        assert!(TuiState::ExecutingTool.can_cancel());
        assert!(TuiState::AwaitingPermission.can_cancel());
        assert!(!TuiState::Idle.can_cancel());
    }

    #[test]
    fn tui_state_is_terminal() {
        assert!(TuiState::Idle.is_terminal());
        assert!(TuiState::ShowingError.is_terminal());
        assert!(!TuiState::Streaming.is_terminal());
    }

    #[test]
    fn tui_state_allows_input_editing() {
        assert!(TuiState::Idle.allows_input_editing());
        assert!(TuiState::Composing.allows_input_editing());
        assert!(TuiState::ShowingError.allows_input_editing());
        assert!(!TuiState::Streaming.allows_input_editing());
    }

    #[test]
    fn tui_state_valid_transitions_idle() {
        let idle = TuiState::Idle;
        let valid = idle.valid_transitions();
        assert!(valid.contains(&TuiState::Composing));
        assert!(valid.contains(&TuiState::Paused));
        assert!(valid.contains(&TuiState::ShowingError));
    }

    #[test]
    fn tui_state_valid_transitions_streaming() {
        let streaming = TuiState::Streaming;
        let valid = streaming.valid_transitions();
        assert!(valid.contains(&TuiState::ExecutingTool));
        assert!(valid.contains(&TuiState::Aborting));
        assert!(valid.contains(&TuiState::Idle));
    }

    #[test]
    fn tui_state_can_transition_to() {
        assert!(TuiState::Idle.can_transition_to(TuiState::Composing));
        assert!(TuiState::Idle.can_transition_to(TuiState::Paused));
        assert!(!TuiState::Idle.can_transition_to(TuiState::Streaming));
        
        assert!(TuiState::Streaming.can_transition_to(TuiState::ExecutingTool));
        assert!(TuiState::Streaming.can_transition_to(TuiState::Aborting));
    }

    #[test]
    fn tui_state_label_returns_correct_string() {
        assert_eq!(TuiState::Idle.label(), "idle");
        assert_eq!(TuiState::Composing.label(), "composing");
        assert_eq!(TuiState::Streaming.label(), "streaming");
        assert_eq!(TuiState::Paused.label(), "paused");
    }
}
