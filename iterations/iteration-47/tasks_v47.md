# Format Module Task List v47

**Version:** 47
**Last Updated:** 2026-04-22
**Priority:** P0 tasks must be completed before P1, P1 before P2

---

## P0 Tasks (Must Fix)

### Create Format Crate Structure

- [x] Create `crates/format/Cargo.toml` with dependencies
- [ ] Create `crates/format/src/lib.rs` with module exports
- [ ] Create `crates/format/src/service.rs` for FormatService
- [ ] Create `crates/format/src/formatters.rs` for built-in formatters
- [ ] Create `crates/format/src/config.rs` for config integration

### Implement Effect Service Layer

- [ ] Define `FormatterStatus` struct (name, extensions, enabled)
- [ ] Define `FormatterContext` struct (directory, worktree)
- [ ] Define `Formatter` trait with `name()`, `extensions()`, `environment()`, `enabled()`
- [ ] Implement `FormatService` with `init()`, `status()`, `file()` methods
- [ ] Integrate with Effect/Layer for dependency injection

### Implement All 25+ Built-in Formatters

- [x] **gofmt** (.go) - `which gofmt`
- [x] **mix** (.ex,.exs,.eex,.heex) - `which mix`
- [x] **prettier** (.js,.ts,.html,.css,.json,.yaml,.md,...) - package.json has prettier dep
- [ ] **oxfmt** (.js,.ts,...) - OPENCODE_EXPERIMENTAL_OXFMT flag + package.json
- [ ] **biome** (.js,.ts,.html,.css,...) - biome.json config + @biomejs/biome
- [ ] **zig** (.zig,.zon) - `which zig`
- [ ] **clang-format** (.c,.cc,.cpp,.h,...) - .clang-format config file
- [x] **ktlint** (.kt,.kts) - `which ktlint`
- [x] **ruff** (.py,.pyi) - `which ruff` + ruff config
- [ ] **uvformat** (.py,.pyi) - `which uv` (fallback when ruff absent)
- [ ] **air** (.R) - `which air` + validates R formatter
- [ ] **rubocop** (.rb,.rake,...) - `which rubocop`
- [ ] **standardrb** (.rb,...) - `which standardrb`
- [x] **htmlbeautifier** (.erb,.html.erb) - `which htmlbeautifier`
- [ ] **dart** (.dart) - `which dart`
- [ ] **ocamlformat** (.ml,.mli) - .ocamlformat config file
- [ ] **terraform** (.tf,.tfvars) - `which terraform`
- [ ] **latexindent** (.tex) - `which latexindent`
- [ ] **gleam** (.gleam) - `which gleam`
- [ ] **shfmt** (.sh,.bash) - `which shfmt`
- [x] **nixfmt** (.nix) - `which nixfmt`
- [ ] **rustfmt** (.rs) - `which rustfmt`
- [ ] **pint** (.php) - composer.json has laravel/pint
- [ ] **ormolu** (.hs) - `which ormolu`
- [ ] **cljfmt** (.clj,.cljs,...) - `which cljfmt`
- [ ] **dfmt** (.d) - `which dfmt`

### Implement status() API

- [ ] Implement `status()` returning `Vec<FormatterStatus>`
- [ ] `status()` returns empty when `formatter: false`
- [ ] `status()` returns all 25+ formatters when `formatter: true`
- [ ] `status()` excludes formatters marked `disabled: true`

---

## P1 Tasks (Should Fix)

### Ruff/UV Linked Disabling

- [ ] When `ruff` is disabled, also disable `uv`
- [ ] When `uv` is disabled, also disable `ruff`
- [ ] Add tests for linked disabling

### InstanceState Integration

- [ ] Integrate FormatService with InstanceState
- [ ] Implement per-directory formatter state scoping
- [ ] Add tests for directory isolation

---

## P2 Tasks (Nice to Have)

### Parallel enabled() Checks

- [ ] Use `tokio::join!` for parallel availability checks
- [ ] Verify formatter availability in parallel

### Unify Glob Pattern Matching

- [ ] Port glob pattern support from formatter.rs to formatter_hook.rs
- [ ] Remove duplicate extension matching logic

### Integration Tests

- [ ] status_empty_when_disabled
- [ ] status_includes_gofmt_when_all_enabled
- [ ] status_excludes_disabled_formatter
- [ ] disabling_ruff_removes_uv
- [ ] disabling_uv_removes_ruff
- [ ] matching_formatters_run_sequentially
- [ ] formatter_state_isolated_per_directory
- [ ] enabled_checks_run_in_parallel
- [ ] file_placeholder_substituted
- [ ] failed_formatter_does_not_panic

### Technical Debt

- [ ] Extract hardcoded `Duration::from_secs(10)` to constant
- [ ] Add formatter name to success log
- [ ] Add tracing spans for observability

---

## Test Mapping

| TypeScript Test | Rust Test |
|-----------------|-----------|
| status() returns empty list when no formatters | status_empty_when_disabled |
| status() returns built-in formatters when formatter is true | status_includes_gofmt_when_all_enabled |
| status() keeps built-in formatters when config object provided | config_object_keeps_all_builtins |
| status() excludes formatters marked as disabled | status_excludes_disabled_formatter |
| status() excludes uv when ruff is disabled | disabling_ruff_removes_uv |
| status() excludes ruff when uv is disabled | disabling_uv_removes_ruff |
| service initializes without error | service_initializes_without_error |
| status() initializes formatter state per directory | formatter_state_isolated_per_directory |
| runs enabled checks for matching formatters in parallel | enabled_checks_run_in_parallel |
| runs matching formatters sequentially for the same file | matching_formatters_run_sequentially |

---

## Verification Commands

```bash
# Build format crate
cargo build -p opencode-format

# Run format crate tests
cargo test -p opencode-format

# Run all tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all -- -D warnings
```

---

*Document Version: 47*