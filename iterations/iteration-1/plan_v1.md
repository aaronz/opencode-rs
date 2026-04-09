# OpenCode Rust Port — Implementation Plan v1

**Version:** 1.0
**Generated:** 2026-04-09
**Based on:** Spec v1 and Gap Analysis
**Status:** Active

---

## 1. Overview

This plan outlines the implementation roadmap for iteration 1, prioritizing P0 blocking issues and building toward Phase 4-6 completion.

## 2. Priority Order

### P0 — Blocking Issues (Must Complete First)

| ID | Issue | Module | FR | Target Phase |
|----|-------|--------|-----|--------------|
| P0-1 | Custom tool file loader | tools | FR-007 | Phase 2 |
| P0-2 | TUI Plugin TypeScript SDK | tui, sdk | FR-018 | Phase 2 |
| P0-3 | Iterations structure | project | FR-019 | Phase 0 |

### P1 — Important Issues (After P0)

| ID | Issue | Module | FR | Target Phase |
|----|-------|--------|-----|--------------|
| P1-1 | GitHub workflow generation | git | FR-016 | Phase 4 |
| P1-2 | GitLab CI component | git | FR-017 | Phase 4 |
| P1-3 | tui.json ownership enforcement | config, tui | FR-009 | Phase 2 |
| P1-4 | Desktop/web/ACP interface | server | FR-015 | Phase 4 |

### P2 — Improvements (After P1)

| ID | Issue | Module | PRD | Target Phase |
|----|-------|--------|-----|--------------|
| P2-1 | VCS worktree root distinction | core | 01-core-arch | Phase 1 |
| P2-2 | AGENTS.md upward scanning | core | 06-config | Phase 1 |
| P2-3 | MCP OAuth CLI commands | cli | 04-mcp | Phase 3 |
| P2-4 | Session compaction boundaries | core | 01-core-arch | Phase 1 |
| P2-5 | Plugin-provided tool registration | plugin | 08-plugin | Phase 2 |
| P2-6 | Skill permission restrictions | core | 12-skills | Phase 3 |

---

## 3. Phase-Based Implementation Plan

### Phase 0: Project Foundation
**Status:** 100% Complete

- [x] Workspace builds
- [x] Tests run
- [x] Clippy clean
- [ ] **P0-3: Iterations structure** ← TODO: Create `iterations/src/` module

### Phase 1: Authority Implementation
**Status:** 90% Complete

Completed:
- [x] Core entity model (FR-001)
- [x] Storage layer (FR-002)
- [x] Config system (FR-003)
- [x] HTTP API surface (FR-004)

Pending P2 work:
- [ ] P2-1: VCS worktree root distinction
- [ ] P2-2: AGENTS.md upward scanning
- [ ] P2-4: Session compaction boundaries verification

### Phase 2: Runtime Core
**Status:** 85% Complete

Completed:
- [x] Agent system (FR-005)
- [x] Plugin system (FR-008)

In Progress:
- [ ] **P0-1: Custom tool file loader** (FR-007)
- [ ] **P0-2: TUI Plugin TypeScript SDK** (FR-018)

Pending:
- [ ] P1-3: tui.json ownership enforcement (FR-009)
- [ ] P2-5: Plugin-provided tool registration

### Phase 3: Infrastructure Subsystems
**Status:** 80% Complete

Completed:
- [x] MCP integration (FR-010)
- [x] LSP integration (FR-011)
- [x] Provider/model system (FR-012)
- [x] Formatters (FR-013)
- [x] Skills system (FR-014)

Pending P2 work:
- [x] P2-3: MCP OAuth CLI commands
- [ ] P2-6: Skill permission restrictions

### Phase 4: Interface Implementations
**Status:** 60% Complete

Pending:
- [ ] **P1-1: GitHub workflow generation** (FR-016)
- [ ] **P1-2: GitLab CI component** (FR-017)
- [ ] **P1-4: Desktop/web/ACP interface** (FR-015)

### Phase 5: Hardening
**Status:** 70% Complete

Completed:
- [x] Phase 5b: Conventions suite green

Pending:
- [ ] Phase 5a: Compatibility suite green

### Phase 6: Release Qualification
**Status:** 0% Pending

- [ ] Non-functional baselines recorded
- [ ] Interface smoke workflows pass

---

## 4. P0 Implementation Details

### P0-1: Custom Tool File Loader (FR-007)

**Module:** `crates/tools/`

**Requirements:**
- Project-level: `.opencode/tools/` directory
- Global-level: `~/.config/opencode/tools/` directory
- File-based tool registration to registry
- Tool definition format: TypeScript/JavaScript files

**Implementation Steps:**
1. Create tool discovery service in `crates/tools/src/discovery.rs`
2. Implement directory scanning for `.opencode/tools/`
3. Implement global path scanning for `~/.config/opencode/tools/`
4. Create TypeScript/JavaScript file parser
5. Integrate with tool registry
6. Add tests

**Deliverables:**
- `crates/tools/src/discovery.rs` — Tool discovery service
- File-based tool registration to registry
- Unit tests for discovery

---

### P0-2: TUI Plugin TypeScript SDK (FR-018)

**Module:** `crates/sdk/`

**Requirements:**
- `@opencode-ai/plugin/tui` package
- `TuiPlugin` type definition
- `TuiPluginModule` type definition
- API surface: commands, routes, dialogs, slots, themes, events, state

**Implementation Steps:**
1. Create TypeScript package structure in `sdk/typescript/`
2. Define `TuiPlugin` and `TuiPluginModule` types
3. Implement API surface:
   - `commands.register()`
   - `routes.register()`
   - `dialogs.register()`
   - `slots.register()`
   - `themes.install()` / `themes.set()`
   - `events.on()`
   - `state.get()` / `state.set()`
4. Implement `onDispose` lifecycle
5. Add build tooling (tsup or similar)
6. Publish to npm

**Deliverables:**
- `sdk/typescript/packages/plugin-tui/` — TypeScript SDK
- Type definitions for all API surfaces
- Build configuration
- Documentation

---

### P0-3: Iterations Structure (FR-019)

**Module:** Project root

**Requirements:**
- Create `iterations/src/` module structure
- Align with `iterate-prd.sh` workflow
- Track implementation progress per iteration

**Implementation Steps:**
1. Create `iterations/src/` directory
2. Create `iterations/src/lib.rs` — Main module
3. Create `iterations/src/tracker.rs` — Progress tracking
4. Create `iterations/src/reporter.rs` — Status reporting
5. Integrate with CI/CD

**Deliverables:**
- `iterations/src/` — Iteration tracking module
- Progress tracking infrastructure

---

## 5. P1 Implementation Details

### P1-1: GitHub Workflow Generation (FR-016)

**Module:** `crates/git/`

**Requirements:**
- `opencode github install` command
- Workflow file at `.github/workflows/opencode.yml`
- Required secrets setup

**Implementation Steps:**
1. Add `github install` subcommand to CLI
2. Create workflow file template
3. Implement GitHub App installation flow
4. Add secrets setup automation

### P1-2: GitLab CI Component (FR-017)

**Module:** `crates/git/`

**Requirements:**
- GitHub workflow trigger examples
- Comment/PR trigger parsing
- CI secret loading for GitHub Actions
- GitLab CI component support

### P1-3: tui.json Ownership Enforcement (FR-009)

**Module:** `crates/config/`, `crates/tui/`

**Requirements:**
- `tui.json` plugin configuration ownership
- Plugin identity — runtime ID resolution
- Plugin deduplication before activation
- `plugin_enabled` semantics

### P1-4: Desktop/Web/ACP Interface (FR-015)

**Module:** `crates/server/`

**Requirements:**
- Desktop app startup flow
- Web server mode
- ACP startup/handshake

---

## 6. Technical Debt Remediation

| ID | Item | Priority | Notes |
|----|------|----------|-------|
| TD-001 | Deprecated `mode` field | Medium | Remove in major version |
| TD-002 | Deprecated `tools` field | Medium | Remove after migration |
| TD-003 | Deprecated `keybinds` field | Low | Moved to tui.json |
| TD-004 | Deprecated `layout` field | Low | Remove field |
| TD-005 | Hardcoded built-in skills | Medium | Consider externalization |
| TD-006 | Magic numbers in compaction | Low | Make configurable |
| TD-007 | SHA256 args hashing | Low | Consider CAS |
| TD-008 | Custom JSONC parser | Medium | Use existing crate |
| TD-009 | `#[serde(other)]` in Part | Low | Explicit error handling |

---

## 7. Release Gates Status

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ |
| Phase 1 | Authority tests green | ✅ |
| Phase 2 | Runtime tests green | ✅ |
| Phase 3 | Subsystem tests green | ✅ |
| Phase 4 | Interface smoke workflows pass | ⏳ |
| Phase 5a | Compatibility suite green | ⏳ |
| Phase 5b | Conventions suite green | ✅ |
| Phase 6 | Non-functional baselines recorded | ⏳ |

---

## 8. Verification

```bash
# Build verification
cargo build --release

# Test suite
cargo test --all-features

# Linting
cargo clippy --all -- -D warnings

# Format check
cargo fmt --all -- --check
```

---

## 9. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-04-09 | Initial plan based on spec v1 |
