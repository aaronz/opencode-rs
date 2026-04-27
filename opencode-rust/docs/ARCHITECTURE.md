# OpenCode RS Architecture

This document describes the architecture of the opencode-rs project, a Rust implementation of an AI coding agent similar to Claude Code / OpenCode.

## Overview

OpenCode RS is organized as a **Cargo workspace** with 23 crates. The project implements an AI coding agent with support for:
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
│   ├── cli/           # CLI binary and commands
│   ├── config/        # Configuration loading and schema
│   ├── control-plane/ # Control plane client integration
│   ├── core/          # Core types and abstractions (see below)
│   ├── file/          # File operations service
│   ├── format/        # Code formatting service (25+ formatters)
│   ├── git/           # Git operations (branch, merge, push, pull, etc.)
│   ├── llm/           # LLM provider integrations
│   ├── logging/       # Structured logging infrastructure
│   ├── lsp/           # Language Server Protocol integration
│   ├── mcp/           # MCP protocol implementation
│   ├── permission/    # Permission system and audit logging
│   ├── plugin/        # Plugin system (WASM-based)
│   ├── sdk/           # SDK for external consumers
│   ├── server/        # HTTP server (Actix-web)
│   ├── storage/       # SQLite-based persistence
│   ├── tools/         # Tool implementations (read, write, grep, etc.)
│   ├── tui/           # Terminal UI (ratatui-based)
│   └── util/           # Utilities
├── integration_tests/  # Integration tests
├── opencode-benches/  # Benchmarks
└── ratatui-testing/   # TUI testing framework
```

## Crate Responsibilities

### Application Crates

| Crate | Responsibility | Public API |
|-------|---------------|------------|
| `opencode-cli` | CLI binary entry point | Binary |
| `opencode-tui` | Terminal user interface | `App`, dialogs, widgets |
| `opencode-server` | HTTP REST API server | Actix-web routes |

### Domain Crates

| Crate | Responsibility | Key Types |
|-------|---------------|-----------|
| `opencode-agent` | Agent implementations | `Agent`, `Task`, `TaskDelegate` |
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

## Core Crate Structure

The `opencode-core` crate contains 53 subdirectories, each following a consistent pattern:

```
crates/core/src/
├── session/           # Session management
│   ├── mod.rs         # Session implementation + tests
│   ├── types.rs       # Session types
│   ├── fork.rs        # Fork logic
│   ├── history.rs     # History management
│   ├── session_info.rs
│   ├── share.rs       # Session sharing
│   └── tool_invocation.rs
├── message/           # Message types
│   ├── mod.rs
│   └── types.rs
├── tool/             # Tool infrastructure
│   ├── mod.rs
│   ├── types.rs
│   └── registry.rs
├── skill/            # Skill system
│   ├── mod.rs
│   ├── types.rs
│   └── match.rs
├── checkpoint/       # Session checkpointing
│   ├── mod.rs
│   ├── types.rs
│   └── manager.rs
└── [45+ more modules...]
```

### Module Naming Convention

- **Types module** (`types.rs`): Data structures, enums, constants
- **Implementation module** (`mod.rs`): Methods, business logic, tests
- **Additional submodules**: Split by concern when a module grows large

## Large Files (Still Present)

These files exceed 1000 lines and could benefit from future refactoring:

| File | Lines | Crate |
|------|-------|-------|
| `runtime.rs` | 1856 | agent |
| `provider_abstraction.rs` | 1502 | llm |
| `auth.rs` | 941 | llm |
| `bedrock.rs` | 760 | llm |
| `openai.rs` | 737 | llm |
| `budget.rs` | 602 | llm |

## Adding New Features

### Adding a New Tool

1. Create tool module in `crates/tools/src/` (e.g., `my_tool.rs`)
2. Implement the `Tool` trait
3. Register in `build_default_registry()` in `crates/tools/src/discovery.rs`
4. Add tests

### Adding a New Module to Core

1. Create new directory `crates/core/src/<module_name>/`
2. Create `types.rs` for data structures
3. Create `mod.rs` for implementation and tests
4. Add `pub mod <module_name>;` to `crates/core/src/lib.rs`

### Adding a New LLM Provider

1. Create provider file in `crates/llm/src/` (e.g., `new_provider.rs`)
2. Implement `LLMProvider` trait
3. Add provider to `crates/llm/src/provider_registry.rs`

### Adding a New TUI Dialog

1. Create dialog in `crates/tui/src/dialogs/`
2. Implement `Dialog` trait
3. Add rendering tests using `ratatui-testing`
4. Handle empty state, keyboard navigation, edge cases

## Testing Strategy

- **Unit tests**: Inside each crate with `#[cfg(test)]` modules
- **Integration tests**: In `integration_tests/` directory
- **TUI tests**: Using `ratatui-testing` crate
- **Benchmarks**: In `opencode-benches/` using Criterion

## CI Pipeline

The CI workflow runs on push to main/dev and PRs:

1. **fmt** - `cargo fmt --all`
2. **clippy** - `cargo clippy --all -- -D warnings`
3. **test** - `cargo test --all-features`
4. **build** - Release build verification

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.

## Known Issues

1. ~~Core crate was monolithic~~ - ✅ Refactored into 53 subdirectories (2026-04-27)
2. Large files in `agent` and `llm` crates still need refactoring
3. Some providers in `llm/` are quite large (1500+ lines)
