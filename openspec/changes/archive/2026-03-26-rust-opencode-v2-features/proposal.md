## Why

The current Rust OpenCode implementation has achieved feature parity with the original TypeScript version in many areas, but there are still significant gaps in functionality that need to be addressed to achieve full compatibility. This proposal outlines the remaining features needed to match the original OpenCode implementation and pass its test suite.

## What Changes

### Advanced Features
- **NEW** Effect system: Functional programming patterns for error handling and async
- **NEW** Bus/Event system: Centralized event handling and messaging
- **NEW** Command system: CLI command architecture with plugins
- **NEW** Format system: Code formatting with language-specific support
- **NEW** IDE integration: Support for VS Code, Zed, and other editors
- **NEW** Installation management: Update and version management
- **NEW** Snapshot/Versioning: File snapshot system for revert capability
- **NEW** Sync system: File synchronization and conflict resolution
- **NEW** Worktree management: Git worktree support for parallel development

### Enhanced Components
- **ENHANCED** Tool system: Add plan tool, external directory tool, truncation directory
- **ENHANCED** Session system: Add message-v2 format, projectors, instruction system
- **ENHANCED** Provider system: Add SDK integration, OAuth support
- **ENHANCED** CLI system: Add subcommands (auth, config, serve, upgrade, etc.)
- **ENHANCED** TUI system: Add interactive components, dialogs, keybindings

### Missing Modules
- **NEW** Account management: User authentication and session management
- **NEW** ACP (Agent Communication Protocol): Multi-agent communication
- **NEW** Auth module: Authentication and authorization
- **NEW** Control Plane: Cloud and enterprise features
- **NEW** File system: Advanced file operations and watching
- **NEW** Flag system: Feature flags and configuration
- **NEW** Global state: Application-wide state management
- **NEW** ID generation: UUID and other ID systems
- **NEW** Patch system: Advanced patch application and management
- **NEW** Permission system: Fine-grained access control
- **NEW** Plugin system: Extensibility and third-party integrations
- **NEW** Project detection: Advanced project type detection
- **NEW** PTY support: Pseudo-terminal for interactive commands
- **NEW** Question system: User input and confirmation dialogs
- **NEW** Server system: HTTP/WS server for API and TUI
- **NEW** Share system: Session sharing and collaboration
- **NEW** Shell integration: Advanced shell command execution
- **NEW** Skill system: Skill registration and execution
- **NEW** Util system: Common utilities and helpers
- **NEW** Storage system: Persistent storage with multiple backends

## Capabilities

### New Capabilities
- `effect-system`: Functional programming patterns for error handling
- `bus-event-system`: Centralized event handling and messaging
- `command-system`: CLI command architecture with plugins
- `format-system`: Code formatting with language-specific support
- `ide-integration`: Support for VS Code, Zed, and other editors
- `installation-management`: Update and version management
- `snapshot-versioning`: File snapshot system for revert capability
- `sync-system`: File synchronization and conflict resolution
- `worktree-management`: Git worktree support for parallel development
- `account-management`: User authentication and session management
- `acp-protocol`: Multi-agent communication protocol
- `auth-module`: Authentication and authorization
- `control-plane`: Cloud and enterprise features
- `file-system`: Advanced file operations and watching
- `flag-system`: Feature flags and configuration
- `global-state`: Application-wide state management
- `id-generation`: UUID and other ID systems
- `patch-system`: Advanced patch application and management
- `permission-system`: Fine-grained access control
- `plugin-system`: Extensibility and third-party integrations
- `project-detection`: Advanced project type detection
- `pty-support`: Pseudo-terminal for interactive commands
- `question-system`: User input and confirmation dialogs
- `server-system`: HTTP/WS server for API and TUI
- `share-system`: Session sharing and collaboration
- `shell-integration`: Advanced shell command execution
- `skill-system`: Skill registration and execution
- `util-system`: Common utilities and helpers
- `storage-system`: Persistent storage with multiple backends

### Modified Capabilities
- `tool-executor`: Add plan tool, external directory tool, truncation directory
- `session-manager`: Add message-v2 format, projectors, instruction system
- `llm-provider`: Add SDK integration, OAuth support
- `rust-cli-core`: Add subcommands (auth, config, serve, upgrade, etc.)
- `tui-renderer`: Add interactive components, dialogs, keybindings

## Impact

### Code Impact
- Add ~30 new modules to the Rust project
- Enhance existing modules with missing features
- Implement ~53,000 lines of TypeScript in Rust

### Dependencies
- Add effect-ts or similar for functional patterns
- Add tokio for async runtime
- Add axum/warp for HTTP server
- Add crossterm for terminal UI
- Add serde for serialization
- Add chrono for time handling
- Add uuid for ID generation

### Systems Impact
- Major architecture changes
- New module dependencies
- Enhanced error handling
- New testing requirements
