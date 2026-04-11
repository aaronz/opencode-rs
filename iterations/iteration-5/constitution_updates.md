# Constitution Updates - Iteration 5

**Generated:** 2026-04-11
**Based on Gap Analysis:** `iteration-5/gap-analysis.md`
**Previous Constitution:** `iteration-4/constitution_updates.md`
**Status:** Amendment Proposal — Addendum to Existing Articles I–VII

---

## Executive Summary

The Iteration 4 Constitution (Articles I–VII) addressed all 20 P0 blockers from that iteration. The Iteration 5 gap analysis reveals:

1. **3 new P0 blockers** — 2 were already constitutionally covered (Art VI §6.1, §6.2) but remain unimplemented; 1 is entirely new (git crate syntax error/code quality).
2. **11 P1 gaps** — TUI slash commands, config error handling, TUI Plugin dialogs/slots, session sharing.
3. **14 P2 gaps** — hardcoded values, path validation, plugin cleanup, experimental features.

**Assessment of existing Constitution:**
- Art VI §6.1–6.2 covers Desktop WebView and ACP transport (**already addressed constitutionally; Iteration 5 must implement**)
- Art III §3.1 covers deprecated fields (**still unimplemented**)
- Art III §3.3 covers hardcoded values (**still unimplemented**)
- **NOT covered:** Build integrity gate, TUI slash command contract, config JSONC error quality, circular expansion detection, TUI Plugin dialog contract

**Net new constitutional content required: 4 new sections across 3 existing articles + 1 new article**

---

## Article I: Coverage Reassessment

### Previously Mandated vs. Iteration 5 Status

| Constitution Reference | Mandate | Iteration 5 Status |
|------------------------|---------|-------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deprecated field sunset | ⚠️ Still present (`mode`, `tools`, `theme`, `keybinds`) |
| Art III §3.2 | Plugin tool registration | ✅ Verified |
| Art III §3.3 | Hardcoded values → config | ❌ Still hardcoded |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ❌ Still stub |
| Art VI §6.2 | ACP HTTP+SSE transport | ❌ Still incomplete |
| Art VII | Compliance checklist | ⚠️ Partially checked |

**Note:** Art VI §6.1 and §6.2 were constitutionally mandated in Iteration 4 but not implemented. No constitutional amendment is needed for these — they are **implementation obligations carried forward**. Iteration 5 MUST deliver them.

---

## Amendment A: Build Integrity Gate (New — Not in Prior Constitution)

### Section A.1 — Crate Compilation as Hard Gate

**Gap Addressed:** Git crate syntax error (P0-new-1) at `crates/git/src/gitlab_ci.rs:611-612`

**Requirement:** ALL crates MUST compile without errors before any PR is merged or any iteration checkpoint is declared.

```
Build Gate Protocol:
  1. cargo build --all-features        → MUST pass (zero errors)
  2. cargo clippy --all -- -D warnings → MUST pass (zero warnings)
  3. cargo fmt --all -- --check        → MUST pass (no diff)
  4. cargo test --all                  → MUST pass
```

**Constraint:** Orphaned or dead code MUST be removed at the time it is identified. It MUST NOT persist across iteration boundaries.

```rust
// PROHIBITED: Orphaned code (unreachable code not in a function/module)
let port = ...;   // ← P0 violation if unreachable
struct Incomplete  // ← P0 violation if body is absent

// REQUIRED: Either complete the construct or remove it
// There is no "TODO: finish later" exception for syntax errors
```

**Verification:**
```bash
# This command MUST succeed with exit code 0
cargo build --all 2>&1 | grep -c "^error" | xargs -I{} test {} -eq 0
```

**Compliance Checklist Addition:**
- [ ] `cargo build --all` exits with code 0 (no errors)
- [ ] No orphaned statements outside function/module scope
- [ ] `cargo clippy --all -- -D warnings` exits with code 0

---

## Amendment B: Configuration System Hardening (Extension of Art IV)

### Section B.1 — JSONC Error Message Quality

**Gap Addressed:** P1-1 — JSONC parsing errors could be clearer

**Requirement:** Config parse errors MUST be actionable:

```rust
// BAD error message
Err("parse error at line 42")

// GOOD error message
Err(ConfigError::ParseError {
    file: PathBuf::from("opencode.json"),
    line: 42,
    column: 7,
    message: "unexpected token '}', expected ':'".to_string(),
    hint: Some("Check for missing commas or mismatched braces".to_string()),
})
```

**CONSTRAINT:** All config parse errors MUST include:
1. File path (absolute)
2. Line and column number
3. Human-readable description
4. Optional remediation hint

### Section B.2 — Circular Variable Expansion Detection

**Gap Addressed:** P1-2 — Circular variable expansion not detected

**Requirement:** Variable expansion MUST detect and reject circular references:

```rust
// Example circular reference
// opencode.json:
// { "base_dir": "${work_dir}", "work_dir": "${base_dir}" }

fn expand_variable(
    key: &str,
    vars: &HashMap<String, String>,
    visited: &mut HashSet<String>,  // ← cycle detection
) -> Result<String, ConfigError> {
    if visited.contains(key) {
        return Err(ConfigError::CircularReference {
            cycle: visited.iter().cloned().collect(),
        });
    }
    visited.insert(key.to_string());
    // ... expand
    visited.remove(key);
    Ok(expanded)
}
```

**CONSTRAINT:** Circular reference detection MUST run at config load time, not lazily at access time. Error MUST clearly identify the cycle.

---

## Amendment C: TUI System Contracts (New — Not in Prior Constitution)

### Section C.1 — Slash Command Completeness Contract

**Gap Addressed:** P1-4 — `/compact`, `/connect`, `/help` incomplete

**Requirement:** Every slash command listed in PRD 09 MUST be fully implemented. A slash command is "complete" if and only if:

1. It appears in `/help` output
2. It executes without panicking
3. It produces observable side effects matching the PRD specification
4. It has at least one integration test in `tests/src/`

**Required slash commands (minimum):**

| Command | Expected Behavior |
|---------|------------------|
| `/compact` | Triggers session compaction, shows token count before/after |
| `/connect` | Connects to remote MCP server, shows connection status |
| `/help` | Lists all available commands with descriptions |
| `/clear` | Clears current session history |
| `/model` | Switches active LLM model |
| `/resume` | Resumes a previous session |

**CONSTRAINT:** Partial slash command stubs (commands that parse but do nothing) are **prohibited**. A command MUST either be fully implemented or absent from the command registry.

```rust
// PROHIBITED: Stub command
SlashCommand::Compact => {
    // TODO: implement
    Ok(())
}

// REQUIRED: Either implement fully or remove from registry
```

### Section C.2 — TUI Plugin Dialog Components Contract

**Gap Addressed:** P1-7 — Dialog components incomplete

**Requirement:** The TUI Plugin API MUST expose all four dialog types:

```rust
pub enum Dialog {
    Alert {
        title: String,
        message: String,
        on_close: Callback,
    },
    Confirm {
        title: String,
        message: String,
        on_confirm: Callback,
        on_cancel: Callback,
    },
    Prompt {
        title: String,
        placeholder: Option<String>,
        on_submit: StringCallback,
        on_cancel: Callback,
    },
    Select {
        title: String,
        options: Vec<SelectOption>,
        on_select: SelectCallback,
        on_cancel: Callback,
    },
}
```

**CONSTRAINT:** All four dialog types MUST be usable from plugin code. Missing dialog types constitute an incomplete plugin API and MUST be treated as P1 blockers for the plugin system.

### Section C.3 — Slots System Contract

**Gap Addressed:** P1-8 — TUI Plugin slots system incomplete

**Requirement:** The slots system MUST support named slot registration:

```rust
pub enum TuiSlot {
    StatusBar,        // Bottom status area
    Sidebar,          // Left/right panel
    ToolbarAction,    // Toolbar button injection
    MessageOverlay,   // Overlay on message area
}

trait TuiPlugin {
    fn register_slots(&self) -> Vec<(TuiSlot, SlotRenderer)> {
        vec![]  // Default: no slots
    }
}
```

**CONSTRAINT:** Slot registration MUST be deterministic (same order every run). Use `IndexMap` or explicit ordering. Multiple plugins MAY register the same slot; rendering order follows plugin registration order.

---

## Amendment D: Non-Functional Hardening

### Section D.1 — Configurable Thresholds (Reinforcement of Art III §3.3)

**Gap Addressed:** P2-8 — Magic numbers still hardcoded

The Iteration 4 Constitution mandated this at Art III §3.3. It remains unimplemented. **This is now escalated to P1.**

**All hardcoded thresholds MUST be moved to config by end of Iteration 5:**

| Constant | Crate | Config Path |
|----------|-------|-------------|
| `COMPACTION_START_THRESHOLD` | compaction.rs | `config.agent.compaction.start_threshold` |
| `COMPACTION_FORCE_THRESHOLD` | compaction.rs | `config.agent.compaction.force_threshold` |
| `MCP_TIMEOUT_MS` | mcp.rs | `config.mcp.timeout_ms` |
| `LSP_TIMEOUT_MS` | lsp.rs | `config.lsp.timeout_ms` |

**Default values MUST be defined as `const` in the config schema, not inline:**

```rust
// BAD
const COMPACTION_START_THRESHOLD: usize = 60000;  // hidden magic number

// GOOD
impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            start_threshold: 60_000,   // documented in config schema
            force_threshold: 120_000,
        }
    }
}
```

### Section D.2 — Deprecated Field Removal Schedule (Reinforcement of Art III §3.1)

**Gap Addressed:** P1-3 — Deprecated fields `mode`, `tools`, `theme`, `keybinds` still present

The Iteration 1 Constitution mandated removal; Iteration 4 deferred to v4.0. **Iteration 5 MUST set a firm deadline.**

| Field | Location | Removal Target |
|-------|----------|----------------|
| `mode` | Config | v4.0 (next major) |
| `tools` (Config) | Config | v4.0 (next major) |
| `theme` | Config | v4.0 — move to `tui.json` |
| `keybinds` | TuiConfig | v4.0 — move to `tui.json` |

**CONSTRAINT:** From Iteration 5 onward, these fields MUST emit a deprecation warning at startup:

```
[WARN] Config field 'mode' is deprecated and will be removed in v4.0.
       Use 'agent.mode' instead. See migration guide at ...
```

### Section D.3 — Experimental Feature Marking

**Gap Addressed:** P2-14 — GitLab Duo and LSP experimental tools unmarked

**Requirement:** Any feature that is environment-dependent, partially implemented, or behind a feature flag MUST be explicitly marked:

```rust
// In code
#[cfg(feature = "experimental")]
pub fn gitlab_duo_integration() { ... }

// In config schema
/// Experimental: requires GitLab Duo subscription.
/// Enable with feature flag `OPENCODE_EXPERIMENTAL=gitlab_duo`
pub gitlab_duo: Option<GitLabDuoConfig>,
```

**CONSTRAINT:** Experimental features MUST NOT be exposed in stable CLI help output without a warning. Users MUST opt in explicitly.

---

## Updated Compliance Checklist (Article VII Amendment)

The following items are added to the Article VII compliance checklist from Iteration 4:

### Build Quality
- [ ] `cargo build --all` exits 0 (zero errors across ALL crates)
- [ ] `cargo clippy --all -- -D warnings` exits 0
- [ ] No orphaned code outside function/module boundaries
- [ ] `cargo fmt --all -- --check` passes

### Configuration System
- [ ] Config parse errors include file path, line, column, hint
- [ ] Circular variable references detected at load time (not lazily)
- [ ] Deprecated fields emit `[WARN]` at startup
- [ ] All magic number thresholds moved to config with documented defaults

### TUI System
- [ ] All slash commands either fully implemented or absent from registry
- [ ] `/compact`, `/connect`, `/help` pass integration tests
- [ ] All four dialog types (Alert/Confirm/Prompt/Select) usable from plugins
- [ ] Slots system supports all `TuiSlot` variants with deterministic ordering

### Interface System (Desktop/Web/ACP — Carried Forward from Art VI)
- [ ] Desktop WebView integration functional (per Art VI §6.1)
- [ ] ACP HTTP+SSE transport functional (per Art VI §6.2)
- [ ] WebView tested on target platform(s)

### Experimental Features
- [ ] Experimental features gated behind `#[cfg(feature = "experimental")]`
- [ ] Experimental features emit warning in CLI output

---

## Appendix A: Gap → Constitution Mapping (Iteration 5)

| Gap ID | Description | Constitution Reference |
|--------|-------------|----------------------|
| P0-new-1 | Git crate syntax error | Amendment A §A.1 (NEW) |
| P0-new-2 | Desktop WebView stub | Art VI §6.1 (carried forward) |
| P0-new-3 | ACP transport incomplete | Art VI §6.2 (carried forward) |
| P1-1 | JSONC error messages unclear | Amendment B §B.1 (NEW) |
| P1-2 | Circular variable expansion | Amendment B §B.2 (NEW) |
| P1-3 | Deprecated fields still present | Art III §3.1 + Amend D §D.2 |
| P1-4 | Slash commands incomplete | Amendment C §C.1 (NEW) |
| P1-7 | TUI Plugin dialogs incomplete | Amendment C §C.2 (NEW) |
| P1-8 | TUI Plugin slots incomplete | Amendment C §C.3 (NEW) |
| P2-8 | Magic numbers hardcoded | Art III §3.3 + Amend D §D.1 |
| P2-14 | Experimental features unmarked | Amendment D §D.3 (NEW) |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles, custom tools, TUI SDK |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin hardening, MCP/LSP, Server API, Desktop/ACP |
| v2.1 | Iteration 5 | I–VII + A–D | Build integrity gate, JSONC errors, circular expansion, slash command contract, TUI dialogs/slots, threshold escalation |

---

## Priority Summary for Iteration 5

| Priority | Amendment | Action Required |
|----------|-----------|-----------------|
| **P0** | Art VI §6.1 | Implement Desktop WebView (MUST deliver) |
| **P0** | Art VI §6.2 | Implement ACP HTTP+SSE (MUST deliver) |
| **P0** | Amend A §A.1 | Fix git crate syntax error (immediate) |
| **P1** | Amend B §B.1-B.2 | Config JSONC errors + circular detection |
| **P1** | Amend C §C.1 | Complete slash commands |
| **P1** | Amend C §C.2-C.3 | TUI Plugin dialogs and slots |
| **P1** | Amend D §D.1 | Escalate hardcoded thresholds to config |
| **P1** | Amend D §D.2 | Add deprecation warnings to legacy fields |
| **P2** | Amend D §D.3 | Mark experimental features |

---

*Constitution v2.1 — Iteration 5*
*Total constitutional articles: 7 (original) + 4 amendments*
*New P0 gaps constitutionally covered: 1 (git syntax error)*
*P0 gaps carried forward from Art VI: 2 (Desktop WebView, ACP transport)*
*New P1 gaps constitutionally covered: 6*
*Report generated: 2026-04-11*
