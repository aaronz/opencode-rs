# Constitution Updates - Iteration 9

**Generated:** 2026-04-12
**Based on Gap Analysis:** `iteration-9/gap-analysis.md`
**Previous Constitution:** `iteration-8/constitution_updates.md` (v2.4)
**Status:** Amendment Proposal — Iteration 9 Review

---

## Executive Summary

Iteration 8's Constitution v2.4 with Amendments J-L addressed clippy linting as a release gate, CLI test quality, and Desktop WebView escalation. The Iteration 9 gap analysis reveals:

1. **P0-8 Clippy unreachable pattern** ✅ FIXED (commit 95c1c0c)
2. **P0-new-2 Desktop WebView** ✅ IMPLEMENTED (commit 131a17e)
3. **P1-2 Circular detection** ✅ IMPLEMENTED (commit 2b43da4)
4. **P1-9 Session sharing** ✅ IMPLEMENTED (commit 43e6564)
5. **P0-9 Clippy failures** ❌ NEW P0 (18 errors in core + ratatui-testing)
6. **All P2 items from iteration-8** ✅ ALL FIXED

**Assessment:** Amendment J (Clippy Linting Hard Gate) inadequately prevented P0-9 because it focused on one specific pattern (unreachable patterns) but did not mandate comprehensive clippy coverage across ALL crates. P0-9 exposes systemic clippy violations across `opencode-core` (17 errors) and `ratatui-testing` (1 error).

---

## Article I: Coverage Reassessment (Iteration 9)

### Previously Mandated vs. Iteration 9 Status

| Constitution Reference | Mandate | Iteration 9 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deterministic hook order | ✅ Verified (IndexMap) |
| Art III §3.2 | Plugin tool registration | ✅ Verified |
| Art III §3.3 | Config ownership boundary | ✅ Verified |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ✅ **IMPLEMENTED** |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ Verified |
| Amend A §A.1 | Build integrity gate | ✅ Verified (builds clean) |
| Amend J §J.1 | Clippy linting gate | ⚠️ **INCOMPLETE (P0-9)** |
| Amend J §J.2 | Pattern matching correctness | ✅ **FIXED** |
| Amend K §K.1 | CLI test quality gate | ✅ **FIXED** |
| Amend L §L.1 | Desktop WebView deadline | ✅ **IMPLEMENTED** |
| Amend B §B.1 | JSONC error messages | ✅ Implemented |
| Amend B §B.2 | Circular variable expansion | ✅ **IMPLEMENTED** |
| Amend C §C.1 | Slash command contract | ✅ Verified |
| Amend C §C.2 | TUI Plugin dialogs | ✅ Implemented |
| Amend C §C.3 | Slots system | ✅ Implemented |
| Amend D §D.1 | Magic number thresholds | ⚠️ Still partial |
| Amend D §D.2 | Deprecated field warnings | 🚧 In progress |
| Amend D §D.3 | Experimental marking | ✅ Verified |
| Amend E §E.1 | Test compilation gate | ✅ Verified |
| Amend F §F.1 | ACP transport verification | ✅ Verified |
| Amend G §G.1 | Desktop WebView milestone | ✅ Implemented |
| Amend H §H.1 | Test code enforcement | ✅ Verified |
| Amend I §I.1 | Code quality debt | ✅ Verified (DC-1 through DC-10 fixed) |

---

## Article II: Iteration 9 Gap Analysis

### P0 Blockers Remaining

| Item | Gap ID | Module | Status | Notes |
|------|--------|--------|--------|-------|
| **Clippy failures (18 errors)** | **P0-9** | **core, ratatui-testing** | ❌ **NEW P0** | `cargo clippy -- -D warnings` fails |

### Issues Fixed Since Iteration 8

| Item | Gap ID | Module | Status | Commit |
|------|--------|--------|--------|--------|
| Clippy unreachable pattern | P0-8 | permission | ✅ FIXED | 95c1c0c |
| Desktop WebView stub | P0-new-2 | cli | ✅ IMPLEMENTED | 131a17e |
| Circular detection | P1-2 | config | ✅ IMPLEMENTED | 2b43da4 |
| Session sharing | P1-9 | cli | ✅ IMPLEMENTED | 43e6564 |
| VCS worktree root | P2-1 | core | ✅ IMPLEMENTED | 8e36b4e |
| Workspace validation | P2-2 | core | ✅ IMPLEMENTED | 6976c92 |
| API error shape | P2-9 | server | ✅ IMPLEMENTED | 4634293 |
| Home view | P2-12 | tui | ✅ IMPLEMENTED | 16b00c7 |
| LLM reasoning budget | P2-13 | llm | ✅ IMPLEMENTED | 76d999b |
| GitLab Duo marking | P2-14 | git | ✅ IMPLEMENTED | 5292612 |
| Git test cleanup | P2-15 | git | ✅ IMPLEMENTED | fced218 |
| CLI test_prompt_history | NEW | cli | ✅ FIXED | 5d8b024 |
| Dead code DC-1..DC-10 | NEW | various | ✅ ALL FIXED | Multiple |

---

## Article III: P0-9 Clippy Failure Analysis

### Error Distribution (18 Total)

**ratatui-testing (1 error):**
| Error | File | Description |
|-------|------|-------------|
| `new_without_default` | `state.rs:6` | `StateTester` lacks `Default` impl |

**opencode-core (17 errors):**
| Error | Count | File | Description |
|-------|-------|------|-------------|
| deprecated `AgentMode` enum | 2 | `config.rs:436` | Using deprecated enum |
| deprecated `AgentConfig::mode` field | 2 | `command.rs:567`, `config.rs:2771` | Using deprecated field |
| `question_mark` | 1 | `config.rs:1594` | Use `?` operator |
| `needless_borrows_for_generic_args` | 1 | `config.rs:2068` | Borrow instead of path |
| `redundant_closure` | 1 | `session_sharing.rs:323` | Use `ok_or` |
| `map_entry` | 1 | `session_sharing.rs:225` | Use entry API |
| `and_then` → `map` | 1 | `crash_recovery.rs:241` | Unnecessary closure |
| `very_complex_type` | 1 | `skill.rs` | Complex type needs factoring |
| `&PathBuf` → `&Path` | 5 | `skill.rs:116` | Wrong type usage |

### Root Cause Analysis

Amendment J §J.1 established `cargo clippy --all -- -D warnings` as a release gate but:
1. Did not explicitly mandate running clippy on **ALL** crates including `ratatui-testing`
2. Did not cover specific clippy lint categories comprehensively
3. Did not address **deprecated API usage** as a constitutional violation

---

## Amendment M: Comprehensive Clippy Coverage (Amendment J Strengthening)

### Section M.1 — Extended Clippy Coverage

**Gap Addressed:** P0-9 — Amendment J did not explicitly cover all crates or deprecated API usage

**CONSTRAINT:** Amendment J §J.1 is HEREBY STRENGTHENED:

The following command MUST exit 0 for ALL release gates:

```bash
cargo clippy --all --all-targets -- -D warnings
```

This includes:
- `opencode-core`
- `opencode-cli`
- `opencode-agent`
- `opencode-tools`
- `opencode-mcp`
- `opencode-lsp`
- `opencode-plugin`
- `opencode-server`
- `opencode-git`
- `opencode-llm`
- `opencode-storage`
- `opencode-permission`
- `ratatui-testing`
- All workspace member crates

### Section M.2 — Deprecated API Prohibition

**Gap Addressed:** P0-9 — Deprecated APIs (`AgentMode`, `AgentConfig::mode`) still in use

**CONSTRAINT:** Deprecated APIs MUST NOT be introduced or continued in new code:

1. No new uses of `#[deprecated]` APIs without explicit `#[allow(deprecated)]` with documented justification
2. Deprecated field/enum continued usage is a **P0-class violation**
3. `AgentMode` enum and `AgentConfig::mode` field are **scheduled for removal in v4.0**

**Verification:**
```bash
# Search for deprecated API usage - MUST return zero matches
grep -r "AgentMode" --include="*.rs" crates/core/src/ | grep -v "deprecation" | grep -v "#\[allow(deprecated)"
grep -r "\.mode" --include="*.rs" crates/core/src/config.rs | grep -v "deprecation"
```

### Section M.3 — Clippy Error Categories Covered

**CONSTRAINT:** The following clippy error categories MUST be fixed in all crates:

| Clippy Lint | Category | Example |
|-------------|----------|---------|
| `new_without_default` | style | Types with `new()` should implement `Default` |
| `question_mark` | style | Use `?` operator instead of `match Ok(x)` |
| `needless_borrows_for_generic_args` | perf | Borrow `Path` instead of `PathBuf` |
| `redundant_closure` | perf | Use `ok_or`/`ok_or_else` instead of closure |
| `map_entry` | perf | Use `HashMap::entry()` API |
| `and_then` | style | `and_then(\|x\| Some(y))` → `map(\|x\| y)` |
| `very_complex_type` | readability | Factor complex types into `type` definitions |
| `deprecated` | warnings | No deprecated APIs without `#[allow(deprecated)]` |

---

## Amendment N: Default Trait Implementation Requirement

### Section N.1 — Public Types with `new()` Must Implement `Default`

**Gap Addressed:** P0-9 — `StateTester` in `ratatui-testing` lacks `Default` implementation

**CONSTRAINT:** Any public type with a `new()` constructor MUST also implement `Default` unless:

1. The type is intentionally non-defaultable (e.g., contains `Pin<>`,
   `Mutex` without `Default`, or other non-defaultable fields)
2. A justification comment `// Reason: <explanation>` is added above the struct

**Verification:**
```rust
// CORRECT:
#[derive(Debug)]
pub struct StateTester {
    // fields...
}

impl Default for StateTester {
    fn default() -> Self {
        Self { /* ... */ }
    }
}

// OR with justification:
#[derive(Debug)]
pub struct NonDefaultable {
    data: Pin<Box<[u8]>>,
}
// Reason: Cannot default-initialize pinned memory
```

---

## Article IV: Updated Compliance Checklist (Iteration 9)

### Build Quality Gate (Amendment A + J + M — STRENGTHENED)
- [ ] `cargo build --all` exits 0 (zero errors across ALL crates)
- [ ] `cargo test --all --no-run` exits 0 (test targets compile with zero warnings)
- [ ] `cargo clippy --all --all-targets -- -D warnings` exits 0 (ZERO clippy warnings) — **STRENGTHENED**
- [ ] `cargo clippy --all --all-targets -- -D warnings` covers **ALL workspace crates** — **NEW**
- [ ] No unreachable patterns in any `match` expression
- [ ] No deprecated API usage without `#[allow(deprecated)]`
- [ ] All public types with `new()` implement `Default` or have justification

### Code Quality (Amendment I + M — Verified)
- [ ] No unused imports across all crates
- [ ] No unused variables (prefixed with `_`)
- [ ] No dead code without `#[allow(dead_code)]` justification
- [ ] No `&PathBuf` where `&Path` suffices — **NEW**
- [ ] No `HashMap::contains_key()` followed by `insert()` — **NEW**
- [ ] Complex types factored into `type` definitions — **NEW**

### Deprecated API Tracking (Amendment M — NEW)
- [ ] `AgentMode` enum: Scheduled for removal in v4.0
- [ ] `AgentConfig::mode` field: Scheduled for removal in v4.0
- [ ] `AgentConfig::tools` field: Scheduled for removal post-migration
- [ ] `AgentConfig::theme` field: Moved to `tui.json`
- [ ] `AgentConfig::keybinds` field: Moved to `tui.json`

### Interface System (Art VI)
- [ ] Desktop WebView integration functional (per Art VI §6.1) — ✅ **IMPLEMENTED**
- [ ] ACP HTTP+SSE transport functional (per Art VI §6.2) — ✅ VERIFIED
- [ ] Desktop WebView shares session state with TUI — ✅ **VERIFIED**

### TUI Plugin System (Amendment C — Verified Complete)
- [ ] All four dialog types (Alert/Confirm/Prompt/Select) implemented
- [ ] Slots system supports all `TuiSlot` variants
- [ ] Slot registration deterministic (IndexMap verified)
- [ ] Multiline input with Shift+Enter (P1-5) ✅ VERIFIED
- [ ] Home view with recent sessions and quick actions ✅ **IMPLEMENTED**

---

## Appendix A: Gap → Constitution Mapping (Iteration 9)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-8 | Clippy unreachable pattern | Amend J §J.2 | ✅ RESOLVED |
| P0-new-2 | Desktop WebView stub | Art VI §6.1 + Amend G + L | ✅ RESOLVED |
| P0-9 | Clippy failures (18 errors) | Amend M (NEW) | ❌ **NEW P0** |
| P1-2 | Circular variable expansion | Amend B §B.2 | ✅ IMPLEMENTED |
| P1-3 | Deprecated fields | Amend D §D.2 | 🚧 In progress |
| P1-9 | Session sharing | Amend K | ✅ IMPLEMENTED |
| P2-1 | VCS worktree root | Art I | ✅ IMPLEMENTED |
| P2-2 | Workspace validation | Art I | ✅ IMPLEMENTED |
| P2-9 | API error shape | Art V | ✅ IMPLEMENTED |
| P2-12 | Home view | Art VI | ✅ IMPLEMENTED |
| P2-13 | LLM reasoning budget | Amend D §D.3 | ✅ IMPLEMENTED |
| P2-14 | GitLab Duo marking | Amend D §D.3 | ✅ IMPLEMENTED |
| P2-15 | Git test code bugs | Amend E + H | ✅ FIXED |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API, Desktop/ACP |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands, dialogs/slots |
| v2.2 | Iteration 6 | I–VII + A–F | Test code quality gate, ACP verification |
| v2.3 | Iteration 7 | I–VII + A–I | Desktop WebView enforcement, Test code enforcement, Code quality debt |
| v2.4 | Iteration 8 | I–VII + A–L | Clippy linting hard gate, CLI test gate, Desktop WebView deadline escalation |
| **v2.5** | **Iteration 9** | **I–VII + A–N** | **Comprehensive clippy coverage (M), Default trait impl requirement (N)** |

---

## Priority Summary for Iteration 9

| Priority | Amendment | Action Required |
|----------|-----------|-----------------|
| **P0** | Amend M §M.1 | Fix P0-9: All 18 clippy errors in `core` and `ratatui-testing` |
| **P1** | Amend D §D.2 | Plan complete deprecated field removal for v4.0 |
| **P2** | Amend D §D.1 | Magic number thresholds (still partial) |

**Constitutional additions in Iteration 9:** 2 amendments (M: Comprehensive clippy coverage strengthening, N: Default trait implementation requirement)

---

## Immediate Actions (Before Next Iteration)

### P0-9: Fix All 18 Clippy Errors

**ratatui-testing (1 error):**
```rust
// File: ratatui-testing/src/state.rs
// Add Default implementation for StateTester
impl Default for StateTester {
    fn default() -> Self {
        Self::new()
    }
}
```

**opencode-core (17 errors):**

1. Fix deprecated `AgentMode` usage (2 errors) — remove or use `#[allow(deprecated)]`
2. Fix deprecated `AgentConfig::mode` field (2 errors) — remove or use `#[allow(deprecated)]`
3. Fix `question_mark` in config.rs:1594 — use `?` operator
4. Fix `needless_borrows_for_generic_args` in config.rs:2068 — borrow correctly
5. Fix `redundant_closure` in session_sharing.rs:323 — use `ok_or`
6. Fix `map_entry` in session_sharing.rs:225 — use entry API
7. Fix `and_then` → `map` in crash_recovery.rs:241
8. Fix `very_complex_type` in skill.rs — factor into type definition
9. Fix `&PathBuf` → `&Path` (5 occurrences) in skill.rs:116

---

## Summary

**Overall Completion: ~90-92%**

**Key Achievements in Iteration 9:**
- ✅ P0-8 Clippy unreachable pattern FIXED
- ✅ P0-new-2 Desktop WebView IMPLEMENTED
- ✅ P1-2 Circular detection IMPLEMENTED
- ✅ P1-9 Session sharing IMPLEMENTED
- ✅ All P2 items from iteration-8 FIXED
- ✅ All dead code (DC-1 through DC-10) CLEANED UP

**Remaining Constitutional Gap:**
- ❌ P0-9: Amendment J insufficient — needs Amendment M strengthening

---

*Constitution v2.5 — Iteration 9*
*Total constitutional articles: 7 (original) + 14 amendments*
*P0 blockers constitutionally covered: 4 (all addressed, 1 remains unimplemented)*
*P0 blocker implementation status: 1 remains (P0-9 clippy failures)*
*Report generated: 2026-04-12*
