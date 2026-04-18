# OpenCode Rust Monorepo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)

A Rust implementation of the OpenCode AI coding agent with a comprehensive TUI testing framework.

## Projects

### opencode-rust
Rust implementation of OpenCode AI coding agent featuring:
- **Core**: Session management, tool registry, and error handling
- **Agent**: Agent implementations with LLM integration
- **LLM**: Provider integrations for various language models
- **Tools**: File operations, grep, git, and other developer tools
- **TUI**: Terminal user interface built with ratatui
- **MCP**: Model Context Protocol support
- **Server**: HTTP server with actix-web
- **Storage**: SQLite-based session persistence

### ratatui-testing
A comprehensive testing framework for Rust TUI applications:
- Snapshot testing for TUI output
- PTY simulation for interactive testing
- CLI testing utilities
- State testing helpers

## Building

```bash
# Build from project root (recommended)
./build.sh

# Build release
cargo build --release

# Build debug
cargo build

# Run all tests
cargo test

# Build with all features
cargo build --all-features
```

### Build Script Options

```bash
./build.sh           # Release build (default)
./build.sh --debug   # Debug build
./build.sh --test    # Build + run tests
```

## Structure

```
opencode-rust/
├── crates/
│   ├── core/       # Core functionality
│   ├── cli/        # CLI commands
│   ├── llm/        # LLM provider integrations
│   ├── tools/      # Tool implementations
│   ├── tui/        # Terminal UI
│   ├── agent/      # Agent implementations
│   └── ...         # Other modules
└── tests/          # Integration tests

ratatui-testing/
├── src/            # Framework source
└── tests/          # Integration tests
```

## Documentation

- [AGENTS.md](./AGENTS.md) - AI agent instructions
- [PRD.md](./docs/PRD.md) - Product requirements
- [docs/](./docs/) - Design and analysis documents
