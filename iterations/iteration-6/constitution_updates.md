# Constitution Updates - Iteration 6

**Generated:** 2026-04-12
**Based on Gap Analysis:** `iteration-6/gap-analysis.md`
**Previous Constitution:** `iteration-5/constitution_updates.md` (v2.1)
**Status:** Amendment Proposal — Iteration 6 Review

---

## Executive Summary

Iteration 5's Constitution (v2.1) with Amendments A-D successfully addressed:
- P0-new-1: Git crate syntax error ✅ RESOLVED
- P0-new-3: ACP HTTP+SSE transport ✅ IMPLEMENTED
- P1-7: TUI Plugin dialogs ✅ IMPLEMENTED
- P1-8: TUI Plugin slots ✅ IMPLEMENTED

**New Gap Identified:**
- P2-15: Git crate has 11 test compilation errors (duplicate test names, missing imports)

**Assessment:** The existing Constitution covers all P0 blockers. Only 1 new P2 issue requires constitutional coverage.

---

## Article I: Coverage Reassessment (Iteration 6)

### Previously Mandated vs. Iteration 6 Status

| Constitution Reference | Mandate | Iteration 6 Status |
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
| Art VI §6.1 | Desktop WebView | ❌ Still stub (P0-new-2) |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ **IMPLEMENTED** |
| Amend A §A.1 | Build integrity gate | ✅ RESOLVED (P0-new-1) |
| Amend B §B.1 | JSONC error messages | ⚠️ Deferred (P1-1) |
| Amend B §B.2 | Circular variable expansion | ⚠️ Deferred (P1-2) |
| Amend C §C.1 | Slash command contract | ✅ VERIFIED |
| Amend C §C.2 | TUI Plugin dialogs | ✅ **IMPLEMENTED** (P1-7) |
| Amend C §C.3 | Slots system | ✅ **IMPLEMENTED** (P1-8) |
| Amend D §D.1 | Magic number thresholds | ⚠️ Still partial |
| Amend D §D.2 | Deprecated field warnings | ⚠️ Still partial |
| Amend D §D.3 | Experimental marking | ✅ VERIFIED |

---

## Article II: Iteration 6 Achievements

### Successfully Resolved (No Constitutional Changes Needed)

| Item | Gap ID | Resolution |
|------|--------|------------|
| P0-new-1 | Git crate syntax error | Build succeeds, tests have separate issues |
| P0-new-3 | ACP HTTP+SSE transport | Full transport layer implemented |
| P1-7 | TUI Plugin dialogs | All 4 dialogs (Alert/Confirm/Prompt/Select) implemented |
| P1-8 | TUI Plugin slots | Full slot registration system implemented |
| P1-10 | Permission inheritance edge cases | Test coverage added |
| P1-11 | Request validation edge cases | Tests added |
| P2-10 | Plugin cleanup/unload | Implemented |

### P0 Blocker Remaining

| Item | Gap ID | Status | Notes |
|------|--------|--------|-------|
| Desktop WebView | P0-new-2 | ❌ Stub | Only HTTP server + browser open |

**Note:** Art VI §6.1 was constitutionally mandated in Iteration 4. ACP transport (same article) was also mandated and has now been implemented. Desktop WebView remains the sole P0 blocker.

---

## Amendment E: Test Code Quality Gate (New — Not in Prior Constitution)

### Section E.1 — Test Compilation as Hard Gate

**Gap Addressed:** P2-15 — Git crate has 11 test compilation errors

```
error[E0428]: name `test_gitlab_pipeline_trigger` defined multiple times
error[E0428]: name `test_gitlab_pipeline_status_monitoring` defined multiple times
error[E0433]: failed to resolve: use of undeclared type `Ordering`
error[E0425]: cannot find function `next_port` in this scope
```

**Root Cause:** Duplicate test module definitions in `crates/git/src/gitlab_ci.rs` (lines 405+ and 697+)

**Requirement:** ALL test targets MUST compile without errors:

```
Test Gate Protocol:
  1. cargo build --all           → MUST pass (zero errors)
  2. cargo test --all --no-run  → MUST pass (test targets compile)
```

**CONSTRAINT:** Test code is code. It MUST follow the same quality standards:

```rust
// PROHIBITED in test modules
mod gitlab_ci_tests {
    mod inner { /* tests */ }  // ← duplicate module names cause E0428
    
    use std::sync::atomic::Ordering;  // ← missing import causes E0433
    
    fn next_port() -> u16 { ... }     // ← helper must be defined or imported
}

// REQUIRED: Unique names, proper imports, explicit helper definitions
```

**CONSTRAINT:** Test modules MUST NOT define duplicate symbols across module boundaries.

**Verification:**
```bash
# This command MUST succeed with exit code 0
cargo test --all --no-run 2>&1 | grep -c "^error" | xargs -I{} test {} -eq 0
```

### Section E.2 — Test Module Structure Guidelines

**Gap Addressed:** P2-15 — Duplicate test names due to module structure issues

**Requirement:** Test modules MUST follow explicit structure:

```rust
// GOOD: Unique module names
#[cfg(test)]
mod tests_gitlab_ci_trigger {
    use super::*;
    
    #[tokio::test]
    async fn test_gitlab_pipeline_trigger() { ... }
}

#[cfg(test)]
mod tests_gitlab_ci_status {
    use super::*;
    
    #[tokio::test]
    async fn test_gitlab_pipeline_status_monitoring() { ... }
}

// GOOD: Use unique test function names across entire crate
#[tokio::test]
async fn test_gitlab_pipeline_trigger_and_monitor_end_to_end() { ... }
```

**CONSTRAINT:** Test function names MUST be unique within their parent module. Consider using longer, descriptive names to avoid collisions.

### Section E.3 — Test Helper Functions

**Gap Addressed:** P2-15 — Missing `next_port()` helper and `Ordering` import

**Requirement:** Test helpers MUST be properly defined or imported:

```rust
// Option 1: Define locally (preferred for portability)
fn next_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    static COUNTER: AtomicU16 = AtomicU16::new(9000);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

// Option 2: Import from common test utilities
use tests::common::helpers::next_port;

// Option 3: Use port reservation pattern
async fn bind_available_port() -> tokio::net::TcpListener {
    tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap()
}
```

**CONSTRAINT:** All `use` statements in test modules MUST resolve. Missing imports are compilation errors.

---

## Amendment F: ACP Transport Verification (Art VI §6.2 Completed)

### Section F.1 — ACP Implementation Verification

**Gap Addressed:** P0-new-3 — ACP HTTP+SSE transport implemented

**Status:** ✅ IMPLEMENTED

| Component | Status | Location |
|-----------|--------|----------|
| ACP handshake | ✅ Done | `crates/server/src/routes/acp.rs` |
| ACP connect | ✅ Done | `crates/cli/src/cmd/acp.rs` |
| ACP HTTP+SSE transport | ✅ Done | Server routes at `/api/acp/*` |
| ACP status endpoint | ✅ Done | `GET /api/acp/status` |

**CONSTRAINT:** ACP transport is now considered **constitutionally verified**. Future changes to ACP MUST maintain SSE semantics (not polling).

---

## Updated Compliance Checklist (Article VII Amendment)

### Build Quality (Amendment A — Reinforced)
- [ ] `cargo build --all` exits 0 (zero errors across ALL crates)
- [ ] `cargo test --all --no-run` exits 0 (test targets compile)
- [ ] No orphaned code outside function/module boundaries
- [ ] No duplicate test names within crate

### TUI Plugin System (Amendment C — Verified Complete)
- [ ] All four dialog types (Alert/Confirm/Prompt/Select) implemented
- [ ] Slots system supports all `TuiSlot` variants
- [ ] Slot registration deterministic (IndexMap verified)

### Interface System (Art VI)
- [ ] Desktop WebView integration functional (per Art VI §6.1) — **P0 REMAINING**
- [ ] ACP HTTP+SSE transport functional (per Art VI §6.2) — ✅ VERIFIED

### Test Quality (Amendment E — NEW)
- [ ] All test modules have unique symbol names
- [ ] All `use` statements in test modules resolve
- [ ] Test helper functions defined or properly imported
- [ ] `cargo test --all --no-run` compiles cleanly

---

## Appendix A: Gap → Constitution Mapping (Iteration 6)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-new-1 | Git crate syntax error | Amend A §A.1 | ✅ RESOLVED |
| P0-new-2 | Desktop WebView stub | Art VI §6.1 | ❌ P0 remains |
| P0-new-3 | ACP HTTP+SSE transport | Art VI §6.2 | ✅ IMPLEMENTED |
| P1-7 | TUI Plugin dialogs | Amend C §C.2 | ✅ IMPLEMENTED |
| P1-8 | TUI Plugin slots | Amend C §C.3 | ✅ IMPLEMENTED |
| **P2-15** | **Git test code bugs** | **Amend E (NEW)** | ❌ NEW |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API, Desktop/ACP |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands, dialogs/slots |
| **v2.2** | **Iteration 6** | **I–VII + A–F** | **Test code quality gate, ACP verification** |

---

## Priority Summary for Iteration 6

| Priority | Amendment | Action Required |
|----------|-----------|-----------------|
| **P0** | Art VI §6.1 | Implement Desktop WebView (ONLY P0 REMAINING) |
| **P2** | Amend E §E.1 | Fix git test code compilation errors |

**Constitutional additions in Iteration 6:** 1 new amendment (E: Test Code Quality)

---

## Remaining Deferred Items (Non-Blocking)

| Item | Gap ID | Constraint | Target |
|------|--------|------------|--------|
| JSONC error messages | P1-1 | Improve error quality | Future |
| Circular variable detection | P1-2 | Add detection algorithm | Future |
| Deprecated field warnings | P1-3, Amend D §D.2 | Emit warnings at startup | Future |
| Multiline input | P1-5 | Shift+Enter support | Future |
| Session sharing | P1-9 | Cross-interface sync | Future |
| Magic number thresholds | Amend D §D.1 | Move to config | Future |

---

*Constitution v2.2 — Iteration 6*
*Total constitutional articles: 7 (original) + 6 amendments*
*P0 blockers constitutionally covered: 3 (all resolved except Desktop WebView)*
*P2 gaps constitutionally covered: 1 new (test code quality)*
*Report generated: 2026-04-12*
