# Specification Document: OpenCode RS (Iteration 29)

**Date:** 2026-04-17
**Iteration:** 29
**Status:** Active Development
**Target:** Full PRD compliance + identified gap resolution

---

## 1. Overview

This document defines the specification for OpenCode RS based on Product Requirements Document (PRD) v1.0 and the Iteration 29 Gap Analysis. It serves as the authoritative reference for all features, implementation status, and prioritized improvements.

**Overall Implementation Status:** ~92% complete by feature count

**Implementation Summary:**
| Category | Status | Coverage |
|----------|--------|----------|
| Crate Structure | ✅ Complete | 15/15 crates implemented |
| P0 Features | ✅ Complete | All P0 requirements met |
| P1 Features | ✅ Complete | 10/10 implemented (2 partially) |
| P2 Features | ⚠️ Partial | 5/9 implemented, 4 out of scope |
| API Endpoints | ✅ Complete | 9/9 implemented |
| Data Models | ✅ Complete | 100% alignment |

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

**Gap Note:** Production code contains `unwrap()` in some tool implementations (see FR-017).

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

**Gap Note (FR-027):** TOML format deprecated, shows warning; needs migration tooling.

---

## 3. Non-Functional Requirements

### FR-017: Production unwrap() Elimination

**Priority:** P0
**Status:** ❌ Not Compliant (3484+ instances)

**Requirements:**
- Zero `.unwrap()` or `.expect()` in production code
- Use proper error propagation with `?`
- Provide meaningful error messages

**High-Risk Locations:**
| File | Line | Issue |
|------|------|-------|
| `crates/tools/src/edit.rs` | 159 | `let idx = index.unwrap();` |
| `crates/tools/src/web_search.rs` | 70 | `let api_key = api_key.unwrap();` |
| `crates/server/src/routes/*.rs` | Various | Untyped String errors |

**Audit Command:**
```bash
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l
```

---

### FR-018: Test Coverage Enforcement

**Priority:** P1
**Status:** ⚠️ Partial (No CI Gate)

**Requirements:**
- Minimum 80% line coverage for all crates
- Use `cargo-llvm-cov` for coverage reporting
- CI gate must fail below 80% threshold

**Current Coverage by Crate:**
| Crate | Current | Target | Delta |
|-------|---------|--------|-------|
| `core` | ~60% | 80%+ | +20% |
| `storage` | ~70% | 80%+ | +10% |
| `llm` | ~55% | 80%+ | +25% |
| `tools` | ~50% | 80%+ | +30% |
| `agent` | ~45% | 80%+ | +35% |
| `server` | ~40% | 80%+ | +40% |
| `tui` | ~60% | 80%+ | +20% |
| `plugin` | ~70% | 80%+ | +10% |
| `auth` | ~75% | 80%+ | +5% |
| `config` | ~70% | 80%+ | +10% |
| `cli` | ~50% | 80%+ | +30% |

---

### FR-019: Benchmark Suite

**Priority:** P2
**Status:** ⚠️ Exists but not in CI

**Requirements:**
- Performance benchmarks in `opencode-benches/`
- Run in CI for regression detection

**Implementation:**
- ✅ Benchmarks exist in `opencode-benches/`
- ❌ Not run in CI pipeline

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

## 4. Gap Analysis Summary

### P0 - Critical (None identified)

All P0 requirements from PRD are implemented.

### P1 - Should Have (Addressed)

| Gap Item | Module | Status | Recommendation |
|----------|--------|--------|----------------|
| WebSocket streaming | server | ⚠️ Verify ws module | FR-025 |
| SDK documentation | sdk | ⚠️ Add to CI | FR-026 |
| Plugin API stability | plugin | ⚠️ Define policy | FR-024 |

### P2 - Nice to Have

| Gap Item | Status | Recommendation |
|----------|--------|----------------|
| Benchmark in CI | ⚠️ Not in pipeline | Add to CI |
| Web UI | ⚠️ Future work | Out of scope per PRD |
| IDE Extensions | ⚠️ Out of scope | Separate packages |
| Formal verification | ❌ Out of scope | N/A |
| Team collaboration | ❌ Out of scope | N/A |

---

## 5. Technical Debt

### FR-023: Unsafe Code Audit

**Priority:** P2
**Status:** ⚠️ Needs SAFETY comments

**Unsafe Blocks Missing SAFETY Comments:**
| File | Line | Issue |
|------|------|-------|
| `crates/plugin/src/lib.rs` | 661 | Missing SAFETY comment |
| `crates/tui/src/app.rs` | 4677, 4690 | Missing SAFETY comments |
| `crates/server/src/routes/validation.rs` | 237, 256 | Missing SAFETY comments |

---

### FR-024: Plugin API Version Stability

**Priority:** P1
**Status:** ⚠️ No version policy defined

**Requirements:**
- Define plugin ABI version lifecycle
- Document compatibility guarantees
- Version stability policy for plugin ecosystem

---

### FR-025: WebSocket Capability Verification

**Priority:** P1
**Status:** ⚠️ Needs Verification

**Action:**
Verify `routes/ws.rs` provides full bidirectional WebSocket streaming vs SSE functionality.

---

### FR-026: SDK Documentation CI

**Priority:** P1
**Status:** ⚠️ Not in CI

**Action:**
Add `cargo doc --no-deps --all-features --no-deps` to CI pipeline.

---

### FR-027: TOML Config Deprecation

**Priority:** P2
**Status:** ⚠️ Shows warning, works

**Action:**
Provide migration tooling or auto-convert TOML to JSONC on load.

---

## 6. Visibility & Rust Conventions

### FR-028: Visibility Boundary Audit

**Priority:** P1
**Status:** ⚠️ Not Compliant (~3896+ pub declarations)

**Requirements:**
- Default to private visibility
- Use `pub(crate)` for internal crate sharing
- Only mark `pub` what is part of public API

---

### FR-029: Error Handling Standardization

**Priority:** P1
**Status:** ⚠️ Partial Compliance (~60%)

**Requirements:**
- All library crates must use `thiserror` for typed errors
- Application crates may use `anyhow` for flexible context
- Production code must not use `.unwrap()` or `.expect()`

---

## 7. Action Items

### P0 — Critical

| ID | Action | Files | Status |
|----|--------|-------|--------|
| FR-017 | Fix production unwrap() in `crates/tools/src/edit.rs:159` | edit.rs:159 | Not Started |
| FR-017 | Fix production unwrap() in `crates/tools/src/web_search.rs:70` | web_search.rs:70 | Not Started |
| FR-017 | Audit remaining production code for `.unwrap()` | All crates | Not Started |
| FR-029 | Convert `crates/server/src/routes/` String errors to thiserror | Route handlers | Not Started |

### P1 — High Priority

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-018 | Add `cargo-llvm-cov` CI gate | 0.5 day | Not Started |
| FR-018 | Increase coverage to 80%+ across all crates | 2-3 weeks | Not Started |
| FR-028 | Visibility audit across all crates | 3 days | Not Started |
| FR-024 | Define plugin API version stability policy | 1 day | Not Started |
| FR-025 | Verify WebSocket streaming capability | 1 day | Not Started |
| FR-026 | Add SDK documentation to CI | 0.5 day | Not Started |

### P2 — Medium Priority

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-023 | Add SAFETY comments to unsafe blocks | 1 day | Not Started |
| FR-019 | Add benchmark to CI pipeline | 0.5 day | Not Started |
| FR-027 | TOML config migration tooling | 2 days | Not Started |

---

## 8. Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test --all

# Build release
cargo build --release

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --fail-under-lines 80

# Count unwraps (excluding tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l

# Audit pub visibility
grep -n "pub fn\|pub struct\|pub enum" crates/core/src/ | head -50

# Audit unsafe blocks
grep -n "unsafe" crates/*/src/*.rs | grep -v "test"
```

---

## 9. Cross-References

- [PRD v1.0](./iterations/iteration-29/prd.md)
- [Gap Analysis](./iterations/iteration-29/gap-analysis.md)
- [AGENTS.md](../../AGENTS.md)
- [Cargo.toml](../../opencode-rust/Cargo.toml)
- [ratatui-testing](../../ratatui-testing/)

---

## 10. Change Log

| Version | Date | Changes |
|---------|------|---------|
| v29 | 2026-04-17 | Full PRD alignment; FR-001 to FR-029 defined; 92% implementation status; gap analysis incorporated |
| v28 | 2026-04-17 | Rust conventions compliance focus; FR-001 to FR-020 defined |

---

**End of Document**