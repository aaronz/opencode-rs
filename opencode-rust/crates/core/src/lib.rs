//! OpenCode Core Library
//!
//! Core types and abstractions for the OpenCode RS agent.

#![allow(unused_imports)]

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
pub mod events;
pub mod executor;
pub mod filesystem;
pub mod flag;
pub mod format;
pub mod formatter;
pub mod global;
pub mod hook;
pub mod id;
pub mod ide;
pub mod installation;
pub mod instance;
pub mod instructions;
pub mod mcp;
pub mod message;
pub mod part;
pub mod paths;
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
pub mod session_sharing;
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
pub mod turn;
pub mod util;
pub mod watcher;
pub mod worktree;

// =============================================================================
// Public re-exports — used by external crates
// =============================================================================

/// Configuration for OpenCode RS application.
/// Loaded from config.toml, environment variables, and command-line arguments.
pub use config::Config;

/// Main error type for OpenCode RS. Covers IO, JSON, Network, Config, Session, Tool, LLM, and Provider errors.
pub use error::OpenCodeError;

/// Domain event — the single source of truth for all events.
/// Supersedes [`bus::InternalEvent`]; new code should use this type directly.
pub use events::DomainEvent;

/// Executes agents with tool access. Use build_default_executor() to create a configured executor.
pub use executor::AgentExecutor;

/// Represents a single message in a conversation with a role (user/assistant/system).
/// Messages are the core unit of conversation in OpenCode RS.
pub use message::{Message, Role};

pub use checkpoint::CheckpointManager;
pub use compaction::TokenBudget;
/// A conversation session containing messages, metadata, and tool invocation history.
/// Sessions can be saved to disk and restored for continued conversations.
pub use session::{Session, SessionInfo, SessionSummaryMetadata, ToolInvocationRecord};
pub use session_state::{is_valid_transition, SessionState};
pub use summary::SummaryGenerator;
pub use token_counter::{CostCalculator, TokenCounter};
pub use turn::{Turn, TurnId, TurnStatus, TaskId};

pub use session_sharing::SessionSharing;
/// Skill management for extending OpenCode with custom capabilities.
/// SkillManager handles skill discovery, matching, and lifecycle.
pub use skill::Skill;
pub use skill::SkillManager;
pub use skill_integration::{SkillResolver, SkillState};

/// Tool execution infrastructure: ToolCall, ToolDefinition, ToolExecutor, ToolParameter.
/// ToolRegistry manages available tools; ToolResult wraps tool execution results.
pub use tool::{
    build_default_registry, ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry, ToolResult,
};

pub use installation::InstallationManager;
pub use instance::Instance;

// =============================================================================
// Crate-internal re-exports — not intended for external use
// =============================================================================

// account
pub(crate) use account::AccountManager;

// acp
pub(crate) use acp::AcpProtocol;

// agents_md
pub(crate) use agents_md::{AgentsMdInfo, AgentsMdScanConfig, AgentsMdScanner};

// bus
pub(crate) use bus::EventBus;

// checkpoint
pub(crate) use checkpoint::{
    create_checkpoint, restore_checkpoint, Checkpoint, CheckpointMetadata,
};

// cli
pub(crate) use cli::CliRegistry;

// command
pub(crate) use command::{Command, CommandRegistry};

// compaction
pub(crate) use compaction::{
    COMPACTION_FORCE_THRESHOLD, COMPACTION_START_THRESHOLD, COMPACTION_WARN_THRESHOLD,
};

// config
pub(crate) use config::DirectoryScanner;

// context
/// Context for agent execution, including token budgeting and message management.
/// Use ContextBuilder to construct contexts with token limits.
pub(crate) use context::{estimate_tokens, trim_to_budget, Context, ContextBudget, ContextBuilder};

// control_plane
pub(crate) use control_plane::ControlPlane;

// directory
pub(crate) use directory::{
    AgentDefinition, CommandDefinition, ModeDefinition, PluginInfo, ThemeInfo,
};

// effect
pub(crate) use effect::{Effect, EffectError, EffectResult};

// env
pub(crate) use env::EnvManager;

// executor
pub(crate) use executor::build_default_executor;

// filesystem
pub(crate) use filesystem::AppFileSystem;

// flag
pub(crate) use flag::types::FlagManager;

// format
pub(crate) use format::FormatUtils;

// formatter
pub(crate) use formatter::{FormatterEngine, FormatterError};

// global
pub use global::GlobalState;

// id — public for external crates
pub use id::{IdGenerator, IdParseError, ProjectId, SessionId, UserId, WorkspaceId};

// ide
pub(crate) use ide::{Ide, IdeExtension, IdeManager, Position};

// instructions
pub(crate) use instructions::{InstructionsError, InstructionsLoader};

// mcp
/// MCP (Model Context Protocol) manager for tool discovery and execution.
pub(crate) use mcp::McpManager;

// part
pub(crate) use part::Part;

// permission
pub use permission::PermissionManager;

// plugin
pub(crate) use plugin::PluginManager;

// project
pub(crate) use project::{
    normalize_path, validate_workspace, ProjectManager, WorkspaceValidationError,
    WorkspaceValidationResult,
};

// prompt
pub(crate) use prompt::PromptManager;

// pty
pub(crate) use pty::PtySession;

// question
pub(crate) use question::QuestionManager;

// revert
pub(crate) use revert::RevertManager;

// server
pub(crate) use server::Server;

// session
pub(crate) use session::ShareError;

// session_sharing
pub(crate) use session_sharing::SharingError;

// session_state
pub(crate) use session_state::StateTransitionError;

// share
pub(crate) use share::ShareManager;

// shell
pub(crate) use shell::Shell;

// skill
pub(crate) use skill::{MatchType, SkillMatch};

// skill_integration — SkillResolver and SkillState already re-exported above as pub

// snapshot
pub(crate) use snapshot::SnapshotManager;

// status
pub(crate) use status::SessionStatus;

// storage
/// Persistent storage abstraction for sessions, skills, and application data.
pub(crate) use storage::Storage;

// summary
pub(crate) use summary::SummaryError;

// sync
pub(crate) use sync::SyncManager;

// tool
pub(crate) use tool::ToolCall;

// tool_config
pub(crate) use tool_config::ToolConfig;

// util
pub(crate) use util::Util;

// worktree
pub(crate) use worktree::WorktreeManager;
