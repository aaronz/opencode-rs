# Rust Testing Best Practices: Implementation Plan

**Date**: 2026-04-24  
**Design Reference**: `docs/DESIGN/test-design.md`  
**Status**: Draft - Awaiting Confirmation

---

## 1. Requirements Restatement

Implement a comprehensive Rust testing infrastructure based on the `test-design.md` best practices guide. The current codebase has solid fundamentals (unit tests, integration tests, async tests, mocks) but lacks:

1. **Property-based and fuzz testing** for invariant validation
2. **Mutation testing** for test suite strength
3. **Snapshot/golden file testing** with `insta`
4. **Dedicated `test-support` crate** for workspace-wide helpers
5. **Fuzzing infrastructure** with corpus management
6. **Staged CI pipeline** with layered test execution
7. **Feature matrix testing** for untested combinations
8. **Slow test isolation** for DB/container tests

---

## 2. Gap Summary

| Area | Severity | Status |
|------|----------|--------|
| Property/Fuzz Testing | P0 | Missing |
| CI Staged Jobs | P0 | Missing |
| Feature Matrix CI | P1 | Missing |
| Mutation Testing | P1 | Missing |
| Snapshot/Golden Workflow | P1 | Partial |
| `test-support` Crate | P1 | Partial |
| Fixtures Organization | P1 | Scattered |
| Slow Test Isolation | P1 | Missing |
| Platform Matrix | P2 | macOS only |
| DB/Container Tests | P2 | Unstructured |

---

## 3. Implementation Phases

### Phase 1: CI Enhancement

**Estimated**: 1-2 days

#### 1.1 Add `cargo nextest` for faster test execution

**Why**: `cargo nextest` provides:
- 2-10x faster test execution through parallelization
- Better CI reporting with test retries and flaky test detection
- Per-test isolation

**Changes**:
- Add to `ci.yml`:
  ```yaml
  - uses: taiki-e/install-action@nextest
  - run: cargo nextest run --workspace
  ```
- Add `.config/nextest.toml` for configuration
- Keep `cargo test` as fallback

#### 1.2 Add feature matrix testing

**Why**: Current CI only tests default features. Feature combinations may have bugs.

**Changes**:
- Add job in `ci.yml`:
  ```yaml
  feature-matrix:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --workspace --no-default-features
      - run: cargo test --workspace --all-features
  ```
- Consider `cargo hack test --workspace --feature-powerset` for comprehensive matrix

#### 1.3 Add platform matrix

**Why**: Path handling, filesystem behavior, and process handling differ across platforms.

**Changes**:
- Modify test job:
  ```yaml
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
  ```

#### 1.4 Split benchmark CI job

**Why**: Benchmarks should actually run, not just compile.

**Changes**:
- Add benchmark execution on main/dev branches
- Compare against baseline on PRs

---

### Phase 2: Test Infrastructure

**Estimated**: 2-3 days

#### 2.1 Add `proptest` for property testing

**Why**: Validates invariants across many inputs, catches edge cases.

**Changes**:

**Step 1**: Add to workspace dev-dependencies:
```toml
# Cargo.toml (workspace)
[workspace.dev-dependencies]
proptest = "1"
```

**Step 2**: Create property tests for high-value invariants:

```rust
// crates/config/tests/property_tests.rs
proptest! {
    #[test]
    fn normalize_is_idempotent(s in "\\PC*") {
        let once = normalize(&s);
        let twice = normalize(&once);
        prop_assert_eq!(once, twice);
    }
}

proptest! {
    #[test]
    fn parse_config_roundtrips(input in "\\PC*") {
        if let Ok(cfg) = parse_config(&input) {
            let encoded = to_string(&cfg)?;
            let decoded = parse_config(&encoded)?;
            prop_assert_eq!(cfg, decoded);
        }
    }
}
```

**Step 3**: Add to existing crates:
- `crates/config/` - Config parsing invariants
- `crates/tools/` - Tool argument validation
- `crates/core/` - Path normalization, message serialization

#### 2.2 Set up `cargo-fuzz` for fuzzing

**Why**: Finds crashes and security bugs in parsers and deserializers.

**Changes**:

**Step 1**: Initialize fuzz directory:
```bash
cargo fuzz init
```

**Step 2**: Create `fuzz/Cargo.toml`:
```toml
[package]
name = "opencode-fuzz"
version = "0.1.0"
edition = "2021"

[dependencies]
opencode-config.workspace = true
opencode-core.workspace = true

[profile.release]
opt-level = 3

[[bin]]
name = "parse_config"
path = "fuzz_targets/parse_config.rs"
```

**Step 3**: Add fuzz targets:
```rust
// fuzz/fuzz_targets/parse_config.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use opencode_config::parse_config;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = parse_config(input);
    }
});
```

**Step 4**: Create corpus directories:
```
fuzz/
  corpus/
    parse_config/
      valid_001.toml
      valid_002.toml
  fuzz_targets/
    parse_config.rs
    decode_message.rs
```

#### 2.3 Add `insta` for snapshot testing

**Why**: Simplifies golden file testing with automatic update workflow.

**Changes**:

**Step 1**: Add dependency:
```toml
# Cargo.toml
insta = { version = "1", features = ["json", "yaml"] }
```

**Step 2**: Update snapshot tests:
```rust
// Example: crates/config/tests/snapshot_tests.rs
#[test]
fn test_config_snapshot() {
    let cfg = Config::load("fixtures/test.toml").unwrap();
    insta::assert_json_snapshot!(cfg);
}
```

**Step 3**: Create snapshot directory:
```
crates/config/
  tests/
    snapshots/
      test_config_snapshot.snap
```

**Step 4**: Add to CI:
```yaml
- name: Review snapshots
  run: cargo insta test && cargo insta review
```

#### 2.4 Add mutation testing

**Why**: Validates test suite strength by checking if tests catch mutated code.

**Changes**:

**Step 1**: Add to workspace:
```toml
[workspace.dev-dependencies]
cargo-mutants = "0.7"
```

**Step 2**: Add nightly CI job:
```yaml
mutation:
  runs-on: ubuntu-latest
  if: github.event_name == 'schedule' # Nightly
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@nightly
    - run: cargo mutants run --workspace --fail-on-mutants
```

**Step 3**: Add to CI with schedule:
```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily
```

---

### Phase 3: Test-Support Crate

**Estimated**: 1-2 days

#### 3.1 Create `crates/test-support/` crate

**Why**: Centralized helpers for workspace-wide test utilities.

**Changes**:

**Step 1**: Create crate:
```
crates/test-support/
  Cargo.toml
  src/
    lib.rs
    fixtures.rs
    temp.rs
    http.rs
    db.rs
    assertions.rs
```

**Step 2**: Define `Cargo.toml`:
```toml
[package]
name = "opencode-test-support"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
tempfile.workspace = true
wiremock = "0.6"
anyhow.workspace = true

# For temp dir helpers
assert_fs.workspace = true
predicates.workspace = true
```

**Step 3**: Migrate from `tests/src/common/`:
- `TempProject` → `crates/test-support/src/temp.rs`
- `MockServer` → `crates/test-support/src/http.rs`
- Add `Fixtures` struct for loading test data

**Step 4**: Update crates to use:
```toml
# crates/storage/Cargo.toml
[dev-dependencies]
opencode-test-support.workspace = true
```

---

### Phase 4: Slow Test Isolation

**Estimated**: 1-2 days

#### 4.1 Add slow test annotations

**Why**: Fast feedback requires isolating slow tests.

**Changes**:

**Step 1**: Add ignore annotations:
```rust
#[tokio::test]
#[ignore = "requires postgres container"]
async fn test_migration_with_real_db() {
    // ...
}
```

**Step 2**: Add feature gates:
```rust
#[cfg(feature = "db-tests")]
#[tokio::test]
async fn test_db_migration() {
    // ...
}
```

#### 4.2 Add DB test container setup

**Why**: Real database behavior testing without polluting CI environment.

**Changes**:

**Step 1**: Add testcontainer dependency:
```toml
[dev-dependencies]
testcontainers = "12"
testcontainers-postgres = "16"
```

**Step 2**: Create test pool helper:
```rust
// crates/test-support/src/db.rs
pub async fn test_db_pool() -> Pool {
    let container = PostgreSQL::new().await.unwrap();
    let pool = Pool::connect(container.connection_string()).await.unwrap();
    pool
}
```

**Step 3**: Add dedicated CI job:
```yaml
db-tests:
  runs-on: ubuntu-latest
  services:
    postgres:
      image: postgres:16
      env:
        POSTGRES_PASSWORD: postgres
  steps:
    - run: cargo test --workspace --features db-tests
```

---

### Phase 5: Fixture Organization

**Estimated**: 0.5-1 day

#### 5.1 Create workspace fixtures directory

**Why**: Centralized, consistent fixture management.

**Changes**:

**Step 1**: Create structure:
```
opencode-rust/
  fixtures/
    configs/
      minimal.toml
      full.toml
    golden/
      expected_output.txt
    corpora/
      parse_config/
        valid/
        invalid/
```

**Step 2**: Create fixture helper:
```rust
// crates/test-support/src/fixtures.rs
pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).unwrap()
}
```

**Step 3**: Move existing fixtures:
- Consolidate all `include_str!("../fixtures/")` calls
- Standardize on `opencode-test-support::fixtures`

---

## 4. Dependency Additions

Summary of new dependencies to add to workspace:

```toml
# Dev dependencies to add
[workspace.dev-dependencies]
proptest = "1"
insta = { version = "1", features = ["json", "yaml"] }
cargo-mutants = "0.7"
testcontainers = "12"
testcontainers-postgres = "16"
wiremock = "0.6"
assert_fs = "1"

# Not in workspace yet - needs adding
# cargo-fuzz (via cargo install)
# cargo-hack (via cargo install)
# cargo-nextest (via cargo install)
```

---

## 5. CI Changes

### New `.github/workflows/ci.yml` structure:

```yaml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Nightly

jobs:
  # Stage 1: Source quality
  fmt-clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings

  # Stage 2: Fast tests
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace
      - run: cargo test --doc --workspace

  # Stage 3: Feature matrix
  feature-matrix:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --no-default-features
      - run: cargo test --workspace --all-features

  # Stage 4: Coverage
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

  # Stage 5: Nightly - Mutation
  mutation:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-mutants
      - run: cargo mutants run --workspace --fail-on-mutants

  # Stage 6: Nightly - Fuzz
  fuzz:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run --all-targets -- -max_total_time=60s

  # Stage 7: Build
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release

  # Stage 8: Benchmarks (baseline tracking)
  benchmarks:
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --workspace
```

---

## 6. File Changes Summary

### New Files
```
docs/PLAN/
  ├── README.md                           # This file
  ├── phase1-ci-enhancement.md            # CI changes details
  ├── phase2-test-infrastructure.md        # Property/fuzz/insta setup
  ├── phase3-test-support-crate.md         # test-support crate
  ├── phase4-slow-test-isolation.md         # DB/container tests
  └── phase5-fixture-organization.md        # Fixture consolidation

opencode-rust/
  ├── crates/test-support/                 # NEW - workspace test helpers
  │   ├── Cargo.toml
  │   └── src/
  │       ├── lib.rs
  │       ├── fixtures.rs
  │       ├── temp.rs
  │       ├── http.rs
  │       └── db.rs
  ├── fuzz/                                # NEW - fuzzing infrastructure
  │   ├── Cargo.toml
  │   ├── fuzz_targets/
  │   └── corpus/
  └── fixtures/                            # NEW - workspace fixtures
      ├── configs/
      ├── golden/
      └── corpora/
```

### Modified Files
```
opencode-rust/
  ├── Cargo.toml                          # Add dev dependencies
  ├── .github/workflows/ci.yml             # Restructure CI
  ├── tests/src/common/                    # Migrate to test-support
  └── crates/*/tests/*.rs                 # Add property tests
```

---

## 7. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| CI time increase from staged jobs | HIGH | Use `cargo nextest` for speed; cache aggressively |
| Mutation tests flaky on CI | MEDIUM | Run only nightly, allow mutants in specific dirs |
| Fuzz corpus bloat | MEDIUM | Size-limit corpus, useReproducer corpus |
| Property tests slow down CI | LOW | Use `#[test]` not `#[proptest]` for slow cases |
| test-support crate circular deps | HIGH | Follow dependency rules: test-support only uses prod crates |

---

## 8. Estimated Complexity

| Phase | Complexity | Time |
|-------|------------|------|
| Phase 1: CI Enhancement | MEDIUM | 1-2 days |
| Phase 2: Test Infrastructure | HIGH | 2-3 days |
| Phase 3: test-support Crate | MEDIUM | 1-2 days |
| Phase 4: Slow Test Isolation | LOW | 1-2 days |
| Phase 5: Fixture Organization | LOW | 0.5-1 day |

**Total**: 5.5-10.5 days

---

## 9. Dependencies Between Phases

```
Phase 1 (CI Enhancement)
    └── Enables faster iteration for all subsequent phases

Phase 2 (Test Infrastructure)
    ├── Phase 2.1 (proptest) can proceed independently
    ├── Phase 2.2 (fuzz) can proceed independently
    ├── Phase 2.3 (insta) can proceed independently
    └── Phase 2.4 (mutants) depends on Phase 1 (CI speed)

Phase 3 (test-support Crate)
    └── Can proceed in parallel with Phase 2

Phase 4 (Slow Test Isolation)
    └── Depends on Phase 3 (test-support db helpers)

Phase 5 (Fixtures)
    └── Depends on Phase 3 (test-support fixtures module)
```

---

## 10. Acceptance Criteria

- [ ] `cargo nextest run --workspace` completes in < 5 minutes on CI
- [ ] Feature matrix tests run on every PR
- [ ] Platform matrix (ubuntu, macos, windows) passes
- [ ] `cargo llvm-cov --workspace` shows > 80% coverage
- [ ] At least 5 property tests added for core invariants
- [ ] At least 3 fuzz targets created with corpus
- [ ] `cargo insta test` works and snapshots are committed
- [ ] Nightly mutation test runs and reports results
- [ ] `crates/test-support/` crate exists and is used by at least 3 other crates
- [ ] Slow tests are isolated behind `#[ignore]` or feature flags
- [ ] All fixtures are loaded via `opencode-test-support::fixtures`

---

## 11. Next Steps

**WAITING FOR CONFIRMATION**: Proceed with this plan?

Reply with:
- **"yes"** or **"proceed"** - Start with Phase 1 (CI Enhancement)
- **"modify: [changes]"** - Suggest modifications
- **"start at phase 2"** - Skip CI changes and begin with test infrastructure
- **"phase 3 only"** - Only create test-support crate