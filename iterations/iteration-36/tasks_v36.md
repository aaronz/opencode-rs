# Task List v36 - Gap Closure

**Project:** opencode-rs
**Iteration:** 36
**Date:** 2026-04-20
**Status:** Draft
**Priority:** P1 Gaps First, then P2

---

## Phase 1: P1 High Priority

### G-001: Plugin WASM Binaries

**Status:** Draft
**Priority:** P1
**Module:** `plugin`
**Effort:** 1 day

**Tasks:**

- [x] **T-G001-1:** Create plugin directory structure ✅ Done
  - [ ] Create `plugins/` directory
  - [ ] Create `plugins/hello_world/` subdirectory
  - [ ] Create `plugins/hello_world/src/` subdirectory

- [x] **T-G001-2:** Create `plugins/hello_world/Cargo.toml` ✅ Done
  - [ ] Package name: `opencode-plugin-hello-world`
  - [ ] WASM target: `wasm32-wasi`
  - [ ] Dependencies: `wasmi`, `serde`, `serde_json`

- [ ] **T-G001-3:** Implement `plugins/hello_world/src/lib.rs`
  - [ ] `plugin_init() -> i32` function
  - [ ] `plugin_execute(command: *const u8, len: usize) -> i32` function
  - [ ] Basic tool registration example

- [x] **T-G001-4:** Create `plugins/hello_world/build.sh` ✅ Done
  - [x] Add shebang `#!/bin/bash`
  - [x] Build for `wasm32-wasi` target
  - [x] Output .wasm to `plugins/bin/`

- [x] **T-G001-5:** Create `scripts/build-plugins.sh` ✅ Done
  - [x] Build all plugins in `plugins/`
  - [x] Create output directory `plugins/bin/`
  - [x] Error handling for failed builds

- [x] **T-G001-6:** Update `crates/plugin/src/lib.rs` ✅ Done
  - [x] Add plugin loader for local .wasm files
  - [x] Example: load from `plugins/bin/*.wasm`
  - [x] Implement WASM module instantiation and function calling using wasmi
  - [x] Handle errors with OpenCodeError enum
  - [x] Add unit tests for invalid WASM handling
  - [x] Add unit tests for valid WASM loading
  - [x] Add integration test for plugin_execute

- [x] **T-G001-7:** Document plugin development in `CONTRIBUTING.md`
  - [ ] How to create a new plugin
  - [ ] How to build and load plugins
  - [ ] Plugin API reference

**Acceptance Criteria:**
- [ ] Example plugin compiles to .wasm
- [ ] `scripts/build-plugins.sh` works
- [ ] Plugin framework can load example plugin
- [ ] Documentation written

---

### G-002: SDK Usage Examples

**Status:** Draft
**Priority:** P1
**Module:** `sdk`
**Effort:** 2 days

**Tasks:**

- [x] **T-G002-1:** Create `crates/sdk/examples/` directory

- [ ] **T-G002-2:** Implement `examples/basic_usage.rs`
  - [ ] Create Client with Config
  - [ ] Create and execute session
  - [ ] Print response
  - [ ] Handle errors

- [ ] **T-G002-3:** Implement `examples/async_session.rs`
  - [ ] Create session with specific ID
  - [ ] Save session
  - [ ] Resume session
  - [ ] Export session to JSON
  - [ ] Import session from JSON

- [x] **T-G002-4:** Implement `examples/tool_execution.rs`
  - [ ] Register custom tool
  - [ ] Execute Read tool
  - [ ] Execute Write tool
  - [ ] Handle tool result

- [x] **T-G002-5:** Implement `examples/provider_config.rs`
  - [x] Configure OpenAI provider
  - [x] Configure Anthropic provider
  - [x] Configure Ollama provider
  - [x] Show environment variable setup

- [ ] **T-G002-6:** Verify all examples compile
  - [ ] `cargo build --examples` succeeds
  - [ ] Fix any compilation errors
  - [ ] Add examples to `Cargo.toml` manifest

- [ ] **T-G002-7:** Update `crates/sdk/README.md`
  - [ ] Add Badges (docs.rs, crates.io)
  - [ ] Add Installation section
  - [ ] Add Quick Start section
  - [ ] Link to examples

**Acceptance Criteria:**
- [ ] `examples/basic_usage.rs` works
- [ ] `examples/async_session.rs` works
- [ ] `examples/tool_execution.rs` works
- [ ] `examples/provider_config.rs` works
- [ ] README has badges and installation

---

## Phase 2: P2 Medium Priority

### G-003: Expand Git Operations

**Status:** Draft
**Priority:** P2
**Module:** `git`
**Effort:** 3 days

**Tasks:**

- [ ] **T-G003-1:** Add branch operations to `crates/git/src/lib.rs`
  - [ ] Add `Branch` variant to `GitOperation` enum
  - [ ] Add `Checkout` variant to `GitOperation` enum

- [ ] **T-G003-2:** Create `crates/git/src/branch.rs`
  - [ ] `git_branch_list() -> Result<Vec<String>>`
  - [ ] `git_branch_create(name: &str) -> Result<()>`
  - [ ] `git_branch_delete(name: &str) -> Result<()>`

- [ ] **T-G003-3:** Create `crates/git/src/checkout.rs`
  - [ ] `git_checkout(branch: &str) -> Result<()>`
  - [ ] `git_checkout_create(name: &str) -> Result<()>`

- [x] **T-G003-4:** Add merge operations
  - [x] `git_merge(branch: &str) -> Result<MergeResult>`
  - [x] Handle clean merge
  - [x] Handle conflicts (return conflict info)

- [ ] **T-G003-5:** Add rebase operations
  - [ ] `git_rebase(branch: &str) -> Result<()>`
  - [ ] `git_rebase_abort() -> Result<()>`
  - [ ] Handle conflicts

- [x] **T-G003-6:** Add stash operations
  - [x] `git_stash() -> Result<()>`
  - [x] `git_stash_pop() -> Result<()>`
  - [x] `git_stash_list() -> Result<Vec<StashEntry>>`
  - [x] `git_stash_drop(index: usize) -> Result<()>`

- [ ] **T-G003-7:** Add push/pull operations
  - [ ] `git_push(remote: Option<&str>) -> Result<PushResult>`
  - [ ] `git_pull(remote: Option<&str>) -> Result<PullResult>`
  - [ ] Handle upstream tracking

- [ ] **T-G003-8:** Add unit tests
  - [ ] Test branch creation/deletion
  - [ ] Test checkout
  - [ ] Test merge (clean and conflict)
  - [ ] Test stash
  - [ ] Test push/pull

**Acceptance Criteria:**
- [ ] `branch`, `checkout`, `merge`, `rebase` operations work
- [ ] `stash`, `push`, `pull` operations work
- [ ] Error handling for common failure cases
- [ ] Unit tests pass

---

### G-004: Expand LSP Capabilities

**Status:** Draft
**Priority:** P2
**Module:** `lsp`
**Effort:** 1 week

**Tasks:**

- [ ] **T-G004-1:** Create `crates/lsp/src/diagnostics.rs`
  - [ ] `Diagnostic` struct with range, severity, message
  - [ ] `publish_diagnostics` handler
  - [ ] Wire up to LSP client

- [x] **T-G004-2:** Create `crates/lsp/src/completion.rs`
  - [ ] `CompletionItem` struct
  - [ ] `completion` handler
  - [ ] Trigger character handling (`.`, `->`, `::`)
  - [ ] Context-aware filtering

- [ ] **T-G004-3:** Create `crates/lsp/src/references.rs`
  - [ ] `references` handler
  - [ ] Find all references to symbol
  - [ ] Include declaration
  - [ ] Workspace-wide or document-only scope

- [ ] **T-G004-4:** Update `crates/lsp/src/lib.rs`
  - [ ] Add new modules to lib
  - [ ] Wire up handlers
  - [ ] Update capability registration

- [ ] **T-G004-5:** Add unit tests
  - [ ] Test diagnostics publishing
  - [ ] Test completion items
  - [ ] Test reference finding

**Acceptance Criteria:**
- [ ] Diagnostics available for open files
- [ ] Completion suggestions work
- [ ] Find references returns all locations
- [ ] Unit tests pass

---

### G-005: Expand Benchmark Suite

**Status:** Draft
**Priority:** P2
**Module:** `benches`
**Effort:** 1 week

**Tasks:**

- [ ] **T-G005-1:** Create `opencode-benches/src/tool_execution.rs`
  - [ ] `bench_read_small_file()` - Read < 1KB file
  - [ ] `bench_read_large_file()` - Read > 1MB file
  - [ ] `bench_write_file()` - Write 1KB file
  - [ ] `bench_grep()` - Grep on 1000-line file
  - [ ] `bench_glob()` - Glob on 100 files

- [ ] **T-G005-2:** Create `opencode-benches/src/session_load.rs`
  - [ ] `bench_session_create()` - New session
  - [ ] `bench_session_save()` - Save to SQLite
  - [ ] `bench_session_resume()` - Load from SQLite
  - [ ] `bench_session_delete()` - Delete session

- [ ] **T-G005-3:** Create `opencode-benches/src/llm_roundtrip.rs`
  - [ ] `bench_llm_first_token()` - Time to first token
  - [ ] `bench_llm_full_response()` - Total response time
  - [ ] `bench_provider_openai()` - OpenAI latency
  - [ ] `bench_provider_anthropic()` - Anthropic latency

- [ ] **T-G005-4:** Update `opencode-benches/src/lib.rs`
  - [ ] Add new benchmark modules
  - [ ] Add criterion measurements
  - [ ] Add report generation

- [ ] **T-G005-5:** Verify benchmarks run
  - [ ] `cargo bench` completes
  - [ ] Report generated
  - [ ] No panics or errors

**Acceptance Criteria:**
- [ ] Tool execution benchmarks exist
- [ ] Session CRUD benchmarks exist
- [ ] LLM streaming benchmarks exist
- [ ] `cargo bench` succeeds

---

### G-006: WebSocket Streaming Verification

**Status:** Draft
**Priority:** P2
**Module:** `server`
**Effort:** 2 days

**Tasks:**

- [ ] **T-G006-1:** Create `tests/integration/test_websocket.rs`
  - [ ] `test_ws_connect()` - Basic connection
  - [ ] `test_ws_execute_stream()` - Execute with streaming
  - [ ] `test_ws_multiple_streams()` - Concurrent streams

- [ ] **T-G006-2:** Implement edge case tests
  - [ ] `test_ws_empty_response()` - Empty streaming response
  - [ ] `test_ws_long_response()` - Very long streaming response
  - [ ] `test_ws_connection_drop()` - Disconnect mid-stream

- [ ] **T-G006-3:** Add memory leak detection
  - [ ] Verify buffers are freed
  - [ ] No growing allocations on repeated streams

- [ ] **T-G006-4:** Update `crates/server/src/routes/execute/ws.rs`
  - [ ] Add debug logging for testing
  - [ ] Ensure proper cleanup on drop

**Acceptance Criteria:**
- [ ] WebSocket test file created
- [ ] Streaming test passes
- [ ] Edge case tests pass
- [ ] Memory leak tests pass

---

### G-007: Publish SDK to crates.io

**Status:** Draft
**Priority:** P2
**Module:** `sdk`
**Effort:** 1 day

**Tasks:**

- [ ] **T-G007-1:** Update `crates/sdk/Cargo.toml`
  - [ ] Add `version = "0.1.0"`
  - [ ] Add `license = "MIT"`
  - [ ] Add `description`
  - [ ] Add `repository` URL
  - [ ] Add `documentation` URL
  - [ ] Add `keywords` and `categories`
  - [ ] Replace workspace deps with specific versions

- [ ] **T-G007-2:** Update `crates/sdk/README.md`
  - [ ] Add crates.io badge
  - [ ] Add docs.rs badge
  - [ ] Add Installation section
  - [ ] Add Quick Start example
  - [ ] Link to full documentation

- [ ] **T-G007-3:** Verify dry-run publish
  - [ ] Run `cargo publish --dry-run`
  - [ ] Fix any warnings/errors
  - [ ] Verify crate metadata

**Acceptance Criteria:**
- [ ] `Cargo.toml` properly configured
- [ ] README has badges and installation
- [ ] `cargo publish --dry-run` succeeds
- [ ] Crates.io account ready (manual step)

---

### G-008: Consolidate Documentation

**Status:** Draft
**Priority:** P2
**Module:** `all`
**Effort:** 3 days

**Tasks:**

- [ ] **T-G008-1:** Create `docs/` directory structure
  - [ ] Create `docs/README.md`
  - [ ] Create `docs/getting-started.md`
  - [ ] Create `docs/sdk-guide.md`
  - [ ] Create `docs/plugin-dev.md`

- [ ] **T-G008-2:** Write `docs/README.md`
  - [ ] Documentation index
  - [ ] Quick links to each section
  - [ ] Link to API reference in crate docs

- [ ] **T-G008-3:** Write `docs/getting-started.md`
  - [ ] Installation instructions
  - [ ] Configuration guide
  - [ ] First session walkthrough
  - [ ] Basic commands

- [ ] **T-G008-4:** Write `docs/sdk-guide.md`
  - [ ] SDK overview
  - [ ] Installation via Cargo
  - [ ] Link to `crates/sdk/examples/`
  - [ ] API reference links

- [ ] **T-G008-5:** Write `docs/plugin-dev.md`
  - [ ] Plugin architecture overview
  - [ ] WASM setup instructions
  - [ ] `hello_world` example walkthrough
  - [ ] Building and loading plugins
  - [ ] Plugin API reference

- [ ] **T-G008-6:** Update crate READMEs with cross-links
  - [ ] `crates/sdk/README.md` - Link to `docs/sdk-guide.md`
  - [ ] `crates/plugin/README.md` - Link to `docs/plugin-dev.md`
  - [ ] Root `README.md` - Link to `docs/`

**Acceptance Criteria:**
- [ ] `docs/` directory with 4 files
- [ ] Getting started guide complete
- [ ] SDK guide with example links
- [ ] Plugin guide with working example
- [ ] Cross-links from crate READMEs

---

## Verification Checklist

For each completed gap:

- [ ] Code compiles without errors (`cargo build`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] Acceptance criteria checked
- [ ] Documentation updated if needed
- [ ] No hardcoded secrets or test data

---

## File Summary

### Files to CREATE (17 new files)

| File | Gap | Type |
|------|-----|------|
| `plugins/hello_world/Cargo.toml` | G-001 | TOML |
| `plugins/hello_world/src/lib.rs` | G-001 | Rust |
| `plugins/hello_world/build.sh` | G-001 | Shell |
| `scripts/build-plugins.sh` | G-001 | Shell |
| `crates/sdk/examples/basic_usage.rs` | G-002 | Rust |
| `crates/sdk/examples/async_session.rs` | G-002 | Rust |
| `crates/sdk/examples/tool_execution.rs` | G-002 | Rust |
| `crates/sdk/examples/provider_config.rs` | G-002 | Rust |
| `crates/git/src/branch.rs` | G-003 | Rust |
| `crates/git/src/remote.rs` | G-003 | Rust |
| `crates/lsp/src/diagnostics.rs` | G-004 | Rust |
| `crates/lsp/src/completion.rs` | G-004 | Rust |
| `crates/lsp/src/references.rs` | G-004 | Rust |
| `opencode-benches/src/tool_execution.rs` | G-005 | Rust |
| `opencode-benches/src/session_load.rs` | G-005 | Rust |
| `opencode-benches/src/llm_roundtrip.rs` | G-005 | Rust |
| `tests/integration/test_websocket.rs` | G-006 | Rust |
| `docs/getting-started.md` | G-008 | Markdown |
| `docs/sdk-guide.md` | G-008 | Markdown |
| `docs/plugin-dev.md` | G-008 | Markdown |

### Files to MODIFY (12 files)

| File | Gap | Changes |
|------|-----|---------|
| `crates/plugin/src/lib.rs` | G-001 | Plugin loader |
| `crates/sdk/src/lib.rs` | G-002 | Examples manifest |
| `crates/sdk/README.md` | G-002, G-007 | Badges, install |
| `crates/git/src/lib.rs` | G-003 | New operations |
| `crates/lsp/src/lib.rs` | G-004 | Expand capabilities |
| `opencode-benches/src/lib.rs` | G-005 | Add benchmarks |
| `crates/server/src/routes/execute/ws.rs` | G-006 | Debug logging |
| `crates/sdk/Cargo.toml` | G-007 | Publishing config |
| `CONTRIBUTING.md` | G-001 | Plugin docs |
| `docs/README.md` | G-008 | Documentation index |
| `crates/*/README.md` | G-008 | Cross-links |

---

## Task Statistics

| Category | Gaps | Tasks | Estimated Days |
|----------|------|-------|-----------------|
| P1 Tasks | 2 | 13 | 3 days |
| P2 Tasks | 6 | 35 | 13 days |
| **Total** | **8** | **48** | **16 days** |

---

## Post-Iteration Status

| Category | Before | After |
|----------|--------|-------|
| Plugin System | 85% | 95% |
| SDK | 90% | 100% |
| Git Integration | 80% | 100% |
| LSP Integration | 70% | 90% |
| Benchmark Suite | 50% | 80% |
| HTTP API (WebSocket) | 95% | 100% |
| Documentation | 75% | 90% |
| **Overall** | **87%** | **~94%** |

---

*Task List Version: 36*
*Created: 2026-04-20*
*Based on: spec_v36.md + gap-analysis.md (Iteration 36)*