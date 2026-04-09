# Constitution Updates - Iteration 1

**Generated:** 2026-04-09
**Based on Gap Analysis:** `gap-analysis.md`
**Status:** New Constitution Recommendation

---

## Executive Summary

No existing Constitution was found in the repository. This document proposes a new **OpenCode RS Constitution** based on the P0/P1 issues identified in the gap analysis, intended to guide future development and prevent recurrence of identified gaps.

---

## Proposed Constitution: OpenCode RS

### Preamble

This Constitution establishes the foundational principles, design constraints, and implementation requirements for OpenCode RS. All contributors and AI agents operating within this codebase shall adhere to these principles.

---

## Article I: Core Design Principles

### Section 1.1 - Modularity First

> **Principle:** All features MUST be implemented as discrete, independently testable modules.

- Each crate MUST have a clearly defined responsibility
- Inter-crate dependencies MUST be minimal and explicit
- No feature shall be implemented as "magic" inline code

**Rationale:** The gap analysis identified `iterations/src/` as empty with no implementation tracking. A modular constitution would have mandated structured implementation tracking.

### Section 1.2 - Configuration Ownership

> **Principle:** Each configuration domain MUST have a single, authoritative source.

| Domain | Authoritative File |
|--------|-------------------|
| Theme, keybindings, TUI plugins | `tui.json` |
| Provider credentials | Environment variables |
| Session persistence | SQLite database |
| Global config | `config.toml` |

**Violation:** Gap analysis found `tui.json` ownership not fully enforced - some TUI config leaking to main config.

### Section 1.3 - Extensibility Gates

> **Principle:** User-extensible systems MUST have documented, stable APIs.

- Custom tools: Files in `.opencode/tools/` (project) or `~/.config/opencode/tools/` (global)
- Plugins: Must expose typed interfaces via `@opencode-ai/plugin/*` packages
- Skills: Discovered via `SKILL.md` files with directory traversal

**Critical Gap:** Custom tool file loader is P0 incomplete.

---

## Article II: P0 Implementation Requirements

### Section 2.1 - Iteration Tracking Structure

**Requirement:** The `iterations/src/` directory MUST contain:

```
iterations/
└── src/
    ├── mod.rs           # Module declaration
    ├── iteration_1.rs   # Implementation tracking
    ├── constitution.rs  # Design principles
    └── changelog.rs     # Decision log
```

**Purpose:** Prevent recurrence of "empty implementation tracking" gap.

### Section 2.2 - Custom Tool Loader

**Requirement:** The tool registry MUST support file-based tool discovery:

```
Tool Discovery Path Priority:
1. Project-level: {project_root}/.opencode/tools/*.ts
2. Global-level: ~/.config/opencode/tools/*.ts
3. Built-in: Compiled into binary
```

**Schema for custom tool files:**
```typescript
interface CustomTool {
  name: string;           // kebab-case, unique
  description: string;   // Human-readable description
  inputSchema: object;    // JSON Schema
  handler: string;        // File path or inline code
}
```

**Implementation Location:** `crates/tools/src/registry.rs`

### Section 2.3 - TUI Plugin TypeScript SDK

**Requirement:** The SDK crate MUST expose TUI plugin types:

```typescript
// @opencode-ai/plugin/tui
import type { TuiPlugin, TuiPluginModule, TuiApi } from "@opencode-ai/plugin/tui";

export interface TuiApi {
  session: SessionApi;
  ui: {
    showDialog(config: DialogConfig): Promise<void>;
    registerWidget(widget: WidgetConfig): void;
    onKeybinding(keys: string[], handler: () => void): void;
  };
}

export type TuiPlugin = (
  api: TuiApi,
  options: Record<string, unknown>,
  meta: PluginMeta
) => Promise<void>;
```

**Implementation Location:** `crates/sdk/src/tui.rs` (new)

---

## Article III: Design Constraints

### Section 3.1 - Deprecated Field Sunset

The following fields are **deprecated** and MUST be removed in the next major version:

| Field | Location | Replacement |
|-------|----------|-------------|
| `mode` | config | `agent` |
| `tools` | config | `permission` |
| `keybinds` | config | `tui.json` |
| `layout` | config | (always stretch) |

**Constraint:** New features MUST NOT reintroduce these field names.

### Section 3.2 - Error Handling Mandate

> **Principle:** Unknown variants MUST NOT be silently ignored.

```rust
// BAD - Silently ignores unknown parts
#[serde(other)]
Unknown,

// GOOD - Explicit error on unknown variants
#[serde(tag = "type")]
enum Part {
    Text(TextPart),
    #[serde(other)]
    Unknown { /* error logging */ },
}
```

**Gap:** `part.rs` uses `#[serde(other)]` - needs revision.

### Section 3.3 - Hardcoded Value Prohibited

The following values are currently hardcoded and MUST become configurable:

| Value | Current | Config Location |
|-------|---------|-----------------|
| `COMPACTION_START_THRESHOLD` | 60000 | `compaction.rs` → config |
| `COMPACTION_FORCE_THRESHOLD` | 120000 | `compaction.rs` → config |

---

## Article IV: GitHub/GitLab Integration

### Section 4.1 - Workflow Generation

**Requirement:** `opencode github install` MUST generate:

1. GitHub App installation via API
2. Workflow file: `.github/workflows/opencode.yml`
3. Required secrets documentation

**Constraint:** Generated files MUST be idempotent (re-run safe).

### Section 4.2 - GitLab CI Component

**Requirement:** CI component at `.gitlab/ci/opencode.yml` MUST:

1. Use GitLab CI YAML anchors for reuse
2. Support `opencode:test` and `opencode:lint` targets
3. Document required CI/CD variables

---

## Article V: Constitution Amendment Process

1. **Proposal:** Any agent may propose constitutional changes
2. **Review:** Changes require review in `iterations/src/constitution.rs`
3. **Adoption:** Merged after +1 from maintainer and passing CI
4. **Versioning:** Major version bump for breaking changes

---

## Article VI: Compliance Checklist

New implementations MUST verify:

- [ ] Module has corresponding test in `tests/src/`
- [ ] Configuration has schema in `crates/core/src/config_schema.rs`
- [ ] Public API documented with doc comments
- [ ] No deprecated fields reintroduced
- [ ] Error handling explicit (no silent `#[serde(other)]`)
- [ ] Integration tests pass: `cargo test -p opencode-integration-tests`

---

## Appendix A: Gap → Constitution Mapping

| Gap ID | Gap Description | Constitution Reference |
|--------|-----------------|------------------------|
| G1 | Empty iterations/src/ | Art II §2.1 |
| G2 | Custom tool loader incomplete | Art II §2.2 |
| G3 | TUI Plugin API unimplemented | Art II §2.3 |
| G4 | tui.json ownership leak | Art I §1.2 |
| G5 | Deprecated fields | Art III §3.1 |
| G6 | Silent serde unknown handling | Art III §3.2 |
| G7 | Hardcoded thresholds | Art III §3.3 |
| G8 | GitHub workflow generation | Art IV §4.1 |
| G9 | GitLab CI component | Art IV §4.2 |

---

## Appendix B: Recommended Next Steps

1. **Immediate:** Create `iterations/src/` structure per Art II §2.1
2. **Short-term:** Implement custom tool loader per Art II §2.2
3. **Short-term:** Implement TUI Plugin SDK per Art II §2.3
4. **Medium-term:** Address all Art III design constraints

---

*This constitution was generated from gap-analysis.md (Iteration 1) and should be reviewed at the start of each iteration.*
