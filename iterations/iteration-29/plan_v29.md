# Implementation Plan - Iteration 29
# OpenCode RS

**Date:** 2026-04-17
**Iteration:** 29
**Status:** Active Development

---

## 1. Overview

This plan outlines implementation tasks based on the Iteration 29 Spec and Gap Analysis. Overall implementation is ~92% complete with focus on P0 critical items and P1 high-priority improvements.

## 2. Priority Classification

### P0 - Critical (Must Fix)
- FR-017: Production `unwrap()` elimination (3484+ instances)
- FR-029: Error handling standardization (thiserror adoption)

### P1 - High Priority
- FR-018: Test coverage enforcement (80% threshold)
- FR-028: Visibility boundary audit (~3896+ pub declarations)
- FR-024: Plugin API version stability policy
- FR-025: WebSocket streaming capability verification
- FR-026: SDK documentation CI integration

### P2 - Medium Priority
- FR-023: Unsafe code SAFETY comments
- FR-019: Benchmark suite CI integration
- FR-027: TOML config migration tooling

---

## 3. P0 Implementation Plan

### FR-017: Production unwrap() Elimination

**Objective:** Zero `.unwrap()` or `.expect()` in production code

**High-Risk Files:**
| File | Issue |
|------|-------|
| `crates/tools/src/edit.rs:159` | `let idx = index.unwrap();` |
| `crates/tools/src/web_search.rs:70` | `let api_key = api_key.unwrap();` |
| `crates/server/src/routes/*.rs` | Untyped String errors |

**Approach:**
1. Audit all production code for `.unwrap()` occurrences
2. Convert to proper error propagation with `?`
3. Use `thiserror` for typed errors in library crates
4. Use `anyhow` for flexible context in application crates

**Verification:**
```bash
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l
```

---

### FR-029: Error Handling Standardization

**Objective:** 100% thiserror adoption in library crates

**Requirements:**
- All library crates use `thiserror` for typed errors
- Application crates may use `anyhow` for flexible context
- No `.unwrap()` or `.expect()` in production code

**Affected Crates:**
- `crates/server/src/routes/` - Convert String errors to thiserror
- `crates/tools/` - Fix unwrap() in edit.rs, web_search.rs
- All other crates - Audit and fix

---

## 4. P1 Implementation Plan

### FR-018: Test Coverage Enforcement

**Objective:** 80% line coverage across all crates

**Current Coverage Gaps:**
| Crate | Current | Target | Delta |
|-------|---------|--------|-------|
| `agent` | ~45% | 80%+ | +35% |
| `server` | ~40% | 80%+ | +40% |
| `tools` | ~50% | 80%+ | +30% |
| `cli` | ~50% | 80%+ | +30% |
| `llm` | ~55% | 80%+ | +25% |
| `core` | ~60% | 80%+ | +20% |
| `tui` | ~60% | 80%+ | +20% |
| `storage` | ~70% | 80%+ | +10% |
| `plugin` | ~70% | 80%+ | +10% |
| `config` | ~70% | 80%+ | +10% |
| `auth` | ~75% | 80%+ | +5% |

**Approach:**
1. Add `cargo-llvm-cov` to CI pipeline
2. Set `--fail-under-lines 80` threshold
3. Incrementally increase coverage in each crate

**CI Command:**
```bash
cargo llvm-cov --fail-under-lines 80
```

---

### FR-028: Visibility Boundary Audit

**Objective:** Reduce ~3896+ pub declarations to necessary public API

**Approach:**
1. Default to private visibility
2. Use `pub(crate)` for internal crate sharing
3. Only mark `pub` what is part of public API
4. Audit each crate systematically

**Verification:**
```bash
grep -n "pub fn\|pub struct\|pub enum" crates/core/src/ | head -50
```

---

### FR-024: Plugin API Version Stability

**Objective:** Define version lifecycle for plugin ecosystem

**Requirements:**
- Document plugin ABI version lifecycle
- Define compatibility guarantees
- Establish stability policy

**Approach:**
1. Define version scheme (major.minor.patch)
2. Document breaking change policy
3. Add version checks to plugin runtime

---

### FR-025: WebSocket Streaming Verification

**Objective:** Verify full bidirectional WebSocket capability

**Current Status:**
- SSE implemented in `routes/stream.rs`
- `routes/ws.rs` module exists

**Verification Needed:**
- Confirm ws module provides full WebSocket vs SSE
- Document capability differences

---

### FR-026: SDK Documentation CI

**Objective:** Add documentation generation to CI pipeline

**Current Status:**
- Doc comments on public API
- No `cargo doc --no-deps` in CI

**Approach:**
1. Add to CI pipeline: `cargo doc --no-deps --all-features --no-deps`
2. Consider docs.rs publishing setup

---

## 5. P2 Implementation Plan

### FR-023: Unsafe Code SAFETY Comments

**Missing SAFETY Comments:**
| File | Line | Issue |
|------|------|-------|
| `crates/plugin/src/lib.rs` | 661 | Missing SAFETY comment |
| `crates/tui/src/app.rs` | 4677, 4690 | Missing SAFETY comments |
| `crates/server/src/routes/validation.rs` | 237, 256 | Missing SAFETY comments |

**Approach:**
1. Audit all `unsafe` blocks
2. Add SAFETY comments explaining justification
3. Verify invariants are documented

**Verification:**
```bash
grep -n "unsafe" crates/*/src/*.rs | grep -v "test"
```

---

### FR-019: Benchmark CI Integration

**Objective:** Add benchmarks to CI pipeline

**Current Status:**
- Benchmarks exist in `opencode-benches/`
- Not run in CI

**Approach:**
1. Add `cargo bench` to CI pipeline
2. Set up performance regression detection
3. Document baseline metrics

---

### FR-027: TOML Config Migration

**Objective:** Migrate from TOML to JSONC configuration

**Current Status:**
- TOML format deprecated
- Shows warning but works

**Approach:**
1. Provide migration tooling
2. Auto-convert TOML to JSONC on load
3. Remove TOML support after transition

---

## 6. CI Pipeline Updates

### Current State
| Stage | Status |
|-------|--------|
| Format check | ✅ |
| Clippy | ⚠️ Warnings exist |
| Unit tests | ✅ |
| Integration | ✅ |
| Build | ✅ |
| Coverage | ❌ Not in CI |
| Audit | ❌ Not in CI |
| Deny | ❌ Not in CI |
| Benchmarks | ❌ Not in CI |

### Required Additions
1. `cargo llvm-cov --fail-under-lines 80`
2. `cargo doc --no-deps --all-features`
3. `cargo bench` (for regression)
4. `cargo audit`
5. `cargo deny check`

---

## 7. Timeline

| Phase | Tasks | Duration |
|-------|-------|----------|
| Phase 1 | P0: unwrap() elimination, error handling | 1-2 weeks |
| Phase 2 | P1: Coverage, visibility audit | 2-3 weeks |
| Phase 3 | P1: Plugin API, WebSocket, SDK docs | 1 week |
| Phase 4 | P2: SAFETY comments, benchmarks, TOML | 1 week |

---

## 8. Success Criteria

- [ ] Zero production `.unwrap()` or `.expect()`
- [ ] All library crates use thiserror
- [ ] 80%+ test coverage across all crates
- [ ] Visibility audit complete
- [ ] Plugin API stability policy documented
- [ ] WebSocket capability verified
- [ ] SDK docs in CI pipeline
- [ ] Benchmark suite in CI
- [ ] TOML migration tooling provided

---

**End of Plan**