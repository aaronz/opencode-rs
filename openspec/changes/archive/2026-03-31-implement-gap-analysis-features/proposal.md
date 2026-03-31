## Why

The Rust OpenCode port (`rust-opencode-port/`) is at ~78% feature completeness. The original gap analysis (`docs/gap-analysis-prd-vs-rust.md`) identified 12 missing critical features, but codebase exploration reveals **several features are already implemented** (agents, tools). The real gaps are in **streaming infrastructure, MCP protocol, and TUI input syntax**. This change addresses the actual missing capabilities to reach v1.0 release readiness.

## What Changes

- **Streaming Infrastructure**: Complete WebSocket and SSE streaming endpoints for real-time agent output
- **MCP Protocol**: Implement Model Context Protocol for external tool integration
- **TUI Input Syntax**: Add `@file`, `!shell`, and `/command` input parsing in the terminal UI
- **LSP Diagnostics**: Complete LSP tool implementation for code quality features
- **Provider Gap**: Add HuggingFace and AI21 LLM providers (low priority, v1.5+)

### Already Implemented (verified in codebase)

The following were listed as "missing" in the gap analysis but are **already implemented**:
- `ReviewAgent`, `RefactorAgent`, `DebugAgent` → `crates/agent/src/{review,refactor,debug}_agent.rs`
- `stat`, `move`, `delete` tools → `crates/tools/src/file_tools.rs` (as `file_stat`, `file_move`, `file_delete`)
- `git_log`, `git_show` tools → `crates/tools/src/git_tools.rs`

## Capabilities

### New Capabilities

- `streaming-websocket`: WebSocket streaming endpoint for real-time agent output
- `streaming-sse`: Server-Sent Events streaming for browser/CLI clients
- `mcp-protocol`: Model Context Protocol implementation for external tool integration
- `tui-input-syntax`: Terminal input parsing for `@file`, `!shell`, `/command` syntax
- `lsp-diagnostics`: Complete LSP diagnostics integration for code quality

### Modified Capabilities

- `server-api`: Add `/ws` and `/sse` endpoints (existing stubs in `crates/server/`)
- `lsp-tool`: Complete LSP diagnostics in `crates/tools/src/lsp_tool.rs`

## Impact

### Affected Crates

- `crates/server/` — Add WebSocket/SSE handlers
- `crates/tui/` — Input parsing for @/!/ syntax
- `crates/tools/` — Complete MCP tool, LSP diagnostics
- `crates/lsp/` — Diagnostics integration

### Dependencies

- WebSocket library (likely `tokio-tungstenite` or `axum` built-in)
- SSE support (likely `axum::response::Sse`)
- MCP protocol types (may need new crate or integration)

### Breaking Changes

- None expected — all additions are new features, not modifications to existing APIs
