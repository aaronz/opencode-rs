# OpenCode Rust Monorepo

A Rust implementation of the OpenCode AI coding agent with a comprehensive TUI testing framework.

## Projects

### rust-opencode-port
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
# Build release
cargo build --release

# Build debug
cargo build

# Run all tests
cargo test

# Build with all features
cargo build --all-features
```

## Structure

```
rust-opencode-port/
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
