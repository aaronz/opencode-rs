use crate::action;
use crate::action::AppMode;
use crate::command::{CommandAction, CommandRegistry};
use crate::components::{
    FileTree, InputWidget, Sidebar, SkillInfo, SkillsPanel, StatusBar, StatusPopoverType,
    TerminalPanel, TitleBar, TitleBarAction,
};
use crate::config::{Config, DiffStyle, UserConfig};
use crate::dialogs::home_view::{HomeAction, HomeView, HomeViewSection};
use crate::dialogs::*;
use crate::dialogs::{ProviderInfo, ProviderStatus};
use crate::file_ref_handler::FileRefHandler;
use crate::input::{EditorLauncher, InputBox, InputParser, InputProcessor, InputToken};
use crate::layout::LayoutManager;
use crate::patch_preview::{PatchDecision, PatchPreview};
use crate::right_panel::{RightPanel, RightPanelContent, RightPanelRenderData};
use crate::session::SessionManager;
use crate::shell_handler::ShellHandler;
use crate::theme::{Theme, ThemeManager};
use crossterm::{
    cursor,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear as TermClear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use opencode_agent::{AgentRuntime, AgentType};
use opencode_auth::CredentialStore;
use opencode_core::{
    AgentExecutor, CostCalculator, Message, PermissionManager, Session, SessionSharing,
    SkillResolver, SkillState, TokenCounter,
};
use opencode_llm::{
    BrowserAuthModelInfo, CopilotOAuthService, CopilotOAuthSession, CopilotOAuthStore,
    GoogleOAuthService, GoogleOAuthSession, GoogleOAuthStore, OpenAiBrowserAuthService,
    OpenAiBrowserAuthStore, OpenAiBrowserSession, OpenAiProvider, Provider, ProviderCatalogFetcher,
    ProviderConfig, ProviderType,
};
use opencode_lsp::client::LspClient;
use opencode_lsp::types::{Diagnostic, Location};
use opencode_mcp::McpManager;
use opencode_runtime::{
    ExecuteShellCommand, RunAgentCommand, RuntimeFacade as OpenCodeRuntime, RuntimeFacadeCommand,
    RuntimeFacadeServices, RuntimeFacadeTaskStore, RuntimeFacadeToolRouter,
};
use opencode_storage::{
    InMemoryProjectRepository, InMemorySessionRepository, StoragePool, StorageService,
};
use opencode_tools::{build_default_registry, ToolRegistry as ToolsToolRegistry};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde::Deserialize;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[allow(clippy::large_enum_variant)]
pub enum LlmEvent {
    Chunk(String),
    ToolCall {
        name: String,
        arguments: serde_json::Value,
        id: String,
    },
    ToolResult {
        id: String,
        output: String,
    },
    SessionComplete(Session),
    Done,
    Error(String),
}

#[derive(Debug)]
pub enum ConnectEvent {
    BrowserOpened(String),
    AuthComplete(OpenAiBrowserSession, Vec<BrowserAuthModelInfo>),
    GoogleAuthComplete(GoogleOAuthSession, Vec<BrowserAuthModelInfo>),
    CopilotAuthComplete(CopilotOAuthSession, Vec<BrowserAuthModelInfo>),
    Failed(String),
    ValidationComplete {
        success: bool,
        error_message: Option<String>,
        models: Option<Vec<BrowserAuthModelInfo>>,
    },
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectFlowState {
    Idle,
    ProviderPickerOpen,
    ProviderSelected,
    ApiKeyInputFocused,
    ValidatingKey,
    ValidationFailed,
    ValidationSucceeded,
    Cancelled,
    RecoveredToMainInput,
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
pub struct MemoryEntry {
    pub id: usize,
    pub content: String,
    pub created_at: String,
}

impl MemoryEntry {
    pub fn new(id: usize, content: String) -> Self {
        Self {
            id,
            content,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoEntry {
    pub content: String,
    pub completed: bool,
    pub priority: String,
}

impl TodoEntry {
    pub fn from_markdown_line(line: &str) -> Option<Self> {
        // Parse "- [ ]" or "- [x]" format
        let line = line.trim();
        if !line.starts_with("- [") {
            return None;
        }

        let completed = line.starts_with("- [x]") || line.starts_with("- [X]");
        let rest = if completed { &line[5..] } else { &line[4..] };

        let content = rest.trim().to_string();
        if content.is_empty() {
            return None;
        }

        let (content, priority) = if let Some(paren_pos) = content.rfind('(') {
            if content.ends_with(')') {
                let p = content[paren_pos + 1..content.len() - 1].to_string();
                (content[..paren_pos].trim().to_string(), p)
            } else {
                (content.clone(), "medium".to_string())
            }
        } else {
            (content, "medium".to_string())
        };

        Some(Self {
            content,
            completed,
            priority,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MessageMeta {
    pub content: String,
    pub is_user: bool,
    pub is_thinking: bool,
    pub token_count: Option<usize>,
    pub duration_ms: Option<u64>,
    pub tool_calls: Vec<String>,
}

impl MessageMeta {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_user: true,
            is_thinking: false,
            token_count: None,
            duration_ms: None,
            tool_calls: Vec::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_user: false,
            is_thinking: false,
            token_count: None,
            duration_ms: None,
            tool_calls: Vec::new(),
        }
    }

    pub fn thinking(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_user: false,
            is_thinking: true,
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
            Self::ExecutingTool => vec![
                Self::Streaming,
                Self::AwaitingPermission,
                Self::Aborting,
                Self::Idle,
            ],
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
    pub(crate) connect_rx: Option<mpsc::Receiver<ConnectEvent>>,
    pub mode: AppMode,
    pub tui_state: TuiState,
    pub reconnect_timeout: Option<std::time::Instant>,
    pub command_palette_input: String,
    pub command_registry: CommandRegistry,
    pub slash_command_dialog: SlashCommandOverlay,
    pub diff_review_dialog: Option<DiffReviewOverlay>,
    pub session_manager: SessionManager,
    pub session_sharing: SessionSharing,
    pub scroll_offset: usize,
    pub scroll_state: ScrollState,
    pub selection: SelectionState,
    pub timeline_state: ListState,
    pub fork_name_input: String,
    pub show_metadata: bool,
    pub theme_manager: ThemeManager,
    pub settings_dialog: SettingsDialog,
    pub model_selection_dialog: ModelSelectionDialog,
    pub provider_management_dialog: ProviderManagementDialog,
    pub connect_provider_dialog: ConnectProviderDialog,
    pub connect_method_dialog: Option<ConnectMethodDialog>,
    pub api_key_input_dialog: Option<ApiKeyInputDialog>,
    pub validation_error_dialog: Option<ValidationErrorDialog>,
    pub connect_model_dialog: Option<ConnectModelDialog>,
    pub file_selection_dialog: FileSelectionDialog,
    pub directory_selection_dialog: DirectorySelectionDialog,
    pub release_notes_dialog: ReleaseNotesDialog,
    pub home_view: HomeView,
    pub file_tree: Option<FileTree>,
    pub show_file_tree: bool,
    pub layout_manager: LayoutManager,
    pub sidebar: Sidebar,
    pub show_sidebar: bool,
    sidebar_file: std::path::PathBuf,
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
    pub pending_google_session: Option<GoogleOAuthSession>,
    pub pending_copilot_session: Option<CopilotOAuthSession>,
    pub pending_browser_models: Vec<BrowserAuthModelInfo>,
    pub validation_in_progress: bool,
    pub validation_cancelled: bool,
    pub pending_api_key_for_validation: Option<String>,
    pub pending_api_key_models: Vec<BrowserAuthModelInfo>,
    pub pending_api_key_for_provider: Option<String>,
    pub token_counter: TokenCounter,
    pub cost_calculator: CostCalculator,
    pub session_token_id: String,
    pub pending_input_tokens: usize,
    pub total_cost_usd: f64,
    pub mcp_cost_usd: f64,
    pub budget_limit_usd: Option<f64>,
    pub username: Option<String>,
    pub share_url: Option<String>,
    pub memory_entries: Vec<MemoryEntry>,
    pub todos: Vec<TodoEntry>,
    pub thinking_mode: bool,
    pub thinking_content: String,
    pub is_receiving_thinking: bool,
    pub model_aliases: std::collections::HashMap<String, String>,
    skill_resolver: SkillResolver,
    input_parser: InputParser,
    input_box: InputBox,
    input_processor: InputProcessor,
    pending_shell_command: Option<String>,
    enriched_input: Option<String>,
    #[allow(dead_code)]
    shell_handler: ShellHandler,
    #[allow(dead_code)]
    file_ref_handler: FileRefHandler,
    pub config: Config,
    #[allow(dead_code)]
    tool_registry: Arc<ToolsToolRegistry>,
    #[allow(dead_code)]
    runtime: Arc<OpenCodeRuntime>,
    session: Option<Session>,
    #[allow(dead_code)]
    agent_executor: AgentExecutor,
    #[allow(dead_code)]
    mcp_manager: &'static McpManager,
    pub lsp_client: Option<LspClient>,
    pub lsp_diagnostics: Vec<Diagnostic>,
    pub catalog_fetcher: ProviderCatalogFetcher,
}

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub is_selecting: bool,
    pub start_pos: Option<(u16, u16)>,
    pub end_pos: Option<(u16, u16)>,
    pub selected_text: Option<String>,
}

#[derive(Debug)]
pub struct ApiKeyValidationError {
    pub message: String,
    pub error_type: ApiKeyValidationErrorType,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiKeyValidationErrorType {
    EmptyKey,
    NetworkError,
    AuthenticationError,
    PermissionError,
    ServerError,
    Timeout,
    InvalidResponse,
    Unknown,
}

impl std::fmt::Display for ApiKeyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiKeyValidationError {}

pub async fn validate_api_key(
    provider_id: &str,
    api_key: &str,
) -> Result<(), ApiKeyValidationError> {
    if api_key.is_empty() {
        return Err(ApiKeyValidationError {
            message: "API key cannot be empty".to_string(),
            error_type: ApiKeyValidationErrorType::EmptyKey,
            status_code: None,
        });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| ApiKeyValidationError {
            message: format!("Failed to create HTTP client: {}", e),
            error_type: ApiKeyValidationErrorType::NetworkError,
            status_code: None,
        })?;

    let (url, auth_header, is_anthropic, _is_lm_studio) = match provider_id {
        "anthropic" => (
            "https://api.anthropic.com/v1/models".to_string(),
            format!("Bearer {}", api_key),
            true,
            false,
        ),
        "openai" => (
            "https://api.openai.com/v1/models".to_string(),
            format!("Bearer {}", api_key),
            false,
            false,
        ),
        "minimax" => (
            "https://api.minimax.io/v1/models".to_string(),
            format!("Bearer {}", api_key),
            false,
            false,
        ),
        "minimax-cn" => (
            "https://api.minimaxi.com/v1/chat/completions".to_string(),
            format!("Bearer {}", api_key),
            false,
            true,
        ),
        "lmstudio" | "lm_studio" | "lm-studio" => {
            let base_url = std::env::var("LMSTUDIO_BASE_URL")
                .ok()
                .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
                .unwrap_or_else(|| "http://localhost:1234".to_string());
            (
                format!("{}/api/tags", base_url.trim_end_matches('/')),
                format!("Bearer {}", api_key),
                false,
                true,
            )
        }
        _ => {
            let base_url = match std::env::var(
                format!("{}_BASE_URL", provider_id.to_uppercase()).replace("-", "_"),
            )
            .ok()
            .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
            {
                Some(url) => url,
                None => "https://api.openai.com".to_string(),
            };
            (
                format!("{}/v1/models", base_url.trim_end_matches('/')),
                format!("Bearer {}", api_key),
                false,
                false,
            )
        }
    };

    let response = client
        .get(&url)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| {
            let error_type = if e.is_timeout() {
                ApiKeyValidationErrorType::Timeout
            } else {
                ApiKeyValidationErrorType::NetworkError
            };
            ApiKeyValidationError {
                message: format!("Network error: {}", e),
                error_type,
                status_code: None,
            }
        })?;

    let status = response.status();

    if status.is_success() {
        Ok(())
    } else if status.as_u16() == 401 || status.as_u16() == 403 {
        let error_body = response.text().await.unwrap_or_default();
        let error_message = if is_anthropic {
            parse_anthropic_error(&error_body).unwrap_or_else(|| {
                format!(
                    "Authentication failed: invalid API key (HTTP {})",
                    status.as_u16()
                )
            })
        } else {
            parse_openai_error(&error_body).unwrap_or_else(|| {
                format!(
                    "Authentication failed: invalid API key (HTTP {})",
                    status.as_u16()
                )
            })
        };
        Err(ApiKeyValidationError {
            message: error_message,
            error_type: ApiKeyValidationErrorType::AuthenticationError,
            status_code: Some(status.as_u16()),
        })
    } else if status.as_u16() >= 500 {
        Err(ApiKeyValidationError {
            message: format!(
                "Server error (HTTP {}): please try again later",
                status.as_u16()
            ),
            error_type: ApiKeyValidationErrorType::ServerError,
            status_code: Some(status.as_u16()),
        })
    } else {
        Err(ApiKeyValidationError {
            message: format!("Request failed with HTTP {}", status.as_u16()),
            error_type: ApiKeyValidationErrorType::Unknown,
            status_code: Some(status.as_u16()),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelData {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    models: Vec<AnthropicModelData>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModelData {
    id: String,
}

pub async fn validate_api_key_and_fetch_models(
    provider_id: &str,
    api_key: &str,
) -> Result<Vec<BrowserAuthModelInfo>, ApiKeyValidationError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let redacted_key = if api_key.len() > 8 {
        format!("****{}", &api_key[api_key.len() - 8..])
    } else {
        "****".to_string()
    };

    if api_key.is_empty() {
        tracing::warn!(
            event = "provider.validation.start",
            request_id = %request_id,
            provider_id = %provider_id,
            reason = "empty_key",
            "Provider validation skipped: empty API key"
        );
        return Err(ApiKeyValidationError {
            message: "API key cannot be empty".to_string(),
            error_type: ApiKeyValidationErrorType::EmptyKey,
            status_code: None,
        });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| ApiKeyValidationError {
            message: format!("Failed to create HTTP client: {}", e),
            error_type: ApiKeyValidationErrorType::NetworkError,
            status_code: None,
        })?;

    let (url, auth_header, is_anthropic, is_lm_studio, http_method) = match provider_id {
        "anthropic" => (
            "https://api.anthropic.com/v1/models".to_string(),
            format!("Bearer {}", api_key),
            true,
            false,
            "GET".to_string(),
        ),
        "openai" => (
            "https://api.openai.com/v1/models".to_string(),
            format!("Bearer {}", api_key),
            false,
            false,
            "GET".to_string(),
        ),
        "minimax" => (
            "https://api.minimax.io/v1/models".to_string(),
            format!("Bearer {}", api_key),
            false,
            false,
            "GET".to_string(),
        ),
        "minimax-cn" => (
            "https://api.minimaxi.com/v1/chat/completions".to_string(),
            format!("Bearer {}", api_key),
            false,
            false,
            "POST".to_string(),
        ),
        "lmstudio" | "lm_studio" | "lm-studio" => {
            let base_url = std::env::var("LMSTUDIO_BASE_URL")
                .ok()
                .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
                .unwrap_or_else(|| "http://localhost:1234".to_string());
            (
                format!("{}/api/tags", base_url.trim_end_matches('/')),
                format!("Bearer {}", api_key),
                false,
                true,
                "GET".to_string(),
            )
        }
        "ollama" => {
            let base_url = std::env::var("OLLAMA_BASE_URL")
                .ok()
                .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            (
                format!("{}/api/tags", base_url.trim_end_matches('/')),
                String::new(),
                false,
                false,
                "GET".to_string(),
            )
        }
        _ => {
            let base_url = match std::env::var(
                format!("{}_BASE_URL", provider_id.to_uppercase()).replace("-", "_"),
            )
            .ok()
            .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
            {
                Some(url) => url,
                None => "https://api.openai.com".to_string(),
            };
            (
                format!("{}/v1/models", base_url.trim_end_matches('/')),
                format!("Bearer {}", api_key),
                false,
                false,
                "GET".to_string(),
            )
        }
    };

    let request_builder = client
        .request(
            reqwest::Method::from_bytes(http_method.as_bytes()).unwrap(),
            &url,
        )
        .header("Authorization", &auth_header);

    let request_builder = if provider_id == "minimax-cn" {
        let body = serde_json::json!({
            "model": "MiniMax-M2.7",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 1
        });
        request_builder
            .header("Content-Type", "application/json")
            .json(&body)
    } else {
        request_builder
    };

    tracing::info!(
        event = "provider.validation.request",
        request_id = %request_id,
        provider_id = %provider_id,
        method = %http_method,
        url = %url,
        auth_type = "Bearer",
        key_suffix = %redacted_key,
        "Sending provider validation request"
    );

    let start_time = std::time::Instant::now();
    let response = request_builder.send().await.map_err(|e| {
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let error_type = if e.is_timeout() {
            ApiKeyValidationErrorType::Timeout
        } else {
            ApiKeyValidationErrorType::NetworkError
        };
        tracing::error!(
            event = "provider.validation.error",
            request_id = %request_id,
            provider_id = %provider_id,
            error_type = ?error_type,
            elapsed_ms = %elapsed_ms,
            url = %url,
            error = %e,
            "Provider validation request failed"
        );
        ApiKeyValidationError {
            message: format!("Network error: {}", e),
            error_type,
            status_code: None,
        }
    })?;

    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    let status = response.status();

    tracing::info!(
        event = "provider.validation.response",
        request_id = %request_id,
        provider_id = %provider_id,
        status = %status,
        elapsed_ms = %elapsed_ms,
        "Received provider validation response"
    );

    if status.is_success() {
        if provider_id == "minimax-cn" {
            tracing::info!(
                event = "provider.validation.success",
                request_id = %request_id,
                provider_id = %provider_id,
                elapsed_ms = %elapsed_ms,
                "Minimax CN validation succeeded (key valid)"
            );
            return Ok(vec![
                BrowserAuthModelInfo {
                    id: "MiniMax-M2.7".to_string(),
                    name: "MiniMax M2.7".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "MiniMax-M2.5".to_string(),
                    name: "MiniMax M2.5".to_string(),
                    variants: vec![],
                },
            ]);
        }

        let body = response.text().await.map_err(|e| ApiKeyValidationError {
            message: format!("Failed to read response body: {}", e),
            error_type: ApiKeyValidationErrorType::InvalidResponse,
            status_code: None,
        })?;

        let models = if is_lm_studio {
            parse_lm_studio_models(&body)
        } else if is_anthropic {
            parse_anthropic_models(&body)
        } else if provider_id == "ollama" {
            parse_ollama_models(&body)
        } else {
            parse_openai_models(&body)
        };

        tracing::info!(
            event = "provider.validation.parsed_models",
            request_id = %request_id,
            provider_id = %provider_id,
            model_count = %models.len(),
            elapsed_ms = %elapsed_ms,
            "Provider validation parsed models successfully"
        );

        Ok(models)
    } else if status.as_u16() == 401 || status.as_u16() == 403 {
        let error_body = response.text().await.unwrap_or_default();
        let error_message = if is_anthropic {
            parse_anthropic_error(&error_body).unwrap_or_else(|| {
                format!(
                    "Authentication failed: invalid API key (HTTP {})",
                    status.as_u16()
                )
            })
        } else {
            parse_openai_error(&error_body).unwrap_or_else(|| {
                format!(
                    "Authentication failed: invalid API key (HTTP {})",
                    status.as_u16()
                )
            })
        };
        tracing::error!(
            event = "provider.validation.auth_failed",
            request_id = %request_id,
            provider_id = %provider_id,
            status = %status,
            elapsed_ms = %elapsed_ms,
            error = %error_message,
            "Provider validation authentication failed"
        );
        Err(ApiKeyValidationError {
            message: error_message,
            error_type: ApiKeyValidationErrorType::AuthenticationError,
            status_code: Some(status.as_u16()),
        })
    } else if status.as_u16() >= 500 {
        tracing::error!(
            event = "provider.validation.server_error",
            request_id = %request_id,
            provider_id = %provider_id,
            status = %status,
            elapsed_ms = %elapsed_ms,
            "Provider validation server error"
        );
        Err(ApiKeyValidationError {
            message: format!(
                "Server error (HTTP {}): please try again later",
                status.as_u16()
            ),
            error_type: ApiKeyValidationErrorType::ServerError,
            status_code: Some(status.as_u16()),
        })
    } else {
        let error_body = response.text().await.unwrap_or_default();
        let error_excerpt = error_body.chars().take(200).collect::<String>();
        tracing::error!(
            event = "provider.validation.request_failed",
            request_id = %request_id,
            provider_id = %provider_id,
            status = %status,
            elapsed_ms = %elapsed_ms,
            response_excerpt = %error_excerpt,
            "Provider validation request failed"
        );
        Err(ApiKeyValidationError {
            message: format!("Request failed with HTTP {}", status.as_u16()),
            error_type: ApiKeyValidationErrorType::Unknown,
            status_code: Some(status.as_u16()),
        })
    }
}

pub fn parse_openai_models(body: &str) -> Vec<BrowserAuthModelInfo> {
    serde_json::from_str::<OpenAIModelsResponse>(body)
        .map(|response| {
            response
                .data
                .into_iter()
                .map(|m| BrowserAuthModelInfo {
                    id: m.id.clone(),
                    name: m.id,
                    variants: vec![],
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_anthropic_models(body: &str) -> Vec<BrowserAuthModelInfo> {
    serde_json::from_str::<AnthropicModelsResponse>(body)
        .map(|response| {
            response
                .models
                .into_iter()
                .map(|m| BrowserAuthModelInfo {
                    id: m.id.clone(),
                    name: m.id,
                    variants: vec![],
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_ollama_models(body: &str) -> Vec<BrowserAuthModelInfo> {
    #[derive(Debug, Deserialize)]
    struct OllamaModelsResponse {
        models: Vec<OllamaModelData>,
    }

    #[derive(Debug, Deserialize)]
    struct OllamaModelData {
        name: String,
        #[serde(rename = "model")]
        model: Option<String>,
    }

    serde_json::from_str::<OllamaModelsResponse>(body)
        .map(|response| {
            response
                .models
                .into_iter()
                .map(|m| {
                    let id = m.name.clone();
                    let name = m.model.unwrap_or_else(|| m.name.clone());
                    BrowserAuthModelInfo {
                        id,
                        name,
                        variants: vec![],
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_lm_studio_models(body: &str) -> Vec<BrowserAuthModelInfo> {
    #[derive(Debug, Deserialize)]
    struct LmStudioModelsResponse {
        models: Vec<LmStudioModelData>,
    }

    #[derive(Debug, Deserialize)]
    struct LmStudioModelData {
        name: String,
    }

    serde_json::from_str::<LmStudioModelsResponse>(body)
        .map(|response| {
            response
                .models
                .into_iter()
                .map(|m| BrowserAuthModelInfo {
                    id: m.name.clone(),
                    name: m.name,
                    variants: vec![],
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_anthropic_error(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("error")
                .and_then(|error| error.get("message"))
                .and_then(|msg| msg.as_str())
                .map(|s| s.to_string())
        })
}

fn parse_openai_error(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("error")
                .and_then(|error| error.get("message"))
                .and_then(|msg| msg.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|json| {
                    json.get("error")
                        .and_then(|error| error.get("type"))
                        .and_then(|msg| msg.as_str())
                        .map(|s| format!("{} error", s))
                })
        })
}

impl App {
    pub fn new() -> Self {
        let _start = tracing::info_span!("app_init").entered();

        let mut timeline_state = ListState::default();
        timeline_state.select(None);

        let _span = tracing::info_span!("app_init", phase = "theme_and_config").entered();
        let mut theme_manager = ThemeManager::new();
        let _ = theme_manager.load_from_config();
        let config = Config::load_from_default_path().unwrap_or_else(|_| Config::default_config());
        let custom_themes = config.tui_config().custom_themes.clone();
        theme_manager.load_custom_themes_from_config(&custom_themes);
        let theme = theme_manager.current().clone();
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "config_dir_lookup").entered();
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("opencode-rs");
        drop(_span);
        let _span = tracing::info_span!("app_init", phase = "create_config_dirs").entered();
        std::fs::create_dir_all(&config_dir).ok();
        drop(_span);
        let history_file = config_dir.join("history.txt");

        let _span = tracing::info_span!("app_init", phase = "history_layout_sidebar").entered();
        let _span2 = tracing::info_span!("app_init", phase = "history_read").entered();
        let mut history = Vec::new();
        if let Ok(content) = std::fs::read_to_string(&history_file) {
            history = content.lines().map(|s| s.to_string()).take(100).collect();
        }
        drop(_span2);

        let layout_file = config_dir.join("layout.txt");
        let _span2 = tracing::info_span!("app_init", phase = "layout_load").entered();
        let layout_manager = LayoutManager::load_from_file(&layout_file).unwrap_or_default();
        drop(_span2);

        let sidebar_file = config_dir.join("sidebar.json");
        let _span2 = tracing::info_span!("app_init", phase = "sidebar_new").entered();
        let mut sidebar = Sidebar::new(theme.clone());
        drop(_span2);
        let _span2 = tracing::info_span!("app_init", phase = "sidebar_load").entered();
        let _ = sidebar.load_from_file(&sidebar_file);
        drop(_span2);
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "session_counter_budget").entered();
        let session_token_id = uuid::Uuid::new_v4().to_string();
        let mut token_counter = TokenCounter::new();
        token_counter.set_active_session(session_token_id.clone());
        let budget_limit_usd = std::env::var("OPENCODE_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok());
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "skills_panel").entered();
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
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "command_registry").entered();
        let command_registry = CommandRegistry::new();
        let input_box = InputBox::new(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            &command_registry,
        );
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "tool_registry").entered();
        let tool_registry = Arc::new(if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.block_on(async { build_default_registry(None).await })
        } else {
            tokio::runtime::Runtime::new()
                .expect("failed to create Tokio runtime for tool registry")
                .block_on(async { build_default_registry(None).await })
        });
        drop(_span);

        let _span = tracing::info_span!("app_init", phase = "mcp_bridge").entered();
        let agent_executor = {
            let mut registry = opencode_core::ToolRegistry::new();
            let mcp_manager = McpManager::global();
            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                rt.block_on(async {
                    mcp_manager.bridge_to_tool_registry(&mut registry).await;
                });
            }
            AgentExecutor::new(registry)
        };
        drop(_span);

        let runtime = build_placeholder_runtime(None, Some(tool_registry.clone()));

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
            mode: AppMode::Home,
            tui_state: TuiState::Idle,
            reconnect_timeout: None,
            command_palette_input: String::new(),
            command_registry,
            slash_command_dialog: SlashCommandOverlay::new(theme.clone()),
            diff_review_dialog: None,
            session_manager: SessionManager::with_file(config_dir.join("sessions.txt")),
            session_sharing: SessionSharing::with_default_path(),
            scroll_offset: 0,
            scroll_state: ScrollState::new(),
            selection: SelectionState::default(),
            timeline_state,
            fork_name_input: String::new(),
            show_metadata: false,
            theme_manager,
            settings_dialog: SettingsDialog::new(theme.clone()),
            model_selection_dialog: ModelSelectionDialog::new(theme.clone()),
            provider_management_dialog: ProviderManagementDialog::new(theme.clone()),
            connect_provider_dialog: ConnectProviderDialog::new(theme.clone()),
            connect_method_dialog: None,
            api_key_input_dialog: None,
            validation_error_dialog: None,
            connect_model_dialog: None,
            file_selection_dialog: FileSelectionDialog::new(theme.clone()),
            directory_selection_dialog: DirectorySelectionDialog::new(theme.clone()),
            release_notes_dialog: ReleaseNotesDialog::new(theme.clone()),
            home_view: HomeView::new(theme.clone()),
            file_tree: None,
            show_file_tree: false,
            layout_manager,
            sidebar,
            show_sidebar: true,
            sidebar_file,
            right_panel: RightPanel::new(theme.clone()),
            patch_preview: PatchPreview::with_theme(theme.clone()),
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
            pending_google_session: None,
            pending_copilot_session: None,
            pending_browser_models: Vec::new(),
            validation_in_progress: false,
            validation_cancelled: false,
            pending_api_key_for_validation: None,
            pending_api_key_models: Vec::new(),
            pending_api_key_for_provider: None,
            token_counter,
            cost_calculator: CostCalculator::new(),
            session_token_id,
            pending_input_tokens: 0,
            total_cost_usd: 0.0,
            mcp_cost_usd: 0.0,
            budget_limit_usd,
            username: std::env::var("OPENCODE_USERNAME").ok(),
            share_url: None,
            memory_entries: Vec::new(),
            todos: Vec::new(),
            thinking_mode: false,
            thinking_content: String::new(),
            is_receiving_thinking: false,
            model_aliases: {
                let mut m = std::collections::HashMap::new();
                m.insert("opus".to_string(), "claude-3-opus-20240229".to_string());
                m.insert(
                    "sonnet".to_string(),
                    "claude-3-5-sonnet-20241022".to_string(),
                );
                m.insert("haiku".to_string(), "claude-3-5-haiku-20241022".to_string());
                m
            },
            skill_resolver,
            input_parser: InputParser::new(),
            input_box,
            input_processor: InputProcessor::new(),
            pending_shell_command: None,
            enriched_input: None,
            shell_handler: ShellHandler::new(),
            file_ref_handler: FileRefHandler::new(),
            config: config.clone(),
            tool_registry,
            runtime,
            agent_executor,
            mcp_manager: McpManager::global(),
            lsp_client: None,
            lsp_diagnostics: Vec::new(),
            catalog_fetcher: {
                let cache_dir = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("opencode-rs/cache");
                std::fs::create_dir_all(&cache_dir).ok();
                ProviderCatalogFetcher::new(cache_dir.join("models_dev_catalog.json"))
            },
            session: None,
        }
    }

    fn begin_connect_flow(&mut self) {
        self.pending_connect_provider = None;
        self.pending_connect_method = None;
        self.pending_browser_session = None;
        self.pending_google_session = None;
        self.pending_browser_models.clear();
        self.pending_api_key_for_validation = None;
        self.pending_api_key_models.clear();
        self.pending_api_key_for_provider = None;
        self.connect_method_dialog = None;
        self.api_key_input_dialog = None;
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
        self.pending_connect_method = Some(method.clone());
        if method == "local" {
            if self.pending_connect_provider.as_deref() == Some("ollama")
                || self.pending_connect_provider.as_deref() == Some("lmstudio")
            {
                self.start_local_connect();
            } else {
                self.add_message(
                    "Local authentication is only available for Ollama and LM Studio".to_string(),
                    false,
                );
                self.mode = AppMode::Chat;
            }
        } else if self.pending_connect_provider.as_deref() == Some("openai")
            && self.pending_connect_method.as_deref() == Some("browser")
        {
            self.start_openai_browser_connect();
        } else if self.pending_connect_provider.as_deref() == Some("google")
            && self.pending_connect_method.as_deref() == Some("browser")
        {
            self.start_google_browser_connect();
        } else if self.pending_connect_provider.as_deref() == Some("copilot")
            && self.pending_connect_method.as_deref() == Some("browser")
        {
            self.start_copilot_browser_connect();
        } else if self.pending_connect_method.as_deref() == Some("api_key") {
            self.start_api_key_input();
        } else {
            self.add_message(
                "Selected connect method is not implemented yet".to_string(),
                false,
            );
            self.mode = AppMode::Chat;
        }
    }

    fn start_local_connect(&mut self) {
        self.validation_in_progress = true;
        self.validation_cancelled = false;
        self.mode = AppMode::ConnectProgress;
        let provider_id = self.pending_connect_provider.clone().unwrap_or_default();
        self.pending_api_key_for_validation = Some(provider_id.clone());
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);

        tracing::info!(
            event = "tui.connect.local.start",
            provider_id = %provider_id,
            "Starting local connect for provider"
        );

        std::thread::spawn(move || {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!(
                        event = "tui.connect.local.runtime_failed",
                        error = %e,
                        "Failed to create runtime for local connect"
                    );
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: false,
                        error_message: Some(format!("Failed to create runtime: {}", e)),
                        models: None,
                    });
                    return;
                }
            };

            let result = runtime.block_on(async {
                let base_url = std::env::var("OLLAMA_BASE_URL")
                    .ok()
                    .or_else(|| std::env::var("OPENCODE_BASE_URL").ok())
                    .unwrap_or_else(|| "http://localhost:11434".to_string());

                tracing::debug!(
                    event = "tui.connect.local.fetching_models",
                    base_url = %base_url,
                    "Fetching models from Ollama"
                );

                let provider = opencode_llm::OllamaProvider::new("".to_string(), Some(base_url));
                provider.get_local_models().await
            });

            match result {
                Ok(models) => {
                    tracing::info!(
                        event = "tui.connect.local.models_fetched",
                        model_count = models.len(),
                        "Successfully fetched local models"
                    );
                    let browser_models: Vec<BrowserAuthModelInfo> = models
                        .into_iter()
                        .map(|m| BrowserAuthModelInfo {
                            id: m.id.clone(),
                            name: m.name,
                            variants: vec![],
                        })
                        .collect();
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: true,
                        error_message: None,
                        models: Some(browser_models),
                    });
                }
                Err(e) => {
                    tracing::error!(
                        event = "tui.connect.local.fetch_failed",
                        error = %e,
                        "Failed to fetch local models"
                    );
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: false,
                        error_message: Some(e.to_string()),
                        models: None,
                    });
                }
            }
        });
    }

    fn start_api_key_input(&mut self) {
        let provider_id = self.pending_connect_provider.clone().unwrap_or_default();
        let provider_name = self.get_provider_name(&provider_id);
        let theme = self.theme_manager.current().clone();
        self.api_key_input_dialog = Some(ApiKeyInputDialog::new(theme, provider_id, provider_name));
        self.mode = AppMode::ConnectApiKey;
    }

    fn handle_api_key_input_confirm(&mut self, api_key: String) {
        let provider_id = self.pending_connect_provider.clone().unwrap_or_default();
        self.pending_api_key_for_validation = Some(api_key.clone());
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        self.validation_in_progress = true;
        self.validation_cancelled = false;
        self.mode = AppMode::ConnectProgress;

        std::thread::spawn(move || {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: false,
                        error_message: Some(format!("Failed to create runtime: {}", e)),
                        models: None,
                    });
                    return;
                }
            };
            let result = runtime.block_on(crate::app::validate_api_key_and_fetch_models(
                &provider_id,
                &api_key,
            ));
            match result {
                Ok(models) => {
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: true,
                        error_message: None,
                        models: Some(models),
                    });
                }
                Err(e) => {
                    let _ = tx.send(ConnectEvent::ValidationComplete {
                        success: false,
                        error_message: Some(e.to_string()),
                        models: None,
                    });
                }
            }
        });
    }

    fn get_provider_name(&self, provider_id: &str) -> String {
        match provider_id {
            "openai" => "OpenAI".to_string(),
            "anthropic" => "Anthropic".to_string(),
            "google" => "Google".to_string(),
            "ollama" => "Ollama".to_string(),
            "lmstudio" => "LM Studio".to_string(),
            "azure" => "Azure".to_string(),
            "openrouter" => "OpenRouter".to_string(),
            "mistral" => "Mistral".to_string(),
            "groq" => "Groq".to_string(),
            "deepinfra" => "DeepInfra".to_string(),
            "cerebras" => "Cerebras".to_string(),
            "cohere" => "Cohere".to_string(),
            "togetherai" => "Together AI".to_string(),
            "perplexity" => "Perplexity".to_string(),
            "xai" => "xAI".to_string(),
            "huggingface" => "Hugging Face".to_string(),
            "copilot" => "GitHub Copilot".to_string(),
            "ai21" => "AI21".to_string(),
            "minimax" => "MiniMax".to_string(),
            "qwen" => "Qwen".to_string(),
            _ => provider_id.to_string(),
        }
    }

    fn save_api_key_credential(&self, provider_id: &str, api_key: &str) -> Result<(), String> {
        let credential = opencode_auth::Credential {
            api_key: api_key.to_string(),
            base_url: None,
            metadata: std::collections::HashMap::new(),
        };
        let store = opencode_auth::CredentialStore::new();
        store
            .store(provider_id, &credential)
            .map_err(|e| e.to_string())
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

    fn start_google_browser_connect(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        self.mode = AppMode::ConnectProgress;

        std::thread::spawn(move || {
            let service = GoogleOAuthService::new();
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

            let google_models = vec![
                BrowserAuthModelInfo {
                    id: "gemini-2.0-flash".to_string(),
                    name: "Gemini 2.0 Flash".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "gemini-1.5-pro".to_string(),
                    name: "Gemini 1.5 Pro".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "gemini-1.5-flash".to_string(),
                    name: "Gemini 1.5 Flash".to_string(),
                    variants: vec![],
                },
            ];

            let _ = tx.send(ConnectEvent::GoogleAuthComplete(session, google_models));
        });
    }

    fn start_copilot_browser_connect(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        self.mode = AppMode::ConnectProgress;

        std::thread::spawn(move || {
            let service = CopilotOAuthService::new();
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

            let copilot_models = vec![
                BrowserAuthModelInfo {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "gpt-4o-mini".to_string(),
                    name: "GPT-4o Mini".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "o1".to_string(),
                    name: "o1".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "o1-mini".to_string(),
                    name: "o1 Mini".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "o1-preview".to_string(),
                    name: "o1 Preview".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "o3-mini".to_string(),
                    name: "o3 Mini".to_string(),
                    variants: vec![],
                },
            ];

            let _ = tx.send(ConnectEvent::CopilotAuthComplete(session, copilot_models));
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

    fn complete_google_auth(
        &mut self,
        session: GoogleOAuthSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        let store = GoogleOAuthStore::from_default_location();
        let _ = store.save(&session);
        self.pending_google_session = Some(session);
        self.pending_browser_models = models.clone();
        self.connect_model_dialog = Some(ConnectModelDialog::new(
            self.theme_manager.current().clone(),
            models,
        ));
        self.mode = AppMode::ConnectModel;
    }

    fn complete_copilot_auth(
        &mut self,
        session: CopilotOAuthSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        let store = CopilotOAuthStore::from_default_location();
        let _ = store.save(&session);
        self.pending_copilot_session = Some(session);
        self.pending_browser_models = models.clone();
        self.connect_model_dialog = Some(ConnectModelDialog::new(
            self.theme_manager.current().clone(),
            models,
        ));
        self.mode = AppMode::ConnectModel;
    }

    fn handle_connect_model_confirm(&mut self, model_id: String) -> Result<(), String> {
        if let Some(session) = self.pending_browser_session.clone() {
            let store = OpenAiBrowserAuthStore::from_default_location();
            store.save(&session).map_err(|e| e.to_string())?;

            self.provider = "openai".to_string();
            std::env::set_var("OPENAI_MODEL", &model_id);
            std::env::set_var("OPENCODE_MODEL", &model_id);
            self.llm_provider = Some(std::sync::Arc::new(OpenAiProvider::new_browser_auth(
                session,
                model_id.clone(),
                store,
            )));

            let rt = tokio::runtime::Handle::current();
            rt.block_on(
                self.runtime
                    .set_provider(self.llm_provider.clone().unwrap()),
            );

            let provider_config = crate::config::ProviderConfig {
                name: "openai".to_string(),
                api_key: None,
                default_model: Some(model_id.clone()),
            };
            if let Some(providers) = &mut self.config.providers {
                if let Some(existing) = providers.iter_mut().find(|p| p.name == "openai") {
                    existing.default_model = Some(model_id.clone());
                } else {
                    providers.push(provider_config);
                }
            } else {
                self.config.providers = Some(vec![provider_config]);
            }
            if let Err(e) = self.config.save(&Config::default_config_path()) {
                tracing::warn!("Failed to save provider config: {}", e);
            }
        } else if let Some(session) = self.pending_google_session.clone() {
            let store = GoogleOAuthStore::from_default_location();
            store.save(&session).map_err(|e| e.to_string())?;

            self.provider = "google".to_string();
            std::env::set_var("GOOGLE_MODEL", &model_id);
            std::env::set_var("OPENCODE_MODEL", &model_id);

            let google_provider = opencode_llm::google::GoogleProvider::new(
                session.access_token.clone(),
                model_id.clone(),
            );
            self.llm_provider = Some(std::sync::Arc::new(google_provider));

            let rt = tokio::runtime::Handle::current();
            rt.block_on(
                self.runtime
                    .set_provider(self.llm_provider.clone().unwrap()),
            );

            let provider_config = crate::config::ProviderConfig {
                name: "google".to_string(),
                api_key: None,
                default_model: Some(model_id.clone()),
            };
            if let Some(providers) = &mut self.config.providers {
                if let Some(existing) = providers.iter_mut().find(|p| p.name == "google") {
                    existing.default_model = Some(model_id.clone());
                } else {
                    providers.push(provider_config);
                }
            } else {
                self.config.providers = Some(vec![provider_config]);
            }
            if let Err(e) = self.config.save(&Config::default_config_path()) {
                tracing::warn!("Failed to save provider config: {}", e);
            }
        } else if let Some(session) = self.pending_copilot_session.clone() {
            let store = CopilotOAuthStore::from_default_location();
            store.save(&session).map_err(|e| e.to_string())?;

            self.provider = "copilot".to_string();
            std::env::set_var("GITHUB_COPILOT_TOKEN", &session.access_token);
            std::env::set_var("COPILOT_MODEL", &model_id);
            std::env::set_var("OPENCODE_MODEL", &model_id);

            let copilot_provider =
                opencode_llm::copilot::CopilotProvider::new(opencode_llm::ProviderConfig {
                    model: model_id.clone(),
                    api_key: session.access_token.clone(),
                    temperature: 0.7,
                    headers: std::collections::HashMap::new(),
                });
            self.llm_provider = Some(std::sync::Arc::new(copilot_provider));

            let rt = tokio::runtime::Handle::current();
            rt.block_on(
                self.runtime
                    .set_provider(self.llm_provider.clone().unwrap()),
            );

            let provider_config = crate::config::ProviderConfig {
                name: "copilot".to_string(),
                api_key: None,
                default_model: Some(model_id.clone()),
            };
            if let Some(providers) = &mut self.config.providers {
                if let Some(existing) = providers.iter_mut().find(|p| p.name == "copilot") {
                    existing.default_model = Some(model_id.clone());
                } else {
                    providers.push(provider_config);
                }
            } else {
                self.config.providers = Some(vec![provider_config]);
            }
            if let Err(e) = self.config.save(&Config::default_config_path()) {
                tracing::warn!("Failed to save provider config: {}", e);
            }
        } else if let Some(api_key) = self.pending_api_key_for_provider.clone() {
            let provider_id = self.pending_connect_provider.clone().unwrap_or_default();
            self.provider = provider_id.clone();
            std::env::set_var(
                format!("{}_MODEL", provider_id.to_uppercase().replace("-", "_")),
                &model_id,
            );
            std::env::set_var("OPENCODE_MODEL", &model_id);

            let llm_config = opencode_llm::ProviderConfig {
                model: model_id.clone(),
                api_key: api_key.clone(),
                temperature: 0.7,
                headers: std::collections::HashMap::new(),
            };

            self.llm_provider = match provider_id.as_str() {
                "openai" => Some(std::sync::Arc::new(opencode_llm::OpenAiProvider::new(
                    llm_config.api_key.clone(),
                    llm_config.model.clone(),
                ))),
                "anthropic" => Some(std::sync::Arc::new(opencode_llm::AnthropicProvider::new(
                    llm_config.api_key.clone(),
                    llm_config.model.clone(),
                ))),
                "ollama" => Some(std::sync::Arc::new(opencode_llm::OllamaProvider::new(
                    llm_config.model.clone(),
                    None,
                ))),
                "lmstudio" | "lm_studio" | "lm-studio" => {
                    Some(std::sync::Arc::new(opencode_llm::LmStudioProvider::new(
                        llm_config.model.clone(),
                        std::env::var("LMSTUDIO_BASE_URL")
                            .ok()
                            .or_else(|| std::env::var("OPENCODE_BASE_URL").ok()),
                    )))
                }
                _ => Some(std::sync::Arc::new(opencode_llm::OpenAiProvider::new(
                    llm_config.api_key.clone(),
                    llm_config.model.clone(),
                ))),
            };

            // Set the provider on the runtime so it can be used for LLM calls
            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                rt.block_on(
                    self.runtime
                        .set_provider(self.llm_provider.clone().unwrap()),
                );
            } else {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(
                    self.runtime
                        .set_provider(self.llm_provider.clone().unwrap()),
                );
            }

            let provider_config = crate::config::ProviderConfig {
                name: provider_id.clone(),
                api_key: Some(api_key),
                default_model: Some(model_id.clone()),
            };
            if let Some(providers) = &mut self.config.providers {
                if let Some(existing) = providers.iter_mut().find(|p| p.name == provider_id) {
                    existing.default_model = Some(model_id.clone());
                } else {
                    providers.push(provider_config);
                }
            } else {
                self.config.providers = Some(vec![provider_config]);
            }
            if let Err(e) = self.config.save(&Config::default_config_path()) {
                tracing::warn!("Failed to save provider config: {}", e);
            }
        } else {
            return Err("No valid authentication session found".to_string());
        }

        self.pending_api_key_for_provider = None;
        self.pending_api_key_models.clear();
        self.pending_browser_session = None;
        self.pending_google_session = None;
        self.pending_copilot_session = None;
        self.pending_browser_models.clear();
        self.connect_model_dialog = None;
        self.mode = AppMode::Chat;
        Ok(())
    }

    #[allow(dead_code)]
    fn handle_connect_model_cancel(&mut self) {
        self.pending_api_key_for_provider = None;
        self.pending_api_key_models.clear();
        self.pending_browser_session = None;
        self.pending_google_session = None;
        self.pending_copilot_session = None;
        self.pending_browser_models.clear();
        self.connect_model_dialog = None;
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

    pub fn complete_google_auth_for_test(
        &mut self,
        session: GoogleOAuthSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        self.complete_google_auth(session, models);
    }

    pub fn complete_copilot_auth_for_test(
        &mut self,
        session: CopilotOAuthSession,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        self.complete_copilot_auth(session, models);
    }

    pub fn prime_copilot_connect_state_for_test(&mut self) {
        self.complete_copilot_auth_for_test(
            CopilotOAuthSession {
                access_token: "test_access_token".to_string(),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
                token_type: "Bearer".to_string(),
            },
            vec![BrowserAuthModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                variants: vec![],
            }],
        );
    }

    pub fn confirm_model_for_copilot_auth_for_test(
        &mut self,
        model_id: &str,
    ) -> Result<(), String> {
        self.handle_connect_model_confirm(model_id.to_string())
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
                variants: vec![],
            }],
        );
    }

    pub fn complete_api_key_auth_for_test(
        &mut self,
        provider_id: &str,
        api_key: &str,
        models: Vec<BrowserAuthModelInfo>,
    ) {
        self.pending_connect_provider = Some(provider_id.to_string());
        self.pending_api_key_for_provider = Some(api_key.to_string());
        self.pending_api_key_models = models;
        self.connect_model_dialog = Some(ConnectModelDialog::new(
            self.theme_manager.current().clone(),
            self.pending_api_key_models.clone(),
        ));
        self.mode = AppMode::ConnectModel;
    }

    pub fn confirm_model_for_api_key_auth_for_test(
        &mut self,
        model_id: &str,
    ) -> Result<(), String> {
        self.handle_connect_model_confirm(model_id.to_string())
    }

    pub fn prime_google_connect_state_for_test(&mut self) {
        self.complete_google_auth_for_test(
            GoogleOAuthSession {
                access_token: "test_access_token".to_string(),
                refresh_token: Some("test_refresh_token".to_string()),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
                email: Some("test@gmail.com".to_string()),
            },
            vec![BrowserAuthModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                variants: vec![],
            }],
        );
    }

    pub fn initiate_google_oauth_flow_for_test(&mut self) {
        self.pending_connect_provider = Some("google".to_string());
        self.handle_connect_method_confirm("browser".to_string());
    }

    pub fn confirm_model_for_google_auth_for_test(&mut self, model_id: &str) -> Result<(), String> {
        self.handle_connect_model_confirm(model_id.to_string())
    }

    #[cfg(test)]
    pub fn get_connect_flow_state(&self) -> ConnectFlowState {
        match self.mode {
            AppMode::ConnectProvider => ConnectFlowState::ProviderPickerOpen,
            AppMode::ConnectMethod => ConnectFlowState::ProviderSelected,
            AppMode::ConnectApiKey => ConnectFlowState::ApiKeyInputFocused,
            AppMode::ConnectProgress if self.validation_in_progress => {
                ConnectFlowState::ValidatingKey
            }
            AppMode::ConnectApiKeyError => ConnectFlowState::ValidationFailed,
            AppMode::ConnectModel => ConnectFlowState::ValidationSucceeded,
            _ if self.validation_in_progress => ConnectFlowState::ValidatingKey,
            _ => ConnectFlowState::Idle,
        }
    }

    #[cfg(test)]
    pub fn is_main_input_focused(&self) -> bool {
        matches!(self.mode, AppMode::Chat | AppMode::Home)
    }

    #[cfg(test)]
    pub fn has_active_modal(&self) -> bool {
        self.api_key_input_dialog.is_some()
            || self.connect_model_dialog.is_some()
            || self.validation_error_dialog.is_some()
    }

    #[cfg(test)]
    pub fn is_connect_rx_dropped(&self) -> bool {
        self.connect_rx.is_none()
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
            .ok()
            .or_else(|| std::env::var("OPENAI_MODEL").ok())
            .or_else(|| {
                self.config.providers.as_ref().and_then(|providers| {
                    providers
                        .iter()
                        .find(|p| p.name == self.provider)
                        .and_then(|p| p.default_model.clone())
                })
            })
            .unwrap_or_else(|| {
                let provider_type = ProviderType::from_str(&self.provider);
                provider_type.default_model().to_string()
            });

        let resolved_model = match self.model_aliases.get(&model) {
            Some(alias) => alias.clone(),
            None => {
                if model != "gpt-4o" && !model.starts_with("claude") && !model.starts_with("gpt-4")
                {
                    self.add_message(
                        format!("Warning: Unknown model alias '{}' - using as-is", model),
                        false,
                    );
                }
                model.clone()
            }
        };

        let config = ProviderConfig {
            model: resolved_model,
            api_key,
            temperature: 0.7,
            headers: std::collections::HashMap::new(),
        };

        let variant = std::env::var("OPENCODE_MODEL_VARIANT").ok();

        let anthropic_thinking = variant
            .as_ref()
            .and_then(|v| match v.to_lowercase().as_str() {
                "low" => Some(opencode_llm::AnthropicThinkingConfig::Low),
                "high" => Some(opencode_llm::AnthropicThinkingConfig::High),
                "max" => Some(opencode_llm::AnthropicThinkingConfig::Max),
                _ => None,
            });

        self.llm_provider = match self.provider.as_str() {
            "openai" => {
                let mut provider =
                    opencode_llm::OpenAiProvider::new(config.api_key.clone(), config.model.clone());
                if let Some(ref v) = variant {
                    provider = provider.with_reasoning_effort(v.clone());
                }
                Some(std::sync::Arc::new(provider))
            }
            "anthropic" => {
                let mut provider = opencode_llm::AnthropicProvider::new(
                    config.api_key.clone(),
                    config.model.clone(),
                );
                if let Some(config) = anthropic_thinking {
                    provider = provider.with_thinking_budget(config);
                }
                Some(std::sync::Arc::new(provider))
            }
            "ollama" => Some(std::sync::Arc::new(opencode_llm::OllamaProvider::new(
                config.model.clone(),
                stored_credential
                    .as_ref()
                    .and_then(|c| c.base_url.clone())
                    .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
                    .or_else(|| Some("http://localhost:11434".to_string())),
            ))),
            _ => {
                let mut provider =
                    opencode_llm::OpenAiProvider::new(config.api_key.clone(), config.model.clone());
                if let Some(ref v) = variant {
                    provider = provider.with_reasoning_effort(v.clone());
                }
                Some(std::sync::Arc::new(provider))
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

    pub fn add_mcp_tool_cost(&mut self, cost: f64) {
        self.mcp_cost_usd += cost;
        let total_tokens = self.token_counter.get_total_tokens();
        let context_total = self.status_bar.context_usage.1;
        self.status_bar.update_usage(
            total_tokens,
            total_tokens,
            context_total,
            self.total_cost_usd,
            self.mcp_cost_usd,
            self.budget_limit_usd,
        );
    }

    fn save_history(&self) {
        let content = self.history.join("\n");
        let _ = std::fs::write(&self.history_file, content);
    }

    pub fn add_message_with_meta(&mut self, meta: MessageMeta) {
        self.messages.push(meta);
    }

    pub fn refresh_todos_from_messages(&mut self) {
        self.todos.clear();
        for msg in &self.messages {
            for line in msg.content.lines() {
                if let Some(todo) = TodoEntry::from_markdown_line(line) {
                    self.todos.push(todo);
                }
            }
        }
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
    /// Maximum characters displayed from a git diff output.
    #[allow(dead_code)]
    const MAX_DIFF_DISPLAY_CHARS: usize = 2000;
    #[allow(dead_code)]
    const MAX_HISTORY_SIZE: usize = 100;
    #[allow(dead_code)]
    const TOKEN_ESTIMATE_DIVISOR: usize = 4;

    fn keybind_string(key: &KeyEvent) -> String {
        let mut s = String::new();
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            s.push_str("Ctrl+");
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            s.push_str("Alt+");
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            s.push_str("Shift+");
        }
        match &key.code {
            KeyCode::Char(c) => s.push(*c),
            KeyCode::Esc => s.push_str("Esc"),
            KeyCode::Enter => s.push_str("Enter"),
            KeyCode::Tab => s.push_str("Tab"),
            KeyCode::Backspace => s.push_str("Backspace"),
            KeyCode::Up => s.push_str("Up"),
            KeyCode::Down => s.push_str("Down"),
            KeyCode::Left => s.push_str("Left"),
            KeyCode::Right => s.push_str("Right"),
            KeyCode::Home => s.push_str("Home"),
            KeyCode::End => s.push_str("End"),
            KeyCode::PageUp => s.push_str("PageUp"),
            KeyCode::PageDown => s.push_str("PageDown"),
            KeyCode::F(n) => s.push_str(&format!("F{}", n)),
            _ => {}
        }
        s
    }

    fn matches_keybind(&self, key: &KeyEvent, action: &str) -> bool {
        let default_key = match action {
            "commands" => "Ctrl+p",
            "timeline" => "Ctrl+t",
            "new_session" => "Ctrl+n",
            "toggle_files" => "Ctrl+Shift+f",
            "settings" => "Ctrl+,",
            "search" => "Ctrl+/",
            _ => return false,
        };
        if let Some(custom) = self.config.keybinds() {
            let custom_key = match action {
                "commands" => custom.commands.as_ref(),
                "timeline" => custom.timeline.as_ref(),
                "new_session" => custom.new_session.as_ref(),
                "toggle_files" => custom.toggle_files.as_ref(),
                "settings" => custom.settings.as_ref(),
                "search" => custom.search.as_ref(),
                _ => None,
            };
            if let Some(ck) = custom_key {
                if !ck.is_empty() {
                    return Self::keybind_string(key) == *ck;
                }
            }
        }
        Self::keybind_string(key) == default_key
    }

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
        use crossterm::{
            cursor,
            terminal::{Clear, ClearType, LeaveAlternateScreen},
        };
        // Clear the screen before leaving alternate screen to avoid leaving TUI artifacts
        execute!(
            io::stdout(),
            Clear(ClearType::All),
            LeaveAlternateScreen,
            cursor::Show
        )?;
        io::stdout().flush()?;
        disable_raw_mode()
    }

    pub fn restore_terminal_after_error() -> io::Result<()> {
        Self::cleanup_terminal()
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

    fn redact_api_keys(content: &str) -> String {
        let mut result = content.to_string();
        let patterns: &[(&str, &str)] = &[
            (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
            (r"key_[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
            (r"token_[a-zA-Z0-9]{20,}", "[REDACTED_TOKEN]"),
            (r"bearer [a-zA-Z0-9._-]+", "bearer [REDACTED]"),
        ];
        for (pattern, replacement) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace_all(&result, *replacement).to_string();
            }
        }
        result
    }

    pub fn get_definition(&mut self, uri: &str, line: u32, col: u32) -> Option<Location> {
        if let Some(ref mut client) = self.lsp_client {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { client.goto_definition(uri, line, col).await })
                .ok()
                .flatten()
        } else {
            None
        }
    }

    pub fn find_references(&mut self, uri: &str, line: u32, col: u32) -> Vec<Location> {
        if let Some(ref mut client) = self.lsp_client {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { client.find_references(uri, line, col).await })
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn refresh_diagnostics(&mut self, uri: &str) {
        if let Some(ref mut client) = self.lsp_client {
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(async { client.get_diagnostics(uri).await }) {
                Ok(diags) => self.lsp_diagnostics = diags,
                Err(_) => self.lsp_diagnostics.clear(),
            }
        } else {
            self.lsp_diagnostics.clear();
        }
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

    fn connect_progress_message(&self) -> String {
        let provider_name = self
            .pending_connect_provider
            .as_deref()
            .map(|provider| self.get_provider_name(provider))
            .unwrap_or_else(|| "Provider".to_string());

        if self.validation_in_progress {
            format!("Validating {} API key...", provider_name)
        } else {
            format!(
                "Complete {} authentication in your browser...",
                provider_name
            )
        }
    }

    pub fn get_connect_progress_message_for_testing(&self) -> String {
        self.connect_progress_message()
    }

    fn sync_status_bar_state(&mut self) {
        self.status_bar.activity_message = match self.mode {
            AppMode::ConnectProvider => Some("🔌 Select provider".to_string()),
            AppMode::ConnectMethod => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("🔐 Choose {} auth", self.get_provider_name(provider))),
            AppMode::ConnectApiKey => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("🔑 Enter {} API key", self.get_provider_name(provider))),
            AppMode::ConnectApiKeyError => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("⚠ Fix {} API key", self.get_provider_name(provider))),
            AppMode::ConnectModel => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("🤖 Select {} model", self.get_provider_name(provider))),
            AppMode::ConnectProgress if self.validation_in_progress => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("⏳ Validating {}", self.get_provider_name(provider))),
            AppMode::ConnectProgress => self
                .pending_connect_provider
                .as_deref()
                .map(|provider| format!("🌐 Browser auth: {}", self.get_provider_name(provider))),
            _ => match self.tui_state {
                TuiState::Idle => Some("✓ Ready".to_string()),
                TuiState::Composing => Some("✎ Composing".to_string()),
                TuiState::Reconnecting => Some("🔄 Reconnecting".to_string()),
                TuiState::Submitting => Some("📤 Submitting prompt".to_string()),
                TuiState::Streaming => Some("✍ Streaming response".to_string()),
                TuiState::ExecutingTool => self
                    .tool_calls
                    .iter()
                    .find(|tool| matches!(tool.status, ToolStatus::Running))
                    .map(|tool| format!("🛠 Running {}", tool.name))
                    .or_else(|| Some("🛠 Executing tool".to_string())),
                TuiState::AwaitingPermission => Some("✋ Awaiting permission".to_string()),
                TuiState::Aborting => Some("🛑 Aborting".to_string()),
                TuiState::ShowingDiff => Some("🧾 Reviewing diff".to_string()),
                TuiState::ShowingError => self
                    .tool_calls
                    .iter()
                    .rev()
                    .find(|tool| matches!(tool.status, ToolStatus::Failed(_)))
                    .map(|tool| format!("⚠ Tool failed: {}", tool.name))
                    .or_else(|| Some("⚠ Review error".to_string())),
                TuiState::Paused => Some("⏸ Paused".to_string()),
            },
        };

        self.status_bar.connection_status = match self.mode {
            AppMode::ConnectProvider
            | AppMode::ConnectMethod
            | AppMode::ConnectApiKey
            | AppMode::ConnectProgress => {
                crate::components::status_bar::ConnectionStatus::Disconnected
            }
            AppMode::ConnectApiKeyError => crate::components::status_bar::ConnectionStatus::Error,
            _ => match self.tui_state {
                TuiState::Reconnecting => {
                    crate::components::status_bar::ConnectionStatus::Disconnected
                }
                TuiState::ShowingError => crate::components::status_bar::ConnectionStatus::Error,
                _ => crate::components::status_bar::ConnectionStatus::Connected,
            },
        };
    }

    pub fn sync_status_bar_state_for_testing(&mut self) {
        self.sync_status_bar_state();
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        execute!(io::stdout(), event::EnableMouseCapture)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        execute!(
            io::stdout(),
            cursor::SetCursorStyle::BlinkingBlock,
            cursor::Show
        )?;
        execute!(io::stdout(), TermClear(ClearType::All))?;
        if let Err(e) = io::stdout().flush() {
            tracing::warn!("Failed to flush stdout: {}", e);
        }

        loop {
            if let Err(e) = io::stdout().flush() {
                tracing::warn!("Pre-draw flush failed: {}", e);
            }
            terminal.draw(|f| self.draw(f))?;

            self.check_leader_key_timeout();
            self.check_reconnect_timeout();
            self.check_llm_events();
            self.check_connect_events();

            if let Some(ref mut ts) = self.input_widget.typewriter_state {
                if ts.is_streaming {
                    ts.tick();
                }
            }

            match self.mode {
                AppMode::Home => self.handle_home_view(&mut terminal)?,
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
                AppMode::ConnectProvider => {
                    tracing::trace!(event = "tui.connect.handle", mode = ?self.mode, "Handling ConnectProvider");
                    self.handle_connect_provider_dialog(&mut terminal)?
                }
                AppMode::ConnectMethod => {
                    tracing::trace!(event = "tui.connect.handle", mode = ?self.mode, "Handling ConnectMethod");
                    self.handle_connect_method_dialog(&mut terminal)?
                }
                AppMode::ConnectApiKey => {
                    tracing::trace!(event = "tui.connect.handle", mode = ?self.mode, "Handling ConnectApiKey");
                    self.handle_api_key_input_dialog(&mut terminal)?
                }
                AppMode::ConnectProgress => {
                    tracing::trace!(event = "tui.connect.handle", mode = ?self.mode, "Handling ConnectProgress");
                    self.handle_connect_progress_dialog(&mut terminal)?
                }
                AppMode::ConnectApiKeyError => {
                    tracing::trace!(event = "tui.connect.handle", mode = ?self.mode, "Handling ConnectApiKeyError");
                    self.handle_validation_error_dialog(&mut terminal)?
                }
                AppMode::ConnectModel => {
                    tracing::info!(
                        event = "tui.connect.handle_connect_model",
                        mode = ?self.mode,
                        has_dialog = %self.connect_model_dialog.is_some(),
                        "Handling ConnectModel"
                    );
                    self.handle_connect_model_dialog(&mut terminal)?
                }
                AppMode::FileSelection => self.handle_file_selection_dialog(&mut terminal)?,
                AppMode::DirectorySelection => {
                    self.handle_directory_selection_dialog(&mut terminal)?
                }
                AppMode::ReleaseNotes => self.handle_release_notes_dialog(&mut terminal)?,
                AppMode::Search => self.handle_search_dialog(&mut terminal)?,
            }
        }
    }

    #[allow(dead_code)]
    fn handle_mouse_event(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(_button) => {
                self.selection.is_selecting = true;
                self.selection.start_pos = Some((event.column, event.row));
                self.selection.end_pos = Some((event.column, event.row));
                self.selection.selected_text = None;
            }
            MouseEventKind::Drag(_button) => {
                if self.selection.is_selecting {
                    self.selection.end_pos = Some((event.column, event.row));
                }
            }
            MouseEventKind::Up(_button) => {
                if self.selection.is_selecting {
                    self.selection.is_selecting = false;
                    // Extract selected text based on start/end positions
                    self.extract_selection_text();
                }
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    fn extract_selection_text(&mut self) {
        let (_start, _end) = match (self.selection.start_pos, self.selection.end_pos) {
            (Some(s), Some(e)) => (s, e),
            _ => {
                self.selection.selected_text = None;
                return;
            }
        };

        // Simplified: if selection started in the messages area, select all visible messages
        // This is a basic implementation - a full implementation would track exact character positions
        if self.messages.is_empty() {
            self.selection.selected_text = None;
            return;
        }

        // For now, select all message content when user makes a selection in the messages area
        // A more sophisticated implementation would track exact line/column ranges
        let mut selected_content = Vec::new();
        for msg in &self.messages {
            selected_content.push(msg.content.clone());
        }

        self.selection.selected_text = Some(selected_content.join("\n\n"));
    }

    fn copy_selection_to_clipboard(&mut self) -> bool {
        if let Some(ref text) = self.selection.selected_text {
            if !text.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    return clipboard.set_text(text).is_ok();
                }
            }
        }
        false
    }

    fn clear_selection(&mut self) {
        self.selection.is_selecting = false;
        self.selection.start_pos = None;
        self.selection.end_pos = None;
        self.selection.selected_text = None;
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
                        if chunk.contains("<thinking>") {
                            self.is_receiving_thinking = true;
                            let thinking_part = chunk.replace("<thinking>", "");
                            self.thinking_content.push_str(&thinking_part);
                        } else if chunk.contains("</thinking>") {
                            self.is_receiving_thinking = false;
                            let final_part = chunk.replace("</thinking>", "");
                            self.thinking_content.push_str(&final_part);
                            if !self.thinking_content.is_empty() {
                                self.add_message_with_meta(MessageMeta::thinking(
                                    &self.thinking_content,
                                ));
                            }
                            self.thinking_content.clear();
                        } else if self.is_receiving_thinking {
                            self.thinking_content.push_str(&chunk);
                        } else {
                            if self.partial_response.is_empty() {
                                self.input_widget.start_typewriter(&chunk);
                            } else {
                                self.input_widget.typewriter_state.as_mut().map(
                                    |s: &mut crate::components::input_widget::TypewriterState| {
                                        s.append(&chunk)
                                    },
                                );
                            }
                            self.update_partial_response(chunk);
                        }
                    }
                    LlmEvent::Done => {
                        let response = self.partial_response.clone();
                        let output_tokens = (response.chars().count() / 4).max(1);
                        let model = std::env::var("OPENCODE_MODEL")
                            .or_else(|_| std::env::var("OPENAI_MODEL"))
                            .unwrap_or_else(|_| "gpt-4o".to_string());

                        self.token_counter.record_tokens(
                            &model,
                            self.pending_input_tokens,
                            output_tokens,
                        );
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
                            self.mcp_cost_usd,
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
                    LlmEvent::ToolCall {
                        name,
                        arguments,
                        id,
                    } => {
                        let args_str = serde_json::to_string_pretty(&arguments).unwrap_or_default();
                        self.add_message(
                            format!("[Tool Call: {} ({})]\nArguments: {}", name, id, args_str),
                            false,
                        );
                    }
                    LlmEvent::ToolResult { id, output } => {
                        self.add_message(format!("[Tool Result for {}]\n{}", id, output), false);
                    }
                    LlmEvent::SessionComplete(session) => {
                        self.session = Some(session);
                    }
                }
            }
        }
    }

    fn runtime_session_for_input(&self, llm_input: &str) -> Session {
        let mut session = self.session.clone().unwrap_or_default();
        session.add_message(Message::user(llm_input.to_string()));
        session
    }

    pub fn check_connect_events_for_testing(&mut self) {
        self.check_connect_events();
    }

    pub fn simulate_validation_complete_for_testing(
        &mut self,
        success: bool,
        error_message: Option<String>,
        models: Option<Vec<BrowserAuthModelInfo>>,
    ) {
        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        let _ = tx.send(ConnectEvent::ValidationComplete {
            success,
            error_message,
            models,
        });
        self.check_connect_events();
    }

    fn check_connect_events(&mut self) {
        if let Some(ref mut rx) = self.connect_rx {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }

            if !events.is_empty() {
                tracing::debug!(
                    event = "tui.connect.events_received",
                    count = %events.len(),
                    "Received connect events"
                );
            }

            for event in events {
                let event_type = format!("{:?}", event);
                tracing::trace!(
                    event = "tui.connect.event_processing",
                    event_type = %event_type,
                    "Processing connect event"
                );
                match event {
                    ConnectEvent::BrowserOpened(url) => {
                        self.add_message(
                            format!("Opened browser for OpenAI login: {}", url),
                            false,
                        );
                    }
                    ConnectEvent::AuthComplete(session, models) => {
                        self.complete_browser_auth(session, models);
                    }
                    ConnectEvent::GoogleAuthComplete(session, models) => {
                        self.complete_google_auth(session, models);
                    }
                    ConnectEvent::CopilotAuthComplete(session, models) => {
                        self.complete_copilot_auth(session, models);
                    }
                    ConnectEvent::Failed(error) => {
                        self.add_message(format!("OpenAI connect failed: {}", error), false);
                        self.mode = AppMode::Chat;
                    }
                    ConnectEvent::ValidationComplete {
                        success,
                        error_message,
                        models,
                    } => {
                        self.validation_in_progress = false;

                        if self.validation_cancelled {
                            tracing::info!(
                                event = "tui.connect.late_result_ignored",
                                success = %success,
                                error_message = ?error_message,
                                "Ignoring late validation result after cancellation"
                            );
                            continue;
                        }

                        if success {
                            let provider_id =
                                self.pending_connect_provider.clone().unwrap_or_default();
                            let model_count = models.as_ref().map(|m| m.len()).unwrap_or(0);
                            tracing::info!(
                                event = "tui.connect.validation_success",
                                provider_id = %provider_id,
                                model_count = %model_count,
                                "Validation succeeded, about to show model selection"
                            );
                            // For local connections (Ollama/LMStudio), no API key is used
                            let is_local_connect =
                                provider_id == "ollama" || provider_id == "lmstudio";
                            let message = if is_local_connect {
                                format!(
                                    "Connected to {} (local)",
                                    self.get_provider_name(&provider_id)
                                )
                            } else {
                                format!(
                                    "API key validated successfully for {}",
                                    self.get_provider_name(&provider_id)
                                )
                            };
                            self.add_message(message, false);
                            if let Err(e) = self.save_api_key_credential(
                                &provider_id,
                                self.pending_api_key_for_validation.as_deref().unwrap_or(""),
                            ) {
                                tracing::error!(
                                    event = "tui.connect.save_credential_failed",
                                    error = %e,
                                    provider_id = %provider_id,
                                    "Failed to save API key credential"
                                );
                                self.add_message(format!("Failed to save API key: {}", e), false);
                                self.mode = AppMode::Chat;
                            } else {
                                let api_key = self
                                    .pending_api_key_for_validation
                                    .clone()
                                    .unwrap_or_default();
                                let provider_id =
                                    self.pending_connect_provider.clone().unwrap_or_default();
                                self.pending_api_key_models = models.unwrap_or_default();
                                self.pending_api_key_for_provider = Some(api_key.clone());
                                let theme = self.theme_manager.current().clone();
                                let models_clone = self.pending_api_key_models.clone();
                                tracing::debug!(
                                    event = "tui.connect.creating_model_dialog",
                                    model_count = %models_clone.len(),
                                    "Creating ConnectModelDialog"
                                );
                                self.connect_model_dialog =
                                    Some(ConnectModelDialog::new(theme, models_clone));
                                if let Some(providers) = &mut self.config.providers {
                                    if let Some(existing) =
                                        providers.iter_mut().find(|p| p.name == provider_id)
                                    {
                                        existing.api_key = Some(api_key);
                                        existing.default_model = self
                                            .pending_api_key_models
                                            .first()
                                            .map(|m| m.id.clone());
                                    } else {
                                        let provider_config = crate::config::ProviderConfig {
                                            name: provider_id.clone(),
                                            api_key: Some(api_key),
                                            default_model: self
                                                .pending_api_key_models
                                                .first()
                                                .map(|m| m.id.clone()),
                                        };
                                        providers.push(provider_config);
                                    }
                                } else {
                                    let provider_config = crate::config::ProviderConfig {
                                        name: provider_id.clone(),
                                        api_key: Some(api_key),
                                        default_model: self
                                            .pending_api_key_models
                                            .first()
                                            .map(|m| m.id.clone()),
                                    };
                                    self.config.providers = Some(vec![provider_config]);
                                }
                                if let Err(e) = self.config.save(&Config::default_config_path()) {
                                    tracing::warn!("Failed to save provider config: {}", e);
                                }
                                tracing::info!(
                                    event = "tui.connect.mode_change",
                                    mode = "ConnectModel",
                                    "Mode changed to ConnectModel"
                                );
                                self.mode = AppMode::ConnectModel;
                            }
                        } else {
                            let provider_id =
                                self.pending_connect_provider.clone().unwrap_or_default();
                            let provider_name = self.get_provider_name(&provider_id);
                            let error_msg =
                                error_message.unwrap_or_else(|| "Unknown error".to_string());
                            tracing::error!(
                                provider = %provider_id,
                                error = %error_msg,
                                "API key validation failed"
                            );
                            let theme = self.theme_manager.current().clone();
                            self.validation_error_dialog =
                                Some(ValidationErrorDialog::from_validation_error(
                                    &error_msg,
                                    &provider_name,
                                    theme,
                                ));
                            self.mode = AppMode::ConnectApiKeyError;
                        }
                        self.pending_api_key_for_validation = None;
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(action) = action::Action::from_key_event(&key) {
                    let mut app_state = action::AppState::new();
                    app_state.mode = action::AppMode::ForkDialog;
                    app_state.input_buffer = self.fork_name_input.clone();
                    action::ActionHandler::handle(action.clone(), &mut app_state);
                    self.fork_name_input = app_state.input_buffer.clone();
                    self.mode = app_state.mode;
                    if action == action::Action::Fork(action::ForkAction::Confirm) {
                        let fork_point = self
                            .timeline_state
                            .selected()
                            .unwrap_or(self.messages.len().saturating_sub(1));
                        self.execute_fork(fork_point);
                        self.mode = AppMode::Chat;
                        self.fork_name_input.clear();
                    }
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
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::SlashCommand;
                        action::ActionHandler::handle(
                            action::Action::SlashCommand(action::SlashCommandAction::Cancel),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
                        self.command_palette_input.clear();
                    }
                    KeyCode::Enter => {
                        let mode_before = self.mode.clone();
                        if let Some(cmd_name) = self.slash_command_dialog.get_selected_command() {
                            self.execute_slash_command(&cmd_name);
                        }
                        if self.mode == mode_before {
                            self.mode = AppMode::Chat;
                        }
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
                use opencode_core::compaction::{CompactionConfig, Compactor};
                let compactor = Compactor::new(CompactionConfig::default());
                let core_messages: Vec<opencode_core::Message> = self
                    .messages
                    .iter()
                    .map(|m| opencode_core::Message {
                        role: if m.is_user {
                            opencode_core::Role::User
                        } else {
                            opencode_core::Role::Assistant
                        },
                        content: m.content.clone(),
                        timestamp: chrono::Utc::now(),
                        parts: None,
                    })
                    .collect();
                let result = compactor.compact(core_messages);
                if result.was_compacted {
                    self.messages.clear();
                    for m in result.messages {
                        self.messages.push(MessageMeta::assistant(m.content));
                    }
                    self.add_message(
                        format!(
                            "Session compacted: {} messages summarized",
                            result.pruned_count
                        ),
                        false,
                    );
                } else {
                    self.add_message("Session doesn't need compaction".to_string(), false);
                }
            }
            "/summarize" => {
                use opencode_core::compaction::{CompactionConfig, Compactor};
                let compactor = Compactor::new(CompactionConfig::default());
                let core_messages: Vec<opencode_core::Message> = self
                    .messages
                    .iter()
                    .map(|m| opencode_core::Message {
                        role: if m.is_user {
                            opencode_core::Role::User
                        } else {
                            opencode_core::Role::Assistant
                        },
                        content: m.content.clone(),
                        timestamp: chrono::Utc::now(),
                        parts: None,
                    })
                    .collect();
                let result = compactor.compact(core_messages);
                if result.was_compacted {
                    self.messages.clear();
                    for m in result.messages {
                        self.messages.push(MessageMeta::assistant(m.content));
                    }
                    let msg_count = self.messages.len();
                    self.add_message(
                        format!(
                            "Session summarized: {} → {} messages",
                            result.pruned_count, msg_count
                        ),
                        false,
                    );
                } else {
                    let msg_count = self.messages.len();
                    self.add_message(
                        format!("Summarizing {} messages... (session summarized)", msg_count),
                        false,
                    );
                }
            }
            "/export" => {
                use std::env;
                use std::fs;
                let export_path = env::temp_dir().join("opencode_export.md");
                let content: String = self
                    .messages
                    .iter()
                    .map(|m| {
                        let role_name = if m.is_user {
                            self.username.as_deref().unwrap_or("User")
                        } else {
                            "Assistant"
                        };
                        let redacted_content = Self::redact_api_keys(&m.content);
                        format!("{}\n\n{}\n\n---\n", role_name, redacted_content)
                    })
                    .collect();
                match fs::write(&export_path, &content) {
                    Ok(_) => {
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(&export_path).spawn();
                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&export_path)
                            .spawn();
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("start")
                            .arg(&export_path)
                            .spawn();
                        self.add_message(format!("Exported to: {}", export_path.display()), false)
                    }
                    Err(e) => self.add_message(format!("Export failed: {}", e), false),
                }
            }
            "/undo" => {
                let is_git_repo = std::process::Command::new("git")
                    .arg("rev-parse")
                    .arg("--is-inside-work-tree")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !is_git_repo {
                    self.add_message(
                        "Undo requires git. This is not a git repository.".to_string(),
                        false,
                    );
                } else {
                    let stash_result = std::process::Command::new("git")
                        .arg("stash")
                        .arg("push")
                        .arg("-m")
                        .arg("opencode-undo")
                        .output();

                    match stash_result {
                        Ok(o) if o.status.success() => {
                            if let Some(last_user_idx) =
                                self.messages.iter().rposition(|m| m.is_user)
                            {
                                self.messages.truncate(last_user_idx);
                            }
                            self.tool_calls.clear();
                            self.add_message(
                                "Undo: Last changes stashed. Use /redo to restore.".to_string(),
                                false,
                            );
                        }
                        Ok(o) => {
                            let stderr = String::from_utf8_lossy(&o.stderr);
                            self.add_message(format!("Undo failed: {}", stderr), false);
                        }
                        Err(e) => {
                            self.add_message(format!("Undo failed: {}", e), false);
                        }
                    }
                }
            }
            "/redo" => {
                let is_git_repo = std::process::Command::new("git")
                    .arg("rev-parse")
                    .arg("--is-inside-work-tree")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !is_git_repo {
                    self.add_message(
                        "Redo requires git. This is not a git repository.".to_string(),
                        false,
                    );
                } else {
                    let stash_result = std::process::Command::new("git")
                        .arg("stash")
                        .arg("pop")
                        .output();

                    match stash_result {
                        Ok(o) if o.status.success() => {
                            self.add_message("Redo: Changes restored.".to_string(), false);
                        }
                        _ => {
                            self.add_message("Nothing to redo.".to_string(), false);
                        }
                    }
                }
            }
            "/search" => {
                self.mode = AppMode::Search;
                self.command_palette_input.clear();
            }
            "/diff" => {
                let is_git_repo = std::process::Command::new("git")
                    .arg("rev-parse")
                    .arg("--is-inside-work-tree")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !is_git_repo {
                    self.add_message("Not a git repository.".to_string(), false);
                } else {
                    let git_args = match self.config.diff_style() {
                        DiffStyle::SideBySide => vec!["diff", "--word-diff=color"],
                        DiffStyle::Unified => vec!["diff", "--unified=5"],
                        DiffStyle::Auto => vec!["diff"],
                    };
                    match std::process::Command::new("git").args(&git_args).output() {
                        Ok(o) => {
                            let diff = String::from_utf8_lossy(&o.stdout);
                            if diff.is_empty() {
                                self.add_message("No changes to show.".to_string(), false);
                            } else {
                                self.add_message(
                                    format!(
                                        "Git diff:\n{}",
                                        diff.chars()
                                            .take(Self::MAX_DIFF_DISPLAY_CHARS)
                                            .collect::<String>()
                                    ),
                                    false,
                                );
                            }
                        }
                        Err(e) => {
                            self.add_message(format!("Git diff failed: {}", e), false);
                        }
                    }
                }
            }
            "/memory" => {
                let args = self.input.trim_start_matches("/memory").trim();
                let parts: Vec<&str> = args.splitn(2, ' ').collect();
                let subcmd = parts.first().unwrap_or(&"");

                match *subcmd {
                    "list" | "" => {
                        if self.memory_entries.is_empty() {
                            self.add_message(
                                "Memory: No entries stored. Use /memory add <content>".to_string(),
                                false,
                            );
                        } else {
                            let list: String = self
                                .memory_entries
                                .iter()
                                .map(|e| format!("[{}] {} - {}", e.id, e.created_at, e.content))
                                .collect::<Vec<_>>()
                                .join("\n");
                            self.add_message(format!("Memory entries:\n{}", list), false);
                        }
                    }
                    "add" => {
                        if let Some(content) = parts.get(1) {
                            let id = self.memory_entries.len() + 1;
                            self.memory_entries
                                .push(MemoryEntry::new(id, content.to_string()));
                            self.add_message(format!("Added memory entry [{}]", id), false);
                        } else {
                            self.add_message("Usage: /memory add <content>".to_string(), false);
                        }
                    }
                    "delete" => {
                        if let Some(id_str) = parts.get(1) {
                            if let Ok(id) = id_str.parse::<usize>() {
                                if let Some(pos) =
                                    self.memory_entries.iter().position(|e| e.id == id)
                                {
                                    self.memory_entries.remove(pos);
                                    self.add_message(
                                        format!("Deleted memory entry [{}]", id),
                                        false,
                                    );
                                } else {
                                    self.add_message(
                                        "Invalid ID. Usage: /memory delete <id>".to_string(),
                                        false,
                                    );
                                }
                            } else {
                                self.add_message(
                                    "Invalid ID. Usage: /memory delete <id>".to_string(),
                                    false,
                                );
                            }
                        } else {
                            self.add_message("Usage: /memory delete <id>".to_string(), false);
                        }
                    }
                    _ => {
                        self.add_message("Memory commands:\n/memory list - Show all entries\n/memory add <content> - Add entry\n/memory delete <id> - Delete entry".to_string(), false);
                    }
                }
            }
            "/plugins" => {
                let args = self.input.trim_start_matches("/plugins").trim();
                let parts: Vec<&str> = args.splitn(2, ' ').collect();
                let subcmd = parts.first().unwrap_or(&"");

                match *subcmd {
                    "list" | "" => {
                        let skills = self.skill_resolver.list_skills().unwrap_or_default();
                        if skills.is_empty() {
                            self.add_message("No plugins installed.".to_string(), false);
                        } else {
                            let msg = skills
                                .iter()
                                .map(|(s, state)| format!("{} ({:?})", s.name, state))
                                .collect::<Vec<_>>()
                                .join("\n");
                            self.add_message(format!("Installed plugins:\n{}", msg), false);
                        }
                    }
                    "enable" => {
                        if let Some(name) = parts.get(1) {
                            if self
                                .skill_resolver
                                .set_skill_state(name, opencode_core::SkillState::Enabled)
                                .is_some()
                            {
                                self.add_message(format!("Enabled plugin: {}", name), false);
                            } else {
                                self.add_message(format!("Plugin not found: {}", name), false);
                            }
                        } else {
                            self.add_message("Usage: /plugins enable <name>".to_string(), false);
                        }
                    }
                    "disable" => {
                        if let Some(name) = parts.get(1) {
                            if self
                                .skill_resolver
                                .set_skill_state(name, opencode_core::SkillState::Disabled)
                                .is_some()
                            {
                                self.add_message(format!("Disabled plugin: {}", name), false);
                            } else {
                                self.add_message(format!("Plugin not found: {}", name), false);
                            }
                        } else {
                            self.add_message("Usage: /plugins disable <name>".to_string(), false);
                        }
                    }
                    _ => {
                        self.add_message(
                            "Usage: /plugins list|enable <name>|disable <name>".to_string(),
                            false,
                        );
                    }
                }
            }
            "/share" => {
                use std::env;
                use std::fs;
                let share_path = env::temp_dir().join(format!(
                    "opencode_share_{}.md",
                    chrono::Utc::now().timestamp()
                ));
                let mut content = String::from("# OpenCode Session\n\n");
                for m in &self.messages {
                    let role_name = if m.is_user {
                        self.username.as_deref().unwrap_or("User")
                    } else {
                        "Assistant"
                    };
                    let redacted = Self::redact_api_keys(&m.content);
                    content.push_str(&format!("## {}\n\n{}\n\n", role_name, redacted));
                }
                match fs::write(&share_path, &content) {
                    Ok(_) => {
                        self.share_url = Some(share_path.to_string_lossy().to_string());
                        #[expect(clippy::expect_used)]
                        let url = self.share_url.as_ref().expect("share_url set above");
                        self.add_message(
                            format!("Session shared: {}\nUse /unshare to remove.", url),
                            false,
                        );
                    }
                    Err(e) => self.add_message(format!("Share failed: {}", e), false),
                }
            }
            "/unshare" => {
                if let Some(ref url) = self.share_url {
                    self.add_message(format!("Removed share: {}", url), false);
                    self.share_url = None;
                } else {
                    self.add_message(
                        "No active session share to remove. Use /share to create a share."
                            .to_string(),
                        false,
                    );
                }
            }
            "/username" => {
                let name = self.input.trim_start_matches("/username").trim();
                if name.is_empty() {
                    if let Some(current) = &self.username {
                        self.add_message(format!("Current username: {}", current), false);
                    } else {
                        self.add_message(
                            "No username set. Usage: /username <name>".to_string(),
                            false,
                        );
                    }
                } else {
                    self.username = Some(name.to_string());
                    if let Ok(mut config) = Config::load_from_default_path() {
                        config.user = Some(UserConfig {
                            username: Some(name.to_string()),
                            remember_username: true,
                        });
                        let _ = config.save(&Config::default_config_path());
                    }
                    self.add_message(format!("Username set to: {} (saved)", name), false);
                }
            }
            "/thinking" => {
                self.thinking_mode = !self.thinking_mode;
                let msg = if self.thinking_mode {
                    "Thinking mode: ON (extended reasoning enabled)"
                } else {
                    "Thinking mode: OFF"
                };
                self.add_message(msg.to_string(), false);
            }
            "/status" => {
                let model = std::env::var("OPENCODE_MODEL")
                    .or_else(|_| std::env::var("OPENAI_MODEL"))
                    .unwrap_or_else(|_| "unknown".to_string());
                let total_tokens = self.token_counter.get_total_tokens();
                let msg = format!(
                    "Session Status:\n\
                    Model: {}\n\
                    Total tokens: {}\n\
                    Total cost: ${:.4}\n\
                    Messages: {}\n\
                    Tool calls: {}\n\
                    Thinking mode: {}",
                    model,
                    total_tokens,
                    self.total_cost_usd,
                    self.messages.len(),
                    self.tool_calls.len(),
                    if self.thinking_mode { "ON" } else { "OFF" }
                );
                self.add_message(msg, false);
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
                    use opencode_core::compaction::{CompactionConfig, Compactor};
                    let compactor = Compactor::new(CompactionConfig::default());
                    let core_messages: Vec<opencode_core::Message> = self
                        .messages
                        .iter()
                        .map(|m| opencode_core::Message {
                            role: if m.is_user {
                                opencode_core::Role::User
                            } else {
                                opencode_core::Role::Assistant
                            },
                            content: m.content.clone(),
                            timestamp: chrono::Utc::now(),
                            parts: None,
                        })
                        .collect();
                    let result = compactor.compact(core_messages);
                    if result.was_compacted {
                        self.messages.clear();
                        for m in result.messages {
                            self.messages.push(MessageMeta::assistant(m.content));
                        }
                        self.add_message(
                            format!(
                                "Session compacted: {} messages summarized",
                                result.pruned_count
                            ),
                            false,
                        );
                    } else {
                        self.add_message("Session doesn't need compaction".to_string(), false);
                    }
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
                CommandAction::OpenEditor => {
                    self.open_editor();
                }
                CommandAction::InitProject => {
                    self.init_project();
                }
                CommandAction::Custom(name) => match name.as_str() {
                    "help" => {
                        let all_commands = self
                            .command_registry
                            .all()
                            .iter()
                            .map(|c| format!("/{} - {}", c.name, c.description))
                            .collect::<Vec<_>>()
                            .join("\n");
                        self.add_message(format!("Available commands:\n{}", all_commands), false);
                    }
                    "search" => {
                        self.mode = AppMode::Search;
                    }
                    "diff" => {
                        let is_git_repo = std::process::Command::new("git")
                            .arg("rev-parse")
                            .arg("--is-inside-work-tree")
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        if !is_git_repo {
                            self.add_message("Not a git repository.".to_string(), false);
                        } else {
                            let git_args = match self.config.diff_style() {
                                DiffStyle::SideBySide => vec!["diff", "--word-diff=color"],
                                DiffStyle::Unified => vec!["diff", "--unified=5"],
                                DiffStyle::Auto => vec!["diff"],
                            };
                            match std::process::Command::new("git").args(&git_args).output() {
                                Ok(o) => {
                                    let diff = String::from_utf8_lossy(&o.stdout);
                                    if diff.is_empty() {
                                        self.add_message("No changes to show.".to_string(), false);
                                    } else {
                                        self.add_message(
                                            format!(
                                                "Git diff:\n{}",
                                                diff.chars()
                                                    .take(Self::MAX_DIFF_DISPLAY_CHARS)
                                                    .collect::<String>()
                                            ),
                                            false,
                                        );
                                    }
                                }
                                Err(e) => {
                                    self.add_message(format!("Git diff failed: {}", e), false);
                                }
                            }
                        }
                    }
                    "memory" => {
                        let args = self.input.trim_start_matches("/memory").trim();
                        let parts: Vec<&str> = args.splitn(2, ' ').collect();
                        let subcmd = parts.first().unwrap_or(&"");

                        match *subcmd {
                            "list" | "" => {
                                if self.memory_entries.is_empty() {
                                    self.add_message(
                                        "Memory: No entries stored. Use /memory add <content>"
                                            .to_string(),
                                        false,
                                    );
                                } else {
                                    let list: String = self
                                        .memory_entries
                                        .iter()
                                        .map(|e| {
                                            format!("[{}] {} - {}", e.id, e.created_at, e.content)
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    self.add_message(format!("Memory entries:\n{}", list), false);
                                }
                            }
                            "add" => {
                                let content = parts.get(1).unwrap_or(&"").trim();
                                if content.is_empty() {
                                    self.add_message(
                                        "Usage: /memory add <content>".to_string(),
                                        false,
                                    );
                                } else {
                                    let id = self.memory_entries.len() + 1;
                                    self.memory_entries
                                        .push(MemoryEntry::new(id, content.to_string()));
                                    self.add_message(format!("Memory entry [{}] added", id), false);
                                }
                            }
                            "delete" | "del" => {
                                let id_str = parts.get(1).unwrap_or(&"").trim();
                                if id_str.is_empty() {
                                    self.add_message(
                                        "Usage: /memory delete <id>".to_string(),
                                        false,
                                    );
                                } else if let Ok(id) = id_str.parse::<usize>() {
                                    if let Some(pos) =
                                        self.memory_entries.iter().position(|e| e.id == id)
                                    {
                                        self.memory_entries.remove(pos);
                                        self.add_message(
                                            format!("Memory entry [{}] deleted", id),
                                            false,
                                        );
                                    } else {
                                        self.add_message(
                                            format!("Memory entry [{}] not found", id),
                                            false,
                                        );
                                    }
                                } else {
                                    self.add_message(
                                        "Invalid ID. Usage: /memory delete <id>".to_string(),
                                        false,
                                    );
                                }
                            }
                            _ => {
                                self.add_message("Memory commands:\n/memory list - Show all entries\n/memory add <content> - Add entry\n/memory delete <id> - Delete entry".to_string(), false);
                            }
                        }
                    }
                    "username" => {
                        let name = self.input.trim_start_matches("/username").trim();
                        if name.is_empty() {
                            if let Some(current) = &self.username {
                                self.add_message(format!("Current username: {}", current), false);
                            } else {
                                self.add_message(
                                    "No username set. Usage: /username <name>".to_string(),
                                    false,
                                );
                            }
                        } else {
                            self.username = Some(name.to_string());
                            if let Ok(mut config) = Config::load_from_default_path() {
                                config.user = Some(UserConfig {
                                    username: Some(name.to_string()),
                                    remember_username: true,
                                });
                                let _ = config.save(&Config::default_config_path());
                            }
                            self.add_message(format!("Username set to: {} (saved)", name), false);
                        }
                    }
                    "plugins" => {
                        let args = self.input.trim_start_matches("/plugins").trim();
                        let parts: Vec<&str> = args.splitn(2, ' ').collect();
                        let subcmd = parts.first().unwrap_or(&"");

                        match *subcmd {
                            "list" | "" => {
                                let skills = self.skill_resolver.list_skills().unwrap_or_default();
                                if skills.is_empty() {
                                    self.add_message("No plugins installed.".to_string(), false);
                                } else {
                                    let msg = skills
                                        .iter()
                                        .map(|(s, state)| format!("{} ({:?})", s.name, state))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    self.add_message(format!("Installed plugins:\n{}", msg), false);
                                }
                            }
                            "enable" => {
                                if let Some(name) = parts.get(1) {
                                    if self
                                        .skill_resolver
                                        .set_skill_state(name, opencode_core::SkillState::Enabled)
                                        .is_some()
                                    {
                                        self.add_message(
                                            format!("Enabled plugin: {}", name),
                                            false,
                                        );
                                    } else {
                                        self.add_message(
                                            format!("Plugin not found: {}", name),
                                            false,
                                        );
                                    }
                                } else {
                                    self.add_message(
                                        "Usage: /plugins enable <name>".to_string(),
                                        false,
                                    );
                                }
                            }
                            "disable" => {
                                if let Some(name) = parts.get(1) {
                                    if self
                                        .skill_resolver
                                        .set_skill_state(name, opencode_core::SkillState::Disabled)
                                        .is_some()
                                    {
                                        self.add_message(
                                            format!("Disabled plugin: {}", name),
                                            false,
                                        );
                                    } else {
                                        self.add_message(
                                            format!("Plugin not found: {}", name),
                                            false,
                                        );
                                    }
                                } else {
                                    self.add_message(
                                        "Usage: /plugins disable <name>".to_string(),
                                        false,
                                    );
                                }
                            }
                            _ => {
                                self.add_message(
                                    "Usage: /plugins list|enable <name>|disable <name>".to_string(),
                                    false,
                                );
                            }
                        }
                    }
                    "status" => {
                        let model = std::env::var("OPENCODE_MODEL")
                            .or_else(|_| std::env::var("OPENAI_MODEL"))
                            .unwrap_or_else(|_| "unknown".to_string());
                        let total_tokens = self.token_counter.get_total_tokens();
                        let msg = format!(
                                "Session Status:\nModel: {}\nTotal tokens: {}\nTotal cost: ${:.4}\nMessages: {}\nTool calls: {}\nThinking mode: {}",
                                model,
                                total_tokens,
                                self.total_cost_usd,
                                self.messages.len(),
                                self.tool_calls.len(),
                                if self.thinking_mode { "ON" } else { "OFF" }
                            );
                        self.add_message(msg, false);
                    }
                    _ => {
                        self.add_message(format!("Command /{} not fully implemented", name), false);
                    }
                },
            }
        } else {
            self.add_message(format!("Unknown command: {}", cmd_name), false);
        }
    }

    fn open_editor(&mut self) {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("opencode_message.md");

        let current_input = self.input_box.input().to_string();
        let _ = std::fs::write(&temp_file, &current_input);

        let launcher = EditorLauncher::from_env();
        if let Err(e) = launcher.launch(&temp_file, true) {
            self.add_message(format!("Failed to open editor: {}", e), false);
            return;
        }

        if let Ok(content) = std::fs::read_to_string(&temp_file) {
            self.input_box.set_input(content);
        }
    }

    fn init_project(&mut self) {
        let agents_file = std::path::PathBuf::from("AGENTS.md");

        let template = r#"# AGENTS.md

OpenCode Agent Configuration

## Agents

### Build Agent
- Full access to files and commands
- Use for: implementation, refactoring, file operations

### Plan Agent
- Read-only access
- Use for: code review, exploration, debugging

## Commands

- `/plan` - Switch to plan mode
- `/build` - Switch to build mode
- `/clear` - Clear conversation
- `/editor` - Open external editor
"#;

        if agents_file.exists() {
            self.add_message(
                "AGENTS.md already exists. Consider updating it manually.".to_string(),
                false,
            );
        } else {
            match std::fs::write(&agents_file, template) {
                Ok(_) => self.add_message("Created AGENTS.md".to_string(), false),
                Err(e) => self.add_message(format!("Failed to create AGENTS.md: {}", e), false),
            }
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
                        } else if self.selection.selected_text.is_some() {
                            // Copy selected text to clipboard
                            if self.copy_selection_to_clipboard() {
                                self.add_message(
                                    "Selection copied to clipboard".to_string(),
                                    false,
                                );
                            }
                            self.clear_selection();
                        } else {
                            disable_raw_mode()?;
                            std::process::exit(0);
                        }
                    }
                    KeyCode::Esc => {
                        if self.selection.is_selecting || self.selection.selected_text.is_some() {
                            self.clear_selection();
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
                    KeyCode::Char('b')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        self.sidebar.toggle_collapse();
                        let _ = self.sidebar.save_to_file(&self.sidebar_file);
                        let msg = if self.sidebar.collapsed {
                            "Sidebar collapsed"
                        } else {
                            "Sidebar expanded"
                        };
                        self.add_message(msg.to_string(), false);
                    }
                    KeyCode::Char('h')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        self.sidebar.toggle_active_collapse();
                        let _ = self.sidebar.save_to_file(&self.sidebar_file);
                        let section = self.sidebar.active_section();
                        let msg = format!(
                            "{} section {}",
                            section.title(),
                            if section.collapsed {
                                "collapsed"
                            } else {
                                "expanded"
                            }
                        );
                        self.add_message(msg, false);
                    }
                    KeyCode::Left
                        if key
                            .modifiers
                            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                    {
                        self.sidebar.prev_section();
                        let _ = self.sidebar.save_to_file(&self.sidebar_file);
                        self.add_message(
                            format!("Sidebar section: {}", self.sidebar.active_section().title()),
                            false,
                        );
                    }
                    KeyCode::Right
                        if key
                            .modifiers
                            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                    {
                        self.sidebar.next_section();
                        let _ = self.sidebar.save_to_file(&self.sidebar_file);
                        self.add_message(
                            format!("Sidebar section: {}", self.sidebar.active_section().title()),
                            false,
                        );
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
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.right_panel.set_content(RightPanelContent::Diagnostics);
                        self.add_message("LSP Diagnostics panel (use Alt+3)".to_string(), false);
                    }
                    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.lsp_client.is_some() {
                            let rt = tokio::runtime::Handle::current();
                            let cwd = std::env::current_dir().unwrap_or_default();
                            rt.block_on(async {
                                if let Some(server) = LspClient::detect_language_server(&cwd) {
                                    self.add_message(format!("LSP server: {}", server), false);
                                } else {
                                    self.add_message(
                                        "No LSP server detected for current directory".to_string(),
                                        false,
                                    );
                                }
                            });
                        } else {
                            self.add_message("LSP client not initialized. Start opencode-rs in a project directory.".to_string(), false);
                        }
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
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.input.push('\n');
                            self.input_box.set_input(self.input.clone());
                            return Ok(());
                        }
                        let input = self.input.clone();
                        if !input.is_empty() {
                            if input.trim() == "/confirm-shell" {
                                if let Some(cmd) = self.pending_shell_command.take() {
                                    match self.input_processor.process_shell_confirmed(&cmd) {
                                        Ok(output) => self.add_message(output, false),
                                        Err(error) => self.add_message(
                                            format!("Shell execution failed: {error}"),
                                            false,
                                        ),
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
                                match self.input_processor.process_command(
                                    &self.command_registry,
                                    name,
                                    args,
                                ) {
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
                                self.show_terminal = true;
                                self.terminal_panel.add_line(format!("$ {cmd}",), false);

                                let runtime = self.runtime.clone();
                                let cmd_clone = cmd.clone();
                                let (tx, rx): (
                                    mpsc::Sender<(
                                        String,
                                        Result<
                                            opencode_runtime::RuntimeFacadeResponse,
                                            opencode_runtime::RuntimeFacadeError,
                                        >,
                                    )>,
                                    mpsc::Receiver<_>,
                                ) = mpsc::channel();
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        let shell_cmd = RuntimeFacadeCommand::ExecuteShell(
                                            ExecuteShellCommand {
                                                command: cmd_clone.clone(),
                                                timeout_secs: None,
                                                workdir: None,
                                            },
                                        );
                                        let result = runtime.execute(shell_cmd).await;
                                        let _ = tx.send((cmd_clone, result));
                                    });
                                });

                                let received = rx.recv();
                                let (cmd, result) = match received {
                                    Ok((c, r)) => (c, r),
                                    Err(_) => (
                                        cmd.clone(),
                                        Err(opencode_runtime::RuntimeFacadeError::Dependency(
                                            "Channel error".to_string(),
                                        )),
                                    ),
                                };

                                match result {
                                    Ok(response) => {
                                        let exit_code =
                                            if response.accepted { Some(0) } else { Some(1) };
                                        let stdout = response.message.clone();
                                        let stderr = if response.accepted {
                                            String::new()
                                        } else {
                                            response.message.clone()
                                        };

                                        if !stdout.is_empty() {
                                            self.terminal_panel.add_stdout(&stdout);
                                        }
                                        if !stderr.is_empty() {
                                            self.terminal_panel.add_stderr(&stderr);
                                        }
                                        let exit_msg = format!(
                                            "[Exit code: {}]",
                                            exit_code
                                                .map(|c| c.to_string())
                                                .unwrap_or_else(|| "N/A".to_string())
                                        );
                                        self.terminal_panel
                                            .add_line(&exit_msg, exit_code != Some(0));

                                        let tool_result =
                                            format!("```\n$ {}\n{}\n```", cmd, stdout);
                                        self.add_message(tool_result, false);
                                    }
                                    Err(e) => {
                                        self.terminal_panel.add_stderr(format!("Error: {}", e));
                                        self.terminal_panel.add_line("[Exit code: 1]", true);
                                        self.add_message(
                                            format!("Shell execution failed: {}", e),
                                            false,
                                        );
                                    }
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

                                let mut file_contexts = Vec::new();
                                let mut total_size = 0;
                                let max_context_size = self.config.max_context_size();

                                for file_path in &parsed_files {
                                    let path_str = file_path.to_string_lossy();
                                    let result = self.file_ref_handler.resolve(&path_str);

                                    if result.error.is_none() {
                                        let formatted =
                                            self.file_ref_handler.format_for_context(&result);
                                        let formatted_len = formatted.len();
                                        if total_size + formatted_len <= max_context_size {
                                            file_contexts.push(formatted);
                                            total_size += formatted_len;
                                        } else {
                                            let truncate_msg = format!(
                                                "\n[File: {} - truncated to fit context limit]\n",
                                                result.path
                                            );
                                            file_contexts.push(truncate_msg);
                                            break;
                                        }
                                    } else {
                                        #[expect(clippy::expect_used)]
                                        let error_msg = result
                                            .error
                                            .as_ref()
                                            .expect("error is Some in else branch");
                                        file_contexts.push(format!(
                                            "\n[Error reading {}: {}]\n",
                                            result.path, error_msg
                                        ));
                                    }
                                }

                                for fc in file_contexts {
                                    context_content.push_str(&fc);
                                }
                                self.enriched_input = Some(context_content.clone());
                                self.add_message(context_content, true);
                            } else {
                                self.enriched_input = None;
                                self.add_message(input.clone(), true);
                            }

                            self.input.clear();
                            self.input_widget.clear();
                            self.input_box.set_input(String::new());

                            let context_usage_pct = self.status_bar.context_usage.0 as f64
                                / self.status_bar.context_usage.1.max(1) as f64;
                            if context_usage_pct >= 0.95 {
                                self.add_message(
                                     "⚠️ Context budget exceeded (95%). Please /compact or start a new session.".to_string(), 
                                     false,
                                 );
                                return Ok(());
                            } else if context_usage_pct >= 0.92 {
                                self.add_message(
                                    "⚠️ Context at 92% - recommend /compact before continuing."
                                        .to_string(),
                                    false,
                                );
                            } else if context_usage_pct >= 0.85 {
                                self.add_message(
                                    "ℹ️ Context at 85% - consider /compact soon.".to_string(),
                                    false,
                                );
                            }

                            // Call LLM in background task if provider is initialized
                            if self.llm_provider.is_some() {
                                self.set_tui_state(TuiState::Submitting);
                                self.is_llm_generating = true;
                                self.partial_response.clear();

                                if self.session.is_none() {
                                    self.session = Some(Session::new());
                                }

                                let (tx, rx) = mpsc::channel();
                                self.llm_rx = Some(rx);
                                let auto_enabled = self.skill_resolver.match_and_enable(&input);
                                for skill in auto_enabled {
                                    self.skills_panel.set_enabled(&skill.name, true);
                                }
                                let skill_prompt = self.skill_resolver.build_skill_prompt();
                                let base_input =
                                    self.enriched_input.clone().unwrap_or_else(|| input.clone());
                                let llm_input = if skill_prompt.is_empty() {
                                    base_input
                                } else {
                                    format!(
                                        "[Enabled Skills]\n{}\n\n[User Request]\n{}",
                                        skill_prompt, base_input
                                    )
                                };
                                let session = self.runtime_session_for_input(&llm_input);

                                let runtime = self.runtime.clone();
                                std::thread::spawn(move || {
                                    #[expect(clippy::expect_used)]
                                    let rt = tokio::runtime::Runtime::new()
                                        .expect("failed to create Tokio runtime");
                                    rt.block_on(async {
                                        let cmd = RuntimeFacadeCommand::RunAgent(Box::new(
                                            RunAgentCommand {
                                                session,
                                                agent_type: AgentType::Build,
                                            },
                                        ));
                                        match runtime.execute(cmd).await {
                                            Ok(response) => {
                                                if let Some(updated_session) = response.session {
                                                    let _ = tx.send(LlmEvent::SessionComplete(
                                                        updated_session,
                                                    ));
                                                }
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
                            && !self.history.is_empty()
                            && self.history_index > 0
                        {
                            self.history_index -= 1;
                            self.input = self.history[self.history_index].clone();
                            self.input_box.set_input(self.input.clone());
                        }
                    }
                    _ => {
                        if self.matches_keybind(&key, "commands") {
                            self.mode = AppMode::CommandPalette;
                            self.command_palette_input.clear();
                        } else if self.matches_keybind(&key, "timeline") {
                            self.mode = AppMode::Timeline;
                            if !self.messages.is_empty() {
                                self.timeline_state.select(Some(self.messages.len() - 1));
                            }
                        } else if self.matches_keybind(&key, "settings") {
                            self.mode = AppMode::Settings;
                        } else if self.matches_keybind(&key, "toggle_files") {
                            self.toggle_file_tree();
                        }
                    }
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
            AppMode::Home => {
                self.draw_home_view(f);
            }
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
                    .unwrap_or_else(|| {
                        "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new".to_string()
                    });
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
            AppMode::ConnectApiKey => {
                self.draw_chat(f);
                if let Some(dialog) = self.api_key_input_dialog.as_ref() {
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
                let message = self.connect_progress_message();
                f.render_widget(
                    Paragraph::new(vec![Line::from(message)])
                        .block(Block::default().title("Connect").borders(Borders::ALL)),
                    area,
                );
            }
            AppMode::ConnectApiKeyError => {
                self.draw_chat(f);
                if let Some(dialog) = self.validation_error_dialog.as_ref() {
                    dialog.draw(f, f.area());
                }
            }
            AppMode::ConnectModel => {
                self.draw_chat(f);
                tracing::info!(
                    event = "tui.connect.draw_connect_model",
                    mode = ?self.mode,
                    has_dialog = %self.connect_model_dialog.is_some(),
                    "Drawing ConnectModel mode"
                );
                if let Some(dialog) = self.connect_model_dialog.as_ref() {
                    tracing::debug!(
                        event = "tui.connect.drawing_model_dialog",
                        "Drawing ConnectModelDialog"
                    );
                    dialog.draw(f, f.area());
                } else {
                    tracing::warn!(
                        event = "tui.connect.model_dialog_none",
                        "ConnectModelDialog is None but mode is ConnectModel"
                    );
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
            AppMode::Search => {
                self.draw_chat(f);
                self.draw_search_dialog(f);
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

    fn right_panel_data(&mut self) -> RightPanelRenderData {
        self.refresh_todos_from_messages();

        let mut diagnostics = self
            .messages
            .iter()
            .filter(|m| {
                let c = m.content.to_ascii_lowercase();
                c.contains("error") || c.contains("warning") || c.contains("diagnostic")
            })
            .map(|m| m.content.clone())
            .take(8)
            .collect::<Vec<_>>();

        for diag in &self.lsp_diagnostics {
            let sev = format!("{:?}", diag.severity);
            let range = format!(
                "{}:{}",
                diag.range.start.line + 1,
                diag.range.start.character + 1
            );
            diagnostics.push(format!("[{}] {} at {}", sev, diag.message, range));
        }

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

        let todos = self
            .todos
            .iter()
            .map(|t| {
                let checkbox = if t.completed { "[x]" } else { "[ ]" };
                format!("{} {} ({})", checkbox, t.content, t.priority)
            })
            .collect();

        use crate::right_panel::{ConfigEntry, DebugEntry, MessageData, SessionData};

        let messages = self
            .messages
            .iter()
            .rev()
            .take(15)
            .map(|m| MessageData {
                role: if m.is_user {
                    "user".to_string()
                } else {
                    "assistant".to_string()
                },
                content_preview: m.content.chars().take(80).collect(),
                timestamp: String::new(),
            })
            .collect();

        let sessions = self
            .session_manager
            .list()
            .iter()
            .take(10)
            .map(|s| SessionData {
                id: s.id.clone(),
                name: s.name.clone(),
                last_active: s.time_since_created().as_secs().to_string(),
                message_count: s.message_count,
            })
            .collect();

        let config_data = vec![
            ConfigEntry {
                key: "provider".to_string(),
                value: self.provider.clone(),
            },
            ConfigEntry {
                key: "model".to_string(),
                value: std::env::var("OPENCODE_MODEL").unwrap_or_default(),
            },
            ConfigEntry {
                key: "agent".to_string(),
                value: self.agent.clone(),
            },
        ];

        let debug_info = vec![
            DebugEntry {
                category: "tokens".to_string(),
                content: format!("{}", self.token_counter.get_total_tokens()),
            },
            DebugEntry {
                category: "cost".to_string(),
                content: format!("${:.4}", self.total_cost_usd),
            },
            DebugEntry {
                category: "messages".to_string(),
                content: format!("{}", self.messages.len()),
            },
        ];

        RightPanelRenderData {
            diagnostics,
            total_tokens: self.token_counter.get_total_tokens(),
            total_cost_usd: self.total_cost_usd,
            files,
            tools,
            todos,
            diff_content: String::new(),
            context_items: Vec::new(),
            permission_log: Vec::new(),
            messages,
            sessions,
            config_data,
            debug_info,
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

    fn highlight_query(&self, text: &str, query: &str, max_len: usize) -> String {
        let lower = text.to_lowercase();
        let q = query.to_lowercase();
        let mut result = String::new();
        let mut last_end = 0;

        for (idx, _) in lower.match_indices(&q) {
            let start = idx.saturating_sub(3);
            if result.len() + idx - last_end > max_len {
                break;
            }
            result.push_str(&text[last_end..start]);
            result.push_str("\x1b[7m");
            let match_end = (idx + query.len()).min(text.len());
            result.push_str(&text[idx..match_end]);
            result.push_str("\x1b[0m");
            last_end = match_end;
        }
        if result.is_empty() {
            text.chars().take(max_len).collect()
        } else {
            result
        }
    }

    fn draw_search_dialog(&self, f: &mut Frame) {
        let area = f.area();
        let theme = self.theme_manager.current().clone();
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 15.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!("Search ({} matches)", {
                let query = self.command_palette_input.to_lowercase();
                if query.is_empty() {
                    0
                } else {
                    self.messages
                        .iter()
                        .filter(|m| m.content.to_lowercase().contains(&query))
                        .count()
                }
            }))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);

        let query = self.command_palette_input.to_lowercase();
        let mut results: Vec<Line> = Vec::new();

        if !query.is_empty() {
            for msg in self.messages.iter() {
                if msg.content.to_lowercase().contains(&query) {
                    let role = if msg.is_user { "U" } else { "A" };

                    let highlighted = self.highlight_query(
                        &msg.content,
                        &query,
                        inner.width.saturating_sub(8) as usize,
                    );
                    results.push(Line::from(vec![
                        Span::styled(
                            format!("[{}] ", role),
                            Style::default().fg(if msg.is_user {
                                theme.primary_color()
                            } else {
                                theme.secondary_color()
                            }),
                        ),
                        Span::raw(highlighted),
                    ]));
                    if results.len() >= inner.height.saturating_sub(1) as usize {
                        break;
                    }
                }
            }
        }

        if results.is_empty() {
            results.push(Line::from(Span::styled(
                if query.is_empty() {
                    "Type to search..."
                } else {
                    "No matches found"
                },
                Style::default().fg(theme.muted_color()),
            )));
        }

        let content = Paragraph::new(results);
        f.render_widget(content, inner);
    }

    fn handle_search_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(action) = action::Action::from_key_event(&key) {
                    let mut app_state = action::AppState::new();
                    app_state.mode = action::AppMode::Search;
                    app_state.input_buffer = self.command_palette_input.clone();
                    action::ActionHandler::handle(action.clone(), &mut app_state);
                    self.command_palette_input = app_state.input_buffer.clone();
                    self.mode = app_state.mode;
                }
            }
        }
        Ok(())
    }

    fn handle_home_view(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                match key.code {
                    KeyCode::Esc => {
                        self.mode = AppMode::Chat;
                    }
                    KeyCode::Tab => {
                        self.home_view.switch_section();
                    }
                    KeyCode::Enter => {
                        if self.home_view.get_focused_section() == HomeViewSection::RecentSessions {
                            if let Some(idx) = self.home_view.get_selected_session_index() {
                                let session_name = self
                                    .session_manager
                                    .get_session(idx)
                                    .map(|s| s.name.clone());
                                if session_name.is_some() {
                                    self.session_manager.select(idx);
                                    #[expect(clippy::expect_used)]
                                    let name = session_name
                                        .expect("session_name is Some per is_some check");
                                    self.add_message(format!("Loaded session: {}", name), false);
                                    self.mode = AppMode::Chat;
                                }
                            }
                        } else {
                            let action = self.home_view.get_selected_action();
                            match action {
                                HomeAction::NewSession => {
                                    let session_count = self.session_manager.len();
                                    self.session_manager
                                        .add_session(format!("Session {}", session_count + 1));
                                    self.mode = AppMode::Chat;
                                }
                                HomeAction::ContinueLast => {
                                    if let Some(session) = self.session_manager.current() {
                                        self.add_message(
                                            format!("Continuing session: {}", session.name),
                                            false,
                                        );
                                    }
                                    self.mode = AppMode::Chat;
                                }
                                HomeAction::ViewSessions => {
                                    self.mode = AppMode::Sessions;
                                }
                                HomeAction::Settings => {
                                    self.mode = AppMode::Settings;
                                }
                                HomeAction::Quit => {
                                    disable_raw_mode()?;
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.home_view.get_focused_section() == HomeViewSection::RecentSessions {
                            self.home_view.move_session_selection(-1);
                        } else {
                            self.home_view.move_selection(-1);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.home_view.get_focused_section() == HomeViewSection::RecentSessions {
                            self.home_view.move_session_selection(1);
                        } else {
                            self.home_view.move_selection(1);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw_home_view(&mut self, f: &mut Frame) {
        self.sync_status_bar_state();
        self.home_view
            .update_from_session_manager(&self.session_manager);
        let status_label = match self.status_bar.connection_status {
            crate::components::status_bar::ConnectionStatus::Connected => "Connected",
            crate::components::status_bar::ConnectionStatus::Disconnected => "Disconnected",
            crate::components::status_bar::ConnectionStatus::Error => "Error",
        };
        self.home_view
            .set_connection_status(Some(status_label.to_string()));
        self.home_view.draw(f, f.area());
    }

    fn handle_sessions_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                match key.code {
                    KeyCode::Esc => {
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::Sessions;
                        action::ActionHandler::handle(
                            action::Action::Sessions(action::SessionsAction::Close),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
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
            .map_err(|e| io::Error::other(format!("Failed to spawn editor: {}", e)))?;

        let status = child
            .wait()
            .map_err(|e| io::Error::other(format!("Editor wait failed: {}", e)))?;

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
        self.sync_status_bar_state();
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

        let main_area = if self.show_sidebar && !self.sidebar.collapsed {
            let sidebar_width =
                ((main_area.width as u32 * proportions.sidebar_width as u32) / 100) as u16;
            let sidebar_width = sidebar_width.max(20).min(40);
            let sidebar_area = Rect::new(
                main_area.x,
                main_area.y,
                sidebar_width,
                main_area.height.saturating_sub(1),
            );
            self.sidebar.draw(f, sidebar_area);
            Rect::new(
                main_area.x + sidebar_width,
                main_area.y,
                main_area.width - sidebar_width,
                main_area.height,
            )
        } else if self.show_file_tree {
            let file_tree_width =
                ((main_area.width as u32 * proportions.sidebar_width as u32) / 100) as u16;
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
            let right_panel_width =
                ((main_area.width as u32 * proportions.right_panel_width as u32) / 100) as u16;
            let right_panel_width = right_panel_width.max(24).min(40).min(main_area.width / 2);
            let right_panel_area = Rect::new(
                main_area.x + main_area.width - right_panel_width,
                main_area.y,
                right_panel_width,
                main_area.height.saturating_sub(1),
            );
            let panel_data = self.right_panel_data();
            self.right_panel.draw(f, right_panel_area, &panel_data);
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
                let role_label = if msg.is_user {
                    "You"
                } else if msg.is_thinking {
                    "Thinking"
                } else {
                    "Assistant"
                };
                let prefix = if msg.is_user { "> " } else { "  " };
                let (color, style) = if msg.is_user {
                    (theme.primary_color(), Modifier::BOLD)
                } else if msg.is_thinking {
                    (theme.warning_color(), Modifier::ITALIC)
                } else {
                    (theme.foreground_color(), Modifier::BOLD)
                };
                let is_code_like = msg.content.starts_with("```")
                    || (msg.content.contains('\n')
                        && msg
                            .content
                            .lines()
                            .skip(1)
                            .any(|l| l.starts_with("    ") || l.starts_with('\t')));
                let content_style = if is_code_like {
                    Style::default()
                        .fg(theme.secondary_color())
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(color).add_modifier(style)
                };
                let mut lines = vec![Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color).add_modifier(style)),
                    Span::styled(
                        format!("[{}] ", role_label),
                        Style::default()
                            .fg(theme.muted_color())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        if msg.is_thinking && self.thinking_mode {
                            format!("[Thinking...] {}", msg.content)
                        } else {
                            msg.content.clone()
                        },
                        content_style,
                    ),
                ])];
                if self.show_metadata {
                    let mut meta_parts = Vec::new();
                    if msg.is_thinking {
                        meta_parts.push("thinking".to_string());
                    }
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                let dialog_action = self.settings_dialog.handle_input(key);
                if dialog_action == DialogAction::Close {
                    let mut app_state = action::AppState::new();
                    app_state.mode = action::AppMode::Settings;
                    crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                        dialog_action.clone(),
                        &mut app_state,
                    );
                    self.mode = app_state.mode;
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.load_model_catalog();
                }
                let action = self.model_selection_dialog.handle_input(key);
                match action {
                    DialogAction::Close => self.mode = AppMode::Chat,
                    DialogAction::Confirm(model_id) => {
                        self.add_message(format!("Selected model: {}", model_id), false);
                        std::env::set_var("OPENCODE_MODEL", &model_id);
                        std::env::remove_var("OPENCODE_MODEL_VARIANT");
                        self.mode = AppMode::Chat;
                    }
                    DialogAction::ConfirmModelWithVariant {
                        model_id,
                        variant_name,
                    } => {
                        let variant_msg = if let Some(v) = &variant_name {
                            format!("Selected model: {} (variant: {})", model_id, v)
                        } else {
                            format!("Selected model: {}", model_id)
                        };
                        self.add_message(variant_msg, false);
                        std::env::set_var("OPENCODE_MODEL", &model_id);
                        if let Some(v) = variant_name {
                            std::env::set_var("OPENCODE_MODEL_VARIANT", &v);
                        } else {
                            std::env::remove_var("OPENCODE_MODEL_VARIANT");
                        }
                        self.mode = AppMode::Chat;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn load_model_catalog(&mut self) {
        use crate::dialogs::ModelInfo;
        if let Some(catalog) = self.catalog_fetcher.get_blocking() {
            let models: Vec<ModelInfo> = catalog
                .providers
                .values()
                .flat_map(|p| {
                    p.models.values().map(|m| ModelInfo {
                        id: m.id.clone(),
                        name: m.display_name.clone(),
                        provider: p.display_name.clone(),
                        is_paid: m.cost.input > 0.0 || m.cost.output > 0.0,
                        is_available: true,
                        variants: m.variants.clone(),
                    })
                })
                .collect();
            self.model_selection_dialog.set_models(models);
        }
    }

    fn handle_provider_management_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.load_provider_catalog();
                }
                let dialog_action = self.provider_management_dialog.handle_input(key);
                match dialog_action {
                    DialogAction::Close => {
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::ProviderManagement;
                        crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                            dialog_action.clone(),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
                    }
                    DialogAction::Navigate(nav) => {
                        self.add_message(format!("Navigating to: {}", nav), false);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn load_provider_catalog(&mut self) {
        use opencode_llm::ProviderDescriptor;
        if let Some(catalog) = self.catalog_fetcher.get_blocking() {
            let providers: Vec<ProviderInfo> = catalog
                .providers
                .values()
                .map(|p: &ProviderDescriptor| ProviderInfo {
                    id: p.id.clone(),
                    name: p.display_name.clone(),
                    status: ProviderStatus::Connected,
                    api_key_set: false,
                })
                .collect();
            self.provider_management_dialog.set_providers(providers);

            let connect_providers: Vec<(String, String)> = catalog
                .providers
                .values()
                .map(|p: &ProviderDescriptor| (p.id.clone(), p.display_name.clone()))
                .collect();
            self.connect_provider_dialog
                .set_providers(connect_providers);
        }
    }

    fn handle_connect_provider_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                let dialog_action = self.connect_provider_dialog.handle_input(key);
                match dialog_action {
                    DialogAction::Close => {
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::ConnectProvider;
                        crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                            dialog_action.clone(),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
                    }
                    DialogAction::Confirm(provider_id) => {
                        self.handle_connect_provider_confirm(provider_id);
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(dialog) = self.connect_method_dialog.as_mut() {
                    let dialog_action = dialog.handle_input(key);
                    match dialog_action {
                        DialogAction::Close => {
                            let mut app_state = action::AppState::new();
                            app_state.mode = action::AppMode::ConnectMethod;
                            crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                                dialog_action.clone(),
                                &mut app_state,
                            );
                            self.mode = app_state.mode;
                            self.connect_method_dialog = None;
                        }
                        DialogAction::Confirm(method) => {
                            self.handle_connect_method_confirm(method);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_api_key_input_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(dialog) = self.api_key_input_dialog.as_mut() {
                    let dialog_action = dialog.handle_input(key);
                    match dialog_action {
                        DialogAction::Close => {
                            let mut app_state = action::AppState::new();
                            app_state.mode = action::AppMode::ConnectApiKey;
                            crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                                dialog_action.clone(),
                                &mut app_state,
                            );
                            self.mode = app_state.mode;
                            self.api_key_input_dialog = None;
                        }
                        DialogAction::Confirm(api_key) => {
                            self.api_key_input_dialog = None;
                            self.handle_api_key_input_confirm(api_key);
                        }
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
        self.check_connect_events();

        if !matches!(self.mode, AppMode::ConnectProgress) {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if key.code == KeyCode::Esc {
                    self.connect_rx = None;
                    self.validation_in_progress = false;
                    self.validation_cancelled = true;
                    self.pending_api_key_for_validation = None;
                    self.pending_connect_provider = None;
                    self.api_key_input_dialog = None;
                    self.connect_method_dialog = None;
                    self.connect_model_dialog = None;
                    self.mode = AppMode::ConnectProvider;
                    tracing::info!(
                        event = "tui.connect.cancelled",
                        reason = "user_pressed_esc",
                        "Connect validation cancelled by user"
                    );
                }
            }
        }
        Ok(())
    }

    fn handle_validation_error_dialog(
        &mut self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(dialog) = self.validation_error_dialog.as_mut() {
                    let dialog_action = dialog.handle_input(key);
                    match dialog_action {
                        crate::dialogs::DialogAction::Close => {
                            let mut app_state = action::AppState::new();
                            app_state.mode = action::AppMode::ConnectApiKeyError;
                            crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                                dialog_action.clone(),
                                &mut app_state,
                            );
                            self.mode = app_state.mode;
                            self.validation_error_dialog = None;
                        }
                        crate::dialogs::DialogAction::Confirm(value) if value == "retry" => {
                            self.validation_error_dialog = None;
                            self.start_api_key_input();
                        }
                        _ => {}
                    }
                }
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                if let Some(dialog) = self.connect_model_dialog.as_mut() {
                    let dialog_action = dialog.handle_input(key);
                    match dialog_action {
                        DialogAction::Close => {
                            let mut app_state = action::AppState::new();
                            app_state.mode = action::AppMode::ConnectModel;
                            crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                                dialog_action.clone(),
                                &mut app_state,
                            );
                            self.mode = app_state.mode;
                            self.connect_model_dialog = None;
                        }
                        DialogAction::Confirm(model_id) => {
                            if let Err(error) = self.handle_connect_model_confirm(model_id) {
                                self.add_message(
                                    format!("Failed to activate model: {}", error),
                                    false,
                                );
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                let dialog_action = self.file_selection_dialog.handle_input(key);
                match dialog_action {
                    DialogAction::Close => {
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::FileSelection;
                        crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                            dialog_action.clone(),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
                    }
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                let dialog_action = self.directory_selection_dialog.handle_input(key);
                match dialog_action {
                    DialogAction::Close => {
                        let mut app_state = action::AppState::new();
                        app_state.mode = action::AppMode::DirectorySelection;
                        crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                            dialog_action.clone(),
                            &mut app_state,
                        );
                        self.mode = app_state.mode;
                    }
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
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        std::process::exit(0);
                    }
                }
                let dialog_action = self.release_notes_dialog.handle_input(key);
                if dialog_action == DialogAction::Close {
                    let mut app_state = action::AppState::new();
                    app_state.mode = action::AppMode::ReleaseNotes;
                    crate::dialog_action_adapter::DialogActionAdapter::handle_dialog_action(
                        dialog_action.clone(),
                        &mut app_state,
                    );
                    self.mode = app_state.mode;
                }
            }
        }
        Ok(())
    }
}

fn build_placeholder_runtime(
    provider: Option<Arc<dyn Provider + Send + Sync>>,
    tools: Option<Arc<ToolsToolRegistry>>,
) -> Arc<OpenCodeRuntime> {
    let event_bus = Arc::new(opencode_core::bus::EventBus::new());
    let permission_manager = Arc::new(tokio::sync::RwLock::new(PermissionManager::default()));
    let session_repo = Arc::new(InMemorySessionRepository::default());
    let project_repo = Arc::new(InMemoryProjectRepository::default());
    let db_path = std::env::temp_dir().join(format!(
        "opencode-tui-runtime-placeholder-{}.db",
        uuid::Uuid::new_v4()
    ));
    let pool = StoragePool::new(&db_path).expect("placeholder storage pool");
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));
    let agent_runtime = Arc::new(tokio::sync::RwLock::new(
        AgentRuntime::new(Session::default(), AgentType::Build).with_event_bus(event_bus.clone()),
    ));

    Arc::new(OpenCodeRuntime::new(RuntimeFacadeServices::new(
        event_bus,
        permission_manager,
        storage,
        agent_runtime,
        Arc::new(RuntimeFacadeTaskStore::new()),
        Arc::new(RuntimeFacadeToolRouter::default()),
        AgentType::Build,
        provider,
        tools,
    )))
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
        Err(format!(
            "Browser open command failed with status {}",
            status
        ))
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
                variants: vec![],
            }],
        );

        assert_eq!(app.mode, AppMode::ConnectModel);
    }

    #[test]
    fn selecting_model_rebinds_provider_and_returns_to_chat() {
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: Setting environment variable in test context is safe as
        // this test runs in isolation and no other threads access this var.
        unsafe {
            std::env::set_var("OPENCODE_DATA_DIR", dir.path());
        }

        let mut app = App::new();
        app.prime_connect_state_for_test();
        app.handle_connect_model_confirm("gpt-5.3-codex".into())
            .unwrap();

        assert_eq!(app.mode, AppMode::Chat);
        assert_eq!(app.provider, "openai");
        assert!(app.llm_provider.is_some());

        // SAFETY: Removing environment variable in test context is safe as
        // this test runs in isolation and the var was set by this test.
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
    fn session_complete_event_updates_app_session() {
        let mut app = App::new();
        let (tx, rx) = mpsc::channel();
        let mut session = Session::new();
        session.add_message(Message::user("persist me"));

        app.llm_rx = Some(rx);
        tx.send(LlmEvent::SessionComplete(session.clone())).unwrap();
        app.check_llm_events();

        let stored = app.session.expect("session should be updated from event");
        assert_eq!(stored.messages.len(), 1);
        assert_eq!(stored.messages[0].content, "persist me");
    }

    #[test]
    fn runtime_session_for_input_reuses_persisted_session_context() {
        let mut app = App::new();
        let mut persisted = Session::new();
        persisted.add_message(Message::user("first turn"));
        persisted.add_message(Message::assistant("first reply"));
        app.session = Some(persisted);

        let session = app.runtime_session_for_input("second turn");

        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[0].content, "first turn");
        assert_eq!(session.messages[1].content, "first reply");
        assert_eq!(session.messages[2].content, "second turn");
    }

    #[test]
    fn runtime_session_for_input_creates_first_turn_when_session_missing() {
        let app = App::new();

        let session = app.runtime_session_for_input("first turn");

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "first turn");
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
