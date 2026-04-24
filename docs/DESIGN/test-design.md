# Rust Testing Organization Best Practices: Implementation-Ready Guide

This guide answers your requested scope: Rust testing organization across unit, integration, CLI, async/service, database, fixtures, snapshots, property/fuzz, benchmarks, workspaces, CI, developer workflow, anti-patterns, and an AI Coding validation blueprint. Your original requirement explicitly asks for practical, engineering-oriented, implementation-ready guidance for real Rust projects, including library crates, binary crates, workspaces, and large modular projects. 

Rust gives you a strong native foundation: `cargo test` discovers tests in `src` files and in `tests/`; tests in `src` are normally unit/doc tests, while files under `tests/` are integration-style tests that import the crate like an external user. ([Rust 文档][1]) The key engineering challenge is that Rust’s default model is intentionally simple, so serious projects need extra conventions for test levels, shared fixtures, slow test isolation, workspace-level E2E, CI matrices, fuzzing, mutation testing, and AI-generated code validation.

---

# 1. Overall Goals of a Good Rust Testing Organization

A well-designed Rust test structure should optimize for **fast local confidence**, **long-term refactorability**, and **clear ownership**.

## 1.1 Core goals

| Goal                      | What it means in practice                                                                                                                             |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Maintainability           | Tests should be easy to locate, read, update, and delete. Test structure should mirror production module boundaries where useful.                     |
| Fast feedback             | Most tests should run in seconds. Slow tests must be isolated behind `#[ignore]`, feature flags, separate commands, or separate CI jobs.              |
| Clear ownership           | Unit tests belong near the module owner. Integration/E2E tests belong to feature/API/CLI/service owners.                                              |
| Separation of test levels | Unit, integration, CLI, property, fuzz, benchmark, and E2E tests should not be mixed randomly.                                                        |
| Reusable test utilities   | Common builders, fixtures, mock servers, temp dirs, and CLI helpers should live in `tests/common`, `test-support` crates, or workspace-level helpers. |
| CI integration            | CI should run layered checks: format, lint, unit, integration, doc tests, features, OS matrix, coverage, security, fuzz smoke, benchmarks.            |
| Refactoring support       | Unit tests verify internal rules; integration tests verify public behavior; contract/golden tests prevent accidental behavior drift.                  |
| Workspace support         | Each crate should be independently testable, while workspace-level tests verify cross-crate behavior.                                                 |

## 1.2 The target shape

A serious Rust test system should have a **test pyramid plus specialist tracks**:

```text
Fast feedback
┌──────────────────────────────────────────┐
│ Unit tests                               │  many, fast, local by default
│ Module tests                             │  many, close to implementation
│ Property tests                           │  selected high-value invariants
├──────────────────────────────────────────┤
│ Crate integration tests                  │  public API behavior
│ CLI tests                                │  binary behavior
│ Snapshot / golden tests                  │  structured output stability
├──────────────────────────────────────────┤
│ Service / DB / network tests             │  slower, isolated dependencies
│ Workspace E2E tests                      │  full scenario tests
│ Compatibility tests                      │  feature/OS/version behavior
├──────────────────────────────────────────┤
│ Fuzzing / mutation / benchmarks          │  scheduled or gated
└──────────────────────────────────────────┘
Slower but higher confidence
```

The mistake to avoid: treating `cargo test` as the whole testing strategy. It is the foundation, not the complete organization.

---

# 2. Rust Native Testing Model

Rust encourages tests to be close to code and easy to run. The standard `cargo test` command compiles and executes unit, integration, and documentation tests. ([Rust 文档][2])

## 2.1 Unit tests inside `src/**/*.rs`

Typical pattern:

```rust
pub fn normalize_name(input: &str) -> String {
    input.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_lowercases_name() {
        assert_eq!(normalize_name(" Alice "), "alice");
    }
}
```

Use this for:

* Pure functions
* Module invariants
* Internal edge cases
* Error mapping
* Trait/generic behavior
* Small async logic if no external service is required

Rust convention is to put unit tests in the same file as the code being tested, inside a `tests` module annotated with `#[cfg(test)]`. ([Rust 文档][3])

## 2.2 Integration tests under `tests/`

Example:

```text
my-lib/
  src/
    lib.rs
  tests/
    public_api.rs
    parsing_contract.rs
```

Each file in `tests/` is compiled as a separate crate. That means integration tests only access public APIs unless you expose test-only helpers or use a helper crate.

```rust
// tests/public_api.rs
use my_lib::parse_config;

#[test]
fn parses_minimal_config() {
    let cfg = parse_config("name = 'demo'").unwrap();
    assert_eq!(cfg.name, "demo");
}
```

Cargo’s guide says integration tests belong in `tests/`, while unit tests belong in the source files they test. ([Rust 文档][1])

## 2.3 Documentation tests

Doc tests live in public API documentation:

````rust
/// Parses a port number.
///
/// # Examples
///
/// ```
/// let port = my_lib::parse_port("8080").unwrap();
/// assert_eq!(port, 8080);
/// ```
pub fn parse_port(s: &str) -> Result<u16, std::num::ParseIntError> {
    s.parse()
}
````

Use doc tests for:

* Public library APIs
* Usage examples
* API contracts
* Preventing documentation drift

Doc tests are especially important for library crates. They make your public docs executable.

## 2.4 Benchmarks

Common structure:

```text
benches/
  parser_bench.rs
  encode_bench.rs
```

Use `criterion` rather than unstable built-in benchmark support for normal stable Rust projects.

## 2.5 Examples

```text
examples/
  basic.rs
  cli_usage.rs
  custom_backend.rs
```

Examples should compile. Some should run in CI using:

```bash
cargo test --examples
```

Examples are useful as:

* Public API demonstrations
* Smoke tests
* Reproducible usage scenarios
* Minimal samples for users

## 2.6 Test-only modules and helper crates

Three common choices:

```text
src/foo.rs              # unit tests near implementation
tests/common/mod.rs     # integration test helpers
crates/test-support/    # reusable workspace-wide helpers
```

Use `tests/common` for one crate. Use `test-support` for large workspaces.

## 2.7 Workspace-level testing

In workspaces:

```bash
cargo test --workspace
cargo test -p my-crate
cargo test -p my-crate --features sqlite
cargo nextest run --workspace
```

`cargo-nextest` is widely used for faster, more CI-friendly Rust test execution; its docs show `cargo nextest run` as the workspace-level test command, and it advertises per-test isolation and CI support. ([nexte.st][4])

## 2.8 What Rust gives you vs. what you must design

| Area              | Rust gives you            | You still need to design                                  |
| ----------------- | ------------------------- | --------------------------------------------------------- |
| Unit tests        | `#[test]`, `#[cfg(test)]` | Naming, fixture strategy, avoiding implementation lock-in |
| Integration tests | `tests/` crates           | Scenario boundaries, shared helpers, slow test isolation  |
| Doc tests         | Executable docs           | Public API coverage policy                                |
| Workspaces        | `cargo test --workspace`  | Cross-crate E2E, feature matrix, CI partitioning          |
| Examples          | `examples/`               | Whether examples are smoke-tested                         |
| Benchmarks        | `benches/` directory      | Criterion setup, baseline strategy                        |
| CI                | Cargo commands            | Layered jobs, cache, OS/MSRV/features/security            |

---

# 3. Test Type Classification

## 3.1 Recommended classification table

| Test type               | Purpose                            | Location                            | Naming                  | Tools                                   | Isolation                       | Local                 | CI                     | Common mistakes                                  |
| ----------------------- | ---------------------------------- | ----------------------------------- | ----------------------- | --------------------------------------- | ------------------------------- | --------------------- | ---------------------- | ------------------------------------------------ |
| Unit tests              | Verify small logic units           | `src/**/*.rs` inside `#[cfg(test)]` | `returns_x_when_y`      | std test, `pretty_assertions`           | No real IO/network              | Always                | Always                 | Testing too much private structure               |
| Module-level tests      | Verify module invariants           | `src/module/tests.rs` or inline     | `module_handles_case`   | std test                                | In-memory                       | Always                | Always                 | Huge test modules mixed with implementation      |
| Crate integration tests | Verify public API                  | `tests/*.rs`                        | `feature_name.rs`       | std test                                | Use public API only             | Often                 | Always                 | Putting all tests in `tests/integration.rs`      |
| API contract tests      | Prevent public behavior drift      | `tests/contracts/*.rs`              | `contract_<api>.rs`     | std, insta/golden                       | Stable inputs/outputs           | Before release        | Always                 | Snapshotting unstable output                     |
| CLI tests               | Verify binary behavior             | `tests/cli/*.rs`                    | `cmd_<scenario>.rs`     | `assert_cmd`, `predicates`, `assert_fs` | Temp dirs/env                   | Often                 | Always                 | Depending on user machine state                  |
| Snapshot tests          | Review structured output changes   | Inline or `tests/snapshots/`        | `scenario.snap`         | `insta`                                 | Normalize paths/time            | Often                 | Always, review changes | Snapshots too large/unreviewed                   |
| Golden file tests       | Compare output to checked-in files | `fixtures/golden/`                  | `<case>.expected`       | std, `insta`, custom diff               | Deterministic output            | Often                 | Always                 | Golden files with platform-specific line endings |
| Property tests          | Check invariants across inputs     | Unit or `tests/property.rs`         | `prop_<invariant>`      | `proptest`, `quickcheck`                | Deterministic seed when failing | Often targeted        | Always or nightly      | Bad invariants, slow generators                  |
| Fuzz tests              | Find crashes/security bugs         | `fuzz/`                             | `fuzz_<target>`         | `cargo-fuzz`, libFuzzer                 | Isolated harness                | Optional              | Smoke/nightly          | Fuzzing whole app instead of pure parser/codec   |
| Mutation tests          | Check test suite strength          | N/A command-based                   | N/A                     | `cargo mutants`                         | Clean workspace                 | Occasionally          | Nightly/scheduled      | Running on every PR too early                    |
| Async tests             | Verify async behavior              | Unit or integration                 | `async_<scenario>`      | `tokio::test`, `timeout`                | Timeouts, controlled runtime    | Always for async code | Always                 | Hanging tasks, real sleeps                       |
| Database tests          | Verify persistence behavior        | `tests/db/*.rs`                     | `db_<scenario>`         | `sqlx`, `testcontainers`, `tempfile`    | Transaction/schema/container    | When touching DB      | Separate CI job        | Shared DB state                                  |
| Network tests           | Verify HTTP/client/server          | `tests/http/*.rs`                   | `http_<scenario>`       | `wiremock`, in-process server           | Mock server/local ports         | Often                 | Always or separate     | Hitting real external APIs                       |
| File system tests       | Verify path/file behavior          | Unit or `tests/fs.rs`               | `fs_<scenario>`         | `tempfile`, `assert_fs`                 | Temp dirs                       | Always                | Always                 | Writing to repo/user dirs                        |
| Cross-platform tests    | OS/path behavior                   | `tests/platform.rs`                 | `windows_path_*`        | std, CI matrix                          | OS-specific temp dirs           | On available OS       | OS matrix              | Hardcoded `/tmp`, `/`                            |
| Performance benchmarks  | Track performance                  | `benches/*.rs`                      | `<component>_bench.rs`  | `criterion`                             | Stable input, controlled env    | Before perf changes   | Scheduled/manual       | Treating noisy CI as exact                       |
| E2E tests               | Verify full system scenario        | workspace `tests/e2e/`              | `e2e_<flow>.rs`         | assert_cmd, containers, service harness | Full isolated environment       | Before major changes  | Dedicated job          | Too many E2E tests                               |
| Compatibility tests     | Verify versions/features/formats   | `tests/compat/`                     | `compat_<version>.rs`   | fixtures, golden                        | Versioned fixtures              | Before release        | Always/nightly         | No old fixture corpus                            |
| Regression tests        | Prevent bug recurrence             | Closest relevant level              | `regression_issue_1234` | std/proptest/fuzz corpus                | Minimal reproducer              | Always                | Always                 | Only adding broad E2E regression                 |

---

# 4. Recommended Project Structures

## 4.1 Small library crate

```text
my-lib/
  Cargo.toml
  src/
    lib.rs
    parser.rs
    encoder.rs
    error.rs
  tests/
    public_api.rs
    parser_contract.rs
    compatibility.rs
    common/
      mod.rs
  benches/
    parser_bench.rs
  examples/
    parse_file.rs
  fixtures/
    valid/
    invalid/
    golden/
```

### Why this structure works

* Unit tests live next to implementation.
* `tests/` verifies only the public API.
* `fixtures/` contains reusable test input/output.
* `benches/` isolates performance testing.
* `examples/` doubles as documentation and smoke tests.

### What goes where

| Location                   | Content                                         |
| -------------------------- | ----------------------------------------------- |
| `src/parser.rs`            | Parser implementation + small parser unit tests |
| `tests/parser_contract.rs` | Public parser behavior and compatibility        |
| `fixtures/`                | Sample inputs and golden outputs                |
| `benches/`                 | Criterion benchmarks                            |
| `examples/`                | User-facing examples                            |

### Avoid leaking test code into production

Good:

```rust
#[cfg(test)]
mod tests {
    use super::*;
}
```

Good for test-only helper:

```rust
#[cfg(test)]
pub(crate) fn make_test_config() -> Config {
    Config::default()
}
```

Avoid:

```rust
pub fn make_test_config() -> Config {
    // production API polluted by test helper
}
```

---

## 4.2 Binary CLI project

```text
my-cli/
  Cargo.toml
  src/
    main.rs
    lib.rs
    cli.rs
    config.rs
    commands/
      mod.rs
      init.rs
      run.rs
  tests/
    cli/
      init_cmd.rs
      run_cmd.rs
      config_cmd.rs
    common/
      mod.rs
  fixtures/
    configs/
    projects/
    golden/
  snapshots/
```

### Recommended split

A CLI should keep most logic in `src/lib.rs` and keep `src/main.rs` thin.

```rust
// src/main.rs
fn main() -> std::process::ExitCode {
    my_cli::run(std::env::args_os())
}
```

```rust
// src/lib.rs
pub fn run<I, S>(args: I) -> std::process::ExitCode
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    // parse args, dispatch command
    std::process::ExitCode::SUCCESS
}
```

### Why this matters

* Unit tests can test command logic without spawning processes.
* CLI tests can verify real binary behavior.
* Cross-platform path behavior is easier to test.
* `assert_cmd` tests stay focused on external behavior.

---

## 4.3 Large workspace

```text
my-workspace/
  Cargo.toml
  crates/
    core/
      src/
      tests/
    cli/
      src/
      tests/
    server/
      src/
      tests/
    sdk/
      src/
      tests/
    test-support/
      src/
        lib.rs
        fixtures.rs
        temp.rs
        http.rs
        db.rs
  tests/
    e2e/
      cli_server_flow.rs
      migration_flow.rs
    compat/
      v1_config_compat.rs
  fixtures/
    configs/
    golden/
    corpora/
  benches/
    workspace_bench.rs
  fuzz/
    Cargo.toml
    fuzz_targets/
      parse_config.rs
  xtask/
    src/
      main.rs
  scripts/
    ci-local.sh
    test-slow.sh
```

### Why this structure works

* Each crate remains independently testable.
* Workspace-level tests verify cross-crate behavior.
* `crates/test-support` centralizes reusable test utilities.
* `xtask` provides stable developer commands.
* `fixtures/` is shared by contract, compatibility, CLI, and E2E tests.
* `fuzz/` is isolated from normal builds.

### Avoid circular dependencies

Bad:

```text
core -> cli -> test-support -> core
```

Better:

```text
core
cli -> core
server -> core
test-support -> core, server
workspace-e2e -> cli, server, test-support
```

Rule:

```text
Production crates must not depend on test-support.
Only dev-dependencies and test crates may depend on test-support.
```

Example:

```toml
# crates/cli/Cargo.toml
[dev-dependencies]
test-support = { path = "../test-support" }
assert_cmd = "..."
assert_fs = "..."
predicates = "..."
```

---

# 5. Unit Test Design

## 5.1 When to put tests next to implementation

Use inline unit tests when:

* The behavior is internal to a module.
* The test needs private function access.
* The logic is small and deterministic.
* Failure should point directly to the source module.

Example:

```rust
// src/range.rs

#[derive(Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

impl Range {
    pub fn new(start: u32, end: u32) -> Result<Self, RangeError> {
        if start > end {
            return Err(RangeError::StartAfterEnd { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn contains(&self, value: u32) -> bool {
        self.start <= value && value <= self.end
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RangeError {
    StartAfterEnd { start: u32, end: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_valid_range() {
        let range = Range::new(1, 3).unwrap();

        assert_eq!(range, Range { start: 1, end: 3 });
    }

    #[test]
    fn new_rejects_start_after_end() {
        let err = Range::new(5, 3).unwrap_err();

        assert_eq!(err, RangeError::StartAfterEnd { start: 5, end: 3 });
    }

    #[test]
    fn contains_is_inclusive() {
        let range = Range::new(1, 3).unwrap();

        assert!(range.contains(1));
        assert!(range.contains(2));
        assert!(range.contains(3));
        assert!(!range.contains(4));
    }
}
```

## 5.2 Testing private functions

Rust unit tests inside the same module can access private items via `use super::*`.

```rust
fn parse_token(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_returns_first_token() {
        assert_eq!(parse_token("hello world"), Some("hello"));
    }
}
```

But avoid over-testing every private helper. Prefer testing private functions only when:

* The logic is complex.
* The helper has meaningful invariants.
* A failure would be hard to diagnose through public behavior alone.

## 5.3 Avoid over-testing implementation details

Bad:

```rust
#[test]
fn parser_calls_tokenizer_three_times() {
    // brittle implementation detail
}
```

Better:

```rust
#[test]
fn parser_accepts_nested_expression() {
    let expr = parse("(add 1 (mul 2 3))").unwrap();
    assert_eq!(expr.evaluate(), 7);
}
```

## 5.4 Table-driven tests

```rust
#[test]
fn parse_bool_accepts_valid_values() {
    let cases = [
        ("true", true),
        ("false", false),
        ("1", true),
        ("0", false),
    ];

    for (input, expected) in cases {
        assert_eq!(
            parse_bool(input).unwrap(),
            expected,
            "input should parse correctly: {input:?}"
        );
    }
}

fn parse_bool(input: &str) -> Result<bool, String> {
    match input {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(format!("invalid bool: {other}")),
    }
}
```

## 5.5 Testing `Result`

Prefer returning `Result` from tests when setup can fail:

```rust
#[test]
fn reads_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("fixtures/configs/basic.toml")?;
    let config = parse_config(&content)?;

    assert_eq!(config.name, "basic");
    Ok(())
}
```

Use `unwrap_err` for intentional failure cases:

```rust
#[test]
fn parse_port_rejects_out_of_range_value() {
    let err = parse_port("99999").unwrap_err();

    assert_eq!(err.kind(), ErrorKind::OutOfRange);
}
```

## 5.6 Testing traits and generics

```rust
trait Encoder {
    fn encode(&self, input: &str) -> Vec<u8>;
}

struct Utf8Encoder;

impl Encoder for Utf8Encoder {
    fn encode(&self, input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }
}

fn assert_encoder_roundtrip<E: Encoder>(encoder: E) {
    let bytes = encoder.encode("hello");
    assert_eq!(bytes, b"hello");
}

#[test]
fn utf8_encoder_encodes_as_bytes() {
    assert_encoder_roundtrip(Utf8Encoder);
}
```

## 5.7 Testing lifetimes

Usually, don’t test lifetimes directly. Test behavior that depends on borrowed data.

```rust
struct View<'a> {
    value: &'a str,
}

impl<'a> View<'a> {
    fn as_uppercase(&self) -> String {
        self.value.to_uppercase()
    }
}

#[test]
fn view_uses_borrowed_value() {
    let source = String::from("abc");
    let view = View { value: &source };

    assert_eq!(view.as_uppercase(), "ABC");
}
```

---

# 6. Integration Test Design

## 6.1 Important Rust rule: each `tests/*.rs` file is a separate crate

This means:

```text
tests/
  api.rs
  cli.rs
```

`api.rs` and `cli.rs` are separate integration test crates. Each imports your crate externally:

```rust
use my_lib::Client;
```

This is excellent for testing public behavior, but not good for sharing helpers directly across files unless you use `tests/common/mod.rs` or a helper crate.

## 6.2 `tests/common/mod.rs`

```text
tests/
  common/
    mod.rs
  api.rs
  config.rs
```

```rust
// tests/common/mod.rs
use tempfile::TempDir;

pub fn temp_project() -> TempDir {
    tempfile::tempdir().expect("create temp project")
}
```

```rust
// tests/api.rs
mod common;

#[test]
fn loads_project_from_temp_dir() {
    let dir = common::temp_project();
    // ...
}
```

Use this for small and medium crates.

## 6.3 Dedicated `test-support` crate

Use a `test-support` crate when:

* Multiple crates need the same helpers.
* You need reusable mock servers.
* You need database/container setup.
* You need fixture loading across the workspace.
* You need custom assertions.

Example:

```text
crates/
  test-support/
    src/
      lib.rs
      fixtures.rs
      temp.rs
      assertions.rs
```

```rust
// crates/test-support/src/fixtures.rs
use std::path::{Path, PathBuf};

pub fn workspace_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}
```

## 6.4 Public API integration test

```rust
// tests/public_api.rs

use my_lib::{Config, Engine};

#[test]
fn engine_runs_with_minimal_config() {
    let config = Config::builder()
        .name("demo")
        .build()
        .unwrap();

    let output = Engine::new(config).run("hello").unwrap();

    assert_eq!(output.status(), "ok");
}
```

## 6.5 Organize integration tests by feature, API, or scenario

Good:

```text
tests/
  parser_contract.rs
  config_contract.rs
  error_handling.rs
  compatibility_v1.rs
```

Also good for larger systems:

```text
tests/
  api/
    users.rs
    projects.rs
  cli/
    init.rs
    run.rs
  compat/
    config_v1.rs
```

Avoid:

```text
tests/
  integration_tests.rs  # 5,000 lines
```

## 6.6 Avoid slow and flaky integration tests

Use:

* Temp dirs
* In-process servers
* Mock HTTP services
* Containers only for dependency-specific tests
* `#[ignore]` for truly slow tests
* `cargo nextest` profiles for slow/serial tests
* No calls to real external SaaS APIs in default CI

---

# 7. CLI Testing

CLI tests should verify the binary as users see it: arguments, stdin/stdout/stderr, exit codes, config files, environment variables, and filesystem effects.

`assert_cmd` is designed to simplify integration testing of CLIs, including finding the crate binary and asserting process results. ([Docs.rs][5]) The Rust CLI testing guide also recommends using `assert_cmd` with `predicates` as dev-dependencies for clear command assertions. ([Rust CLI][6])

## 7.1 Recommended structure

```text
my-cli/
  src/
    main.rs
    lib.rs
  tests/
    cli/
      init.rs
      run.rs
      config.rs
    common/
      mod.rs
  fixtures/
    projects/
    configs/
    golden/
```

## 7.2 Dev dependencies

```toml
[dev-dependencies]
assert_cmd = "2"
assert_fs = "1"
predicates = "3"
tempfile = "3"
```

## 7.3 Test stdout, stderr, and exit code

```rust
// tests/cli/init.rs

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn init_prints_success_message() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();

    cmd.arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn unknown_command_exits_with_error() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();

    cmd.arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized"));
}
```

## 7.4 Test config files and working directory

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn run_uses_config_file_from_current_directory() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child("my-cli.toml")
        .write_str("name = 'demo'\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("my-cli").unwrap();

    cmd.current_dir(temp.path())
        .arg("run")
        .assert()
        .success()
        .stdout(predicates::str::contains("demo"));

    temp.close().unwrap();
}
```

## 7.5 Test environment variables

```rust
#[test]
fn env_var_overrides_config() {
    let mut cmd = assert_cmd::Command::cargo_bin("my-cli").unwrap();

    cmd.arg("show-mode")
        .env("MY_CLI_MODE", "ci")
        .assert()
        .success()
        .stdout(predicates::str::contains("ci"));
}
```

## 7.6 Cross-platform path behavior

Bad:

```rust
assert_eq!(output, "/tmp/project/file.txt");
```

Better:

```rust
let expected = std::path::Path::new("project").join("file.txt");
assert!(output.contains(&expected.display().to_string()));
```

Or normalize before snapshotting:

```rust
fn normalize_paths(s: &str) -> String {
    s.replace('\\', "/")
}
```

## 7.7 Interactive or TUI behavior

For interactive CLIs:

* Keep core state machine testable without terminal IO.
* Use unit tests for key bindings/state transitions.
* Use integration tests only for smoke-level terminal behavior.
* Consider separating `TerminalUi` from `AppState`.

Example:

```rust
#[derive(Debug, PartialEq, Eq)]
enum Action {
    Quit,
    MoveUp,
    MoveDown,
}

fn handle_key(input: char) -> Option<Action> {
    match input {
        'q' => Some(Action::Quit),
        'k' => Some(Action::MoveUp),
        'j' => Some(Action::MoveDown),
        _ => None,
    }
}

#[test]
fn q_exits_tui() {
    assert_eq!(handle_key('q'), Some(Action::Quit));
}
```

---

# 8. Async and Service Testing

## 8.1 Basic async test

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

```rust
#[tokio::test]
async fn fetch_user_returns_user() {
    let user = fetch_user("u1").await.unwrap();

    assert_eq!(user.id, "u1");
}
```

## 8.2 Runtime configuration

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn processes_jobs_concurrently() {
    // test concurrent behavior
}
```

Use single-threaded runtime when testing deterministic scheduling:

```rust
#[tokio::test(flavor = "current_thread")]
async fn state_machine_is_deterministic() {
    // good for local async state machine tests
}
```

## 8.3 Always protect async tests from hanging

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn operation_completes_quickly() {
    let result = timeout(Duration::from_secs(2), do_work()).await;

    assert!(result.is_ok(), "operation timed out");
}
```

## 8.4 Testing cancellation

```rust
#[tokio::test]
async fn worker_stops_when_cancelled() {
    let handle = tokio::spawn(async {
        loop {
            tokio::task::yield_now().await;
        }
    });

    handle.abort();

    let result = handle.await;
    assert!(result.is_err());
    assert!(result.unwrap_err().is_cancelled());
}
```

## 8.5 Testing channels

```rust
#[tokio::test]
async fn worker_sends_result_on_channel() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        tx.send("done").await.unwrap();
    });

    let msg = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        rx.recv(),
    )
    .await
    .unwrap();

    assert_eq!(msg, Some("done"));
}
```

## 8.6 Avoid real sleeps

Bad:

```rust
tokio::time::sleep(Duration::from_secs(5)).await;
```

Better:

```rust
tokio::time::timeout(Duration::from_secs(1), operation()).await?;
```

For time-based logic, abstract time or use Tokio’s paused time utilities where appropriate.

## 8.7 In-process HTTP service test

```rust
#[tokio::test]
async fn health_endpoint_returns_ok() {
    let app = build_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let body = reqwest::get(format!("http://{addr}/health"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(body, "ok");
}
```

## 8.8 Mock external HTTP with `wiremock`

```rust
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn client_handles_remote_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/u1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "u1",
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    let client = ApiClient::new(server.uri());
    let user = client.get_user("u1").await.unwrap();

    assert_eq!(user.name, "Alice");
}
```

---

# 9. Database and External Dependency Testing

## 9.1 Classify dependency tests

| Dependency test type   | Recommended strategy                                      |
| ---------------------- | --------------------------------------------------------- |
| Pure repository logic  | Unit tests with fake repository                           |
| SQL query correctness  | Real DB test                                              |
| Migration correctness  | Real DB/container test                                    |
| Redis/cache behavior   | Real Redis container or fake for simple cases             |
| Message queue behavior | Container or embedded test broker if available            |
| External SaaS API      | Mock server by default, real API only in gated/manual job |

## 9.2 Recommended directory

```text
tests/
  db/
    user_repository.rs
    migrations.rs
  external/
    redis_cache.rs
    queue_consumer.rs
```

## 9.3 Isolation strategies

| Strategy             | When to use              | Notes                                          |
| -------------------- | ------------------------ | ---------------------------------------------- |
| Transaction rollback | Fast DB tests            | Each test runs in a transaction and rolls back |
| Temporary schema     | Parallel DB tests        | Unique schema per test                         |
| Temporary database   | Strong isolation         | More expensive                                 |
| Testcontainers       | Real dependency behavior | Good CI fit, slower                            |
| Seed data fixture    | Contract/compatibility   | Keep small and versioned                       |

## 9.4 Example: transaction rollback

```rust
#[tokio::test]
async fn inserts_and_loads_user() {
    let pool = test_support::db::pool().await;
    let mut tx = pool.begin().await.unwrap();

    let repo = UserRepository::new(&mut tx);
    repo.insert("u1", "Alice").await.unwrap();

    let user = repo.find("u1").await.unwrap().unwrap();
    assert_eq!(user.name, "Alice");

    tx.rollback().await.unwrap();
}
```

## 9.5 Example: unique schema

```rust
fn unique_schema_name() -> String {
    format!("test_{}", uuid::Uuid::new_v4().simple())
}
```

## 9.6 Separating fast and slow tests

Options:

### Option A: ignored tests

```rust
#[tokio::test]
#[ignore = "requires postgres"]
async fn runs_migrations_against_postgres() {
    // ...
}
```

Run manually:

```bash
cargo test -- --ignored
```

### Option B: feature-gated tests

```rust
#[cfg(feature = "db-tests")]
#[tokio::test]
async fn db_test() {
    // ...
}
```

Run:

```bash
cargo test --features db-tests
```

### Option C: separate CI job

```yaml
db-tests:
  services:
    postgres:
      image: postgres:16
  steps:
    - run: cargo test --workspace --features db-tests
```

Best practice:

```text
Default PR path: no real external dependencies.
Dedicated CI path: DB/container tests.
Nightly path: exhaustive slow tests.
```

---

# 10. Fixtures, Test Data, Golden Files, and Snapshots

## 10.1 Recommended fixture layout

```text
fixtures/
  configs/
    minimal.toml
    invalid_missing_name.toml
  golden/
    cli_init_stdout.txt
    formatted_config.json
  corpora/
    parser/
      valid_001.txt
      invalid_001.txt
  large/
    README.md
```

## 10.2 Loading fixtures safely

```rust
use std::path::{Path, PathBuf};

pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn parses_fixture() {
    let path = fixture_path("configs/minimal.toml");
    let content = std::fs::read_to_string(path).unwrap();

    let cfg = parse_config(&content).unwrap();

    assert_eq!(cfg.name, "minimal");
}
```

For workspace-level fixtures, use a `test-support` crate.

## 10.3 Snapshot tests with `insta`

`insta` is a Rust snapshot testing tool; its docs describe snapshots as assertions against managed reference values, especially useful for large or frequently changing reference values. ([Insta Snapshots][7])

```toml
[dev-dependencies]
insta = { version = "1", features = ["json"] }
```

```rust
#[test]
fn formatted_config_snapshot() {
    let cfg = Config {
        name: "demo".into(),
        retries: 3,
    };

    insta::assert_json_snapshot!(cfg);
}
```

Review snapshots:

```bash
cargo insta test
cargo insta review
```

## 10.4 Golden file test

```rust
#[test]
fn formatter_matches_golden_file() {
    let input = include_str!("../fixtures/configs/minimal.toml");
    let expected = include_str!("../fixtures/golden/minimal.formatted.toml");

    let actual = format_config(input).unwrap();

    assert_eq!(normalize_newlines(&actual), normalize_newlines(expected));
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
```

## 10.5 Deterministic random data

Bad:

```rust
let value = rand::random::<u64>();
```

Better:

```rust
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;

let mut rng = ChaCha8Rng::seed_from_u64(42);
let value = rng.gen::<u64>();
```

## 10.6 Large files

Guidelines:

* Keep small fixtures in Git.
* Compress large corpora if needed.
* Avoid huge binary files in normal tests.
* Use a separate corpus download step only for fuzz/perf/nightly jobs.
* Add README explaining fixture origin and update policy.

## 10.7 Cross-platform fixture rules

Normalize:

* Line endings
* Path separators
* Timestamps
* Locale-dependent formatting
* Random IDs
* Absolute paths

---

# 11. Property-Based Testing and Fuzzing

## 11.1 When to use property-based testing

Use property tests for:

* Parsers
* Serializers/deserializers
* Codecs
* Normalizers
* State machines
* Sorting/ranking logic
* Permission logic
* Round-trip behavior
* Idempotency
* Algebraic invariants

`proptest` supports generated test cases and shrinking. Its docs show the `proptest!` macro and configurable number of generated cases; its project description emphasizes automatically finding minimal failing cases. ([altsysrq.github.io][8])

## 11.2 Good invariants

| Component          | Invariant                                                          |
| ------------------ | ------------------------------------------------------------------ |
| Parser + formatter | `parse(format(x)) == x` for normalized x                           |
| Serializer         | `decode(encode(x)) == x`                                           |
| Normalizer         | `normalize(normalize(x)) == normalize(x)`                          |
| Sorter             | Output is sorted and contains same elements                        |
| Permission engine  | Deny rules cannot become allow rules accidentally                  |
| CLI config         | CLI args override env, env overrides file, file overrides defaults |

## 11.3 Example: normalize idempotency

```toml
[dev-dependencies]
proptest = "1"
```

```rust
use proptest::prelude::*;

fn normalize(input: &str) -> String {
    input.trim().to_lowercase()
}

proptest! {
    #[test]
    fn normalize_is_idempotent(s in "\\PC*") {
        let once = normalize(&s);
        let twice = normalize(&once);

        prop_assert_eq!(once, twice);
    }
}
```

## 11.4 Example: encode/decode round trip

```rust
proptest! {
    #[test]
    fn message_roundtrips(id in any::<u64>(), body in "\\PC*") {
        let msg = Message { id, body };
        let encoded = encode(&msg).unwrap();
        let decoded = decode(&encoded).unwrap();

        prop_assert_eq!(decoded, msg);
    }
}
```

## 11.5 Regression cases

When a property test finds a failure:

* Keep the generated regression file.
* Add a small explicit unit test if the bug is important.
* Add the minimized input to a corpus if it also matters for fuzzing.

## 11.6 Fuzzing with `cargo-fuzz`

The Rust Fuzz Book says `cargo-fuzz` is the recommended tool for fuzz testing Rust code and currently invokes libFuzzer through `libfuzzer-sys`. ([rust-fuzz.github.io][9])

Recommended layout:

```text
fuzz/
  Cargo.toml
  fuzz_targets/
    parse_config.rs
    decode_message.rs
  corpus/
    parse_config/
  artifacts/
```

Example target:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = my_lib::parse_config(input);
    }
});
```

Run:

```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz run parse_config
```

## 11.7 How to choose fuzz targets

Good fuzz targets are:

* Pure or mostly pure
* Fast
* Panic-sensitive
* Input-parsing-heavy
* Security-sensitive
* Easy to assert with simple invariants

Good targets:

```text
parse_config(bytes)
decode_message(bytes)
deserialize_request(bytes)
normalize_path(input)
evaluate_policy(json)
```

Bad targets:

```text
start_entire_server()
run_full_cli_with_network()
call_real_database()
```

## 11.8 Combining unit, property, and fuzz tests

| Bug source               | Best tool                          |
| ------------------------ | ---------------------------------- |
| Known edge case          | Unit test                          |
| Many possible inputs     | Property test                      |
| Unknown weird bytes      | Fuzz test                          |
| Public behavior drift    | Integration/golden test            |
| Crash from fuzz artifact | Regression unit test + fuzz corpus |

---

# 12. Benchmark and Performance Testing

## 12.1 Recommended tool: Criterion

Use `criterion` for stable Rust benchmarking.

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "parser_bench"
harness = false
```

```rust
// benches/parser_bench.rs

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_parse_config(c: &mut Criterion) {
    let input = include_str!("../fixtures/configs/large.toml");

    c.bench_function("parse large config", |b| {
        b.iter(|| my_lib::parse_config(input).unwrap())
    });
}

criterion_group!(benches, bench_parse_config);
criterion_main!(benches);
```

Run:

```bash
cargo bench
```

## 12.2 Microbenchmarks vs macrobenchmarks

| Type             | Example                 | Purpose                                     |
| ---------------- | ----------------------- | ------------------------------------------- |
| Microbenchmark   | Parse one config        | Detect local algorithmic regressions        |
| Macrobenchmark   | Run full CLI command    | Detect user-visible performance regressions |
| Load benchmark   | HTTP service throughput | Capacity planning                           |
| Memory benchmark | Large corpus processing | Detect allocations/regressions              |

## 12.3 Benchmark stability rules

* Avoid benchmarking debug builds.
* Avoid network and real external dependencies.
* Use stable input fixtures.
* Avoid measuring logging or random filesystem state.
* Run benchmarks on controlled machines if used for gating.
* Use CI benchmarks for trend detection, not exact truth.

## 12.4 CI benchmark strategy

Recommended:

```text
PR:
  - Compile benchmark targets
  - Optional smoke run

Nightly:
  - Run full benchmarks
  - Compare against baseline
  - Publish trend

Release:
  - Run full benchmark suite
  - Block only on major regression thresholds
```

---

# 13. Workspace-Level Testing Strategy

## 13.1 Test each crate independently

```bash
cargo test -p core
cargo test -p cli
cargo test -p server
```

This keeps ownership clear and makes failures easier to diagnose.

## 13.2 Test the whole workspace

```bash
cargo test --workspace
cargo nextest run --workspace
```

Prefer `cargo nextest run --workspace` for larger projects because of speed, isolation, and CI reporting support. ([nexte.st][10])

## 13.3 Cross-crate behavior

Use workspace-level tests when behavior spans crates:

```text
tests/
  e2e/
    cli_invokes_server.rs
    sdk_talks_to_server.rs
  compat/
    old_config_still_loads.rs
```

## 13.4 `test-support` crate design

```rust
// crates/test-support/src/lib.rs

pub mod fixtures;
pub mod temp;
pub mod cli;
pub mod http;
pub mod db;
```

Guidelines:

* Put reusable helpers here.
* Keep it dev-only for production crates.
* Avoid depending on CLI unless specifically testing CLI.
* Avoid global mutable state.
* Provide strong cleanup guarantees.

## 13.5 Dev-dependency or workspace crate?

| Situation                                    | Use                        |
| -------------------------------------------- | -------------------------- |
| One crate only                               | `tests/common/mod.rs`      |
| Multiple crates share helpers                | `crates/test-support`      |
| Helper has complex setup                     | `crates/test-support`      |
| Helper would require production dependencies | Separate dev-only crate    |
| Helper is actually reusable production code  | Move to real library crate |

## 13.6 Feature flag tests

Run default features:

```bash
cargo test --workspace
```

Run all features:

```bash
cargo test --workspace --all-features
```

Run no default features:

```bash
cargo test --workspace --no-default-features
```

Run selected feature:

```bash
cargo test -p server --features postgres
```

For feature matrix, use `cargo-hack`:

```bash
cargo hack test --workspace --feature-powerset
```

Use feature matrix testing for libraries and reusable crates. For applications, test important feature combinations instead of the full powerset if it explodes.

---

# 14. CI/CD Strategy

## 14.1 Recommended CI stages

```text
Stage 1: Source quality
  - cargo fmt --check
  - cargo clippy --all-targets --all-features -- -D warnings

Stage 2: Fast tests
  - cargo nextest run --workspace
  - cargo test --doc --workspace

Stage 3: Feature and platform matrix
  - no-default-features
  - all-features
  - selected feature combos
  - Linux/macOS/Windows

Stage 4: Slow dependency tests
  - DB tests
  - Redis/queue tests
  - Docker/Testcontainers tests

Stage 5: Quality depth
  - coverage
  - mutation testing
  - security/license checks
  - fuzz smoke

Stage 6: Performance
  - benchmark compile check on PR
  - full benchmark on nightly/release
```

## 14.2 Concrete commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo test --workspace
cargo test --doc --workspace
cargo test --workspace --all-features
cargo test --workspace --no-default-features

cargo nextest run --workspace
cargo nextest run --workspace --all-features

cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo mutants
cargo audit
cargo deny check
cargo bench --no-run
```

`cargo-llvm-cov` wraps Rust LLVM source-based coverage instrumentation; the Rust compiler docs describe coverage via `-C instrument-coverage`, and `cargo-llvm-cov` documents itself as a Cargo subcommand for LLVM source-based coverage. ([Rust 文档][11])

## 14.3 Example GitHub Actions pipeline

```yaml
name: Rust CI

on:
  pull_request:
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  fmt-clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace
      - run: cargo test --doc --workspace

  feature-matrix:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --no-default-features
      - run: cargo test --workspace --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-audit
      - uses: taiki-e/install-action@cargo-deny
      - run: cargo audit
      - run: cargo deny check
```

## 14.4 Slow dependency test job

```yaml
  db-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_USER: postgres
          POSTGRES_DB: app_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    env:
      DATABASE_URL: postgres://postgres:postgres@localhost:5432/app_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --features db-tests
```

## 14.5 MSRV checks

```yaml
  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.76.0
      - run: cargo check --workspace --all-targets
```

Use MSRV checks when your crate promises a minimum supported Rust version.

---

# 15. Test Execution and Developer Workflow

## 15.1 Fast loop during coding

```bash
cargo test -p my-crate
cargo test parser
cargo test parse_config_accepts_minimal_input
cargo test -p my-crate --lib
```

## 15.2 Run one test with logs

```bash
RUST_LOG=debug cargo test parse_config -- --nocapture
```

With backtrace:

```bash
RUST_BACKTRACE=1 cargo test failing_test -- --nocapture
```

## 15.3 Run ignored tests

```bash
cargo test -- --ignored
cargo test --features db-tests -- --ignored
```

## 15.4 Run tests with features

```bash
cargo test -p server --features postgres
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

## 15.5 Run with nextest

```bash
cargo nextest run
cargo nextest run --workspace
cargo nextest run -p my-crate
cargo nextest run parse_config
```

## 15.6 Full local verification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace
cargo test --examples --workspace
cargo llvm-cov --workspace --all-features
```

## 15.7 Debugging failed tests

Use:

```bash
cargo test failing_test -- --nocapture
RUST_BACKTRACE=full cargo test failing_test
RUST_LOG=trace cargo test failing_test -- --nocapture
```

For async hangs:

```bash
RUST_BACKTRACE=1 cargo test hanging_test -- --nocapture
```

Add timeout wrappers around async tests rather than relying on CI timeouts.

## 15.8 Pre-commit checks

Recommended lightweight pre-commit:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
```

Recommended pre-push:

```bash
cargo nextest run --workspace
cargo test --doc --workspace
```

---

# 16. Anti-Patterns

| Anti-pattern                            | Consequence                                          | Better alternative                                 |
| --------------------------------------- | ---------------------------------------------------- | -------------------------------------------------- |
| One huge `tests/integration.rs`         | Hard to navigate, slow recompilation, poor ownership | Split by feature/API/scenario                      |
| Using integration tests for all logic   | Slow feedback, hard diagnosis                        | Put pure logic tests in unit tests                 |
| Over-testing private details            | Refactoring becomes painful                          | Test invariants and public behavior                |
| Duplicating fixtures across crates      | Drift and inconsistent behavior                      | Central `fixtures/` or `test-support`              |
| Mixing slow tests with fast tests       | Local workflow becomes painful                       | Use feature flags, `#[ignore]`, separate CI jobs   |
| Real external services in default tests | Flaky and slow CI                                    | Use mock servers or containers                     |
| Test-only code in public APIs           | Polluted API, accidental dependency                  | `#[cfg(test)]`, `dev-dependencies`, `test-support` |
| Poor feature coverage                   | Bugs hidden behind optional features                 | Feature matrix tests                               |
| No doc tests for public libraries       | Docs drift from API                                  | Add executable examples                            |
| No cross-platform path tests            | Windows/macOS bugs                                   | OS matrix and path normalization                   |
| No regression tests                     | Fixed bugs return                                    | Add minimal regression test per bug                |
| Snapshots accepted blindly              | Snapshots lose meaning                               | Review every snapshot diff                         |
| Fuzzing full application                | Slow, noisy, hard to triage                          | Fuzz pure parsers/codecs                           |
| Benchmarks used as exact CI gates       | False failures from noise                            | Use trends and thresholds                          |
| Database tests share global state       | Flaky parallel tests                                 | Transaction rollback or isolated schema            |
| Async tests without timeout             | CI hangs                                             | Wrap futures with `timeout`                        |

---

# 17. Recommended Final Blueprint for a Serious Rust Project

## 17.1 Directory layout

```text
serious-rust-project/
  Cargo.toml
  Cargo.lock

  crates/
    core/
      Cargo.toml
      src/
        lib.rs
        parser.rs
        model.rs
      tests/
        public_api.rs
        parser_contract.rs

    cli/
      Cargo.toml
      src/
        main.rs
        lib.rs
        commands/
      tests/
        cli_init.rs
        cli_run.rs

    server/
      Cargo.toml
      src/
        lib.rs
        routes/
        services/
      tests/
        http_contract.rs
        db_repository.rs

    test-support/
      Cargo.toml
      src/
        lib.rs
        fixtures.rs
        temp.rs
        cli.rs
        http.rs
        db.rs
        assertions.rs

  tests/
    e2e/
      full_cli_flow.rs
      client_server_flow.rs
    compat/
      config_v1.rs
      config_v2.rs

  fixtures/
    configs/
    golden/
    corpora/
    projects/

  benches/
    parser_bench.rs
    cli_macro_bench.rs

  fuzz/
    Cargo.toml
    fuzz_targets/
      parse_config.rs
      decode_message.rs

  examples/
    basic_usage.rs
    custom_config.rs

  xtask/
    Cargo.toml
    src/main.rs

  scripts/
    ci-local.sh
    test-slow.sh
```

## 17.2 Test categories

| Category         | Default local? |               PR CI? |               Nightly? | Tooling                 |
| ---------------- | -------------: | -------------------: | ---------------------: | ----------------------- |
| Unit             |            Yes |                  Yes |                    Yes | std                     |
| Integration      |            Yes |                  Yes |                    Yes | std, nextest            |
| Doc tests        |   Yes for libs |                  Yes |                    Yes | cargo test --doc        |
| CLI              |            Yes |                  Yes |                    Yes | assert_cmd              |
| Snapshot/golden  |            Yes |                  Yes |                    Yes | insta                   |
| Property         |       Targeted |                  Yes | Yes, larger case count | proptest                |
| DB/container     |       Optional |         Separate job |                    Yes | testcontainers/services |
| Fuzz             |             No |           Smoke only |               Full run | cargo-fuzz              |
| Mutation         |             No | Optional small scope |                    Yes | cargo mutants           |
| Benchmarks       |             No |         Compile only |               Full run | criterion               |
| Coverage         |       Optional |       Yes or nightly |                    Yes | cargo llvm-cov          |
| Security/license |       Optional |                  Yes |                    Yes | cargo audit, cargo deny |

## 17.3 Naming conventions

```text
Unit test function:
  returns_error_when_config_is_missing
  normalizes_path_separators
  preserves_existing_fields_when_merging

Integration file:
  tests/parser_contract.rs
  tests/config_compat.rs
  tests/cli_init.rs

Regression test:
  regression_issue_1234_empty_config_does_not_panic

Property test:
  prop_encode_decode_roundtrips
  prop_normalize_is_idempotent

Benchmark:
  parser_bench.rs
  config_load_bench.rs

Fuzz target:
  parse_config
  decode_message
```

## 17.4 Tooling choices

| Need                                | Tool                                    |
| ----------------------------------- | --------------------------------------- |
| Standard unit/integration/doc tests | `cargo test`                            |
| Faster CI test runner               | `cargo nextest`                         |
| CLI testing                         | `assert_cmd`, `predicates`, `assert_fs` |
| Temp dirs/files                     | `tempfile`, `assert_fs`                 |
| Snapshots                           | `insta`                                 |
| Property tests                      | `proptest`                              |
| Fuzzing                             | `cargo-fuzz`                            |
| Benchmarks                          | `criterion`                             |
| Coverage                            | `cargo llvm-cov`                        |
| Mutation testing                    | `cargo mutants`                         |
| Dependency security                 | `cargo audit`                           |
| License/dependency policy           | `cargo deny`                            |
| Feature matrix                      | `cargo hack`                            |
| Local task orchestration            | `xtask`, `just`, or `make`              |

## 17.5 Local workflow

```bash
# Fast coding loop
cargo test -p core parser

# Before committing
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace

# Before opening PR
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace
cargo test --examples --workspace

# Before release
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo llvm-cov --workspace --all-features
cargo audit
cargo deny check
cargo bench
```

## 17.6 CI pipeline blueprint

```text
PR required:
  fmt
  clippy
  nextest default
  doc tests
  feature matrix subset
  OS matrix for key crates
  security checks

PR optional/separate:
  coverage
  DB/container tests
  benchmark compile

Nightly:
  all features
  no default features
  full DB/container tests
  fuzz run
  mutation test
  full coverage
  full benchmarks

Release:
  compatibility fixtures
  migration tests
  benchmark comparison
  security/license gate
```

---

# 18. Guidelines for Adding New Tests

When adding a test, ask:

1. **What behavior am I protecting?**

   * Internal invariant → unit test.
   * Public API behavior → integration test.
   * CLI behavior → CLI test.
   * Output shape → snapshot/golden test.
   * Input space invariant → property test.
   * Crash/security robustness → fuzz target.
   * Performance → benchmark.
   * Past bug → regression test.

2. **How fast should it be?**

   * < 100 ms: default unit/integration.
   * < 2 seconds: default integration acceptable.
   * > 2 seconds: consider slow suite.
   * External dependency: separate CI job unless mocked.

3. **What is the isolation boundary?**

   * No shared mutable global state.
   * Temp dirs for filesystem.
   * Mock server for HTTP.
   * Transaction/schema/container for DB.
   * Deterministic random seed.

4. **What is the ownership boundary?**

   * Crate-specific behavior stays in crate.
   * Cross-crate behavior goes to workspace E2E.
   * Reusable helpers go to `test-support`.

5. **Can this test survive refactoring?**

   * Avoid asserting private call order.
   * Prefer behavior and invariants.
   * Use explicit regression names for bug fixes.

---

# 19. Guidelines for AI Coding / LLM-Generated Rust Code Validation

For AI-generated Rust code, testing organization should be stricter than normal human-written code because LLMs often produce plausible but incomplete behavior.

## 19.1 AI-generated Rust validation pipeline

```text
LLM generates code
  ↓
cargo fmt --check
  ↓
cargo clippy --all-targets --all-features -- -D warnings
  ↓
cargo test --workspace
  ↓
cargo nextest run --workspace --all-features
  ↓
cargo test --doc --workspace
  ↓
feature matrix
  ↓
snapshot/golden contract review
  ↓
property tests for invariants
  ↓
fuzz smoke for parsers/codecs
  ↓
coverage + mutation testing for critical modules
  ↓
human review of behavior, API, and snapshots
```

## 19.2 Require LLMs to generate tests with code

For each generated change, require:

```text
- Unit tests for new internal logic
- Integration tests for public API changes
- CLI tests for command behavior
- Regression test if fixing a bug
- Property test for parser/codec/normalizer logic
- Snapshot/golden update if output format changes
- Doc test if public API is introduced
```

## 19.3 AI Coding prompt template

```text
You are modifying a Rust workspace.

For every code change:
1. Add unit tests next to the implementation using #[cfg(test)].
2. Add integration tests under tests/ when public API behavior changes.
3. Add CLI tests with assert_cmd when binary behavior changes.
4. Add regression tests named regression_issue_<id>_<case> for bug fixes.
5. Add property tests with proptest for parsers, serializers, normalizers, or state machines.
6. Do not expose test-only helpers in production public APIs.
7. Use dev-dependencies or crates/test-support for test utilities.
8. Use tempfile/assert_fs for filesystem isolation.
9. Use mock servers instead of real external APIs.
10. Ensure these commands pass:

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace

Return:
- Changed files
- Test files added/updated
- Commands run
- Any behavior not covered and why
```

## 19.4 Extra checks for LLM-generated code

| Risk                       | Required validation                       |
| -------------------------- | ----------------------------------------- |
| Incorrect edge cases       | Table-driven unit tests                   |
| Wrong public API semantics | Integration contract tests                |
| Hidden panics              | Fuzz/property tests                       |
| Over-broad snapshots       | Human snapshot review                     |
| Poor error handling        | Negative tests using `unwrap_err`         |
| Platform assumptions       | Windows/macOS/Linux CI                    |
| Feature flag breakage      | `--all-features`, `--no-default-features` |
| Dead code                  | Clippy + coverage                         |
| Weak tests                 | Mutation testing                          |

---

# 20. Final Checklist

## Project structure

* [ ] Unit tests live near implementation.
* [ ] Public API tests live under `tests/`.
* [ ] CLI tests use `assert_cmd`.
* [ ] Shared helpers are in `tests/common` or `crates/test-support`.
* [ ] Fixtures are centralized and documented.
* [ ] Slow dependency tests are isolated.
* [ ] Workspace E2E tests are separate from crate tests.

## Test quality

* [ ] Tests assert behavior, not private call order.
* [ ] Regression tests exist for fixed bugs.
* [ ] Public library APIs have doc tests.
* [ ] Parser/codec/normalizer logic has property or fuzz coverage.
* [ ] Snapshot/golden tests normalize time, paths, and line endings.
* [ ] Async tests have timeouts.
* [ ] DB tests isolate state with transactions, schemas, or containers.

## Tooling

* [ ] `cargo fmt --check`
* [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
* [ ] `cargo nextest run --workspace`
* [ ] `cargo test --doc --workspace`
* [ ] `cargo test --examples --workspace`
* [ ] `cargo llvm-cov`
* [ ] `cargo audit`
* [ ] `cargo deny check`
* [ ] `cargo mutants` for critical code
* [ ] `cargo fuzz` for parser/codec/security-sensitive code

## CI

* [ ] Fast PR pipeline exists.
* [ ] OS matrix covers Linux/macOS/Windows where relevant.
* [ ] Feature matrix covers `default`, `all-features`, and `no-default-features`.
* [ ] Slow DB/container tests run separately.
* [ ] Coverage/security checks are automated.
* [ ] Fuzz/mutation/benchmark checks run nightly or pre-release.

## AI Coding validation

* [ ] LLM-generated changes must include tests.
* [ ] LLM-generated public APIs must include doc/integration tests.
* [ ] LLM-generated parsers/codecs must include property or fuzz tests.
* [ ] LLM-generated CLI behavior must include `assert_cmd` tests.
* [ ] LLM-generated output changes must include reviewed snapshots/golden files.
* [ ] Required commands are enforced before merge.

[1]: https://doc.rust-lang.org/cargo/guide/tests.html?utm_source=chatgpt.com "Tests - The Cargo Book"
[2]: https://doc.rust-lang.org/cargo/commands/cargo-test.html?utm_source=chatgpt.com "cargo test - The Cargo Book"
[3]: https://doc.rust-lang.org/book/ch11-03-test-organization.html?utm_source=chatgpt.com "Test Organization - The Rust Programming Language"
[4]: https://nexte.st/docs/running/?utm_source=chatgpt.com "Running tests"
[5]: https://docs.rs/assert_cmd?utm_source=chatgpt.com "assert_cmd - Rust"
[6]: https://rust-cli.github.io/book/tutorial/testing.html?utm_source=chatgpt.com "Testing - Command Line Applications in Rust"
[7]: https://insta.rs/docs/?utm_source=chatgpt.com "Documentation"
[8]: https://altsysrq.github.io/proptest-book/proptest/getting-started.html?utm_source=chatgpt.com "Getting started - Proptest"
[9]: https://rust-fuzz.github.io/book/cargo-fuzz.html?utm_source=chatgpt.com "Fuzzing with cargo-fuzz - Rust Fuzz Book"
[10]: https://nexte.st/?utm_source=chatgpt.com "cargo-nextest: Home"
[11]: https://doc.rust-lang.org/rustc/instrument-coverage.html?utm_source=chatgpt.com "Instrumentation-based Code Coverage - The rustc book"
