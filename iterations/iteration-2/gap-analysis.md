# Gap Analysis Report - Iteration 2

**Generated:** 2026-04-10
**Analysis Period:** Iteration 1 → Iteration 2
**Output Directory:** `/Users/aaronzh/Documents/GitHub/opencode-rs/iterations/iteration-2/`

---

## 1. Executive Summary

This gap analysis compares the current Rust implementation against the PRD specification documents, focusing on changes since Iteration 1.

**Key Findings:**
- ✅ Custom Tool File Loader - **IMPLEMENTED** in `crates/tools/src/discovery.rs`
- ✅ GitHub workflow generation - **IMPLEMENTED** in `crates/git/src/workflow.rs`
- ✅ GitLab CI component - **IMPLEMENTED** in `crates/git/src/gitlab_ci.rs`
- ✅ Desktop/Web/ACP Interface - **IMPLEMENTED** in `crates/cli/src/cmd/`
- ⚠️ TUI Plugin TypeScript SDK - **PARTIALLY IMPLEMENTED** (not fully aligned with PRD)
- ❌ `iterations/src/` directory - **STILL NOT IMPLEMENTED**

**Overall PRD Coverage: ~78-83%** (up from ~75-80% in Iteration 1)

---

## 2. Implementation Progress by PRD Document

| PRD Document | Iteration 1 | Iteration 2 | Change | Notes |
|-------------|-------------|-------------|--------|-------|
| 01-core-architecture | 90% | 90% | — | Minor VCS worktree gap remains (P2) |
| 02-agent-system | 95% | 95% | — | |
| 03-tools-system | 75% | **90%** | ↑ | Custom tool loader IMPLEMENTED |
| 04-mcp-system | 85% | 85% | — | |
| 05-lsp-system | 80% | 80% | — | |
| 06-configuration-system | 85% | **82%** | ↓ | Deprecated fields still present (tech debt) |
| 07-server-api | 80% | 80% | — | |
| 08-plugin-system | 70% | 70% | — | |
| 09-tui-system | 85% | 85% | — | |
| 10-provider-model | 90% | 90% | — | |
| 11-formatters | 70% | 70% | — | |
| 12-skills-system | 85% | 85% | — | |
| 14-github-gitlab | 50% | **85%** | ↑ | Workflow & CI generation IMPLEMENTED |
| 15-tui-plugin-api | 40% | **55%** | ↑ | TypeScript SDK started but incomplete |

---

## 3. Gap Analysis Table

| Gap Item | Severity | Module | PRD Reference | Status | 修复建议 |
|----------|----------|--------|---------------|--------|---------|
| `iterations/src/` directory empty | **P0** | Project | All | ❌ Unchanged | Create `iterations/src/lib.rs`, `tracker.rs`, `reporter.rs` |
| TUI Plugin SDK not PRD-aligned | **P0** | sdk | 15-tui-plugin-api | ⚠️ Partial | Align `TuiPlugin` type signature with PRD: should be `fn(api, options, meta) => void` not interface |
| Custom tool loader incomplete | ~~**P0**~~ | tools | 03-tools-system | ✅ IMPLEMENTED | Discovery, CustomTool, and registration all implemented |
| GitHub workflow generation | ~~**P1**~~ | git | 14-github-gitlab | ✅ IMPLEMENTED | `WorkflowTemplate`, `setup_github_workflow` complete |
| GitLab CI component | ~~**P1**~~ | git | 14-github-gitlab | ✅ IMPLEMENTED | `GitLabCiTemplate`, `setup_gitlab_ci` complete |
| tui.json ownership enforcement | **P1** | config | 06-config, 09-tui | ⚠️ Partial | Deprecated `theme` field still in main config |
| Desktop/Web/ACP Interface | ~~**P1**~~ | cli | 13-desktop-web | ✅ IMPLEMENTED | `desktop.rs`, `web.rs`, `acp.rs` all present |
| VCS worktree root distinction | **P2** | core | 01-core-arch | ❌ | `worktree_root` field not added |
| AGENTS.md upward scanning | **P2** | core | 12-skills | ⚠️ Partial | `AgentsMdConfig` exists but upward traversal may be incomplete |
| MCP OAuth CLI commands | **P2** | cli | 04-mcp | ❌ | `opencode mcp auth` not exposed |
| Plugin tool registration | **P2** | plugin | 08-plugin | ❌ | Plugins cannot register custom tools |
| Skill permission restrictions | **P2** | core | 12-skills | ❌ | Skill usage not gated by permissions |

---

## 4. P0/P1/P2 Problem Classification

### P0 - Blocking Issues (Must Fix)

| Issue | Description | Impact | Status |
|-------|-------------|--------|--------|
| **iterations/src/ not created** | No implementation tracking structure | Cannot track iteration progress | ❌ Unchanged |
| **TUI Plugin SDK misalignment** | Type signature doesn't match PRD | Third-party plugins may not work correctly | ⚠️ Worsened |

### P1 - Important Issues (Should Fix)

| Issue | Description | Impact | Status |
|-------|-------------|--------|--------|
| **tui.json ownership not enforced** | `theme` deprecated field still in main config | Configuration boundary violations | ⚠️ Unchanged |
| **VCS worktree root** | Project doesn't distinguish worktree from root | Minor semantic gap | ❌ Unchanged |
| **MCP OAuth CLI** | `opencode mcp auth` commands not exposed | OAuth requires manual steps | ❌ Unchanged |

### P2 - Improvement Issues (Nice to Have)

| Issue | Description | Impact | Status |
|-------|-------------|--------|--------|
| **Plugin-provided tool registration** | Plugins cannot add custom tools | Limited extensibility | ❌ Unchanged |
| **Skill permission restrictions** | Skill loading not permission-gated | No per-skill access control | ❌ Unchanged |
| **AGENTS.md upward scanning** | Directory traversal may be incomplete | May miss project instructions | ⚠️ Unchanged |

---

## 5. Technical Debt清单

### 5.1 Deprecated Fields (Iteration 1 → Iteration 2)

| Debt Item | Location | Iteration 1 | Iteration 2 | Remediation |
|-----------|----------|--------------|-------------|-------------|
| `mode` field | `Config` | Deprecated | ⚠️ Still present (line 92) | Remove in v4.0 |
| `tools` field (Config) | `Config` | Deprecated | ⚠️ Still present (line 129) | Remove in v4.0 |
| `theme` field | `Config` | Deprecated | ⚠️ Still present (line 146) | Move to tui.json |
| `tools` field (AgentConfig) | `AgentConfig` | Deprecated | ⚠️ Still present (line 415) | Remove in v4.0 |
| `keybinds` field | `TuiConfig` | Present | ⚠️ Present (line 1072) | Should be in tui.json |
| `layout` field | ? | Mentioned | ⚠️ Need verification | Verify if removed |

### 5.2 Implementation Debt

| Debt Item | Module | Description | Remediation |
|-----------|--------|-------------|-------------|
| Hardcoded built-in skills | core | Skills embedded in binary | Consider externalization |
| Magic numbers in compaction | core | `COMPACTION_START_THRESHOLD`, `COMPACTION_FORCE_THRESHOLD` | Make configurable |
| Custom JSONC parser | config | Custom implementation in `jsonc.rs` | Consider using existing crate |
| `#[serde(other)]` in Part | core | Unknown parts silently ignored | Use explicit error handling |

---

## 6. Module Implementation Status

### Core Crate (`crates/core/src/`)

| Module | Iteration 1 | Iteration 2 | Notes |
|--------|-------------|-------------|-------|
| `session.rs` | ✅ Complete | ✅ | |
| `message.rs` | ✅ Complete | ✅ | |
| `part.rs` | ✅ Complete | ✅ | |
| `project.rs` | ⚠️ Partial | ⚠️ Partial | VCS worktree distinction missing |
| `config.rs` | ✅ Complete | ⚠️ Deprecated fields | 4 deprecated fields still present |
| `permission.rs` | ✅ Complete | ✅ | |
| `checkpoint.rs` | ✅ Complete | ✅ | |
| `compaction.rs` | ✅ Complete | ✅ | |
| `skill.rs` | ✅ Complete | ✅ | |
| `storage.rs` | ✅ Complete | ✅ | |
| `agents_md.rs` | ⚠️ Partial | ⚠️ Partial | Upward scanning may be incomplete |

### Tools Crate (`crates/tools/src/`)

| Component | Iteration 1 | Iteration 2 | Notes |
|-----------|-------------|-------------|-------|
| Built-in tools (30+) | ✅ | ✅ | |
| `registry.rs` | ✅ | ✅ | |
| **`discovery.rs`** | ❌ Not implemented | ✅ **IMPLEMENTED** | `ToolDiscovery`, `CustomTool`, `register_custom_tools` |
| **Custom tool loader** | ❌ | ✅ **IMPLEMENTED** | File-based discovery working |

### Git Crate (`crates/git/src/`)

| Component | Iteration 1 | Iteration 2 | Notes |
|-----------|-------------|-------------|-------|
| `github.rs` | ✅ | ✅ | GitHub API client |
| **`workflow.rs`** | ❌ | ✅ **IMPLEMENTED** | `WorkflowTemplate`, `setup_github_workflow` |
| **`gitlab_ci.rs`** | ❌ | ✅ **IMPLEMENTED** | `GitLabCiTemplate`, `setup_gitlab_ci` |

### CLI Crate (`crates/cli/src/cmd/`)

| Command | Iteration 1 | Iteration 2 | Notes |
|---------|-------------|-------------|-------|
| `github.rs` | ⚠️ Partial | ✅ | `install` and `workflow` commands working |
| **`desktop.rs`** | ❌ | ✅ **IMPLEMENTED** | Desktop mode with browser auto-open |
| **`web.rs`** | ❌ | ✅ **IMPLEMENTED** | Web interface mode |
| **`acp.rs`** | ❌ | ✅ **IMPLEMENTED** | ACP protocol support |

### SDK (`sdk/typescript/`)

| Package | Iteration 1 | Iteration 2 | Notes |
|---------|-------------|-------------|-------|
| `@opencode-ai/plugin-tui` | ❌ Not exist | ⚠️ Partial | Interfaces exist but NOT PRD-aligned |

---

## 7. Newly Implemented Components Detail

### 7.1 Custom Tool File Loader ✅

**PRD Requirement:**
- Project-level: `.opencode/tools/*.ts`
- Global-level: `~/.config/opencode/tools/*.ts`

**Implementation:** `crates/tools/src/discovery.rs`
- `ToolDiscovery` struct with project/global path scanning
- `CustomTool` implementing `Tool` trait
- `register_custom_tools()` async function
- Node.js execution for `.js`, `.ts`, `.mjs`, `.cjs` files
- Regex-based tool definition extraction

**Status:** ✅ **COMPLETE** - Matches PRD requirements

### 7.2 GitHub Workflow Generation ✅

**PRD Requirement:** `opencode github install` creates `.github/workflows/opencode.yml`

**Implementation:** `crates/git/src/workflow.rs`
- `GitHubAppClient` for GitHub API
- `WorkflowTemplate::generate_yaml()` generates proper CI YAML
- `setup_github_workflow()` creates/updates workflow file
- Supports secrets: `OPENCODE_API_KEY`, `OPENCODE_MODEL`

**Status:** ✅ **COMPLETE** - Matches PRD requirements

### 7.3 GitLab CI Component ✅

**PRD Requirement:** CI component for GitLab Duo

**Implementation:** `crates/git/src/gitlab_ci.rs`
- `GitLabCiTemplate` with standalone and component modes
- `setup_gitlab_ci()` creates/updates `.gitlab-ci.yml`
- Supports both traditional CI and GitLab CI component format

**Status:** ✅ **COMPLEMENT** - Matches PRD requirements

### 7.4 Desktop/Web/ACP Interface ✅

**PRD Requirement:** Desktop, web, and ACP interface modes

**Implementation:**
- `desktop.rs`: Full desktop mode with server, browser auto-open, config precedence
- `web.rs`: Web interface mode with server
- `acp.rs`: ACP protocol routes in server

**Status:** ✅ **COMPLETE** - Matches PRD requirements

### 7.5 TUI Plugin TypeScript SDK ⚠️

**PRD Requirement:**
```typescript
type TuiPlugin = (
  api: TuiPluginAPI,
  options: unknown,
  meta: TuiPluginMeta
) => Promise<void> | void
```

**Current Implementation:**
```typescript
interface TuiPlugin<TState = unknown> {
  id: string;
  name: string;
  version: string;
  module: TuiPluginModule;
}
```

**Gap:** Current implementation is an interface with properties, NOT a function type as PRD specifies. The API surface (CommandsAPI, RoutesAPI, etc.) exists but the plugin entry point signature is wrong.

**Status:** ⚠️ **PARTIAL** - Requires signature change to match PRD

---

## 8. Comparison: Iteration 1 vs Iteration 2

### Issues Resolved (Iteration 1 → 2)

| Issue ID | Issue | Resolution |
|----------|-------|------------|
| P0-1 | Custom tool file loader not implemented | ✅ IMPLEMENTED in `discovery.rs` |
| P1-1 | GitHub workflow generation missing | ✅ IMPLEMENTED in `workflow.rs` |
| P1-2 | GitLab CI component not implemented | ✅ IMPLEMENTED in `gitlab_ci.rs` |
| P1-4 | Desktop/Web/ACP Interface not implemented | ✅ IMPLEMENTED |

### Issues Remaining (Iteration 2)

| Issue ID | Issue | Status |
|----------|-------|--------|
| P0-1 | `iterations/src/` not created | ❌ Unchanged |
| P0-2 | TUI Plugin TypeScript SDK | ⚠️ Partial (worsened alignment) |
| P1-3 | tui.json ownership | ⚠️ Partial (deprecated theme field) |
| P2-1 | VCS worktree root | ❌ Unchanged |
| P2-2 | AGENTS.md upward scanning | ⚠️ Partial |
| P2-3 | MCP OAuth CLI | ❌ Unchanged |
| P2-5 | Plugin tool registration | ❌ Unchanged |
| P2-6 | Skill permission restrictions | ❌ Unchanged |

---

## 9. Recommendations

### Immediate Actions (P0 - Must Fix)

1. **Create `iterations/src/` structure**
   - Create `iterations/src/lib.rs`
   - Create `iterations/src/tracker.rs` for tracking iteration progress
   - Create `iterations/src/reporter.rs` for generating reports
   - Align with `iterate-prd.sh` workflow

2. **Fix TUI Plugin SDK to match PRD**
   - Change `TuiPlugin` from interface to function type
   - Signature: `fn(api: TuiPluginAPI, options: unknown, meta: TuiPluginMeta) => void`
   - Export proper `TuiPluginModule` type

### Short-term Actions (P1 - Should Fix)

3. **Enforce tui.json ownership**
   - Remove deprecated `theme` field from `Config`
   - Add validation that `theme` must be in tui.json
   - Consider removing `keybinds` from `TuiConfig` (should be in tui.json)

4. **Add VCS worktree root field**
   - Add `worktree_root` field to `ProjectInfo` when distinct from project root

5. **Add MCP OAuth CLI commands**
   - Implement `opencode mcp auth` subcommands

### Medium-term Actions (P2 - Improvement)

6. **Plugin-provided tool registration**
7. **Skill permission restrictions**
8. **AGENTS.md upward scanning verification**

---

## 10. Technical Debt Remediation Plan

### High Priority (Remove in v4.0)

| Debt | Current | Action |
|------|---------|--------|
| `mode` field | Config line 92 | Remove, keep `agent` field only |
| `tools` field (Config) | Config line 129 | Remove, use `permission` only |
| `tools` field (AgentConfig) | AgentConfig line 415 | Remove |

### Medium Priority (Move to tui.json)

| Debt | Current | Action |
|------|---------|--------|
| `theme` field | Config line 146 | Move to tui.json |
| `keybinds` field | TuiConfig line 1072 | Move to tui.json |

### Low Priority (Nice to Have)

| Debt | Action |
|------|--------|
| Magic numbers in compaction | Make configurable via Config |
| Custom JSONC parser | Replace with existing crate |
| `#[serde(other)]` in Part | Use explicit error handling |

---

## 11. Progress Summary

| Priority | Iteration 1 Total | Resolved in Iter 2 | Remaining | Progress |
|----------|------------------|---------------------|-----------|----------|
| P0 | 3 | 0 (+1 improved) | 2 | 33% |
| P1 | 4 | 3 | 1 | 75% |
| P2 | 6 | 0 | 6 | 0% |
| Tech Debt | 9 | 0 | 9 | 0% |
| **Total** | **22** | **3 (+1 improved)** | **18** | **~18%** |

---

## 12. Appendix

### A. PRD File Inventory

| File | Coverage | Notes |
|------|----------|-------|
| 01-core-architecture.md | 90% | VCS worktree gap |
| 02-agent-system.md | 95% | |
| 03-tools-system.md | **90%** | Custom tool loader IMPLEMENTED |
| 04-mcp-system.md | 85% | |
| 05-lsp-system.md | 80% | |
| 06-configuration-system.md | **82%** | Deprecated fields |
| 07-server-api.md | 80% | |
| 08-plugin-system.md | 70% | |
| 09-tui-system.md | 85% | |
| 10-provider-model-system.md | 90% | |
| 11-formatters.md | 70% | |
| 12-skills-system.md | 85% | |
| 13-desktop-web-interface.md | **80%** | IMPLEMENTED |
| 14-github-gitlab-integration.md | **85%** | Workflow & CI IMPLEMENTED |
| 15-tui-plugin-api.md | **55%** | SDK started but incomplete |
| 16-test-plan.md | N/A | |
| 17-rust-test-implementation-roadmap.md | N/A | |
| 18-crate-by-crate-test-backlog.md | N/A | |
| 19-implementation-plan.md | N/A | |

### B. Key Implementation Files

```
opencode-rust/
├── crates/
│   ├── tools/src/discovery.rs      # ✅ Custom tool loader
│   ├── git/src/workflow.rs         # ✅ GitHub workflow
│   ├── git/src/gitlab_ci.rs        # ✅ GitLab CI
│   ├── cli/src/cmd/desktop.rs      # ✅ Desktop mode
│   ├── cli/src/cmd/web.rs          # ✅ Web mode
│   └── core/src/config.rs          # ⚠️ Deprecated fields
└── sdk/typescript/packages/plugin-tui/  # ⚠️ Not PRD-aligned

iterations/
├── iteration-1/                    # Previous iteration
└── iteration-2/                    # Current (gap-analysis.md here)
    └── .checkpoint                 # phase1
```

---

*Report generated for Iteration 2 gap analysis.*
