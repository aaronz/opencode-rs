# Contributing to OpenCode RS

This guide helps developers understand the OpenCode RS project structure and how to add new features.

## Quick Links

- [Project Structure](#project-structure)
- [Development Setup](#development-setup)
- [Adding New Code](#adding-new-code)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)

## Project Structure

```
opencode-rs/
├── opencode-rust/           # Main Rust workspace
│   ├── crates/
│   │   ├── core/           # Core types (53 subdirectories)
│   │   ├── agent/          # Agent implementations
│   │   ├── tools/          # Tool implementations
│   │   ├── llm/            # LLM provider integrations
│   │   ├── cli/            # CLI binary
│   │   ├── tui/            # Terminal UI
│   │   └── [20+ more]      # Other crates
│   ├── integration_tests/   # Integration tests
│   ├── docs/               # Architecture docs
│   └── opencode-benches/   # Benchmarks
├── ratatui-testing/        # TUI testing framework
└── docs/                  # General documentation
```

## Development Setup

### Prerequisites

- Rust 1.70+ (with `cargo`)
- Git

### Build Commands

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Run all tests
cargo test

# Run tests for specific crate
cargo test -p opencode-core

# Run clippy (linting)
cargo clippy --all -- -D warnings

# Format code
cargo fmt --all
```

### Build Script

```bash
# Build release (default)
./build.sh

# Build debug
./build.sh --debug

# Build and test
./build.sh --test
```

## Adding New Code

### Adding a New Tool

Tools are in `crates/tools/src/`. Each tool is a separate file.

1. **Create tool file** (e.g., `my_tool.rs`):
```rust
use async_trait::async_trait;
use opencode_core::{Tool, ToolCall, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MyTool;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyToolArgs {
    pub input: String,
}

#[async_trait]
impl Tool for MyTool {
    name = "my_tool"
    description = "Does something useful"

    async fn execute(
        &self,
        args: ToolCall,
        _context: ToolContext,
    ) -> ToolResult {
        let args: MyToolArgs = args.parse()?;
        // Implementation
        Ok("result".into())
    }
}
```

2. **Register in registry** (`discovery.rs`):
```rust
pub fn build_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    // ... existing tools
    registry.register(MyTool);
    registry
}
```

3. **Add tests** in the same file or in `tests/` directory

### Adding a New Agent

Agents are in `crates/agent/src/`. Each agent type has its own file.

1. **Create agent file** (e.g., `my_agent.rs`)
2. **Implement the `Agent` trait**
3. **Register in `lib.rs`** with `pub mod my_agent;`

### Adding a New LLM Provider

Providers are in `crates/llm/src/`.

1. **Create provider file** (e.g., `new_provider.rs`)
2. **Implement `LLMProvider` trait**:
```rust
impl LLMProvider for NewProvider {
    fn name(&self) -> &str { "newprovider" }
    // ... other trait methods
}
```
3. **Register in `provider_registry.rs`**:
```rust
pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(NewProvider::new());
    // ... other providers
    registry
}
```

### Adding a New Core Module

The `opencode-core` crate uses a consistent subdirectory structure.

1. **Create directory**: `crates/core/src/<module_name>/`
2. **Create `types.rs`** for data structures:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyData {
    pub field: String,
}
```
3. **Create `mod.rs`** for implementation:
```rust
use super::types::MyData;

pub struct MyModule {
    data: MyData,
}

impl MyModule {
    pub fn new() -> Self { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() { /* ... */ }
}
```
4. **Declare in `lib.rs`**: `pub mod module_name;`

### Adding a New TUI Dialog

Dialogs are in `crates/tui/src/dialogs/`.

1. **Create dialog file** implementing `Dialog` trait
2. **Handle edge cases**:
   - Empty state rendering
   - Keyboard navigation bounds
   - Enter on empty selection
3. **Add tests** with `ratatui-testing`

### Adding a New CLI Command

Commands are in `crates/cli/src/cmd/`.

1. **Create command module** (e.g., `my_command.rs`)
2. **Implement `run()` function**
3. **Add to `Commands` enum** in `main.rs`

## Testing

### Unit Tests

Unit tests live inside `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(2), 4);
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p opencode-core

# With output
cargo test -- --nocapture

# Single test
cargo test test_name_here
```

### Integration Tests

Integration tests are in `integration_tests/src/`.

### TUI Testing

Use the `ratatui-testing` crate for dialog tests:

```rust
#[test]
fn test_dialog_renders() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let dialog = MyDialog::new();
        dialog.draw(f, f.area());
    }).unwrap();
    // assertions
}
```

## Code Style

### Rust Formatting

- **Edition**: Rust 2021
- **Formatter**: `cargo fmt`
- **Indentation**: Standard Rust formatting (4 spaces)

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | snake_case | `get_session` |
| Variables | snake_case | `session_id` |
| Types | CamelCase | `Session` |
| Enums | CamelCase | `OpenCodeError` |
| Traits | CamelCase | `Agent` |
| Constants | SCREAMING_SNAKE | `MAX_TOKEN_BUDGET` |

### Error Handling

Use `thiserror` for structured errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Async/Await

- Use `#[tokio::test]` for async tests
- Use `async_trait` for async trait methods
- Handle errors explicitly

## Submitting Changes

### Commit Guidelines

1. **Small, focused commits** - Each commit should do one thing
2. **Clear commit messages**:
   ```
   feat(tools): add my_tool for doing something

   Implements the MyTool struct with execute() method.
   Closes #123
   ```
3. **Prefixes**: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

### Pull Request Checklist

- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated if needed
- [ ] Commit message is clear

### CI Pipeline

All PRs must pass:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all -- -D warnings`
3. `cargo test --all-features`
4. `cargo build --release`

## Architecture

For detailed architecture information, see [ARCHITECTURE.md](./ARCHITECTURE.md).

## Getting Help

- Check [docs/](.) for documentation
- Look at existing similar code for patterns
- Ask in issues for clarification
