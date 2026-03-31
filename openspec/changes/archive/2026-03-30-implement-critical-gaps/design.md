## Context

The Rust implementation (`rust-opencode-port/`) is a Cargo workspace with 14 crates covering agents, tools, LLM providers, server, and TUI components. The current implementation has critical gaps that block the MVP release:

- **Agents**: 7/10 implemented (missing ReviewAgent, RefactorAgent, DebugAgent)
- **Streaming**: WebSocket/SSE endpoints exist as stubs only
- **Tools**: Missing stat, move, delete, git_log, git_show
- **LSP**: Diagnostics incomplete

The codebase follows a modular architecture with separate crates for agents, tools, server, and LLM providers. Existing agents (BuildAgent, PlanAgent, GeneralAgent, ExploreAgent, CompactionAgent, TitleAgent, SummaryAgent) provide templates for new implementations.

## Goals / Non-Goals

**Goals:**
1. Implement ReviewAgent, RefactorAgent, DebugAgent as PRD-required agent types
2. Complete WebSocket/SSE streaming infrastructure for real-time responses
3. Add missing file system tools (stat, move, delete)
4. Add missing Git tools (git_log, git_show)
5. Complete LSP diagnostics implementation

**Non-Goals:**
- OAuth login and GitHub integration (v1.5+ features)
- Additional LLM providers (HuggingFace, AI21)
- MCP protocol full implementation
- TUI input syntax (@/!/commands)

## Decisions

### 1. Agent Architecture

**Decision**: Create a shared base agent trait/struct in `crates/agent` that all agents extend.

**Rationale**: Existing agents follow similar patterns. A shared base reduces code duplication and ensures consistent behavior (prompt handling, tool execution, state management).

**Alternative**: Each agent as completely standalone - Rejected due to maintenance burden.

### 2. Streaming Implementation

**Decision**: Use `tokio-tungstenite` for WebSocket and `axum` SSE for Server-Sent Events.

**Rationale**: 
- `tokio-tungstenite` is the standard for WebSocket in Rust async ecosystem
- `axum` already used for REST endpoints; SSE integrates naturally
- Both support the existing async runtime

**Alternative**: Use `websocket` crate - Rejected; less maintained and fewer Tokio integrations.

### 3. Tool Implementation Pattern

**Decision**: Follow existing tool patterns in `crates/tools` using the `Tool` trait.

**Rationale**: Ensures consistency with existing 26 tools. New tools integrate seamlessly with permission system and execution framework.

### 4. LSP Diagnostics

**Decision**: Implement full LSP diagnostics using the existing LSP client in `crates/lsp`.

**Rationale**: Reuses existing LSP infrastructure. Diagnostics are essential for code quality checks in ReviewAgent.

## Risks / Trade-offs

- **[Risk]** WebSocket streaming complexity → **Mitigation**: Start with SSE (simpler), then add WebSocket
- **[Risk]** Agent prompt engineering for Review/Refactor/Debug → **Mitigation**: Leverage existing agent patterns, iterate on prompts
- **[Risk]** LSP diagnostics performance → **Mitigation**: Cache diagnostics, lazy loading for large projects
- **[Trade-off]** Scope: Implementing all features in one release → **Mitigation**: Prioritize Critical gaps first (agents + streaming)

## Migration Plan

1. **Phase 1**: Implement 3 missing agents (ReviewAgent, RefactorAgent, DebugAgent)
2. **Phase 2**: Complete WebSocket/SSE streaming infrastructure
3. **Phase 3**: Add missing tools (stat, move, delete, git_log, git_show)
4. **Phase 4**: Complete LSP diagnostics

No migration needed - all net new functionality.
