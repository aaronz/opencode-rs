pub mod account;
pub mod acp;
pub mod agents_md;
pub mod bus;
pub mod checkpoint;
pub mod cli;
pub mod command;
pub mod compaction;
pub mod config;
pub mod context;
pub mod control_plane;
pub mod crash_recovery;
pub mod directory;
pub mod effect;
pub mod env;
pub mod error;
pub mod executor;
pub mod filesystem;
pub mod flag;
pub mod format;
pub mod formatter;
pub mod global;
pub mod id;
pub mod ide;
pub mod installation;
pub mod instance;
pub mod instructions;
pub mod mcp;
pub mod message;
pub mod part;
pub mod permission;
pub mod plugin;
pub mod processor;
pub mod project;
pub mod prompt;
pub mod pty;
pub mod question;
pub mod revert;
pub mod server;
pub mod session;
pub mod session_state;
pub mod share;
pub mod shell;
pub mod skill;
pub mod skill_integration;
pub mod snapshot;
pub mod status;
pub mod storage;
pub mod summary;
pub mod sync;
pub mod token_counter;
pub mod tool;
pub mod tool_config;
pub mod util;
pub mod watcher;
pub mod worktree;

pub use account::AccountManager;
pub use acp::AcpProtocol;
pub use agents_md::{AgentsMdInfo, AgentsMdScanConfig, AgentsMdScanner};
pub use bus::EventBus;
pub use checkpoint::{
    create_checkpoint, restore_checkpoint, Checkpoint, CheckpointManager, CheckpointMetadata,
};
pub use cli::CliRegistry;
pub use command::{Command, CommandRegistry};
pub use compaction::{
    TokenBudget, COMPACTION_FORCE_THRESHOLD, COMPACTION_START_THRESHOLD, COMPACTION_WARN_THRESHOLD,
};
/// Configuration for OpenCode RS application.
/// Loaded from config.toml, environment variables, and command-line arguments.
pub use config::Config;

/// Context for agent execution, including token budgeting and message management.
/// Use ContextBuilder to construct contexts with token limits.
pub use context::{estimate_tokens, trim_to_budget, Context, ContextBudget, ContextBuilder};
pub use control_plane::ControlPlane;
pub use directory::{
    AgentDefinition, CommandDefinition, DirectoryScanner, ModeDefinition, PluginInfo, ThemeInfo,
};
pub use effect::{Effect, EffectError, EffectResult};
pub use env::EnvManager;
/// Main error type for OpenCode RS. Covers IO, JSON, Network, Config, Session, Tool, LLM, and Provider errors.
pub use error::OpenCodeError;
/// Executes agents with tool access. Use build_default_executor() to create a configured executor.
pub use executor::{build_default_executor, AgentExecutor};
pub use filesystem::AppFileSystem;
pub use flag::FlagManager;
pub use format::FormatUtils;
pub use formatter::{FormatterEngine, FormatterError};
pub use global::GlobalState;
pub use id::IdGenerator;
pub use ide::{Ide, IdeExtension, IdeManager, Position};
pub use installation::InstallationManager;
pub use instance::Instance;
pub use instructions::{InstructionsError, InstructionsLoader};
/// MCP (Model Context Protocol) manager for tool discovery and execution.
pub use mcp::McpManager;
/// Represents a single message in a conversation with a role (user/assistant/system).
/// Messages are the core unit of conversation in OpenCode RS.
pub use message::{Message, Role};
pub use part::Part;
pub use permission::PermissionManager;
pub use plugin::PluginManager;
pub use project::{
    normalize_path, validate_workspace, ProjectManager, WorkspaceValidationError,
    WorkspaceValidationResult,
};
pub use prompt::PromptManager;
pub use pty::PtySession;
pub use question::QuestionManager;
pub use revert::RevertManager;
pub use server::Server;
/// A conversation session containing messages, metadata, and tool invocation history.
/// Sessions can be saved to disk and restored for continued conversations.
pub use session::{Session, SessionInfo, SessionSummaryMetadata, ShareError, ToolInvocationRecord};
pub use session_state::{is_valid_transition, SessionState, StateTransitionError};
pub use share::ShareManager;
pub use shell::Shell;
/// Skill management for extending OpenCode with custom capabilities.
/// SkillManager handles skill discovery, matching, and lifecycle.
pub use skill::{MatchType, Skill, SkillManager, SkillMatch};
pub use skill_integration::{SkillResolver, SkillState};
pub use snapshot::SnapshotManager;
pub use status::SessionStatus;
/// Persistent storage abstraction for sessions, skills, and application data.
pub use storage::Storage;
pub use summary::{SummaryError, SummaryGenerator};
pub use sync::SyncManager;
pub use token_counter::{CostCalculator, TokenCounter};
/// Tool execution infrastructure: ToolCall, ToolDefinition, ToolExecutor, ToolParameter.
/// ToolRegistry manages available tools; ToolResult wraps tool execution results.
pub use tool::{
    build_default_registry, ToolCall, ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry,
    ToolResult,
};
pub use tool_config::ToolConfig;
pub use util::Util;
pub use worktree::WorktreeManager;
