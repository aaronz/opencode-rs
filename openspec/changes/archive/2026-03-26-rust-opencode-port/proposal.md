## Why

OpenCode is a powerful open-source AI coding agent with 131k+ GitHub stars, yet it's built primarily in TypeScript (56%) and MDX (40%). Porting to Rust will provide:
- **Performance**: Native binary distribution, faster startup, lower memory usage
- **Portability**: Single executable, no Node.js dependency
- **Integration**: Better system-level integration, easier packaging for various platforms
- **Developer experience**: Rust's strong typing and safety guarantees for maintainability

This port makes OpenCode accessible to developers who prefer Rust tooling or need a lightweight CLI alternative.

## What Changes

### Core Architecture Changes
- **Rewrite in Rust**: Full port from TypeScript/Node.js to Rust
- **Client/Server Architecture**: Maintain the existing architecture with possible improvements
- **CLI First**: Terminal-based UI as primary interface
- **Plugin System**: Extensible architecture for future capabilities

### Feature Parity (Phase 1)
- **Agent System**: Implement build, plan, and general subagents
- **LLM Integration**: Multi-provider support (OpenAI, Anthropic, Google, local models)
- **File Operations**: Read, write, edit, glob, grep capabilities
- **Tool System**: Web search, code search, Git operations
- **Session Management**: Conversation history, context preservation

### New Capabilities
- **Enhanced LSP Integration**: Native LSP protocol implementation
- **Streaming Responses**: Efficient real-time token streaming
- **Tool Call Parallelization**: Concurrent tool execution

## Capabilities

### New Capabilities
- `rust-cli-core`: Rust-based command-line interface with interactive terminal
- `agent-system`: Multi-agent architecture with build/plan/general agents
- `llm-provider`: Unified LLM provider abstraction supporting multiple backends
- `tool-executor`: Tool definition, execution, and result handling system
- `session-manager`: Conversation and session state management
- `lsp-client`: Language Server Protocol client implementation
- `tui-renderer`: Terminal UI rendering with modern terminal features

### Modified Capabilities
- None initially - this is a greenfield port

## Impact

### Code Impact
- New Rust codebase in `src/` directory
- Replace all TypeScript packages with Rust crates
- New Cargo workspace structure

### Dependencies
- **tokio**: Async runtime
- **tui-rs / ratatui**: Terminal UI rendering
- **reqwest**: HTTP client for LLM APIs
- **serde**: Serialization/deserialization
- **tracing**: Logging
- **anyhow / thiserror**: Error handling
- **async-openai / async-anthropic**: LLM SDKs

### Systems Impact
- Package manager changes (npm → cargo)
- Build system changes (turbo → cargo build)
- Test framework changes (vitest → tokio test)

### Breaking Changes
- **BREAKING**: Binary name changes from `opencode` to `opencode-rs` (or `ox` for brevity)
- **BREAKING**: Config file format may change from YAML to TOML
- **BREAKING**: API responses may have different JSON structure
