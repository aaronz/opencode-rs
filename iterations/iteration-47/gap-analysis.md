# Format Module Gap Analysis Report

**Generated:** 2026-04-22
**PRD Source:** `packages/opencode/src/format/` (TypeScript)
**Implementation Source:** `opencode-rust/crates/core/src/formatter.rs`

---

## 1. Gap Summary

| Gap Item | Severity | Module |修复建议 |
|----------|----------|--------|---------|
| No dedicated `opencode-format` crate | P0 | Architecture | Create `crates/format/` crate |
| No Effect-based service layer | P0 | Architecture | Implement `FormatService` with Effect/Layer |
| No built-in formatter registry (25+ formatters) | P0 | formatter.rs | Add all 25+ formatters with `enabled()` checks |
| No `status()` API | P0 | formatter.rs | Implement `status()` returning `FormatterStatus[]` |
| Ruff/UV linked disabling not implemented | P1 | formatter.rs | Add logic to remove both when either is disabled |
| No per-directory state scoping via InstanceState | P1 | Architecture | Integrate with InstanceState for directory-scoped formatters |
| No environment variable support in FormatterEntry | P1 | config | Already exists, verify integration |
| `$FILE` placeholder not substituted in `formatter_hook.rs` | P1 | formatter_hook.rs | Already implemented, verify |
| No parallel `enabled()` checks | P2 | formatter.rs | Use `tokio::join!` or similar for parallel checks |
| Limited glob pattern matching in `formatter_hook.rs` | P2 | formatter_hook.rs | Use same logic as `formatter.rs` |
| No integration tests for format module | P2 | Testing | Add tests in `crates/format/tests/` |

---

## 2. P0 Blockers (Must Fix)

### 2.1 No Dedicated `opencode-format` Crate

**Current State:** Formatter logic is scattered across `crates/core/src/formatter.rs` and `crates/tools/src/formatter_hook.rs`.

**PRD Requirement:** "Rust Crate: `opencode-format` (or within `opencode-tools`)"

**Gap:** No dedicated `crates/format/` directory. The formatter module is not isolated as its own crate.

**Fix:** Create `crates/format/` with:
```
crates/format/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── service.rs      (FormatService)
    ├── formatters/     (individual formatter implementations)
    └── config.rs
```

---

### 2.2 No Effect-Based Service Layer

**Current State:** `FormatterEngine` in `crates/core/src/formatter.rs` is a simple struct with async methods.

**PRD Requirement:** "An Effect-based service that... Initializes formatter state per project instance (directory-scoped)"

**Gap:** No `Effect`, `Layer`, `Context.Service` usage. The PRD specifies:
```rust
export class Service extends Context.Service<Service, Interface>()("@opencode/Format") {}
export const layer: Layer.Layer<Service, never, Config.Service | ChildProcessSpawner>
export const defaultLayer: Layer.Layer<Service>
```

**Fix:** Implement Effect service layer with proper dependency injection.

---

### 2.3 No Built-in Formatter Registry (25+ Formatters)

**Current State:** `FormatterEngine` accepts config but has no built-in formatters.

**PRD Requirement:** 25+ built-in formatters with `enabled()` checks:
- gofmt, mix, prettier, oxfmt, biome, zig, clang-format, ktlint, ruff, rlang/air, uvformat, rubocop, standardrb, htmlbeautifier, dart, ocamlformat, terraform, latexindent, gleam, shfmt, nixfmt, rustfmt, pint, ormolu, cljfmt, dfmt

**Gap:** None of these are implemented. Config only accepts user-defined formatters.

**Fix:** Implement all 25+ formatters with `enabled()` checks that detect binary availability.

---

### 2.4 No `status()` API

**Current State:** `FormatterEngine` has `match_formatters()` but no `status()` returning `FormatterStatus[]`.

**PRD Requirement:**
```rust
export const Status = z.object({
  name: z.string(),
  extensions: z.string().array(),
  enabled: z.boolean(),
})
export type Status = { name: string; extensions: string[]; enabled: boolean }

export interface Interface {
  readonly init: () => Effect.Effect<void>
  readonly status: () => Effect.Effect<Status[]>
  readonly file: (filepath: string) => Effect.Effect<void>
}
```

**Gap:** No `status()` method returning enabled/disabled status of all formatters.

**Fix:** Implement `status()` that returns `Vec<FormatterStatus>` with name, extensions, and enabled flag.

---

## 3. P1 Issues (Should Fix)

### 3.1 Ruff/UV Linked Disabling Not Implemented

**Current State:** Ruff and UV are independent.

**PRD Requirement:** "Disabling either `ruff` OR `uv` removes BOTH from the active formatters (they share the same backend binary)."

**Gap:** Disabling ruff does not disable uv and vice versa.

**Fix:** Add logic in `match_formatters()` or `status()` to check if either ruff or uv is disabled and remove both.

---

### 3.2 No Per-Directory State Scoping via InstanceState

**Current State:** `FormatterEngine::new()` takes `FormatterConfig` but state is not scoped per directory.

**PRD Requirement:** "Formatter state is scoped per project directory (InstanceState)"

**Gap:** Each `FormatterEngine` instance is independent; no per-directory scoping mechanism.

**Fix:** Integrate with InstanceState or similar to maintain per-directory formatter state.

---

### 3.3 Environment Variable Support Not Fully Integrated

**Current State:** `FormatterEntry` has `environment: Option<HashMap<String, String>>` field.

**Gap:** Need to verify `environment` is passed to child processes in `format_file()`.

**Current code in `formatter.rs:99-101`:**
```rust
if let Some(environment) = formatter.environment.as_ref() {
    cmd.envs(environment);
}
```
This looks correct.

---

### 3.4 `$FILE` Placeholder Substitution

**Current State:** Implemented in `formatter.rs:91-94`.

**Gap:** Need to verify it works correctly and also in `formatter_hook.rs`.

**Current code in `formatter.rs`:**
```rust
let args = command[1..]
    .iter()
    .map(|arg| arg.replace("$FILE", file_path))
    .collect();
```

This is correct.

---

## 4. P2 Issues (Nice to Have)

### 4.1 No Parallel `enabled()` Checks

**Current State:** `enabled()` checks are not performed; formatters are configured statically.

**PRD Requirement:** "Checks formatter availability in parallel across different formatters"

**Gap:** No `enabled()` async check mechanism for built-in formatters.

**Fix:** When implementing built-in formatters, use `tokio::join!` or `futures::future::join_all` to check availability in parallel.

---

### 4.2 Limited Glob Pattern Matching in `formatter_hook.rs`

**Current State:** `formatter_hook.rs:102-113` uses simple extension matching.

**Gap:** `formatter.rs:153-182` uses glob patterns but `formatter_hook.rs` uses simpler logic.

**Fix:** Unify matching logic or port glob pattern support to `formatter_hook.rs`.

---

### 4.3 No Integration Tests

**Current State:** Basic unit tests exist in `formatter.rs`.

**PRD Requirement:** Tests for all acceptance criteria including:
- status_empty_when_disabled
- status_includes_gofmt_when_all_enabled
- disabling_ruff_removes_uv
- disabling_uv_removes_ruff
- matching_formatters_run_sequentially
- formatter_state_isolated_per_directory
- enabled_checks_run_in_parallel

**Gap:** No dedicated test file for format module.

**Fix:** Create `crates/format/tests/format_tests.rs` with comprehensive tests.

---

## 5. Technical Debt

| Item | Description | Impact |
|------|-------------|--------|
| Duplicate formatter logic | `formatter.rs` and `formatter_hook.rs` have overlapping code | Maintenance burden |
| No dedicated crate | Format module mixed with core | Harder to maintain, test, publish |
| Hardcoded timeout | `Duration::from_secs(10)` in two places | Magic number |
| No logging of formatter name in success case | Only logs on failure | Debug difficulty |
| No metrics/observability | No tracing spans for formatter execution | Operational blindness |

---

## 6. Implementation Progress

### Completed ✓
- [x] `FormatterConfig` enum in `crates/config/src/lib.rs:598-600`
- [x] `FormatterEntry` struct in `crates/config/src/lib.rs:604-616`
- [x] `$FILE` placeholder substitution in `formatter.rs:91-94`
- [x] Environment variable passing in `formatter.rs:99-101`
- [x] Sequential formatter execution (ordering preserved by sorting name)
- [x] Best-effort formatting (failures don't propagate)
- [x] Timeout handling with kill on timeout
- [x] Basic unit tests in `formatter.rs`

### Not Started / Incomplete
- [ ] Create `crates/format/` crate
- [ ] Implement Effect service layer
- [ ] Implement all 25+ built-in formatters with `enabled()` checks
- [ ] Implement `status()` API
- [ ] Implement Ruff/UV linked disabling
- [ ] Per-directory state scoping via InstanceState
- [ ] Parallel `enabled()` checks
- [ ] Comprehensive integration tests

---

## 7. Acceptance Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| `Format.Service` via Effect DI | ❌ Not implemented | No Effect layer |
| `status()` returns empty when `formatter: false` | ❌ Not implemented | No `status()` method |
| `status()` returns all 25+ formatters when `formatter: true` | ❌ Not implemented | No built-in formatters |
| `status()` excludes formatters marked `disabled: true` | ✅ Implemented | `formatter.rs:56` |
| Disabling `ruff` removes `uv` | ❌ Not implemented | No linked disabling |
| Disabling `uv` removes `ruff` | ❌ Not implemented | No linked disabling |
| `file(path)` runs ALL matching formatters | ✅ Implemented | `formatter.rs:79-143` |
| Multiple formatters run sequentially | ✅ Implemented | Sorted by name |
| Availability checks run in parallel | ❌ Not implemented | No `enabled()` checks |
| Custom `command` override works | ✅ Implemented | Via config |
| Custom `extensions` override works | ✅ Implemented | Via config |
| Environment variables passed | ✅ Implemented | `formatter.rs:99-101` |
| `$FILE` placeholder substituted | ✅ Implemented | `formatter.rs:93` |
| Failed formatter doesn't throw | ✅ Implemented | Best-effort |
| Formatter state per directory | ❌ Not implemented | No InstanceState integration |

---

## 8. Recommended Fix Order

1. **Create `crates/format/` crate** - Establish proper module structure
2. **Implement Effect service layer** - `FormatService` with `init()`, `status()`, `file()`
3. **Add built-in formatters** - Implement all 25+ formatters with `enabled()` checks
4. **Implement Ruff/UV linked disabling** - Add special case logic
5. **Add per-directory state scoping** - Integrate with InstanceState
6. **Add parallel enabled checks** - Use `tokio::join!`
7. **Write integration tests** - Comprehensive test coverage

---

## Appendix: Current File Locations

| File | Purpose |
|------|---------|
| `crates/core/src/formatter.rs` | Main formatter engine |
| `crates/tools/src/formatter_hook.rs` | Hook for formatting after file writes |
| `crates/config/src/lib.rs:598-616` | Config types (`FormatterConfig`, `FormatterEntry`) |
| `crates/core/src/format.rs` | Unrelated utility functions (`FormatUtils`) |
