# AGENTS.md

This file provides guidance for AI coding agents operating in this repository.

**Important**: The content of this file is automatically loaded into the agent's system prompt when working in this project. Keep this file concise and focused on project-specific guidelines that the agent should follow.

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

Use test helpers from `integration_tests/src/common/`:
- `TempProject`: Create temporary projects for testing
- `MockServer`: Mock HTTP server for testing
- `MockLLMProvider`: Mock LLM provider for testing

### TUI Dialog Testing

Dialogs have common edge cases that must be tested:

#### Required Dialog Tests

Every dialog with a list/selection must have these tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 1. Empty collection + Enter should close the dialog
    #[test]
    fn empty_list_enter_closes() {
        let mut dialog = create_dialog_with_items(vec![]);
        assert_eq!(dialog.handle_input(enter_key()), DialogAction::Close);
    }

    // 2. Empty collection + navigation should not panic
    #[test]
    fn empty_list_up_does_not_panic() {
        let mut dialog = create_dialog_with_items(vec![]);
        dialog.handle_input(up_key());
    }

    // 3. Single item navigation (Down at end should stay at 0)
    #[test]
    fn single_item_down_stays_at_zero() {
        let mut dialog = create_dialog_with_items(vec!["item"]);
        dialog.handle_input(down_key());
    }

    // 4. Render with empty state must show something visible
    #[test]
    fn test_dialog_renders_empty_state() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            let dialog = create_dialog_with_items(vec![]);
            dialog.draw(f, f.area());
        }).unwrap();
        let buffer = terminal.backend().buffer();
        let has_border = buffer.content.iter()
            .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
        assert!(has_border, "Empty dialog should render border");
    }
}
```

#### Rendering Tests Pattern

Use `ratatui::backend::TestBackend` to verify UI renders correctly:

```rust
use ratatui::{backend::TestBackend, Frame, Terminal};

#[test]
fn test_dialog_renders_models() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f: &mut Frame| {
        let dialog = ConnectModelDialog::new(Theme::default(), vec![]);
        dialog.draw(f, f.area());
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let has_border = buffer.content.iter()
        .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
    assert!(has_border, "Dialog should render with border");
}
```

#### Dialog Index Safety Rules

When dialogs have filtering, `selected_indices` must track **original** indices:

```rust
// WRONG: Mixing filtered index with original indices
let filtered = self.filtered_entries();
let is_selected = self.selected_indices.contains(&filtered_idx); // Bug!

// CORRECT: filtered_entries returns (original_idx, item)
let filtered = self.filtered_entries(); // Vec<(usize, &Item)>
let is_selected = self.selected_indices.contains(original_idx);
```

#### Empty State Rendering Rule

When a list is empty, always render a visible message:

```rust
if self.items.is_empty() {
    let empty_msg = Paragraph::new("No items match filter")
        .style(Style::default().fg(self.theme.muted_color()));
    f.render_widget(empty_msg, inner_area);
    return;
}
```

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
│   ├── core/               # Core functionality (error, session, tool, hook, etc.)
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
│   ├── mcp/                # MCP protocol
│   ├── acp/                # ACP protocol client
│   ├── file/               # File operations
│   ├── format/             # Code formatting
│   ├── logging/            # Logging infrastructure
│   ├── sdk/                # SDK components
│   ├── util/               # Utilities
│   └── runtime/            # Runtime facade
├── integration_tests/      # Integration tests
│   └── src/
│       ├── agent_tool_tests.rs
│       ├── agent_llm_tests.rs
│       ├── mcp_protocol_tests.rs
│       ├── session_storage_tests.rs
│       └── common/         # Test helpers
├── opencode-benches/       # Benchmarks
└── ratatui-testing/        # TUI testing framework library
    ├── src/
    │   ├── lib.rs
    │   ├── state.rs       # State testing
    │   ├── pty.rs         # PTY simulation
    │   ├── cli.rs         # CLI testing
    │   ├── diff.rs        # Buffer diffing
    │   ├── dsl.rs         # Test DSL
    │   └── dialog_tester.rs # Dialog rendering test helpers
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

### Adding a New Dialog

When adding a dialog to `crates/tui/src/dialogs/`:

1. **Implement the `Dialog` trait**
2. **Handle empty state**: Always show a visible message when the list is empty
3. **Handle Enter on empty**: Return `DialogAction::Close` not `None`
4. **Track indices correctly**: If using filtering + selection, store original indices
5. **Add unit tests**:
   - Empty collection + Enter closes
   - Empty collection + navigation doesn't panic
   - Single item navigation
   - Filter reduces to empty
6. **Add rendering tests** (in `tests/` folder):
   - Verify empty state renders with border
   - Verify content renders with models/items
   - Use `ratatui::backend::TestBackend`

Example empty state handling:
```rust
if self.items.is_empty() {
    let empty_msg = Paragraph::new("No items available")
        .style(Style::default().fg(self.theme.muted_color()));
    f.render_widget(empty_msg, inner_area);
    return;
}
```

### Desktop/Web/ACP Interface

The project supports multiple interface modes:

#### CLI Commands

```bash
# Start TUI mode (default)
opencode

# Start web interface mode
opencode web --port 3000

# Start API server mode
opencode serve --port 8080

# Start desktop mode (TUI + server + browser)
opencode desktop --port 3000

# ACP protocol commands
opencode acp status
opencode acp handshake --client-id <id> --capabilities chat,tasks
opencode acp connect --url <url>
```

#### Configuration

Desktop and ACP settings can be configured in `config.json`:

```json
{
  "server": {
    "port": 3000,
    "hostname": "127.0.0.1",
    "desktop": {
      "enabled": true,
      "auto_open_browser": true
    },
    "acp": {
      "enabled": true,
      "server_id": "local",
      "version": "1.0"
    }
  }
}
```

#### ACP Routes

The ACP (Agent Communication Protocol) provides REST endpoints:

- `GET /api/acp/status` - Get ACP status
- `POST /api/acp/handshake` - Perform handshake
- `POST /api/acp/connect` - Connect to a server
- `POST /api/acp/ack` - Acknowledge handshake
