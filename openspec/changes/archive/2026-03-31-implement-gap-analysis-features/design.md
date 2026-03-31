## Context

The Rust OpenCode port has a solid foundation with 14 crates covering agents, tools, LLM providers, and basic server APIs. The architecture uses async Rust patterns (tokio) with a trait-based plugin system for agents and tools.

Current state:
- REST APIs complete (`crates/server/`)
- WebSocket/SSE endpoints exist as **stubs**
- MCP protocol exists as **stub** only
- TUI input parsing doesn't handle `@file`, `!shell`, `/command` syntax
- LSP tool has basic structure but diagnostics incomplete

Constraints:
- Must maintain compatibility with existing REST APIs
- WebSocket/SSE should reuse existing session/agent infrastructure
- TUI changes must not break existing keyboard input flow

## Goals / Non-Goals

**Goals:**
1. Complete WebSocket streaming for real-time agent output
2. Complete SSE streaming for browser/CLI clients
3. Implement MCP protocol for external tool integration
4. Add TUI input syntax for file inclusion, shell execution, and commands
5. Complete LSP diagnostics for code quality feedback

**Non-Goals:**
- OAuth/GitHub integration (v1.5+)
- Additional LLM providers (HuggingFace, AI21) - low priority
- Performance optimization (separate effort)
- Breaking changes to existing APIs

## Decisions

### Decision 1: WebSocket Implementation

**Choice**: Use `axum::extract::ws` (already in dependencies via axum)

**Rationale**:
- Axum is already the HTTP framework in use
- Built-in WebSocket support, no new dependencies
- Tokio-native, async-first design

**Alternatives considered**:
- `tokio-tungstenite`: More flexible but adds dependency
- Raw hyper: Too low-level

**Approach**:
- Add `ws_handler` to existing router in `crates/server/src/router.rs`
- WebSocket connections create a streaming session
- Agent output streams as JSON messages over the socket
- Reuse existing `Session` and `Agent` types

### Decision 2: SSE Implementation

**Choice**: Use `axum::response::Sse` with `tokio-stream`

**Rationale**:
- Lighter than WebSocket for one-way streaming
- Better for browser EventSource API
- Axum has built-in SSE support

**Approach**:
- Add `/sse/:session_id` endpoint
- Stream agent output as `data:` events
- Support `Last-Event-ID` for reconnection

### Decision 3: MCP Protocol

**Choice**: Implement MCP JSON-RPC 2.0 protocol

**Rationale**:
- MCP is the emerging standard for LLM tool integration
- JSON-RPC 2.0 is well-defined and testable
- Enables external tool servers

**Approach**:
- Create `crates/mcp/` crate for protocol types
- Add MCP server in `crates/server/src/mcp.rs`
- Implement tool discovery, execution, and result streaming
- Bridge MCP tools to existing `ToolRegistry`

### Decision 4: TUI Input Syntax

**Choice**: Parse input in `crates/tui/src/input.rs` before sending to agent

**Rationale**:
- Early parsing allows syntax-specific handling
- Doesn't require agent changes
- Clean separation of concerns

**Syntax**:
- `@file.txt` → Inject file contents into context
- `!command` → Execute shell command, inject output
- `/command` → Execute TUI command (e.g., `/help`, `/clear`)

**Approach**:
- Add input parser that detects prefix characters
- `@` → Read file, append to message
- `!` → Run via `BashTool`, append output
- `/` → Route to command handler

### Decision 5: LSP Diagnostics

**Choice**: Complete existing `LSPTool` with diagnostics

**Rationale**:
- `crates/lsp/` has foundation in place
- Need to connect LSP client to tool output
- Diagnostics are the most useful LSP feature for coding agents

**Approach**:
- Extend `LSPTool` to support `diagnostics` action
- Connect to LSP client in `crates/lsp/`
- Return diagnostics as structured JSON

## Risks / Trade-offs

### Risk: WebSocket Connection Management
- **Risk**: Memory leaks from unclosed connections
- **Mitigation**: Implement connection timeout, heartbeat, and cleanup on disconnect

### Risk: MCP Protocol Complexity
- **Risk**: MCP spec is evolving, implementation may need updates
- **Mitigation**: Start with core protocol (tools), avoid premature optimization

### Risk: TUI Input Parsing Edge Cases
- **Risk**: `@`, `!`, `!` in legitimate messages
- **Mitigation**: Support escape syntax (`\@`, `\!`), make prefixes configurable

### Risk: LSP Performance
- **Risk**: LSP queries can be slow for large codebases
- **Mitigation**: Cache diagnostics, implement timeout, allow opt-out

## Migration Plan

1. **Phase 1**: WebSocket streaming (highest value, lowest risk)
2. **Phase 2**: SSE streaming (similar to WebSocket)
3. **Phase 3**: TUI input syntax (isolated to TUI crate)
4. **Phase 4**: LSP diagnostics (extends existing tool)
5. **Phase 5**: MCP protocol (largest scope, do last)

Rollback: Each phase is independent. Can ship partial completion.

## Open Questions

1. **WebSocket authentication**: Should WS connections require auth tokens?
   - Recommendation: Yes, reuse existing session auth

2. **MCP tool discovery**: Static config or dynamic discovery?
   - Recommendation: Start with static config, add discovery later

3. **TUI command registry**: Extend existing command system or new registry?
   - Recommendation: Extend existing in `crates/tui/src/commands.rs`
