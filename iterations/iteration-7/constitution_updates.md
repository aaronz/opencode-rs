# Constitution Updates - Iteration 7

**Generated:** 2026-04-12
**Based on Gap Analysis:** `iteration-7/gap-analysis.md`
**Previous Constitution:** `iteration-6/constitution_updates.md` (v2.2)
**Status:** Amendment Proposal — Iteration 7 Review

---

## Executive Summary

Iteration 6's Constitution (v2.2) with Amendments A-F addressed multiple P0/P1 issues. The Iteration 7 gap analysis reveals:

1. **P1-5: Multiline input** ✅ IMPLEMENTED (Shift+Enter support)
2. **P2-7: Context cost warnings** ✅ IMPLEMENTED
3. **P2-6: Per-server OAuth** ✅ VERIFIED
4. **P2-10: Plugin cleanup** ✅ VERIFIED
5. **P0-new-2: Desktop WebView** ❌ Still stub (only P0 remaining)
6. **P2-15: Git test code bugs** ❌ Still buggy (Amendment E not enforced)

**Assessment:** Art VI §6.1 (Desktop WebView) is constitutionally covered but unimplemented. Amendment E (Test Code Quality Gate) was added but P2-15 persists. Constitution v2.3 requires strengthening test code enforcement and escalating Desktop WebView to explicit milestone.

---

## Article I: Coverage Reassessment (Iteration 7)

### Previously Mandated vs. Iteration 7 Status

| Constitution Reference | Mandate | Iteration 7 Status |
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
| Amend A §A.1 | Build integrity gate | ✅ Verified |
| Amend B §B.1 | JSONC error messages | ✅ **IMPLEMENTED** (P1-1) |
| Amend B §B.2 | Circular variable expansion | ⚠️ Deferred (P1-2) |
| Amend C §C.1 | Slash command contract | ✅ Verified |
| Amend C §C.2 | TUI Plugin dialogs | ✅ **IMPLEMENTED** (P1-7) |
| Amend C §C.3 | Slots system | ✅ **IMPLEMENTED** (P1-8) |
| Amend D §D.1 | Magic number thresholds | ⚠️ Still partial |
| Amend D §D.2 | Deprecated field warnings | ⚠️ Still partial |
| Amend D §D.3 | Experimental marking | ✅ VERIFIED |
| Amend E §E.1 | Test compilation gate | ❌ **NOT ENFORCED** (P2-15 persists) |
| Amend F §F.1 | ACP transport verification | ✅ VERIFIED |

---

## Article II: Iteration 7 Achievements

### Successfully Resolved (No Constitutional Changes Needed)

| Item | Gap ID | Resolution |
|------|--------|------------|
| P1-5 | Multiline input terminal support | ✅ Shift+Enter implemented in `input_widget.rs` |
| P2-7 | Context cost warnings | ✅ Implemented in `context_cost.rs` |
| P2-6 | Per-server OAuth | ✅ Verified |
| P2-10 | Plugin cleanup/unload | ✅ Verified |

### P0 Blocker Remaining

| Item | Gap ID | Status | Notes |
|------|--------|--------|-------|
| Desktop WebView | P0-new-2 | ❌ Stub | Only HTTP server + browser open |

**Note:** Art VI §6.1 has been constitutionally mandated since Iteration 4. This is the **ONLY remaining P0 blocker**.

### P2 Issue Still Buggy

| Item | Gap ID | Status | Notes |
|------|--------|--------|-------|
| Git test code bugs | P2-15 | ❌ Buggy | 8 duplicate test names, unused imports |

**Note:** Amendment E was added in Iteration 6 but P2-15 persists. The amendment is adequate; enforcement is lacking.

---

## Amendment G: Desktop WebView Enforcement (Art VI §6.1 Escalation)

### Section G.1 — Desktop WebView Milestone

**Gap Addressed:** P0-new-2 — Desktop WebView remains stub after 3 iterations

**Status:** ❌ Still stub since Iteration 4

| Component | Status | Location |
|-----------|--------|----------|
| `desktop.rs` WebView creation | ❌ Stub | `crates/cli/src/cmd/desktop.rs` |
| `wry` WebView integration | Partial | `crates/cli/src/webview.rs` |
| Desktop mode lifecycle | ❌ Incomplete | Not integrated with app state |

**CONSTRAINT:** Art VI §6.1 has been unimplemented for 3 iterations. This is now a **critical architectural debt**.

**Requirement:** Desktop WebView MUST be delivered with the following properties:

```rust
// DESKTOP SHELL ARCHITECTURE (Required)
trait DesktopShell {
    fn spawn_webview(&self) -> Result<WebViewHandle>;
    fn share_session(&self, session: &Session) -> Result<()>;
    fn close(&self) -> Result<()>;
}

// WebView MUST share state with:
// 1. TUI input/output
// 2. Server session store
// 3. Agent runtime

// PROHIBITED: WebView as isolated browser window
// REQUIRED: WebView as integrated desktop shell component
```

**CONSTRAINT:** Desktop mode is NOT complete if:
1. The WebView cannot receive agent output in real-time
2. User input in WebView does not reach the agent runtime
3. Session state diverges between TUI and WebView

**Verification:**
```bash
# Desktop mode MUST:
# 1. Build with --features desktop
# 2. Create WebView window (not just spawn browser)
# 3. Pass integration test: input in WebView → agent receives it
```

---

## Amendment H: Test Code Enforcement (Amendment E Strengthening)

### Section H.1 — Test Code Compilation Hard Gate

**Gap Addressed:** P2-15 — Git crate test compilation errors persist despite Amendment E

**Status:** Amendment E (Iteration 6) was added but NOT enforced

**Current Issues in `crates/git/src/gitlab_ci.rs`:**

```
warning: function `next_port` is never used
warning: struct `GitLabMockServer` is never constructed
warning: associated items `new`, `handle_request`, `url`, and `stop` are never used
```

**Root Cause:** Amendment E specified requirements but did not mandate automated enforcement in CI.

**CONSTRAINT:** Amendment E §E.1 is REITERATED with enforcement:

```bash
# MUST be added to CI pipeline (if not already present)
cargo test --all --no-run 2>&1 | grep -c "^error" | xargs -I{} test {} -eq 0
```

**CONSTRAINT:** Any crate with test compilation warnings MUST be treated as a build failure for PR merging.

**Required Actions:**
1. Remove or use `next_port()` function in `git/gitlab_ci.rs:413`
2. Remove or use `GitLabMockServer` struct in `git/gitlab_ci.rs:706`
3. Clean up all associated unused items

### Section H.2 — Orphaned Test Code Prohibition

**Gap Addressed:** P2-15 — Unused test helpers and mock servers

**CONSTRAINT:** Test code that is never called or constructed is technical debt and MUST be removed:

```rust
// PROHIBITED: Orphaned test infrastructure
#[cfg(test)]
mod unused_mocks {
    struct GitLabMockServer { ... }  // ← never constructed
    fn next_port() -> u16 { ... }   // ← never called
}

// REQUIRED: Either delete or integrate
#[cfg(test)]
mod test_infrastructure {
    use super::*;
    
    // Helper used by actual tests
    pub fn next_port() -> u16 { ... }
    
    // Mock used by actual tests  
    pub struct MockServer { ... }
}
```

---

## Amendment I: Code Quality Debt (New)

### Section I.1 — Low-Severity Warnings as Debt

**Gap Addressed:** CQ-1 through CQ-9 — Accumulated unused code warnings

**CONSTRAINT:** Unused imports, variables, and functions are technical debt. They MUST be cleaned before PR merge:

| ID | Item | Location | Fix |
|----|------|----------|-----|
| CQ-1 | Unused `Message` import | core/crash_recovery.rs:1 | Remove `Message` from import |
| CQ-2 | Unused `SecretStorage` methods | core/config/secret_storage.rs:36 | Remove or mark `#[allow(dead_code)]` with justification |
| CQ-3 | Unused `e` variable | tools/lsp_tool.rs:311,526,626,783 | Rename to `_e` |
| CQ-4 | Unused `body` variable | git/github.rs:566 | Rename to `body: _` |
| CQ-5 | Unused `next_port` function | git/gitlab_ci.rs:413 | Remove or use |
| CQ-6 | Unused `GitLabMockServer` | git/gitlab_ci.rs:706 | Remove or use |
| CQ-7 | Unused imports | cli/src/cmd/quick.rs:5-6 | Remove `save_session_records`, `SessionRecord` |
| CQ-8 | Unused `save_session_records` | cli/src/cmd/session.rs:42 | Remove or use |
| CQ-9 | Unused `complete` variable | cli/src/cmd/mcp_auth.rs:216 | Rename to `_complete` |

**CONSTRAINT:** `#[allow(dead_code)]` MAY be used only if:
1. The item is public API required for trait conformance
2. A comment explains why it must exist but cannot be used

---

## Updated Compliance Checklist (Article VII Amendment)

### Build Quality (Amendment A — Verified)
- [ ] `cargo build --all` exits 0 (zero errors across ALL crates)
- [ ] `cargo test --all --no-run` exits 0 (test targets compile with zero warnings)
- [ ] No orphaned code outside function/module boundaries
- [ ] No duplicate test names within crate

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

### Code Quality (Amendment I — NEW)
- [ ] No unused imports across all crates
- [ ] No unused variables (prefixed with `_`)
- [ ] No dead code without `#[allow(dead_code)]` justification

---

## Appendix A: Gap → Constitution Mapping (Iteration 7)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-new-1 | Git crate syntax error | Amend A §A.1 | ✅ RESOLVED |
| P0-new-2 | Desktop WebView stub | Art VI §6.1 + Amend G | ❌ **P0 remains** |
| P0-new-3 | ACP HTTP+SSE transport | Art VI §6.2 | ✅ VERIFIED |
| P1-1 | JSONC error messages | Amend B §B.1 | ✅ IMPLEMENTED |
| P1-2 | Circular variable expansion | Amend B §B.2 | ⚠️ Deferred |
| P1-3 | Deprecated fields | Art III §3.1 + Amend D §D.2 | ⚠️ Deferred |
| P1-5 | Multiline input | Amend C §C.1 | ✅ **IMPLEMENTED** |
| P1-7 | TUI Plugin dialogs | Amend C §C.2 | ✅ IMPLEMENTED |
| P1-8 | TUI Plugin slots | Amend C §C.3 | ✅ IMPLEMENTED |
| P2-6 | Per-server OAuth | Art IV | ✅ VERIFIED |
| P2-7 | Context cost warnings | Art IV | ✅ IMPLEMENTED |
| P2-10 | Plugin cleanup | Art VIII | ✅ VERIFIED |
| P2-14 | Experimental marking | Amend D §D.3 | ✅ VERIFIED |
| **P2-15** | **Git test code bugs** | **Amend E + H** | ❌ **Persists** |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API, Desktop/ACP |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands, dialogs/slots |
| v2.2 | Iteration 6 | I–VII + A–F | Test code quality gate, ACP verification |
| **v2.3** | **Iteration 7** | **I–VII + A–I** | **Desktop WebView enforcement, Test code enforcement strengthening, Code quality debt** |

---

## Priority Summary for Iteration 7

| Priority | Amendment | Action Required |
|----------|-----------|-----------------|
| **P0** | Art VI §6.1 + Amend G | Implement Desktop WebView (ONLY P0 REMAINING) |
| **P1** | Amend H §H.1 | Enforce Amendment E — fix P2-15 git test code |
| **P2** | Amend I §I.1 | Clean up 9 code quality warnings |
| **P2** | Amend D §D.1 | Magic number thresholds (still partial) |

**Constitutional additions in Iteration 7:** 3 amendments (G: Desktop WebView enforcement, H: Test code enforcement strengthening, I: Code quality debt)

---

## Remaining Deferred Items (Non-Blocking)

| Item | Gap ID | Constraint | Target |
|------|--------|------------|--------|
| JSONC error messages | P1-1 | ✅ Implemented | - |
| Circular variable detection | P1-2 | Add detection algorithm | Future |
| Deprecated field warnings | P1-3, Amend D §D.2 | Emit warnings at startup | Future |
| Session sharing | P1-9 | Cross-interface sync | Future |
| Magic number thresholds | Amend D §D.1 | Move to config | Future |

---

*Constitution v2.3 — Iteration 7*
*Total constitutional articles: 7 (original) + 9 amendments*
*P0 blockers constitutionally covered: 3 (all resolved except Desktop WebView)*
*P0 blocker implementation status: Desktop WebView (P0-new-2) is ONLY remaining*
*P2 gaps constitutionally covered: 2 new (test code enforcement, code quality debt)*
*Report generated: 2026-04-12*
