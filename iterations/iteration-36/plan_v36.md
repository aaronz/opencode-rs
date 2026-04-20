# Implementation Plan v36 - Gap Closure

**Project:** opencode-rs
**Iteration:** 36
**Date:** 2026-04-20
**Status:** Draft
**Priority:** P1 Gaps First, then P2

---

## 1. Executive Summary

This plan addresses the remaining gaps identified in the Gap Analysis Report (Iteration 36). With P0 features fully implemented and overall implementation at ~87%, this iteration focuses on closing P1 gaps (Plugin WASM, SDK examples) and addressing key P2 gaps (Git/LSP expansion, benchmarks, WebSocket verification, SDK publishing, documentation consolidation).

### Key Constraints
- No subagents/task tools allowed
- All analysis must be done directly in current session
- Use only Read, Write, Edit, Grep, LSP tools

---

## 2. Priority Matrix

### P1 - High Priority (Should Fix)

| Gap ID | Issue | Module | Estimated Effort |
|--------|-------|--------|-----------------|
| G-001 | Plugin WASM binaries not in repo | `plugin` | 1 day |
| G-002 | SDK lacks usage examples | `sdk` | 2 days |

### P2 - Medium Priority (Nice to Have)

| Gap ID | Issue | Module | Estimated Effort |
|--------|-------|--------|-----------------|
| G-003 | Git tool missing branch/merge/rebase/stash/push/pull | `git` | 3 days |
| G-004 | LSP integration lacks diagnostics, completion, references | `lsp` | 1 week |
| G-005 | Benchmark suite has limited scenarios | `benches` | 1 week |
| G-006 | WebSocket streaming not verified end-to-end | `server` | 2 days |
| G-007 | SDK not published to crates.io | `sdk` | 1 day |
| G-008 | Documentation scattered across crates | `all` | 3 days |

---

## 3. Implementation Phases

### Phase 1: P1 High Priority

#### 1.1 G-001: Plugin WASM Binaries

**Objective:** Provide deployable WASM plugin binaries or build infrastructure

**Current Status:** WASM framework complete, but no .wasm files in repo

**Files to Create:**
- `plugins/hello_world/Cargo.toml` - Plugin manifest
- `plugins/hello_world/src/lib.rs` - Example plugin source
- `plugins/hello_world/build.sh` - Plugin build script
- `scripts/build-plugins.sh` - Build all plugins

**Files to Modify:**
- `crates/plugin/src/lib.rs` - Add plugin loader integration
- `CONTRIBUTING.md` - Document plugin development

**Implementation Steps:**

1. Create example plugin structure:
   ```bash
   plugins/
   ├── hello_world/
   │   ├── Cargo.toml
   │   ├── src/
   │   │   └── lib.rs
   │   └── build.sh
   ```

2. Implement minimal WASM plugin:
   ```rust
   use wasmi::{Engine, Linker, Module, Store};

   #[no_mangle]
   pub extern "C" fn plugin_init() -> i32 {
       // Initialize plugin
       0
   }

   #[no_mangle]
   pub extern "C" fn plugin_execute(command: *const u8, len: usize) -> i32 {
       // Execute plugin command
       0
   }
   ```

3. Create build script to compile to WASM:
   ```bash
   #!/bin/bash
   cargo build --target wasm32-wasi --release -p hello_world
   ```

4. Document plugin API for external developers

**Acceptance Criteria:**
- [ ] Example plugin source in `plugins/hello_world/`
- [ ] Build script produces working .wasm file
- [ ] Plugin framework can load the example plugin
- [ ] Documentation for plugin development

---

#### 1.2 G-002: SDK Usage Examples

**Objective:** Add comprehensive usage examples to `opencode-sdk/examples/`

**Current Status:** Public API exists in `opencode-sdk/src/lib.rs`, no examples

**Files to Create:**
- `crates/sdk/examples/basic_usage.rs` - Simple SDK usage
- `crates/sdk/examples/async_session.rs` - Async session management
- `crates/sdk/examples/tool_execution.rs` - Tool execution example
- `crates/sdk/examples/provider_config.rs` - LLM provider setup

**Implementation Steps:**

1. Create `crates/sdk/examples/` directory structure

2. Implement `basic_usage.rs`:
   ```rust
   use opencode_sdk::{Client, Config};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn Error>> {
       let config = Config::default();
       let client = Client::new(config)?;

       let session = client.create_session().await?;
       let response = session.execute("Hello, world!").await?;

       println!("Response: {}", response);
       Ok(())
   }
   ```

3. Implement `async_session.rs`:
   - Show session creation, save, resume, export
   - Demonstrate error handling

4. Implement `tool_execution.rs`:
   - Show how to register custom tools
   - Execute tools via SDK

5. Implement `provider_config.rs`:
   - Show different provider configurations
   - Environment variable setup

**Acceptance Criteria:**
- [ ] `examples/basic_usage.rs` - Simple hello world example
- [ ] `examples/async_session.rs` - Session management
- [ ] `examples/tool_execution.rs` - Tool execution
- [ ] `examples/provider_config.rs` - Provider configuration
- [ ] All examples compile and run
- [ ] Documentation in `crates/sdk/README.md`

---

### Phase 2: P2 Medium Priority

#### 2.1 G-003: Expand Git Operations

**Objective:** Add branch, merge, rebase, stash, push, pull to Git tool

**Current Status:** Git tool supports status, diff, log, commit only

**Files to Modify:**
- `crates/git/src/lib.rs` - Add new operations
- `crates/git/src/branch.rs` - New module for branch operations
- `crates/git/src/remote.rs` - New module for push/pull

**Implementation Steps:**

1. Add branch operations:
   ```rust
   pub enum GitOperation {
       Status,
       Diff,
       Log,
       Commit,
       Branch,      // NEW
       Checkout,    // NEW
       Merge,       // NEW
       Rebase,      // NEW
       Stash,       // NEW
       Push,        // NEW
       Pull,        // NEW
   }
   ```

2. Implement `branch.rs`:
   - `git branch` - List branches
   - `git branch <name>` - Create branch
   - `git branch -d <name>` - Delete branch

3. Implement `checkout.rs`:
   - `git checkout <branch>` - Switch branches
   - `git checkout -b <name>` - Create and switch

4. Implement `merge.rs`:
   - `git merge <branch>` - Merge branch
   - Handle merge conflicts (return conflict info)

5. Implement `rebase.rs`:
   - `git rebase <branch>` - Rebase onto branch
   - Handle interactive rebase flag

6. Implement `stash.rs`:
   - `git stash` - Stash changes
   - `git stash pop` - Apply and drop
   - `git stash list` - List stashes

7. Implement `push_pull.rs`:
   - `git push` - Push to remote
   - `git pull` - Pull from remote
   - Handle remote selection

**Acceptance Criteria:**
- [ ] `branch`, `checkout`, `merge` operations work
- [ ] `rebase`, `stash`, `push`, `pull` operations work
- [ ] Error handling for common failure cases
- [ ] Unit tests for all new operations

---

#### 2.2 G-004: Expand LSP Capabilities

**Objective:** Add diagnostics, completion, and references support

**Current Status:** Basic LSP integration only

**Files to Modify:**
- `crates/lsp/src/lib.rs` - Expand capabilities
- `crates/lsp/src/diagnostics.rs` - New module
- `crates/lsp/src/completion.rs` - New module
- `crates/lsp/src/references.rs` - New module

**Implementation Steps:**

1. Add diagnostics support:
   ```rust
   pub struct LspDiagnostics {
       pub file: PathBuf,
       pub diagnostics: Vec<Diagnostic>,
   }

   pub enum DiagnosticSeverity {
       Error,
       Warning,
       Information,
       Hint,
   }
   ```

2. Implement completion provider:
   - `textDocument/completion` - Provide completions
   - Trigger characters (`.`, `->`, `::`)
   - Filter based on cursor context

3. Implement references finder:
   - `textDocument/references` - Find all references
   - Include declaration
   - Filter by current document or workspace

4. Integrate with existing LSP client

**Acceptance Criteria:**
- [ ] Diagnostics available for open files
- [ ] Completion suggestions work
- [ ] Find references returns all locations
- [ ] Tests for new LSP features

---

#### 2.3 G-005: Expand Benchmark Suite

**Objective:** Add comprehensive benchmark scenarios

**Current Status:** `opencode-benches/` exists with limited scenarios

**Files to Create:**
- `opencode-benches/src/tool_execution.rs` - Tool benchmarks
- `opencode-benches/src/session_load.rs` - Session load benchmarks
- `opencode-benches/src/llm_roundtrip.rs` - LLM latency benchmarks

**Files to Modify:**
- `opencode-benches/src/lib.rs` - Add benchmarks

**Implementation Steps:**

1. Add tool execution benchmarks:
   ```rust
   #[tokio::test]
   async fn bench_read_file_small() {
       let tool = ReadTool::new();
       let params = json!({"path": "README.md"});
       let start = Instant::now();
       tool.execute(params).await;
       let duration = start.elapsed();
       crit::report("read_file_small", duration);
   }
   ```

2. Add session load benchmarks:
   - Session creation
   - Session save to SQLite
   - Session resume

3. Add LLM roundtrip benchmarks:
   - Time to first token (streaming)
   - Total response time
   - Provider comparison

**Acceptance Criteria:**
- [ ] Tool execution benchmarks (Read, Write, Grep)
- [ ] Session CRUD benchmarks
- [ ] LLM streaming latency benchmarks
- [ ] `cargo bench` runs successfully

---

#### 2.4 G-006: WebSocket Streaming Verification

**Objective:** Verify WebSocket streaming end-to-end

**Current Status:** WebSocket implemented, not verified

**Files to Modify:**
- `crates/server/src/routes/execute/ws.rs` - Add streaming tests
- `tests/integration/test_websocket.rs` - New integration test

**Implementation Steps:**

1. Create integration test:
   ```rust
   #[tokio::test]
   async fn test_websocket_streaming() {
       let server = TestServer::new();
       let client = WebSocketClient::connect(server.uri()).await;

       // Send execute request
       client.send_execute("say hello").await;

       // Verify streaming response
       let mut stream = client.receive_stream().await;
       let first_token = stream.next().await;
       assert!(first_token.is_some());

       // Verify complete response
       let full = stream.collect::<String>().await;
       assert!(!full.is_empty());
   }
   ```

2. Test edge cases:
   - Empty response
   - Very long response
   - Connection drop mid-stream
   - Multiple concurrent streams

**Acceptance Criteria:**
- [ ] WebSocket test file created
- [ ] Streaming test passes
- [ ] Edge case tests pass
- [ ] No memory leaks in streaming

---

#### 2.5 G-007: Publish SDK to crates.io

**Objective:** Publish `opencode-sdk` to crates.io

**Files to Modify:**
- `crates/sdk/Cargo.toml` - Add publishing config
- `crates/sdk/README.md` - Add badges, installation instructions

**Implementation Steps:**

1. Update `Cargo.toml`:
   ```toml
   [package]
   name = "opencode-sdk"
   version = "0.1.0"
   edition = "2021"
   license = "MIT"
   description = "Official Rust SDK for OpenCode RS"
   repository = "https://github.com/anomalyco/opencode-rs"
   documentation = "https://docs.opencode.ai/sdk/rust"
   keywords = ["opencode", "sdk", "ai", "coding"]
   categories = ["api-bindings", "development-tools"]

   [dependencies]
   # Update to specific versions, remove workspace deps
   ```

2. Update `README.md` with:
   - Crates.io badge
   - Installation instructions
   - Quick start example
   - Link to full documentation

3. Verify crate publishes cleanly:
   ```bash
   cargo publish --dry-run
   ```

**Acceptance Criteria:**
- [ ] `Cargo.toml` properly configured for publishing
- [ ] README has badges and installation
- [ ] `cargo publish --dry-run` succeeds
- [ ] Crates.io account ready for publish

---

#### 2.6 G-008: Consolidate Documentation

**Objective:** Create centralized documentation guide

**Files to Create:**
- `docs/README.md` - Central documentation index
- `docs/getting-started.md` - Quick start guide
- `docs/sdk-guide.md` - SDK documentation
- `docs/plugin-dev.md` - Plugin development guide

**Files to Modify:**
- `crates/*/README.md` - Add cross-links

**Implementation Steps:**

1. Create `docs/` directory structure

2. Write `docs/README.md`:
   ```markdown
   # OpenCode RS Documentation

   ## Quick Links
   - [Getting Started](getting-started.md)
   - [SDK Guide](sdk-guide.md)
   - [Plugin Development](plugin-dev.md)
   - [API Reference](../crates/*/src/lib.rs)
   ```

3. Write `docs/getting-started.md`:
   - Installation
   - Configuration
   - First session

4. Write `docs/sdk-guide.md`:
   - Basic usage
   - Examples link
   - API reference

5. Write `docs/plugin-dev.md`:
   - Plugin architecture
   - WASM setup
   - Example plugin walkthrough

**Acceptance Criteria:**
- [ ] `docs/` directory with 4 files
- [ ] Quick start guide complete
- [ ] SDK guide links to examples
- [ ] Plugin guide with working example
- [ ] Cross-links from crate READMEs

---

## 4. Technical Debt

| TD | Issue | Priority | Fix |
|----|-------|----------|-----|
| TD-001 | Plugin WASM files not in repo | High | Add example plugin (G-001) |
| TD-002 | SDK needs examples | Medium | Add examples (G-002) |
| TD-003 | Limited LSP tools | Medium | Expand LSP (G-004) |
| TD-004 | Git tool limited | Low | Expand Git (G-003) |
| TD-005 | No benchmarks | Low | Add benchmarks (G-005) |
| TD-006 | SDK not published | Low | Publish to crates.io (G-007) |

---

## 5. File Inventory

### Files to CREATE (15 new files)

| File | Purpose | Gap |
|------|---------|-----|
| `plugins/hello_world/Cargo.toml` | Plugin manifest | G-001 |
| `plugins/hello_world/src/lib.rs` | Plugin source | G-001 |
| `plugins/hello_world/build.sh` | Plugin build | G-001 |
| `scripts/build-plugins.sh` | Build all plugins | G-001 |
| `crates/sdk/examples/basic_usage.rs` | SDK example | G-002 |
| `crates/sdk/examples/async_session.rs` | SDK example | G-002 |
| `crates/sdk/examples/tool_execution.rs` | SDK example | G-002 |
| `crates/sdk/examples/provider_config.rs` | SDK example | G-002 |
| `crates/git/src/branch.rs` | Git branch ops | G-003 |
| `crates/git/src/remote.rs` | Git push/pull | G-003 |
| `crates/lsp/src/diagnostics.rs` | LSP diagnostics | G-004 |
| `crates/lsp/src/completion.rs` | LSP completion | G-004 |
| `crates/lsp/src/references.rs` | LSP references | G-004 |
| `opencode-benches/src/tool_execution.rs` | Benchmarks | G-005 |
| `docs/getting-started.md` | Documentation | G-008 |
| `docs/sdk-guide.md` | Documentation | G-008 |
| `docs/plugin-dev.md` | Documentation | G-008 |

### Files to MODIFY (12 files)

| File | Changes | Gap |
|------|---------|-----|
| `crates/plugin/src/lib.rs` | Plugin loader | G-001 |
| `crates/sdk/src/lib.rs` | Add examples manifest | G-002 |
| `crates/sdk/README.md` | Add badges, install | G-002, G-007 |
| `crates/git/src/lib.rs` | Add new operations | G-003 |
| `crates/lsp/src/lib.rs` | Expand capabilities | G-004 |
| `opencode-benches/src/lib.rs` | Add benchmarks | G-005 |
| `crates/server/src/routes/execute/ws.rs` | Streaming tests | G-006 |
| `tests/integration/test_websocket.rs` | WS integration test | G-006 |
| `crates/sdk/Cargo.toml` | Publishing config | G-007 |
| `CONTRIBUTING.md` | Plugin development | G-001 |
| `docs/README.md` | Documentation index | G-008 |
| `crates/*/README.md` | Cross-links | G-008 |

---

## 6. Dependencies

### Internal Dependencies
- G-002 examples depend on G-007 (SDK being publishable)
- G-008 docs depend on G-001, G-002 completion

### External Dependencies
- `wasm32-wasi` target for plugin compilation (G-001)
- `tokio-test` for benchmarks (already in workspace)
- `crates.io` account for publishing (G-007)

---

## 7. Testing Strategy

### Unit Tests
- Git operations: branch, merge, rebase, stash, push, pull
- LSP: diagnostics, completion, references
- SDK examples: compile-only smoke tests

### Integration Tests
- WebSocket streaming (G-006)
- Plugin loading (G-001)
- SDK examples run end-to-end

### Benchmark Tests
- Tool execution latency
- Session CRUD throughput
- LLM streaming time-to-first-token

---

## 8. Timeline Estimate

| Phase | Tasks | Total Effort |
|-------|-------|--------------|
| Phase 1 (P1) | G-001, G-002 | 3 days |
| Phase 2 (P2) | G-003, G-004, G-005, G-006, G-007, G-008 | 13 days |
| **Total** | **8 gaps** | **16 days** |

---

## 9. Verification

Before marking each gap complete:
1. All relevant tests pass (`cargo test`)
2. Code compiles without warnings (`cargo clippy -- -D warnings`)
3. Acceptance criteria checked off
4. Documentation updated if needed

---

## 10. Post-Iteration Status

After closing all gaps:

| Category | Before | After |
|----------|--------|-------|
| Plugin System | 85% | 95% |
| SDK | 90% | 100% |
| Git Integration | 80% | 100% |
| LSP Integration | 70% | 90% |
| Benchmark Suite | 50% | 80% |
| HTTP API | 95% | 100% |
| Documentation | 75% | 90% |
| **Overall** | **87%** | **~94%** |

---

*Plan Version: 36*
*Created: 2026-04-20*
*Based on: spec_v36.md + gap-analysis.md (Iteration 36)*