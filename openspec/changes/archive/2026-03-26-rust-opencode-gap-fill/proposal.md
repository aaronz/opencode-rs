## Why

The initial Rust port of OpenCode implemented core functionality but is missing many features from the original TypeScript version. To achieve full feature parity and pass the original test suite, we need to implement missing tools, enhance the provider system, add MCP support, and complete the session/LSP implementations.

## What Changes

### Tool System Gaps
- **NEW** `bash` tool: Execute shell commands with permission control
- **NEW** `apply_patch` tool: Apply code patches in diff format
- **NEW** `edit` tool: Edit files with exact string matching
- **NEW** `batch` tool: Execute multiple tools in parallel
- **NEW** `codesearch` tool: Search code across repositories
- **NEW** `ls` tool: List directory contents
- **NEW** `lsp` tool: Query LSP for symbols
- **NEW** `multiedit` tool: Multiple edits to same file
- **NEW** `question` tool: Ask user for input
- **NEW** `read` tool: Read file with line numbers
- **NEW** `todowrite` tool: Manage todo lists
- **NEW** `webfetch` tool: Fetch web content
- **NEW** `write` tool: Write files
- **NEW** `skill` tool: Execute skills
- **NEW** `task` tool: Spawn subagents
- **NEW** `truncate` tool: Truncate large outputs

### Provider System Gaps
- **ENHANCED** Provider: Add auth system with API key validation
- **ENHANCED** Provider: Add model registry with capabilities
- **ENHANCED** Provider: Add error handling with retries
- **ENHANCED** Provider: Add message transformation pipeline

### Session System Gaps
- **NEW** Session: Add compaction for long conversations
- **NEW** Session: Add message versioning (v2 format)
- **NEW** Session: Add session processor with hooks
- **NEW** Session: Add prompt management system
- **NEW** Session: Add session status tracking
- **NEW** Session: Add session summary generation
- **NEW** Session: Add session revert capability

### LSP System Gaps
- **ENHANCED** LSP: Add full server implementation
- **ENHANCED** LSP: Add language detection
- **ENHANCED** LSP: Add launch configuration

### CLI Gaps
- **NEW** CLI: Add all subcommands (auth, config, serve, etc.)
- **NEW** CLI: Add UI components for terminal

### Missing Systems
- **NEW** MCP: Model Context Protocol support
- **NEW** Config: Configuration management system
- **NEW** Storage: Persistent storage system
- **NEW** Git: Git integration utilities
- **NEW** Permission: Permission control system
- **NEW** Plugin: Plugin architecture
- **NEW** Project: Project detection and management
- **NEW** Format: Code formatting utilities

## Capabilities

### New Capabilities
- `bash-tool`: Execute shell commands with permission control
- `apply-patch-tool`: Apply code patches in diff format
- `edit-tool`: Edit files with exact string matching
- `batch-tool`: Execute multiple tools in parallel
- `codesearch-tool`: Search code across repositories
- `ls-tool`: List directory contents
- `lsp-tool`: Query LSP for symbols
- `multiedit-tool`: Multiple edits to same file
- `question-tool`: Ask user for input
- `read-tool`: Read file with line numbers
- `todowrite-tool`: Manage todo lists
- `webfetch-tool`: Fetch web content
- `write-tool`: Write files
- `skill-tool`: Execute skills
- `task-tool`: Spawn subagents
- `truncate-tool`: Truncate large outputs
- `provider-auth`: Provider authentication system
- `provider-models`: Model registry and capabilities
- `provider-error`: Error handling with retries
- `provider-transform`: Message transformation pipeline
- `session-compaction`: Session compaction for long conversations
- `session-processor`: Session processor with hooks
- `session-prompt`: Prompt management system
- `session-status`: Session status tracking
- `session-summary`: Session summary generation
- `session-revert`: Session revert capability
- `mcp-support`: Model Context Protocol support
- `config-management`: Configuration management
- `storage-system`: Persistent storage
- `git-integration`: Git integration utilities
- `permission-control`: Permission control system
- `plugin-architecture`: Plugin architecture
- `project-management`: Project detection and management
- `format-utilities`: Code formatting utilities

### Modified Capabilities
- `rust-cli-core`: Add new CLI commands and UI
- `agent-system`: Add agent registry and subagent spawning
- `llm-provider`: Add auth, models, error, transform
- `tool-executor`: Add all missing tools
- `session-manager`: Add compaction, processor, prompt, status, summary, revert
- `lsp-client`: Add server, language detection, launch

## Impact

### Code Impact
- Add 16 new tool implementations
- Enhance provider system with auth, models, error, transform
- Add session management features (compaction, processor, etc.)
- Add MCP support
- Add config, storage, git, permission, plugin, project, format modules

### Dependencies
- Add serde for JSON serialization
- Add reqwest for HTTP client
- Add tokio for async runtime
- Add clap for CLI
- Add ratatui for TUI
- Add walkdir for file operations
- Add regex for pattern matching
- Add glob for glob patterns
- Add chrono for time handling
- Add uuid for unique IDs
- Add directories for standard directories
- Add toml for config
- Add anyhow/thiserror for error handling
- Add tracing for logging
- Add async-trait for async traits
- Add futures for async utilities

### Systems Impact
- Tool registry expansion
- Provider system enhancement
- Session management enhancement
- LSP system completion
- CLI command addition
- New module creation (MCP, Config, Storage, etc.)
