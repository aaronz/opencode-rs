# Specification Document: OpenCode RS (Iteration 30)

**Date:** 2026-04-17
**Iteration:** 30
**Status:** Active Development
**Target:** Full PRD compliance + ratatui-testing crate documentation + gap resolution

---

## 1. Overview

This document defines the specification for OpenCode RS based on Product Requirements Document (PRD) v1.0 and the Iteration 30 Gap Analysis focusing on the `ratatui-testing` crate. It serves as the authoritative reference for all features, implementation status, and prioritized improvements.

**ratatui-testing Implementation Status:** ~95% complete

**Implementation Summary:**
| Category | Status | Coverage |
|----------|--------|----------|
| ratatui-testing Core Modules | ✅ Complete | 5/5 modules |
| ratatui-testing Extended Modules | ✅ Complete | 1/1 (DialogRenderTester) |
| Dependencies | ✅ Complete | 9/9 required present |
| Test Coverage | ✅ Complete | 160+ unit tests |

---

## 2. Feature Requirements (FR-XXX)

### FR-001: Crate Architecture

**Priority:** P0
**Status:** ✅ Complete

**Requirements:**
- 15 crates covering core, cli, llm, tools, agent, tui, lsp, storage, server, auth, permission, plugin, git, mcp, sdk

**Implementation:**

| Crate | Path | Status |
|-------|------|--------|
| `opencode-core` | `crates/core/` | ✅ |
| `opencode-cli` | `crates/cli/` | ✅ |
| `opencode-llm` | `crates/llm/` | ✅ |
| `opencode-tools` | `crates/tools/` | ✅ |
| `opencode-agent` | `crates/agent/` | ✅ |
| `opencode-tui` | `crates/tui/` | ✅ |
| `opencode-lsp` | `crates/lsp/` | ✅ |
| `opencode-storage` | `crates/storage/` | ✅ |
| `opencode-server` | `crates/server/` | ✅ |
| `opencode-auth` | `crates/auth/` | ✅ |
| `opencode-permission` | `crates/permission/` | ✅ |
| `opencode-plugin` | `crates/plugin/` | ✅ |
| `opencode-git` | `crates/git/` | ✅ |
| `opencode-mcp` | `crates/mcp/` | ✅ |
| `opencode-sdk` | `crates/sdk/` | ✅ |

---

### FR-002: LLM Provider Support

**Priority:** P0
**Status:** ✅ Complete (Extended)

**Requirements:**
- OpenAI (GPT-4, GPT-3.5)
- Anthropic Claude (Claude 3 Opus, Sonnet, Haiku)
- Ollama local (Llama2, Mistral, custom models)
- Configurable via environment variables or config files
- Streaming support for real-time responses

**Implementation:**
| Provider | Status | Models |
|----------|--------|--------|
| OpenAI | ✅ | GPT-4, GPT-3.5, GPT-4 Turbo, GPT-4o |
| Anthropic | ✅ | Claude 3 Opus, Sonnet, Haiku, Claude 3.5 |
| Ollama | ✅ | Llama2, Mistral, custom models |
| Azure OpenAI | ✅ | GPT-4, GPT-3.5 |
| Google Gemini | ✅ | Gemini Pro, Gemini Ultra |
| AWS Bedrock | ✅ | Claude, Llama, Titan |
| Cohere | ✅ | Command R, Command R+ |
| 20+ additional providers | ✅ | Extended provider support |

**Configuration:**
```bash
OPENCODE_LLM_PROVIDER=openai|anthropic|ollama|azure|gemini|bedrock|cohere
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
OLLAMA_BASE_URL=http://localhost:11434
```

---

### FR-003: Tool System

**Priority:** P0
**Status:** ✅ Complete (Extended)

**Built-in Tools:**

| Tool | Description | Priority | Status |
|------|-------------|----------|--------|
| `Read` | Read file contents with line range support | P0 | ✅ |
| `Write` | Create or overwrite files | P0 | ✅ |
| `Edit` | Apply targeted edits to files | P0 | ✅ |
| `Grep` | Search file contents with regex | P0 | ✅ |
| `Glob` | Find files by pattern | P1 | ✅ |
| `Git` | Git operations (status, diff, log, commit) | P1 | ✅ |
| `Bash` | Execute shell commands | P1 | ✅ |
| `WebSearch` | Search the web | P2 | ✅ |
| `Terminal` | Terminal operations | P1 | ✅ |
| `process` | Process management | P1 | ✅ |
| `http` | HTTP client operations | P2 | ✅ |

**Tool Registry:**
- ✅ Plugins can register custom tools
- ✅ Tools have names, descriptions, parameter schemas
- ✅ Permission system controls tool access

---

### FR-004: Agent Modes

**Priority:** P0
**Status:** ✅ Complete (Extended)

| Mode | Capabilities | Use Case | Status |
|------|--------------|----------|--------|
| `Build` | Full tool access, can modify files | Implementation | ✅ |
| `Plan` | Read-only, analysis only | Planning, review | ✅ |
| `General` | Search and research | Investigation | ✅ |
| `Expert` | Enhanced reasoning | Complex tasks | ✅ |
| `Review` | Code review focus | Code review | ✅ |
| `Debug` | Debugging assistance | Bug investigation | ✅ |

---

### FR-005: Session Management

**Priority:** P0
**Status:** ✅ Complete

**Requirements:**
- SQLite-based storage
- Session Types: Conversations, implementations, code reviews
- Resume: Continue interrupted sessions
- Export: JSON export/import for portability

**Data Model:**
```json
{
  "id": "uuid",
  "created_at": "timestamp",
  "updated_at": "timestamp",
  "mode": "build|plan|general|expert|review|debug",
  "messages": [
    {
      "role": "user|assistant|system",
      "content": "string",
      "timestamp": "timestamp"
    }
  ],
  "metadata": {}
}
```

**Implementation:**
- ✅ `Session` struct with all required fields
- ✅ `SessionRepository` trait for persistence
- ✅ SQLite storage backend
- ✅ JSON export/import support

---

### FR-006: User Interfaces

**Priority:** P0
**Status:** ✅ Complete

| Interface | Description | Status |
|-----------|-------------|--------|
| TUI | Interactive terminal UI with ratatui | ✅ |
| HTTP API | REST endpoints for remote access | ✅ |
| CLI | Shell-like command interface | ✅ |
| SDK | Rust library for programmatic access | ✅ |

**TUI Features:**
- ✅ Interactive command palette
- ✅ Session history browser
- ✅ Real-time output streaming
- ✅ Keybinding support

**HTTP API Endpoints:**
| Endpoint | Method | Status |
|----------|--------|--------|
| `/api/status` | GET | ✅ |
| `/api/session` | POST | ✅ |
| `/api/session/{id}` | GET | ✅ |
| `/api/session/{id}/execute` | POST | ✅ |
| `/api/session/{id}/history` | GET | ✅ |

**ACP Routes:**
| Route | Method | Status |
|-------|--------|--------|
| `/api/acp/status` | GET | ✅ |
| `/api/acp/handshake` | POST | ✅ |
| `/api/acp/connect` | POST | ✅ |
| `/api/acp/ack` | POST | ✅ |

---

### FR-007: MCP (Model Context Protocol)

**Priority:** P1
**Status:** ✅ Complete

**Requirements:**
- Connect to external MCP servers
- Tool discovery from MCP servers
- Remote MCP server connections
- Local MCP server support

**Implementation:**
- ✅ MCP client implementation
- ✅ MCP server implementation
- ✅ Tool discovery protocol
- ✅ Multiple concurrent connections

---

### FR-008: Plugin System

**Priority:** P1
**Status:** ✅ Complete

**Requirements:**
- WASM-based plugin runtime
- Plugin can register custom tools
- Plugin API stability (see gap note)

**Implementation:**
- ✅ WASM runtime integration
- ✅ Tool registration API
- ✅ Plugin lifecycle management

**Gap Note (FR-024):** Plugin API version stability policy not defined.

---

### FR-009: Permission System

**Priority:** P1
**Status:** ✅ Complete

**Requirements:**
- Role-based tool access
- Permission enforcement per endpoint
- Permission levels: read, write, admin

**Implementation:**
- ✅ `PermissionScope` enum
- ✅ Role-based access control
- ✅ Tool permission enforcement

---

### FR-010: Auth System

**Priority:** P1
**Status:** ✅ Complete

**Requirements:**
- JWT for API authentication
- Argon2/bcrypt for password hashing
- OAuth support

**Implementation:**
- ✅ JWT authentication
- ✅ Argon2/bcrypt password hashing
- ✅ OAuth 2.0 integration
- ✅ Saml authentication

---

### FR-011: Git Integration

**Priority:** P1
**Status:** ✅ Complete

**Requirements:**
- Git operations (status, diff, log, commit)
- GitHub integration
- GitLab integration

**Implementation:**
- ✅ `GitManager` for git operations
- ✅ GitHub client
- ✅ GitLab client

---

### FR-012: WebSocket Streaming

**Priority:** P1
**Status:** ⚠️ Partial - Requires Verification

**Requirements:**
- WebSocket support for real-time streaming
- SSE (Server-Sent Events) for streaming

**Implementation:**
- ✅ SSE implemented in `routes/stream.rs`
- ⚠️ `routes/ws.rs` module exists, full bidirectional streaming capability needs verification

**Gap Note (FR-025):** Verify ws module provides full WebSocket streaming vs SSE.

---

### FR-013: SDK Documentation

**Priority:** P1
**Status:** ⚠️ Partial

**Requirements:**
- Public API docs via `cargo doc --no-deps`
- docs.rs publishing consideration

**Implementation:**
- ✅ Doc comments on public API
- ❌ No `cargo doc --no-deps` in CI workflow
- ❌ No docs.rs publishing configured

**Gap Note (FR-026):** Add SDK documentation to CI pipeline.

---

### FR-014: LSP Integration

**Priority:** P1
**Status:** ⚠️ Partial

**Requirements:**
- Language Server Protocol integration
- IDE editor support (VSCode/Neovim extensions)

**Implementation:**
- ✅ `LspManager` for LSP lifecycle
- ✅ `LspClient` for protocol communication
- ✅ Built-in language server registry
- ⚠️ VSCode/Neovim extension packages not in scope (per PRD)

**Gap Note:** Server-side LSP is complete; client editor extensions are future work.

---

### FR-015: HTTP API Completeness

**Priority:** P1
**Status:** ✅ Complete

**All PRD-specified HTTP endpoints implemented:**
- ✅ `GET /api/status`
- ✅ `POST /api/session`
- ✅ `GET /api/session/{id}`
- ✅ `POST /api/session/{id}/execute`
- ✅ `GET /api/session/{id}/history`

---

### FR-016: Configuration System

**Priority:** P0
**Status:** ✅ Complete

**Environment Variables:**
| Variable | Status |
|----------|--------|
| `OPENCODE_LLM_PROVIDER` | ✅ |
| `OPENAI_API_KEY` | ✅ |
| `ANTHROPIC_API_KEY` | ✅ |
| `OLLAMA_BASE_URL` | ✅ |
| `OPENCODE_DB_PATH` | ✅ |

**Config File (config.toml/jsonc):**
| Section | Status |
|---------|--------|
| `[server]` | ✅ |
| `[server.desktop]` | ✅ |
| `[server.acp]` | ✅ |

---

## 3. ratatui-testing Crate Specification

### FR-030: PtySimulator Module

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/pty.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| PTY creation | Creates PTY master/slave pair on Unix | ✅ |
| Write operations | Writes strings to PTY slave | ✅ |
| Read operations | Reads output from PTY master with timeout | ✅ |
| Window resize | Resizes PTY window (cols/rows) | ✅ |
| KeyEvent injection | Injects KeyEvent via crossterm | ✅ |
| MouseEvent injection | Injects MouseEvent via crossterm | ✅ |
| Unix implementation | Fully functional on Unix | ✅ |
| Windows implementation | Stub with descriptive errors | ✅ |

**Public API:**
```rust
pub struct PtySimulator { /* ... */ }

impl PtySimulator {
    pub fn new() -> Result<Self>;
    pub fn write(&mut self, data: &str) -> Result<()>;
    pub fn read(&mut self, timeout: Duration) -> Result<String>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn inject_key_event(&mut self, key: KeyEvent) -> Result<()>;
    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>;
}
```

**Known Limitation:** Windows PTY support is a stub with descriptive errors (ConPTY implementation is difficult per PRD).

---

### FR-031: BufferDiff Module

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/diff.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| Cell-by-cell comparison | Compares two Buffers cell-by-cell | ✅ |
| Difference location | Reports exact x,y of differences | ✅ |
| Color ignoring | Supports ignoring foreground color | ✅ |
| Background ignoring | Supports ignoring background color | ✅ |
| Attribute ignoring | Supports ignoring attributes (bold, italic, etc.) | ✅ |
| Human-readable output | Provides human-readable diff output | ✅ |
| String comparison | String-based comparison (diff_str) | ✅ |
| Unit tests | 40+ unit tests covering all features | ✅ |

**Public API:**
```rust
pub struct BufferDiff { /* ... */ }

impl BufferDiff {
    pub fn compare(a: &Buffer, b: &Buffer) -> BufferDiffResult;
    pub fn compare_with_ignore(a: &Buffer, b: &Buffer, ignore: DiffIgnore) -> BufferDiffResult;
    pub fn diff_str(expected: &str, actual: &str) -> StringDiffResult;
}

pub struct DiffIgnore {
    pub ignore_fg: bool,
    pub ignore_bg: bool,
    pub ignore_attrs: bool,
}
```

---

### FR-032: StateTester Module

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/state.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| State capture | Captures serializable state to JSON | ✅ |
| Snapshot comparison | Compares current state to captured snapshot | ✅ |
| Mismatch reporting | Reports mismatches with JSON diff | ✅ |
| Named snapshots | Named snapshots support | ✅ |
| Terminal capture | Terminal state capture (from Buffer) | ✅ |
| Clear/remove API | Clear/remove snapshots API | ✅ |
| Unit tests | 30+ unit tests | ✅ |

**Public API:**
```rust
pub struct StateTester<S: Serialize> { /* ... */ }

impl<S: Serialize> StateTester<S> {
    pub fn new(state: S) -> Self;
    pub fn capture(&mut self) -> Result<()>;
    pub fn compare(&self, name: &str) -> Result<bool>;
    pub fn save_snapshot(&mut self, name: &str) -> Result<()>;
    pub fn load_snapshot(&self, name: &str) -> Result<S>;
    pub fn clear_snapshots(&mut self) -> Result<()>;
}
```

---

### FR-033: TestDsl Module

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/dsl.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| Widget rendering | Renders widget to Buffer | ✅ |
| Composition | Composes PTY, BufferDiff, StateTester | ✅ |
| Fluent API | Fluent API chains correctly | ✅ |
| Wait-for predicate | Wait-for predicate support | ✅ |
| Async wait support | Async wait support (`wait_for_async`) | ✅ |
| Snapshot integration | Snapshot save/load integration | ✅ |
| Unit tests | 70+ unit tests | ✅ |

**Public API:**
```rust
pub struct TestDsl { /* ... */ }

impl TestDsl {
    pub fn new() -> Self;
    pub fn with_backend(backend: T) -> Self;
    pub fn render<W: Widget>(&mut self, widget: &W) -> Result<&mut Self>;
    pub fn with_pty(self) -> PtySimulator;
    pub fn wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>
    where F: Fn() -> bool;
    pub fn wait_for_async<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>
    where F: Fn() -> future::Future<Output = bool>;
    pub fn poll_until<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>
    where F: FnMut() -> Option<T>;
    pub fn snapshot(&mut self, name: &str) -> Result<&mut Self>;
}
```

---

### FR-034: CliTester Module

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/cli.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| Process spawning | Spawns process with args | ✅ |
| Output capture | Captures stdout/stderr | ✅ |
| Exit code | Returns exit code | ✅ |
| Temp cleanup | Cleans up temp directories | ✅ |
| Working directory | Working directory support | ✅ |
| Environment vars | Environment variable support | ✅ |
| Output assertions | Output assertion helpers | ✅ |
| Unit tests | 20+ unit tests | ✅ |

**Public API:**
```rust
pub struct CliTester { /* ... */ }

pub struct ChildProcess { /* ... */ }

impl CliTester {
    pub fn new(cmd: &str, args: &[&str]) -> Self;
    pub fn working_dir(mut self, dir: PathBuf) -> Self;
    pub fn env(mut self, key: &str, val: &str) -> Self;
    pub fn run(&mut self) -> Result<ChildProcess>;
}

impl ChildProcess {
    pub fn exit_code(&self) -> Option<i32>;
    pub fn stdout(&self) -> &str;
    pub fn stderr(&self) -> &str;
    pub fn assert_success(&self) -> Result<()>;
    pub fn assert_output_contains(&self, text: &str) -> Result<()>;
}
```

---

### FR-035: Snapshot Management

**Priority:** P0
**Status:** ✅ Complete (100%)

**File:** `ratatui-testing/src/snapshot.rs`

**Requirements:**
| Feature | Description | Status |
|---------|-------------|--------|
| Buffer snapshots | Save/load Buffer snapshots | ✅ |
| Color serialization | Color/style serialization | ✅ |
| Configurable directory | Configurable directory via env var | ✅ |
| Path sanitization | Path sanitization | ✅ |
| Unit tests | 6+ unit tests | ✅ |

**Public API:**
```rust
pub struct SerializedBuffer {
    pub version: u32,  // Added in FR-038
    pub width: u16,
    pub height: u16,
    pub cells: Vec<SerializedCell>,
    pub area: Option<SerializedArea>,
}

impl SerializedBuffer {
    pub fn from_buffer(buffer: &Buffer) -> Self;
    pub fn to_buffer(&self) -> Buffer;
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
}
```

---

### FR-036: DialogRenderTester Module

**Priority:** P1
**Status:** ✅ Extra (Not in PRD - Approved Extension)

**File:** `ratatui-testing/src/dialog_tester.rs`

**Description:**
Extended module for dialog rendering testing, providing utilities for testing TUI dialogs. This module exceeds the original PRD scope but is approved as a useful extension.

**Features:**
| Feature | Description | Status |
|---------|-------------|--------|
| Border detection | Border detection in rendered dialogs | ✅ |
| Content checking | Content checking | ✅ |
| Title detection | Title detection | ✅ |
| Line counting | Line counting utilities | ✅ |

**Public API:**
```rust
pub struct DialogRenderTester { /* ... */ }

impl DialogRenderTester {
    pub fn new() -> Self;
    pub fn with_backend(backend: TestBackend) -> Self;
    pub fn has_border(&self) -> bool;
    pub fn get_title(&self) -> Option<String>;
    pub fn get_content_lines(&self) -> Vec<String>;
    pub fn assert_has_border(&self) -> Result<()>;
    pub fn assert_content_contains(&self, text: &str) -> Result<()>;
}
```

---

## 4. Non-Functional Requirements

### FR-017: Production unwrap() Elimination

**Priority:** P0
**Status:** ❌ Not Compliant (3484+ instances)

**Requirements:**
- Zero `.unwrap()` or `.expect()` in production code
- Use proper error propagation with `?`
- Provide meaningful error messages

---

### FR-018: Test Coverage Enforcement

**Priority:** P1
**Status:** ⚠️ Partial (No CI Gate)

**Requirements:**
- Minimum 80% line coverage for all crates
- Use `cargo-llvm-cov` for coverage reporting
- CI gate must fail below 80% threshold

---

### FR-019: Benchmark Suite

**Priority:** P2
**Status:** ⚠️ Exists but not in CI

**Requirements:**
- Performance benchmarks in `opencode-benches/`
- Run in CI for regression detection

---

### FR-020: CI Pipeline

**Priority:** P0
**Status:** ✅ Mostly Complete

| Stage | Command | Status |
|-------|---------|--------|
| Format check | `cargo fmt --all -- --check` | ✅ Pass |
| Clippy | `cargo clippy --all -- -D warnings` | ⚠️ Warnings exist |
| Unit tests | `cargo test --lib` | ✅ Pass |
| Integration | `cargo test --test '*'` | ✅ Pass |
| Build | `cargo build --release` | ✅ Pass |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | ❌ Not in CI |
| Audit | `cargo audit` | ❌ Not in CI |
| Deny | `cargo deny check` | ❌ Not in CI |
| Benchmarks | `cargo bench` | ❌ Not in CI |

---

### FR-021: Security Requirements

**Priority:** P0
**Status:** ✅ Compliant (~95%)

| Requirement | Status |
|-------------|--------|
| No hardcoded credentials | ✅ |
| Argon2/bcrypt password hashing | ✅ |
| AES-GCM for data at rest | ✅ |
| JWT for API authentication | ✅ |
| SQL injection prevention | ✅ |
| Path normalization | ⚠️ Needs verification |

---

### FR-022: Platform Compatibility

**Priority:** P0
**Status:** ✅ Complete

| Platform | Status |
|----------|--------|
| macOS | ✅ |
| Linux | ✅ |
| Windows | ✅ |
| Rust 1.70+ | ✅ |

---

## 5. Gap Analysis Summary

### P0 - Critical (None identified for ratatui-testing)

All ratatui-testing P0 requirements from PRD are implemented.

### P1 - High Priority Issues (ratatui-testing)

| Gap Item | Module | Status | Recommendation |
|----------|--------|--------|----------------|
| Windows PTY returns generic errors | pty.rs | Acknowledged limitation | See PRD documentation |

### P2 - Medium Priority Issues (ratatui-testing)

| Gap Item | Module | Status | Recommendation |
|----------|--------|--------|----------------|
| Missing `similar-asserts` dev-dep | Cargo.toml | Not critical | FR-037 |
| Extra module not in PRD | dialog_tester.rs | Approved extension | FR-036 |
| Missing async wait API in PRD | dsl.rs | Added | FR-033 |
| Snapshot versioning | snapshot.rs | Added | FR-038 |

---

## 6. Technical Debt (ratatui-testing)

### FR-037: Optional Dev Dependencies

**Priority:** P2
**Status:** ⚠️ Not Implemented

**Item:** Missing `similar-asserts = "1.5"` dev dependency

**Recommendation:** Add to dev-dependencies if visual snapshot diffing is desired.

---

### FR-038: Snapshot Format Versioning

**Priority:** P2
**Status:** ✅ Implemented

**Item:** Added `version` field to `SerializedBuffer` struct for forward compatibility.

**Implementation:**
```rust
pub struct SerializedBuffer {
    pub version: u32,  // Added for format migration
    pub width: u16,
    pub height: u16,
    pub cells: Vec<SerializedCell>,
    pub area: Option<SerializedArea>,
}
```

---

## 7. Dependencies Analysis (ratatui-testing)

### Required by PRD ✅ All Present

| Dependency | Version | Status |
|------------|---------|--------|
| ratatui | 0.28 | ✅ |
| crossterm | 0.28 | ✅ |
| portable-pty | 0.8 | ✅ |
| anyhow | 1.0 | ✅ |
| thiserror | 2.0 | ✅ |
| serde | 1.0 | ✅ |
| serde_json | 1.0 | ✅ |
| tempfile | 3.14 | ✅ |
| tokio | 1.45 | ✅ |

### Optional (Recommended for Enhanced Testing)

| Dependency | Version | Status |
|------------|---------|--------|
| similar-asserts | 1.5 | ⚠️ Not present (optional) |

---

## 8. File Structure (ratatui-testing)

### Expected (PRD) vs Actual

| Expected | Actual | Status |
|----------|--------|--------|
| Cargo.toml | Cargo.toml | ✅ |
| src/lib.rs | src/lib.rs | ✅ |
| src/pty.rs | src/pty.rs | ✅ |
| src/diff.rs | src/diff.rs | ✅ |
| src/state.rs | src/state.rs | ✅ |
| src/dsl.rs | src/dsl.rs | ✅ |
| src/cli.rs | src/cli.rs | ✅ |
| src/snapshot.rs | src/snapshot.rs | ✅ |
| - | src/dialog_tester.rs | ✅ Extra (Approved) |
| tests/pty_tests.rs | tests/pty_tests.rs | ✅ |
| tests/buffer_diff_tests.rs | tests/buffer_diff_tests.rs | ✅ |
| tests/state_tests.rs | tests/state_tests.rs | ✅ |
| tests/dsl_tests.rs | tests/dsl_tests.rs | ✅ |
| tests/integration_tests.rs | tests/integration_tests.rs | ✅ |
| - | tests/dialog_tests.rs | ✅ Extra |
| - | tests/snapshot_tests.rs | ✅ Extra |
| - | tests/dsl_integration_tests.rs | ✅ Extra |

---

## 9. Action Items

### P0 — Critical

| ID | Action | Status |
|----|--------|--------|
| FR-017 | Fix production unwrap() in all crates | Not Started |

### P1 — High Priority

| ID | Action | Status |
|----|--------|--------|
| FR-018 | Add `cargo-llvm-cov` CI gate | Not Started |
| FR-018 | Increase coverage to 80%+ across all crates | Not Started |
| FR-028 | Visibility audit across all crates | ✅ Done |
| FR-024 | Define plugin API version stability policy | Not Started |
| FR-025 | Verify WebSocket streaming capability | Not Started |
| FR-026 | Add SDK documentation to CI | Not Started |

### P2 — Medium Priority (ratatui-testing)

| ID | Action | Status |
|----|--------|--------|
| FR-037 | Add `similar-asserts` to dev-dependencies | Optional |
| FR-036 | Document DialogRenderTester as approved extension | Completed |
| FR-033 | Document `wait_for_async` in spec | Completed |
| FR-038 | Add version field to snapshot format | Completed |

---

## 10. Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test -p ratatui-testing --all

# Build release
cargo build --release

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --fail-under-lines 80

# Run ratatui-testing specific tests
cargo test -p ratatui-testing

# Count unwraps (excluding tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l
```

---

## 11. Cross-References

- [PRD v1.0](./iterations/iteration-29/prd.md)
- [Gap Analysis (Iteration 30)](./iterations/iteration-30/gap-analysis.md)
- [Gap Analysis (Iteration 29)](./iterations/iteration-29/gap-analysis.md)
- [AGENTS.md](../../AGENTS.md)
- [Cargo.toml](../../opencode-rust/Cargo.toml)
- [ratatui-testing](../../ratatui-testing/)

---

## 12. Change Log

| Version | Date | Changes |
|---------|------|---------|
| v30 | 2026-04-17 | Added FR-030 to FR-038 for ratatui-testing crate; DialogRenderTester approved as PRD extension; snapshot versioning implemented; async wait support documented |
| v29 | 2026-04-17 | Full PRD alignment; FR-001 to FR-029 defined; 92% implementation status |
| v28 | 2026-04-17 | Rust conventions compliance focus; FR-001 to FR-020 defined |

---

**End of Document**