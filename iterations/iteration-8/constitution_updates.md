# Constitution Updates - Iteration 8

**Generated:** 2026-04-12
**Based on Gap Analysis:** `iteration-8/gap-analysis.md`
**Previous Constitution:** `iteration-7/constitution_updates.md` (v2.3)
**Status:** Amendment Proposal — Iteration 8 Review

---

## Executive Summary

Iteration 7's Constitution (v2.3) with Amendments G-I addressed Desktop WebView enforcement, test code enforcement, and code quality debt. The Iteration 8 gap analysis reveals:

1. **P0-8 NEW: Clippy unreachable pattern** ❌ CRITICAL (blocks release)
2. **P0-new-2: Desktop WebView** ❌ Still stub (persists from Iteration 4)
3. **CLI e2e test failures** ❌ NEW (prompt history tests failing)

**Assessment:** Constitution v2.3 inadequately covers clippy linting as a hard gate. Amendment A §A.1 references build integrity but does not explicitly mandate `cargo clippy --all -D warnings` as a release gate. The new P0-8 (clippy unreachable pattern) is a direct consequence of this gap.

---

## Article I: Coverage Reassessment (Iteration 8)

### Previously Mandated vs. Iteration 8 Status

| Constitution Reference | Mandate | Iteration 8 Status |
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
| Art VI §6.1 | Desktop WebView | ❌ **Still stub (P0-new-2)** |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ **VERIFIED** |
| Amend A §A.1 | Build integrity gate | ⚠️ **INCOMPLETE** (clippy fails) |
| Amend B §B.1 | JSONC error messages | ✅ Implemented |
| Amend B §B.2 | Circular variable expansion | ⚠️ Deferred |
| Amend C §C.1 | Slash command contract | ✅ Verified |
| Amend C §C.2 | TUI Plugin dialogs | ✅ **IMPLEMENTED** |
| Amend C §C.3 | Slots system | ✅ **IMPLEMENTED** |
| Amend D §D.1 | Magic number thresholds | ⚠️ Still partial |
| Amend D §D.2 | Deprecated field warnings | ⚠️ Still partial |
| Amend D §D.3 | Experimental marking | ✅ Verified |
| Amend E §E.1 | Test compilation gate | ⚠️ Partial enforcement |
| Amend F §F.1 | ACP transport verification | ✅ VERIFIED |
| Amend G §G.1 | Desktop WebView milestone | ❌ **STILL STUB** |
| Amend H §H.1 | Test code enforcement | ⚠️ Partial (CLI tests failing) |
| Amend I §I.1 | Code quality debt | ⚠️ **NEW P0-8** |

---

## Article II: Iteration 8 Gap Analysis

### P0 Blockers Remaining

| Item | Gap ID | Module | Status | Notes |
|------|--------|--------|--------|-------|
| **Clippy unreachable pattern** | **P0-8** | **permission** | ❌ **NEW** | `intersect()` at models.rs:28 fails clippy |
| Desktop WebView integration | P0-new-2 | cli | ❌ **STUB** | Only HTTP server + browser open |

### New Issues Identified

| Item | Gap ID | Module | Status | Notes |
|------|--------|--------|--------|-------|
| Clippy unreachable pattern | P0-8 | permission | ❌ NEW P0 | `models.rs:28` - logical error |
| CLI e2e test failures | NEW | cli | ❌ NEW | `e2e_prompt_history.rs` - 2 tests failing |

---

## Amendment J: Clippy Linting Hard Gate (NEW - CRITICAL)

### Section J.1 — Clippy as Release Gate

**Gap Addressed:** P0-8 — Clippy fails with unreachable pattern, blocking release with `-D warnings`

**Status:** Amendment A §A.1 inadequately covered clippy linting

**Current Issue:**
```
error: unreachable pattern
  --> crates/permission/src/models.rs:28:51
   |
28 |             (AgentPermissionScope::ReadOnly, _) | (_, AgentPermissionScope::ReadOnly) => {
   |                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no value can reach this
```

**Root Cause:** `intersect()` function has logically redundant pattern matching:
- Line 22 handles `(Full, other) | (other, Full)` → catches `(ReadOnly, Full)` and `(Full, ReadOnly)`
- Lines 23-27 handle `(Restricted, other) | (other, Restricted)` → catches `(ReadOnly, Restricted)` and `(Restricted, ReadOnly)`
- Line 28 `(ReadOnly, _) | (_, ReadOnly)` is therefore unreachable

**CONSTRAINT:** Amendment A §A.1 is HEREBY AMENDED to explicitly include clippy:

```bash
# MUST pass ALL of the following before release:
cargo build --all                    # Zero errors
cargo test --all --no-run            # Test targets compile
cargo clippy --all -- -D warnings    # ZERO warnings (INCLUDES clippy)
```

**CONSTRAINT:** `cargo clippy --all -- -D warnings` MUST exit 0 for:
1. PR merging
2. Release tagging
3. Phase gate completion

### Section J.2 — Pattern Matching Correctness

**Gap Addressed:** P0-8 — Unreachable patterns indicate logical errors

**CONSTRAINT:** Pattern matching in `match` expressions MUST be semantically meaningful:

```rust
// PROHIBITED: Unreachable patterns (logical error)
match (a, b) {
    (Full, _) | (_, Full) => ...,
    (Restricted, other) | (other, Restricted) => match other { ... },
    (ReadOnly, _) | (_, ReadOnly) => ...  // ← UNREACHABLE
}

// REQUIRED: Exhaustive and mutually exclusive patterns
match (a, b) {
    (None, _) | (_, None) => None,
    (Full, other) | (other, Full) => other,
    (Restricted, ReadOnly) | (ReadOnly, Restricted) => Restricted,
    (ReadOnly, ReadOnly) => ReadOnly,
}
```

**Verification:**
```bash
# This MUST pass for all crates
cargo clippy --all -- -D warnings 2>&1 | grep -c "unreachable pattern" | xargs -I{} test {} -eq 0
```

---

## Amendment K: CLI Test Quality Gate (NEW)

### Section K.1 — CLI Integration Tests Must Pass

**Gap Addressed:** NEW — `e2e_prompt_history.rs` tests failing:
- `test_prompt_history_persistence` - assertion failed
- `test_prompt_history_navigation` - history.len() >= 3 assertion failed

**CONSTRAINT:** All CLI integration tests MUST pass before release:

```bash
cargo test -p opencode-cli 2>&1 | grep -E "(test result|FAILED)"
```

**CONSTRAINT:** Failing integration tests are P0 blockers, equivalent to unit test failures.

---

## Amendment L: Desktop WebView Deadline Escalation (Amendment G Update)

### Section L.1 — Desktop WebView Milestone Deadline

**Gap Addressed:** P0-new-2 — Desktop WebView stub persists since Iteration 4

**Status:** ❌ Still stub after 4 iterations

**CONSTRAINT:** Art VI §6.1 and Amendment G are HEREBY REITERATED with escalation:

The Desktop WebView issue has persisted for **4 iterations** (Iteration 4 → 8). This is now a **critical architectural debt** and **release blocker**.

**Required Resolution:**
1. Either implement the Desktop WebView per Amendment G §G.1 specification
2. Or formally deprecate the `desktop` feature and remove it from the roadmap

**Verification:**
```bash
# If desktop feature exists, it MUST:
# 1. Build with --features desktop
# 2. Create actual WebView window (not just spawn browser)
# 3. Pass integration test: WebView input → agent runtime

# OR if desktop feature is deprecated:
# 1. Remove desktop feature from Cargo.toml
# 2. Remove desktop.rs and webview.rs
# 3. Update PRD 13 to reflect deprecation
```

---

## Updated Compliance Checklist (Article VII Amendment)

### Build Quality Gate (Amendment A + J — STRENGTHENED)
- [ ] `cargo build --all` exits 0 (zero errors across ALL crates)
- [ ] `cargo test --all --no-run` exits 0 (test targets compile with zero warnings)
- [ ] `cargo clippy --all -- -D warnings` exits 0 (ZERO clippy warnings) — **NEW**
- [ ] No unreachable patterns in any `match` expression — **NEW**
- [ ] No orphaned code outside function/module boundaries

### CLI Integration Tests (Amendment K — NEW)
- [ ] `cargo test -p opencode-cli` exits 0 (ALL tests pass)
- [ ] Specifically: `e2e_prompt_history` tests must pass

### TUI Plugin System (Amendment C — Verified Complete)
- [ ] All four dialog types (Alert/Confirm/Prompt/Select) implemented
- [ ] Slots system supports all `TuiSlot` variants
- [ ] Slot registration deterministic (IndexMap verified)
- [ ] Multiline input with Shift+Enter (P1-5) ✅ VERIFIED

### Interface System (Art VI)
- [ ] Desktop WebView integration functional (per Art VI §6.1) — **P0 REMAINING**
- [ ] ACP HTTP+SSE transport functional (per Art VI §6.2) — ✅ VERIFIED
- [ ] Desktop WebView shares session state with TUI — **REQUIRED**

### Test Quality (Amendment E + H — STRENGTHENED)
- [ ] All test modules have unique symbol names
- [ ] All `use` statements in test modules resolve
- [ ] Test helper functions defined or properly imported
- [ ] `cargo test --all --no-run` compiles cleanly (ZERO warnings)
- [ ] No orphaned test infrastructure code

### Code Quality (Amendment I — Verified)
- [ ] No unused imports across all crates
- [ ] No unused variables (prefixed with `_`)
- [ ] No dead code without `#[allow(dead_code)]` justification

---

## Appendix A: Gap → Constitution Mapping (Iteration 8)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-new-1 | Git crate syntax error | Amend A §A.1 | ✅ RESOLVED |
| P0-new-2 | Desktop WebView stub | Art VI §6.1 + Amend G | ❌ **P0 remains (4 iterations)** |
| P0-new-3 | ACP HTTP+SSE transport | Art VI §6.2 | ✅ VERIFIED |
| **P0-8** | **Clippy unreachable pattern** | **Amend J (NEW)** | ❌ **NEW P0** |
| P1-1 | JSONC error messages | Amend B §B.1 | ✅ IMPLEMENTED |
| P1-2 | Circular variable expansion | Amend B §B.2 | ⚠️ Deferred |
| P1-3 | Deprecated fields | Amend D §D.2 | ⚠️ In progress |
| P1-5 | Multiline input | Amend C §C.1 | ✅ **IMPLEMENTED** |
| P1-7 | TUI Plugin dialogs | Amend C §C.2 | ✅ IMPLEMENTED |
| P1-8 | TUI Plugin slots | Amend C §C.3 | ✅ IMPLEMENTED |
| P2-6 | Per-server OAuth | Art IV | ✅ VERIFIED |
| P2-7 | Context cost warnings | Art IV | ✅ IMPLEMENTED |
| P2-10 | Plugin cleanup | Art VIII | ✅ VERIFIED |
| P2-14 | Experimental marking | Amend D §D.3 | ✅ VERIFIED |
| P2-15 | Git test code bugs | Amend E + H | ⚠️ Partial |
| **NEW** | **CLI e2e test failures** | **Amend K (NEW)** | ❌ **NEW** |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API, Desktop/ACP |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands, dialogs/slots |
| v2.2 | Iteration 6 | I–VII + A–F | Test code quality gate, ACP verification |
| v2.3 | Iteration 7 | I–VII + A–I | Desktop WebView enforcement, Test code enforcement strengthening, Code quality debt |
| **v2.4** | **Iteration 8** | **I–VII + A–L** | **Clippy linting hard gate, CLI test gate, Desktop WebView deadline escalation** |

---

## Priority Summary for Iteration 8

| Priority | Amendment | Action Required |
|----------|-----------|-----------------|
| **P0** | Amend J §J.1 | Fix P0-8: clippy unreachable pattern in `permission/models.rs:28` |
| **P0** | Art VI §6.1 + Amend G + L | Implement or deprecate Desktop WebView |
| **P0** | Amend K §K.1 | Fix CLI e2e test failures in `e2e_prompt_history.rs` |
| **P1** | Amend H §H.1 | Enforce Amendment E — fix remaining test code issues |
| **P2** | Amend D §D.1 | Magic number thresholds (still partial) |

**Constitutional additions in Iteration 8:** 3 amendments (J: Clippy linting hard gate, K: CLI test quality gate, L: Desktop WebView deadline escalation)

---

## Immediate Actions (Before Next Iteration)

### P0-8: Fix Clippy Unreachable Pattern
```rust
// File: opencode-rust/crates/permission/src/models.rs
// Fix the intersect() function to remove unreachable pattern at line 28

// Current (BUGGY):
pub fn intersect(self, other: AgentPermissionScope) -> AgentPermissionScope {
    match (self, other) {
        (None, _) | (_, None) => None,
        (Full, other) | (other, Full) => other,
        (Restricted, other) | (other, Restricted) => match other {
            None => None,
            _ => other,
        },
        (ReadOnly, _) | (_, ReadOnly) => ReadOnly,  // UNREACHABLE
    }
}

// REQUIRED FIX:
pub fn intersect(self, other: AgentPermissionScope) -> AgentPermissionScope {
    match (self, other) {
        (None, _) | (_, None) => None,
        (Full, other) | (other, Full) => other,
        (Restricted, ReadOnly) | (ReadOnly, Restricted) => Restricted,
        (ReadOnly, ReadOnly) => ReadOnly,
    }
}
```

### K.1: Fix CLI e2e Test Failures
- `test_prompt_history_persistence` — investigate history persistence
- `test_prompt_history_navigation` — investigate history.len() >= 3

---

*Constitution v2.4 — Iteration 8*
*Total constitutional articles: 7 (original) + 12 amendments*
*P0 blockers constitutionally covered: 4 (Git crate, Desktop WebView, Clippy gate, CLI tests)*
*P0 blocker implementation status: 2 remain (Desktop WebView since Iter 4, Clippy P0-8 NEW)*
*Report generated: 2026-04-12*