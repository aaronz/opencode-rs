# OpenCode RS Architecture

This document describes the architecture of the opencode-rs project, a Rust implementation of an AI coding agent similar to Claude Code / OpenCode.

## Overview

OpenCode RS is organized as a **Cargo workspace** with 24 crates. The project implements an AI coding agent with support for:
- Terminal UI (TUI) and web interfaces
- Multiple LLM provider integrations (OpenAI, Anthropic, Google, Azure, etc.)
- Tool execution and file operations
- Session management and persistence
- MCP (Model Context Protocol) integration
- Git operations and workspace awareness
- Plugin system for extensibility

## Crate Structure

```
opencode-rust/
├── crates/
│   ├── acp/           # Agent Communication Protocol client
│   ├── agent/         # Agent implementations (BuildAgent, DebugAgent, etc.)
│   ├── auth/          # Authentication (JWT, OAuth, password hashing)
│   ├── cli/           # CLI binary and commands (42 subcommands)
│   ├── config/        # Configuration loading and schema
│   ├── control-plane/ # Control plane client integration
│   ├── core/          # Core types and abstractions (⚠️ large, see below)
│   ├── file/          # File operations service
│   ├── format/        # Code formatting service (25+ formatters)
│   ├── git/           # Git operations (branch, merge, push, pull, etc.)
│   ├── llm/          # LLM provider integrations
│   ├── logging/       # Structured logging infrastructure
│   ├── lsp/          # Language Server Protocol integration
│   ├── mcp/          # MCP protocol implementation
│   ├── permission/   # Permission system and audit logging
│   ├── plugin/        # Plugin system (WASM-based)
│   ├── sdk/          # SDK for external consumers
│   ├── server/        # HTTP server (Actix-web)
│   ├── storage/       # SQLite-based persistence
│   ├── tools/         # Tool implementations (read, write, grep, etc.)
│   ├── tui/          # Terminal UI (ratatui-based)
│   └── util/          # Utilities
├── integration_tests/  # Integration tests
├── opencode-benches/  # Benchmarks
└── ratatui-testing/   # TUI testing framework
```

## Crate Responsibilities

### Application Crates

| Crate | Responsibility | Public API |
|-------|---------------|------------|
| `opencode-cli` | CLI binary entry point, 42 subcommands | Binary |
| `opencode-tui` | Terminal user interface | `App`, dialogs, widgets |
| `opencode-server` | HTTP REST API server | Actix-web routes |

### Domain Crates

| Crate | Responsibility | Key Types |
|-------|---------------|-----------|
| `opencode-agent` | Agent implementations | `Agent`, `Task`, `TaskDelegate` |
| `opencode-session` | Session management | `Session`, `SessionState` |
| `opencode-project` | Project/workspace management | `ProjectManager` |
| `opencode-skill` | Skill system | `Skill`, `SkillManager` |
| `opencode-tool-core` | Tool infrastructure | `Tool`, `ToolRegistry`, `ToolCall` |
| `opencode-tools` | Tool implementations | `BashTool`, `ReadTool`, `WriteTool`, etc. |
| `opencode-git` | Git operations | `GitManager` |

### Infrastructure Crates

| Crate | Responsibility | Key Types |
|-------|---------------|-----------|
| `opencode-config` | Configuration | `Config`, `ConfigSource` |
| `opencode-storage` | SQLite persistence | `Storage`, `Repository` |
| `opencode-logging` | Structured logging | `AgentLogger`, `LogStore` |
| `opencode-llm` | LLM provider abstraction | `LLMProvider`, `ModelRegistry` |

### Integration Crates

| Crate | Responsibility |
|-------|---------------|
| `opencode-mcp` | MCP protocol client/server |
| `opencode-lsp` | LSP client implementation |
| `opencode-plugin` | WASM plugin runtime |
| `opencode-acp` | Agent Communication Protocol |

## Dependency Direction

```
CLI/TUI/Server (application layer)
    ↓
Agent, Tools, Skills (domain layer)
    ↓
Session, Project, Config, Storage (core domain)
    ↓
LLM, MCP, LSP, Git (provider/integration layer)
    ↓
Logging, Util (infrastructure layer)
```

**Rule**: Domain logic must NOT depend on application layers (CLI, TUI, Server).

## Core Crate (⚠️ Attention)

The `opencode-core` crate is currently **too large** with 64 modules and ~25K lines of code. It contains:

- Session management (session.rs - 2461 lines)
- Project management (project.rs - 2203 lines)
- Skill system (skill.rs - 1485 lines)
- Tool infrastructure (tool.rs - 1088 lines)
- Command system (command.rs - 1196 lines)

### Future Extraction Plan

The following extractions are planned (not yet implemented):

1. `opencode-session` - Extract session, session_state, snapshot
2. `opencode-project` - Extract project, filesystem, directory
3. `opencode-skill-core` - Extract skill, skill_integration
4. `opencode-tool-core` - Extract tool trait, ToolCall, ToolResult

## Large Files

The following files exceed 1000 lines and should be split:

| File | Lines | Domain |
|------|-------|--------|
| `crates/core/src/session.rs` | 2461 | Session management |
| `crates/core/src/project.rs` | 2203 | Project management |
| `crates/core/src/skill.rs` | 1485 | Skill system |
| `crates/core/src/command.rs` | 1196 | Command execution |
| `crates/core/src/tool.rs` | 1088 | Tool infrastructure |
| `crates/tools/src/registry.rs` | 2640 | Tool registry |
| `crates/tools/src/lsp_tool.rs` | 1660 | LSP tool |

## Adding New Features

### Adding a New Tool

1. Create tool module in `crates/tools/src/`
2. Implement the `Tool` trait
3. Register in `build_default_registry()` in `crates/tools/src/discovery.rs`
4. Add tests

### Adding a New CLI Command

1. Create command module in `crates/cli/src/cmd/`
2. Add module and `run()` function
3. Register in `main.rs` command dispatch
4. Add subcommand to `Commands` enum

### Adding a New LLM Provider

1. Implement `LLMProvider` trait in `crates/llm/src/`
2. Add provider to `crates/llm/src/provider_registry.rs`
3. Add model catalog entry if needed

### Adding a New TUI Dialog

1. Create dialog in `crates/tui/src/dialogs/`
2. Implement `Dialog` trait
3. Add to appropriate screen/component
4. Add rendering tests using `ratatui-testing`

## Testing Strategy

- **Unit tests**: Inside each crate with `#[cfg(test)]` modules
- **Integration tests**: In `integration_tests/` directory
- **TUI tests**: Using `ratatui-testing` crate
- **Benchmarks**: In `opencode-benches/` using Criterion

## CI Pipeline

The CI workflow runs on push to main/dev and PRs:

1. **fmt-clippy** - Formatting and linting
2. **test** - Cross-platform tests (Ubuntu, macOS, Windows)
3. **feature-matrix** - No-default-features and all-features tests
4. **coverage** - 80% line coverage requirement
5. **build** - Release build
6. **benchmarks** - Benchmark compilation
7. **fuzz-smoke** - Nightly fuzzing smoke test

## Known Issues

1. **Core crate is monolithic** - Needs refactoring into smaller domain crates
2. **Large files** - Several files exceed 1000 lines
3. **CLI has 42 commands** - Could be grouped into submodules
4. **Logging crate has TUI code** - `logging/src/tui/` should move to TUI crate

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines and plugin development documentation.