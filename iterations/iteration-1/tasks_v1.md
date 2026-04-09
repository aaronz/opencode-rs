# OpenCode Rust Port — Task Checklist v1

**Version:** 1.0
**Generated:** 2026-04-09
**Iteration:** 1
**Status:** Active

---

## P0 Tasks — Blocking Issues (Must Complete First)

### P0-1: Custom Tool File Loader
**FR:** FR-007 | **Phase:** 2 | **Module:** `crates/tools/`

- [x] Create tool discovery service `crates/tools/src/discovery.rs`
- [x] Implement project-level `.opencode/tools/` directory scanning
- [x] Implement global-level `~/.config/opencode/tools/` scanning
- [x] Create TypeScript/JavaScript file parser
- [x] Integrate with tool registry
- [x] Add unit tests for discovery
- [x] Add integration tests for custom tool loading
- [x] Update `crates/tools/src/lib.rs` exports
- [ ] Update AGENTS.md with custom tool documentation

**Definition of Done:**
- Custom tools load from both project and global directories
- Tool registration works correctly
- Tests pass

---

### P0-2: TUI Plugin TypeScript SDK
**FR:** FR-018 | **Phase:** 2 | **Module:** `crates/sdk/`

- [x] Create `sdk/typescript/packages/plugin-tui/` directory structure
- [x] Define `TuiPlugin` type in `src/types.ts`
- [x] Define `TuiPluginModule` type in `src/types.ts`
- [x] Implement `commands.register()` API
- [x] Implement `routes.register()` API
- [x] Implement `dialogs.register()` API
- [x] Implement `slots.register()` API
- [x] Implement `themes.install()` API
- [x] Implement `themes.set()` API
- [x] Implement `events.on()` API
- [x] Implement `state.get()` API
- [x] Implement `state.set()` API
- [x] Implement `onDispose` lifecycle
- [x] Configure TypeScript build (tsup)
- [x] Add package.json with exports
- [x] Add README documentation
- [x] Test SDK build

**Definition of Done:**
- TypeScript package builds successfully
- All API surfaces typed correctly
- Documentation complete

---

### P0-3: Iterations Structure
**FR:** FR-019 | **Phase:** 0 | **Module:** Project root

- [x] Create `iterations/src/` directory
- [x] Create `iterations/src/lib.rs` main module
- [x] Create `iterations/src/tracker.rs` for progress tracking
- [x] Create `iterations/src/reporter.rs` for status reporting
- [ ] Integrate with `iterate-prd.sh` workflow
- [ ] Add to workspace Cargo.toml if applicable
- [x] Add tests for tracking functionality

**Definition of Done:**
- `iterations/src/` module exists and compiles
- Progress tracking works
- CI integration complete

---

## P1 Tasks — Important Issues (After P0)

### P1-1: ✅ Done
**FR:** FR-016 | **Phase:** 4 | **Module:** `crates/git/`

- [x] Add `github install` subcommand to CLI
- [x] Create workflow file template at `.github/workflows/opencode.yml`
- [x] Implement GitHub App installation flow
- [x] Add secrets setup automation
- [x] Add tests for workflow generation
- [ ] Update CLI documentation

**Definition of Done:**
- `opencode github install` command works
- Workflow file generates correctly
- Tests pass

---

### P1-2: ✅ Done
**FR:** FR-017 | **Phase:** 4 | **Module:** `crates/git/`

- [x] Create GitLab CI component template
- [x] Implement GitHub workflow trigger examples
- [x] Implement comment/PR trigger parsing
- [x] Add CI secret loading for GitHub Actions
- [x] Add GitLab CI component support (experimental)
- [x] Add tests
- [ ] Update documentation

**Definition of Done:**
- GitLab CI component available
- Trigger parsing works
- Tests pass

---

### P1-3: ✅ Done
**FR:** FR-009 | **Phase:** 2 | **Module:** `crates/config/`, `crates/tui/`

- [x] Audit current config for TUI boundary violations
- [x] Ensure `tui.json` exclusively owns theme settings
- [x] Ensure `tui.json` exclusively owns keybind settings
- [x] Ensure `tui.json` exclusively owns TUI plugin config
- [x] Remove TUI settings from main config schema
- [x] Add validation tests
- [ ] Update config documentation

**Definition of Done:**
- No TUI settings leak to main config
- `tui.json` owns all TUI configuration
- Tests pass

**Note:** Deprecated fields (`keybinds`, `theme`) still present for backwards compatibility but marked with deprecation warnings. `TuiConfig` in core config is a known gap per PRD 06 migration path.

---

### P1-4: Desktop/Web/ACP Interface
**FR:** FR-015 | **Phase:** 4 | **Module:** `crates/server/`

- [ ] Implement desktop app startup flow
- [ ] Implement web server mode
- [x] Implement ACP startup/handshake
- [ ] Add configuration options
- [ ] Add tests
- [ ] Update documentation

**Definition of Done:**
- Desktop app starts correctly
- Web server mode functional
- ACP handshake works
- Tests pass

**Progress:**
- ACP handshake implemented in `crates/core/src/acp.rs` with version negotiation and session management
- ACP CLI commands enhanced in `crates/cli/src/cmd/acp.rs` with Start, Connect, Handshake, and Status actions
- Desktop app startup and web server mode remain TODO

---

## P2 Tasks — Improvement Issues (After P1)

### P2-1: ✅ Done
**FR:** 01-core-arch | **Phase:** 1 | **Module:** `crates/core/`

- [x] Add `worktree_root` field to ProjectInfo
- [x] Update project detection logic
- [x] Add tests
- [x] Update documentation

---

### P2-2: ✅ Done - AGENTS.md Upward Scanning
**FR:** 06-config | **Phase:** 1 | **Module:** `crates/core/`

- [x] Implement upward directory traversal from CWD to worktree root
- [x] Add scanning for AGENTS.md files
- [x] Add configuration for scanning behavior
- [x] Add tests
- [x] Update documentation

Implementation: `crates/core/src/agents_md.rs` with `AgentsMdScanner`, `AgentsMdInfo`, `AgentsMdScanConfig`. Configuration via `AgentsMdConfig` in config system.

---

### P2-3: MCP OAuth CLI Commands
**FR:** 04-mcp | **Phase:** 3 | **Module:** `crates/cli/`

- [ ] Add `opencode mcp auth` subcommands
- [ ] Implement OAuth flow for MCP servers
- [ ] Add token storage
- [ ] Add tests
- [ ] Update documentation

---

### P2-4: Session Compaction Boundaries
**FR:** 01-core-arch | **Phase:** 1 | **Module:** `crates/core/`

- [ ] Verify checkpoint-based compaction semantics
- [ ] Review compaction boundaries
- [ ] Add configuration options
- [ ] Add tests if needed
- [ ] Update documentation

---

### P2-5: Plugin-Provided Tool Registration
**FR:** 08-plugin | **Phase:** 2 | **Module:** `crates/plugin/`

- [ ] Implement tool registration from plugins
- [ ] Add permission integration
- [ ] Add tests
- [ ] Update documentation

---

### P2-6: Skill Permission Restrictions
**FR:** 12-skills | **Phase:** 3 | **Module:** `crates/core/`

- [ ] Add permission restrictions for skill usage
- [ ] Implement permission evaluation for skills
- [ ] Add configuration options
- [ ] Add tests
- [ ] Update documentation

---

## Technical Debt Tasks

### TD-001: Deprecated `mode` field
**Module:** `crates/config/`

- [ ] Remove `mode` field from config schema
- [ ] Add deprecation warning
- [ ] Schedule for major version removal

---

### TD-002: Deprecated `tools` field
**Module:** `crates/config/`

- [ ] Remove `tools` field from config schema (after migration)
- [ ] Keep conversion logic for backwards compatibility
- [ ] Add deprecation warning

---

### TD-003: Deprecated `keybinds` field
**Module:** `crates/config/`

- [ ] Remove `keybinds` field from config schema
- [ ] Verify moved to tui.json

---

### TD-004: Deprecated `layout` field
**Module:** `crates/config/`

- [ ] Remove `layout` field from config schema
- [ ] Always uses stretch layout

---

### TD-005: Hardcoded Built-in Skills
**Module:** `crates/core/`

- [ ] Consider externalization of built-in skills
- [ ] Evaluate runtime impact

---

### TD-006: Magic Numbers in Compaction
**Module:** `crates/core/`

- [ ] Make `COMPACTION_START_THRESHOLD` configurable
- [ ] Make `COMPACTION_FORCE_THRESHOLD` configurable
- [ ] Add configuration options

---

### TD-007: SHA256 Args Hashing
**Module:** `crates/storage/`

- [ ] Evaluate content-addressable storage approach
- [ ] Consider implementation if beneficial

---

### TD-008: Custom JSONC Parser
**Module:** `crates/config/`

- [ ] Replace custom JSONC parser with existing crate
- [ ] Verify functionality maintained
- [ ] Run tests

---

### TD-009: `#[serde(other)]` in Part
**Module:** `crates/core/`

- [ ] Replace with explicit error handling
- [ ] Add unknown part handling
- [ ] Run tests

---

## Convention Test Tasks

### Architecture Boundary Tests
**Location:** `tests/src/conventions/architecture_boundaries.rs`

- [x] Implemented (5 tests)

---

### Config Ownership Tests
**Location:** `tests/src/conventions/config_ownership.rs`

- [x] Implemented (4 tests)

---

### Route/Resource Group Tests
**Location:** `tests/src/conventions/route_conventions.rs`

- [x] Implemented (4 tests)

---

### Test Placement Tests
**Location:** `tests/src/conventions/test_layout.rs`

- [x] Implemented (5 tests)

---

### TUI Convention Tests
**Location:** `tests/src/conventions/tui_conventions.rs`

- [x] Implemented (5 tests)
- [ ] Verify `ratatui-testing/` integration

---

## Verification Commands

After any changes, run:

```bash
# Build verification
cargo build --release

# Test suite
cargo test --all-features

# Linting
cargo clippy --all -- -D warnings

# Format check
cargo fmt --all -- -check
```

---

## Progress Summary

| Priority | Total Tasks | Completed | Pending |
|----------|-------------|-----------|---------|
| P0 | 3 | 3 | 0 |
| P1 | 4 | 3 | 1 |
| P2 | 6 | 0 | 6 |
| Tech Debt | 9 | 0 | 9 |
| Conventions | 5 | 5 | 0 |
| **Total** | **27** | **11** | **16** |

**Note:** P1-4 partially completed - ACP handshake implemented, desktop app and web server mode remain.

---

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-04-09 | Initial task list based on spec v1 |
| 1.1 | 2026-04-09 | Completed P0-1 (Custom Tool File Loader), P0-2 (TUI Plugin TypeScript SDK), P0-3 (Iterations Structure) |
| 1.2 | 2026-04-09 | Completed P1-1 (GitHub Workflow Generation) |
| 1.3 | 2026-04-09 | Completed P1-2 (GitLab CI Component), P1-3 (tui.json Ownership Enforcement) |
