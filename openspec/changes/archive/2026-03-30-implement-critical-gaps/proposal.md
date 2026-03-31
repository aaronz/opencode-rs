## Why

The Rust implementation (`rust-opencode-port/`) currently covers ~65-70% of PRD v1.1 requirements. Critical gaps block the MVP release: 3 essential agent types are missing, streaming infrastructure is incomplete, and several core tools are absent. Addressing these gaps is essential for v1 release readiness.

## What Changes

1. **Add ReviewAgent** - Provides code review functionality with AI-powered analysis
2. **Add RefactorAgent** - Offers intelligent code refactoring suggestions and implementation
3. **Add DebugAgent** - Enables debugging assistance with error analysis and fix suggestions
4. **Implement WebSocket/SSE Streaming** - Real-time streaming responses for better UX
5. **Add Missing File System Tools** - stat, move, delete file operations
6. **Add Missing Git Tools** - git_log, git_show for extended Git operations
7. **Implement LSP Diagnostics** - Complete code diagnostics via LSP protocol

## Capabilities

### New Capabilities

- `review-agent`: AI-powered code review agent that analyzes code changes, identifies issues, and provides improvement suggestions
- `refactor-agent`: Intelligent refactoring agent that suggests and applies code improvements
- `debug-agent`: Debugging assistant that analyzes errors and provides fix recommendations
- `websocket-streaming`: Real-time WebSocket and SSE streaming support for agent responses
- `file-operations-extended`: Extended file operations including stat, move, delete
- `git-extended`: Extended Git operations including log and show commands
- `lsp-diagnostics`: Complete LSP diagnostics implementation for code quality checks

### Modified Capabilities

(None - all new capabilities)

## Impact

- **New Agents**: New agent implementations in `rust-opencode-port/src/agents/`
- **Streaming**: New WebSocket/SSE server infrastructure in `rust-opencode-port/src/server/`
- **Tools**: New tool implementations in `rust-opencode-port/src/tools/`
- **LSP**: Enhanced LSP client in `rust-opencode-port/src/lsp/`
- **API**: New streaming endpoints added to server API surface
