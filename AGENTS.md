# AGENTS.md

This file provides guidance for AI coding agents operating in this repository.

## Repository Overview

This is a monorepo containing:
- **opencode-rust/**: Rust implementation of OpenCode AI coding agent
- **ratatui-testing/**: TUI testing framework for Rust applications

## Build Commands

### Rust Project (opencode-rust)

```bash
# Build release
cargo build --release

# Build debug
cargo build

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests with all features
cargo test --all-features

# Run a single test
cargo test <test_name>
# Example: cargo test test_tool_registry_register_and_execute

# Run tests for specific package
cargo test -p opencode-core
cargo test -p opencode-tools
cargo test -p opencode-integration-tests

# Run doc tests
cargo test --doc

# Check formatting
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all

# Run clippy (linting)
cargo clippy --all -- -D warnings

# Build with all features
cargo build --all-features

# Generate docs
cargo doc --all-features --no-deps
```

### Using the Build Script

```bash
# Build release (default)
./build.sh

# Build debug
./build.sh --debug

# Build and run tests
./build.sh --test
```

### ratatui-testing Library

```bash
cd ratatui-testing

# Build
cargo build

# Run tests
cargo test

# Run tests with all features
cargo test --all-features

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Code Style Guidelines

### Rust Formatting

- **Edition**: Rust 2021 (`edition = "2021"`)
- **Formatter**: Use `cargo fmt` before committing
- **Indentation**: Standard Rust formatting (4 spaces default, let rustfmt handle)
- **Line length**: Default rustfmt settings

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | snake_case | `get_session`, `execute_tool` |
| Variables | snake_case | `session_id`, `tool_name` |
| Types | CamelCase | `Session`, `ToolRegistry` |
| Enums | CamelCase | `OpenCodeError`, `ToolResult` |
| Traits | CamelCase | `Agent`, `Tool` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_TOKEN_BUDGET` |
| Module names | snake_case | `tool`, `session` |

### Error Handling

The project uses a structured error system in `crates/core/src/error.rs`:

```rust
// Use thiserror for enum variants
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenCodeError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool arguments: {0}")]
    ToolInvalidArgs(String),

    // With structured fields
    #[error("Token expired")]
    TokenExpired { detail: Option<String> },
}
```

Error code ranges:
- 1xxx: Authentication errors
- 2xxx: Authorization errors
- 3xxx: Provider errors
- 4xxx: Tool errors
- 5xxx: Session errors
- 6xxx: Config errors
- 7xxx: Validation errors
- 9xxx: Internal errors

### Async/Await

- Use `#[tokio::test]` for async tests
- Use `async_trait` for async trait methods
- Always handle errors explicitly in async code

```rust
#[tokio::test]
async fn test_tool_execution() {
    let result = tool.execute(params, None).await;
    assert!(result.is_ok());
}
```

### Serialization

Uses `serde` with derive macros:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}
```

### Module Organization

```rust
// lib.rs - public API with re-exports
pub mod tool;
pub use tool::{Tool, ToolRegistry};

// Use doc comments for public API
/// A tool that can execute file operations.
/// Use ToolRegistry to manage multiple tools.
pub use tool::{Tool, ToolRegistry};
```

### Imports

```rust
// Order: std → external → internal
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use opencode_core::{Session, Tool};
```

### Testing Conventions

```rust
#[tokio::test]
async fn test_feature() {
    // Setup
    let item = Item::new();

    // Action
    let result = item.do_something().await;

    // Assertion
    assert!(result.is_ok());
}
```

Use test helpers from `tests/src/common/`:
- `TempProject`: Create temporary projects for testing
- `MockServer`: Mock HTTP server for testing
- `MockLLMProvider`: Mock LLM provider for testing

### Documentation

Document public API with doc comments:

```rust
/// Represents a conversation session containing messages and metadata.
/// Sessions can be saved to disk and restored for continued conversations.
pub struct Session { ... }

/// Configuration for OpenCode RS application.
/// Loaded from config.toml, environment variables, and command-line arguments.
pub use config::Config;
```

## Project Structure

```
opencode-rust/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Core functionality (error, session, tool, etc.)
│   ├── cli/                # CLI commands
│   ├── llm/                # LLM provider integrations
│   ├── tools/              # Tool implementations (read, write, grep, etc.)
│   ├── tui/                # Terminal UI (ratatui)
│   ├── agent/              # Agent implementations
│   ├── lsp/                # LSP integration
│   ├── storage/            # Database/storage layer
│   ├── server/             # HTTP server (actix-web)
│   ├── permission/         # Permission system
│   ├── auth/               # Authentication
│   ├── control-plane/      # Control plane client
│   ├── plugin/             # Plugin system
│   ├── git/                # Git operations
│   └── mcp/                # MCP protocol
├── tests/                  # Integration tests
│   └── src/
│       ├── agent_tool_tests.rs
│       ├── agent_llm_tests.rs
│       ├── mcp_protocol_tests.rs
│       ├── session_storage_tests.rs
│       └── common/         # Test helpers
└── opencode-benches/       # Benchmarks

ratatui-testing/            # TUI testing framework library
├── src/
│   ├── lib.rs
│   ├── state.rs           # State testing
│   ├── pty.rs             # PTY simulation
│   ├── cli.rs             # CLI testing
│   ├── diff.rs            # Buffer diffing
│   └── dsl.rs             # Test DSL
└── tests/
```

## Key Dependencies

- **tokio**: Async runtime (`features = ["full"]`)
- **serde/serde_json**: Serialization
- **thiserror**: Error enums
- **anyhow**: Contextual errors
- **tracing**: Logging
- **ratatui**: Terminal UI
- **rusqlite**: SQLite database
- **async-trait**: Async trait methods

## CI Pipeline

The CI runs on push to main/dev and PRs:
1. Check formatting (`cargo fmt`)
2. Run clippy (`cargo clippy -D warnings`)
3. Build release
4. Run all tests

## Common Tasks

### Adding a New Tool

1. Create tool module in `crates/tools/src/`
2. Implement the `Tool` trait
3. Register in `ToolRegistry::build_default_registry()`
4. Add tests in `tests/src/agent_tool_tests.rs`

### Adding a New CLI Command

1. Create command module in `crates/cli/src/cmd/`
2. Add to `CliRegistry` in `crates/core/src/cli.rs`
3. Implement command parser and handler

### Adding a New Agent Type

1. Create agent module in `crates/agent/src/`
2. Implement `Agent` trait
3. Export from `crates/agent/src/lib.rs`
