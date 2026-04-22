# Format Module Implementation Plan v47

**Version:** 47
**Last Updated:** 2026-04-22
**Status:** Not Started

---

## 1. Overview

This plan implements the Format Module based on Spec v47. The module provides automatic code formatting after file edits by detecting and running the appropriate formatter for each file extension.

**Target:** Support 25+ formatters across many languages via Effect-based service layer.

---

## 2. Implementation Phases

### Phase 1: Foundation (P0)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 1.1 | Create `crates/format/` crate structure | ❌ | Cargo.toml + src/lib.rs |
| 1.2 | Define `Formatter` trait and `FormatterContext` | ❌ | Based on spec section 4.3 |
| 1.3 | Define `FormatterStatus` struct | ❌ | name, extensions, enabled |
| 1.4 | Implement `FormatService` with Effect layer | ❌ | init(), status(), file() methods |

### Phase 2: Built-in Formatters (P0)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 2.1 | Implement gofmt (.go) | ❌ | |
| 2.2 | Implement mix (.ex,.exs,.eex,.heex) | ❌ | |
| 2.3 | Implement prettier (.js,.ts,.html,.css,.json,.yaml,.md,...) | ❌ | |
| 2.4 | Implement oxfmt (.js,.ts,...) | ❌ | OPENCODE_EXPERIMENTAL_OXFMT flag |
| 2.5 | Implement biome (.js,.ts,.html,.css,...) | ❌ | |
| 2.6 | Implement zig (.zig,.zon) | ❌ | |
| 2.7 | Implement clang-format (.c,.cc,.cpp,.h,...) | ❌ | |
| 2.8 | Implement ktlint (.kt,.kts) | ❌ | |
| 2.9 | Implement ruff (.py,.pyi) | ❌ | |
| 2.10 | Implement uvformat (.py,.pyi) | ❌ | fallback when ruff absent |
| 2.11 | Implement air (.R) | ❌ | |
| 2.12 | Implement rubocop (.rb,.rake,...) | ❌ | |
| 2.13 | Implement standardrb (.rb,...) | ❌ | |
| 2.14 | Implement htmlbeautifier (.erb,.html.erb) | ❌ | |
| 2.15 | Implement dart (.dart) | ❌ | |
| 2.16 | Implement ocamlformat (.ml,.mli) | ❌ | |
| 2.17 | Implement terraform (.tf,.tfvars) | ❌ | |
| 2.18 | Implement latexindent (.tex) | ❌ | |
| 2.19 | Implement gleam (.gleam) | ❌ | |
| 2.20 | Implement shfmt (.sh,.bash) | ❌ | |
| 2.21 | Implement nixfmt (.nix) | ❌ | |
| 2.22 | Implement rustfmt (.rs) | ❌ | |
| 2.23 | Implement pint (.php) | ❌ | |
| 2.24 | Implement ormolu (.hs) | ❌ | |
| 2.25 | Implement cljfmt (.clj,.cljs,...) | ❌ | |
| 2.26 | Implement dfmt (.d) | ❌ | |

### Phase 3: Ruff/UV Linked Disabling (P1)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 3.1 | Add linked disabling logic | ❌ | Disabling ruff removes uv, and vice versa |

### Phase 4: InstanceState Integration (P1)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 4.1 | Integrate with InstanceState | ❌ | Per-directory formatter state scoping |

### Phase 5: Parallel Execution (P2)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 5.1 | Use `tokio::join!` for parallel `enabled()` checks | ❌ | |

### Phase 6: Testing & Polish (P2)

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 6.1 | Add comprehensive integration tests | ❌ | |
| 6.2 | Add tracing spans for observability | ❌ | |

---

## 3. File Locations

| File | Purpose |
|------|---------|
| `crates/format/src/lib.rs` | Crate entry point |
| `crates/format/src/service.rs` | FormatService implementation |
| `crates/format/src/formatters.rs` | Built-in formatters |
| `crates/format/src/config.rs` | Config integration |
| `crates/core/src/formatter.rs` | Existing formatter engine (to be refactored) |
| `crates/tools/src/formatter_hook.rs` | Existing hook (to be refactored) |

---

## 4. Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| tokio | Async runtime | ✅ |
| serde | Serialization | ✅ |
| glob | Pattern matching | ✅ |
| which | Binary detection | ❌ Need to add |
| effect | Service layer | ❌ Need to add |
| tracing | Logging | ✅ |

---

## 5. Acceptance Criteria

| Criteria | Status | Verification |
|----------|--------|--------------|
| `Format.Service` via Effect DI | ❌ | |
| `status()` returns empty when `formatter: false` | ❌ | |
| `status()` returns all 25+ formatters when `formatter: true` | ❌ | |
| `status()` excludes formatters marked `disabled: true` | ✅ | `formatter.rs:56` |
| Disabling `ruff` removes `uv` | ❌ | |
| Disabling `uv` removes `ruff` | ❌ | |
| `file(path)` runs ALL matching formatters | ✅ | `formatter.rs:79-143` |
| Multiple formatters run sequentially | ✅ | Sorted by name |
| Availability checks run in parallel | ❌ | No `enabled()` checks |
| Custom `command` override works | ✅ | Via config |
| Custom `extensions` override works | ✅ | Via config |
| Environment variables passed | ✅ | `formatter.rs:99-101` |
| `$FILE` placeholder substituted | ✅ | `formatter.rs:93` |
| Failed formatter doesn't throw | ✅ | Best-effort |
| Formatter state per directory | ❌ | No InstanceState integration |

---

## 6. Technical Debt to Address

| Item | Description | Fix |
|------|-------------|-----|
| Duplicate formatter logic | formatter.rs and formatter_hook.rs overlap | Unify in crates/format |
| No dedicated crate | Format module mixed with core | Create crates/format |
| Hardcoded timeout | `Duration::from_secs(10)` in two places | Extract to constant |
| No formatter name in success log | Only logs on failure | Add success logging |

---

*Document Version: 47*