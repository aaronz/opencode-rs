# Format Module Specification v47

**Version:** 47
**Date:** 2026-04-22
**Source PRD:** `packages/opencode/src/format/` (TypeScript)
**Implementation Source:** `opencode-rust/crates/core/src/formatter.rs`, `opencode-rust/crates/tools/src/formatter_hook.rs`

---

## 1. Module Overview

| Field | Value |
|-------|-------|
| **Module Name** | format |
| **Source Path** | `opencode-rust/crates/core/src/formatter.rs`, `opencode-rust/crates/tools/src/formatter_hook.rs` |
| **Type** | Service / Effect Layer |
| **Rust Crate** | `opencode-format` (planned) |
| **Purpose** | Provides automatic code formatting after file edits by detecting and running the appropriate formatter for each file extension. Supports 25+ formatters across many languages. |

---

## 2. Current Implementation State

### 2.1 Existing Components

| Component | Location | Status |
|-----------|----------|--------|
| `FormatterEngine` | `crates/core/src/formatter.rs` | ✅ Implemented |
| `FormatterConfig` | `crates/config/src/lib.rs:598-600` | ✅ Implemented |
| `FormatterEntry` | `crates/config/src/lib.rs:604-616` | ✅ Implemented |
| `format_file_after_write` hook | `crates/tools/src/formatter_hook.rs` | ✅ Implemented |

### 2.2 Current Capabilities

- `$FILE` placeholder substitution ✅ (`formatter.rs:91-94`)
- Environment variable passing ✅ (`formatter.rs:99-101`)
- Sequential formatter execution (sorted by name) ✅
- Best-effort formatting (failures don't propagate) ✅
- Timeout handling with kill on timeout ✅
- Basic unit tests ✅
- `FormatterConfig::Disabled(false)` returns empty matcher ✅
- `FormatterConfig::Formatters` accepts custom formatters ✅

---

## 3. Functionality Specification

### 3.1 Core Features

#### FR-001: Formatter Engine Service
- [x] `FormatterEngine` struct manages formatter configuration
- [x] `FormatterConfig` supports `Disabled(bool)` and `Formatters(HashMap)` variants
- [x] `match_formatters(file_path)` returns matching `FormatterEntry` list sorted by name
- [ ] Implement Effect-based service layer with `init()`, `status()`, `file()` methods
- [ ] Integrate with InstanceState for per-directory state scoping

#### FR-002: Format File Execution
- [x] `format_file(file_path, project_root)` executes matching formatters sequentially
- [x] `$FILE` placeholder replaced with actual file path
- [x] Environment variables passed to child processes
- [x] Timeout (10s default) with kill on timeout
- [x] Best-effort: failures log warnings but don't propagate errors
- [ ] Parallel `enabled()` checks for different formatters

#### FR-003: Built-in Formatter Registry
- [ ] Implement all 25+ built-in formatters with `enabled()` checks:

| Formatter | Extensions | Binary Detection |
|-----------|-----------|------------------|
| `gofmt` | `.go` | `which gofmt` |
| `mix` | `.ex,.exs,.eex,.heex` | `which mix` |
| `prettier` | `.js,.ts,.html,.css,.json,.yaml,.md,...` | `package.json` has `prettier` dep |
| `oxfmt` | `.js,.ts,...` | `OPENCODE_EXPERIMENTAL_OXFMT` flag + `package.json` |
| `biome` | `.js,.ts,.html,.css,...` | `biome.json` config + `@biomejs/biome` bin |
| `zig` | `.zig,.zon` | `which zig` |
| `clang-format` | `.c,.cc,.cpp,.h,...` | `.clang-format` config file |
| `ktlint` | `.kt,.kts` | `which ktlint` |
| `ruff` | `.py,.pyi` | `which ruff` + ruff config |
| `uvformat` | `.py,.pyi` | `which uv` (fallback when ruff absent) |
| `air` | `.R` | `which air` + validates R formatter |
| `rubocop` | `.rb,.rake,...` | `which rubocop` |
| `standardrb` | `.rb,...` | `which standardrb` |
| `htmlbeautifier` | `.erb,.html.erb` | `which htmlbeautifier` |
| `dart` | `.dart` | `which dart` |
| `ocamlformat` | `.ml,.mli` | `.ocamlformat` config file |
| `terraform` | `.tf,.tfvars` | `which terraform` |
| `latexindent` | `.tex` | `which latexindent` |
| `gleam` | `.gleam` | `which gleam` |
| `shfmt` | `.sh,.bash` | `which shfmt` |
| `nixfmt` | `.nix` | `which nixfmt` |
| `rustfmt` | `.rs` | `which rustfmt` |
| `pint` | `.php` | `composer.json` has `laravel/pint` |
| `ormolu` | `.hs` | `which ormolu` |
| `cljfmt` | `.clj,.cljs,...` | `which cljfmt` |
| `dfmt` | `.d` | `which dfmt` |

#### FR-004: Status API
- [ ] Implement `status()` returning `Vec<FormatterStatus>` with:
  - `name: String`
  - `extensions: Vec<String>`
  - `enabled: bool`

#### FR-005: Ruff/UV Linked Disabling
- [ ] When `ruff` is disabled, also disable `uv`
- [ ] When `uv` is disabled, also disable `ruff`
- [ ] They share the same backend binary, so disabling either removes both

### 3.2 Config Integration

| Config Value | Behavior |
|--------------|----------|
| `formatter: false` | No formatters loaded (engine disabled) |
| `formatter: true` | All built-in formatters loaded (when implemented) |
| `formatter: { <name>: {...} }` | Per-formatter override (disabled, command, extensions) |

### 3.3 Formatter Override Shape

```rust
pub struct FormatterEntry {
    pub disabled: Option<bool>,
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    pub extensions: Option<Vec<String>>,
}
```

---

## 4. API Surface

### 4.1 FormatterEngine (Current)

```rust
impl FormatterEngine {
    pub fn new(config: FormatterConfig) -> Self
    pub fn is_enabled(&self) -> bool
    pub fn match_formatters(&self, file_path: &str) -> Vec<&FormatterEntry>
    pub async fn format_file(&self, file_path: &str, project_root: &Path) -> Result<(), FormatterError>
}
```

### 4.2 Planned FormatService (Effect-based)

```rust
// Status returned by status()
#[derive(Serialize)]
pub struct FormatterStatus {
    pub name: String,
    pub extensions: Vec<String>,
    pub enabled: bool,
}

// Effect-based service interface
pub trait FormatService {
    async fn init(&self) -> Result<(), FormatterError>;
    async fn status(&self) -> Vec<FormatterStatus>;
    async fn file(&self, filepath: &Path) -> Result<(), FormatterError>;
}
```

### 4.3 FormatterTrait (for built-in formatters)

```rust
#[async_trait]
pub trait Formatter: Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn environment(&self) -> Option<&HashMap<String, String>> { None }
    async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>>;
}

pub struct FormatterContext {
    pub directory: PathBuf,
    pub worktree: PathBuf,
}
```

---

## 5. Data Structures

### 5.1 Internal State (Current)

```rust
pub struct FormatterEngine {
    config: HashMap<String, FormatterEntry>,
    timeout: Duration,
    enabled: bool,
}
```

### 5.2 Planned Per-Directory State

```rust
pub struct FormatServiceState {
    formatters: HashMap<String, Box<dyn Formatter>>,
    commands: Mutex<HashMap<String, Option<Vec<String>>>>,
}
```

---

## 6. Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| `tokio` | Async runtime for child processes | ✅ |
| `tokio::process::Command` | Spawning formatter subprocesses | ✅ |
| `serde` / `serde_json` | Config deserialization | ✅ |
| `glob::Pattern` | Pattern matching for file extensions | ✅ |
| `which` crate | Finding binaries in PATH | Planned |
| `tracing` | Logging formatter activity | ✅ |
| `effect` | Service/layer injection | Planned |
| `tempfile` | Test support | ✅ |

---

## 7. File Locations

| File | Purpose |
|------|---------|
| `opencode-rust/crates/core/src/formatter.rs` | Main formatter engine |
| `opencode-rust/crates/tools/src/formatter_hook.rs` | Hook for formatting after writes |
| `opencode-rust/crates/config/src/lib.rs:598-616` | Config types |
| `opencode-rust/crates/core/src/format.rs` | Unrelated `FormatUtils` |
| `opencode-rust/crates/format/` | **Planned**: Dedicated format crate |

---

## 8. Gap Analysis Summary

### P0 Blockers (Must Fix)

| Gap | Severity | Fix |
|-----|----------|-----|
| No dedicated `opencode-format` crate | P0 | Create `crates/format/` |
| No Effect-based service layer | P0 | Implement `FormatService` with Effect/Layer |
| No built-in formatter registry (25+ formatters) | P0 | Add all formatters with `enabled()` checks |
| No `status()` API | P0 | Implement `status()` returning `FormatterStatus[]` |

### P1 Issues (Should Fix)

| Gap | Severity | Fix |
|-----|----------|-----|
| Ruff/UV linked disabling not implemented | P1 | Add logic to remove both when either disabled |
| No per-directory state scoping via InstanceState | P1 | Integrate with InstanceState |
| Environment variable support | P1 | ✅ Already implemented |

### P2 Issues (Nice to Have)

| Gap | Severity | Fix |
|-----|----------|-----|
| No parallel `enabled()` checks | P2 | Use `tokio::join!` |
| Limited glob matching in `formatter_hook.rs` | P2 | Unify with `formatter.rs` logic |
| No integration tests for format module | P2 | Add tests in `crates/format/tests/` |

---

## 9. Acceptance Criteria

| Criteria | Status | Notes |
|----------|--------|-------|
| `Format.Service` via Effect DI | ❌ | Not implemented |
| `status()` returns empty when `formatter: false` | ❌ | No `status()` method |
| `status()` returns all 25+ formatters when `formatter: true` | ❌ | No built-in formatters |
| `status()` excludes formatters marked `disabled: true` | ✅ | `formatter.rs:56` |
| Disabling `ruff` removes `uv` | ❌ | No linked disabling |
| Disabling `uv` removes `ruff` | ❌ | No linked disabling |
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

## 10. Test Design

### 10.1 Unit Tests (Existing)

- `match_formatters_matches_typescript_extension`
- `format_file_executes_command_with_file_replaced`
- `disabled_config_reports_not_enabled`
- `multiple_formatters_execute_in_order`
- `timeout_is_enforced_and_failure_is_not_fatal`

### 10.2 Planned Unit Tests

| Test | Description |
|------|-------------|
| `status_empty_when_disabled` | `status()` returns empty when `formatter: false` |
| `status_includes_gofmt_when_all_enabled` | `status()` includes gofmt when all enabled |
| `status_excludes_disabled_formatter` | `status()` excludes formatters marked disabled |
| `disabling_ruff_removes_uv` | Disabling ruff also removes uv |
| `disabling_uv_removes_ruff` | Disabling uv also removes ruff |
| `matching_formatters_run_sequentially` | Multiple formatters run in order |
| `formatter_state_isolated_per_directory` | State is per-directory scoped |
| `file_placeholder_substituted` | `$FILE` replaced with actual path |
| `failed_formatter_does_not_panic` | Failures don't propagate |
| `enabled_checks_run_in_parallel` | `enabled()` checks run concurrently |

### 10.3 Integration Test Mapping

| TS Test | Rust Equivalent |
|---------|-----------------|
| `status() returns empty list when no formatters` | `status_empty_when_disabled` |
| `status() returns built-in formatters when formatter is true` | `status_includes_gofmt_when_all_enabled` |
| `status() keeps built-in formatters when config object provided` | `config_object_keeps_all_builtins` |
| `status() excludes formatters marked as disabled` | `status_excludes_disabled_formatter` |
| `status() excludes uv when ruff is disabled` | `disabling_ruff_removes_uv` |
| `status() excludes ruff when uv is disabled` | `disabling_uv_removes_ruff` |
| `service initializes without error` | `service_initializes_without_error` |
| `status() initializes formatter state per directory` | `formatter_state_isolated_per_directory` |
| `runs enabled checks for matching formatters in parallel` | `enabled_checks_run_in_parallel` |
| `runs matching formatters sequentially for the same file` | `matching_formatters_run_sequentially` |

---

## 11. Implementation Roadmap

### Phase 1: Foundation (P0)
1. Create `crates/format/` crate with `Cargo.toml` and `src/lib.rs`
2. Define `Formatter` trait and `FormatterContext`
3. Implement `FormatService` with Effect layer

### Phase 2: Built-in Formatters (P0)
4. Implement all 25+ formatters with `enabled()` checks
5. Add binary detection using `which` crate
6. Implement `status()` API returning `FormatterStatus[]`

### Phase 3: Enhanced Features (P1)
7. Implement Ruff/UV linked disabling
8. Integrate with InstanceState for per-directory scoping
9. Add parallel `enabled()` checks with `tokio::join!`

### Phase 4: Testing & Polish (P2)
10. Add comprehensive integration tests
11. Unify glob pattern matching across modules
12. Add tracing spans for observability

---

## 12. Technical Debt

| Item | Description | Impact |
|------|-------------|--------|
| Duplicate formatter logic | `formatter.rs` and `formatter_hook.rs` overlap | Maintenance |
| No dedicated crate | Format module mixed with core | Harder to test/publish |
| Hardcoded timeout | `Duration::from_secs(10)` in two places | Magic number |
| No formatter name in success log | Only logs on failure | Debug difficulty |
| No metrics/observability | No tracing spans | Operations |

---

*Document Version: 47*
*Last Updated: 2026-04-22*
