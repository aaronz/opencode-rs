# OpenCode Modules PRD Index

This directory contains detailed Product Requirement Documents for each module in the OpenCode codebase.

## Purpose

These PRDs serve as implementation guides for LLM code agents to build the Rust equivalent of the OpenCode TypeScript codebase.

## Directory Structure

```
modules/
├── agent.md            # Core AI agent implementation
├── session.md          # Conversation session management
├── tool.md             # Tool implementations (26 tools)
├── provider.md         # AI provider abstraction (75+ providers)
├── cli.md              # Command-line interface (22 commands)
├── server.md           # HTTP server and API routes
├── storage.md          # SQLite database with Drizzle ORM
├── config.md           # Configuration management
├── lsp.md              # Language Server Protocol integration
├── mcp.md              # Model Context Protocol implementation
├── plugin.md           # Plugin system for extensibility
├── auth.md             # Authentication and credentials
├── project.md          # Project detection and management
├── acp.md              # Agent Communication Protocol
├── util.md             # General utilities (logging, errors, fs)
├── effect.md           # Effect-based functional programming
├── flag.md             # Feature flags and runtime config
├── global.md           # Global state and paths
├── env.md              # Environment variable handling
├── file.md             # File system utilities
├── git.md              # Git operations wrapper
├── pty.md              # Pseudo-terminal (PTY) management
├── sync.md             # State synchronization / SSE streaming
├── shell.md            # Shell command execution
├── bus.md              # In-process event bus
├── snapshot.md         # File snapshot / diff utilities
├── worktree.md         # Git worktree management
├── id.md               # Typed identifier generation
├── skill.md            # Skills / agent capability registry
├── account.md          # User account and subscription management
├── ide.md              # IDE/editor integration
├── share.md            # Session sharing functionality
├── control-plane.md    # Control plane API client
├── installation.md     # Installation and update management
├── permission.md       # Permission / access control system
├── question.md         # Interactive question / confirmation prompts
├── v2.md               # Session V2 schema & event system
├── format.md           # Code formatter service (25+ formatters)
├── npm.md              # NPM package manager service
├── patch.md            # Apply patch tool (custom patch format)
├── opencode-models-dev-integration.md  # models.dev integration PRD
└── opencode-modules-reference.md       # Modules overview reference
```

## Module Categories

### Core Modules (4)
Essential modules that form the heart of the agent:
- [`agent.md`](agent.md) - Orchestrates tool execution and LLM interaction
- [`session.md`](session.md) - Manages conversation context and prompts
- [`tool.md`](tool.md) - Implements 26 tools (read, write, bash, grep, etc.)
- [`provider.md`](provider.md) - Unified interface to 75+ AI providers

### Infrastructure Modules (3)
Modules that provide system-level functionality:
- [`cli.md`](cli.md) - Entry point and 22 CLI commands
- [`server.md`](server.md) - HTTP API for remote access
- [`storage.md`](storage.md) - SQLite persistence with Drizzle ORM

### Integration Modules (6)
Modules that integrate with external systems:
- [`lsp.md`](lsp.md) - Code intelligence via Language Server Protocol
- [`mcp.md`](mcp.md) - Extended capabilities via Model Context Protocol
- [`plugin.md`](plugin.md) - Extensibility via external plugins
- [`auth.md`](auth.md) - API key and OAuth credential management
- [`project.md`](project.md) - Project type detection
- [`acp.md`](acp.md) - Inter-agent communication

### Utility Modules (30)
Supporting modules for common operations:

| Module | File | Description |
|--------|------|-------------|
| util | [`util.md`](util.md) | Logging, errors, filesystem helpers |
| effect | [`effect.md`](effect.md) | Functional Effect monad wrappers |
| flag | [`flag.md`](flag.md) | Feature flags and runtime config |
| global | [`global.md`](global.md) | Global paths and shared state |
| env | [`env.md`](env.md) | Environment variable handling |
| file | [`file.md`](file.md) | File operations |
| git | [`git.md`](git.md) | Git operations wrapper |
| config | [`config.md`](config.md) | Configuration management |
| pty | [`pty.md`](pty.md) | Pseudo-terminal management |
| sync | [`sync.md`](sync.md) | State synchronization / SSE streaming |
| shell | [`shell.md`](shell.md) | Shell command execution |
| bus | [`bus.md`](bus.md) | In-process event bus |
| snapshot | [`snapshot.md`](snapshot.md) | File snapshot / diff utilities |
| worktree | [`worktree.md`](worktree.md) | Git worktree management |
| id | [`id.md`](id.md) | Typed identifier generation |
| skill | [`skill.md`](skill.md) | Skills / agent capability registry |
| account | [`account.md`](account.md) | User account and subscription management |
| ide | [`ide.md`](ide.md) | IDE/editor integration |
| share | [`share.md`](share.md) | Session sharing functionality |
| control-plane | [`control-plane.md`](control-plane.md) | Control plane API client |
| installation | [`installation.md`](installation.md) | Installation and update management |
| permission | [`permission.md`](permission.md) | Permission / access control system |
| question | [`question.md`](question.md) | Interactive question / confirmation prompts |
| v2 | [`v2.md`](v2.md) | Session V2 schema & streaming event system |
| format | [`format.md`](format.md) | Code formatter service (25+ formatters) |
| npm | [`npm.md`](npm.md) | NPM package manager service |
| patch | [`patch.md`](patch.md) | Apply patch tool (custom patch format) |

## Usage

Each PRD contains:
1. **Module Overview** - Name, source path, type, purpose
2. **Functionality** - Core features and capabilities
3. **API Surface** - Key interfaces and types
4. **Data Structures** - Important data types
5. **Dependencies** - External dependencies
6. **Acceptance Criteria** - Success conditions
7. **Rust Implementation Guidance** - Specific recommendations for Rust implementation
8. **Test Design** - Unit and integration test cases derived from TypeScript tests

## Total Modules: 41 individual PRDs

| Category | Count |
|----------|-------|
| Core | 4 |
| Infrastructure | 3 |
| Integration | 6 |
| Utility | 27 |
| Reference | 2 |

## Reference

- Original OpenCode TypeScript source: `/packages/opencode/src/`
- models.dev API: `https://models.dev/api.json`
