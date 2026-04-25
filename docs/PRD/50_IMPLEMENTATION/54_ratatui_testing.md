# PRD: ratatui-testing

## Overview

`ratatui-testing` is a TUI testing framework for Rust applications built on `ratatui`. It provides infrastructure for automated testing of terminal user interfaces including PTY simulation, buffer diffing, event injection, and snapshot testing.

---

## Current State

The crate currently exists as a **stub implementation** with placeholder modules:

| Module | Status | Description |
|--------|--------|-------------|
| `PtySimulator` | Stub | Methods exist but are no-ops |
| `BufferDiff` | Stub | Methods exist but are no-ops |
| `StateTester` | Stub | Methods exist but are no-ops |
| `TestDsl` | Stub | Methods exist but are no-ops |
| `CliTester` | Stub | Methods exist but are no-ops |

All methods return `Ok(())` or empty strings without performing actual work. The `tests/` directory is empty.

---

## Goals

1. **PTY Simulation** — Create pseudo-terminals for injecting keyboard/mouse input and capturing terminal output
2. **Buffer Diffing** — Compare ratatui `Buffer` output to detect rendering regressions
3. **Event Simulation** — Generate and inject `KeyEvent`, `MouseEvent` into TUI applications
4. **Snapshot Testing** — Capture and replay UI states for regression testing
5. **State Testing** — Verify application state transitions based on events
6. **CLI Testing** — Test TUI application startup and command-line behavior

---

## Architecture

### Core Components

#### 1. PtySimulator

Wrapper around POSIX PTY (or cross-platform `portable-pty` crate).

```rust
pub struct PtySimulator {
    master: PtyMaster,
    writer: Box<dyn Write>,
    reader: Box<dyn BufRead>,
}

impl PtySimulator {
    pub fn new() -> Result<Self>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn write_input(&mut self, input: &str) -> Result<()>;
    pub fn read_output(&mut self, timeout: Duration) -> Result<String>;
    pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>;
    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>;
}
```

**Dependencies:**
- `portable-pty` for cross-platform PTY
- `crossterm` for event types

#### 2. BufferDiff

Compares ratatui buffer output to detect rendering differences.

```rust
pub struct BufferDiff {
    ignore_fg: bool,
    ignore_bg: bool,
    ignore_attributes: bool,
}

impl BufferDiff {
    pub fn new() -> Self;
    pub fn ignore_fg(mut self, ignore: bool) -> Self;
    pub fn ignore_bg(mut self, ignore: bool) -> Self;
    pub fn ignore_attributes(mut self, ignore: bool) -> Self;
    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult;
    pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult;
}

pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
}

pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}
```

**Dependencies:**
- `ratatui` for `Buffer` and `Cell` types

#### 3. StateTester

Verifies application state after events are injected.

```rust
pub struct StateTester {
    snapshot: Option<serde_json::Value>,
}

impl StateTester {
    pub fn new() -> Self;
    pub fn capture<S>(&mut self, state: &S) -> Result<()>
    where
        S: serde::Serialize;
    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where
        S: serde::Serialize;
    pub fn assert_state_matches(&self, expected: &serde_json::Value) -> Result<()>;
}
```

**Dependencies:**
- `serde` / `serde_json`

#### 4. TestDsl

Fluent interface for composing test scenarios.

```rust
pub struct TestDsl {
    pty: Option<PtySimulator>,
    buffer_diff: BufferDiff,
    state_tester: StateTester,
}

impl TestDsl {
    pub fn new() -> Self;
    pub fn with_pty(mut self) -> Result<Self>;
    pub fn render(&self, widget: &impl Widget) -> Result<Buffer>;
    pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>;
    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;
    pub fn wait_for(&mut self, timeout: Duration, predicate: impl Fn(&str) -> bool) -> Result<&mut Self>;
    pub fn capture_state<S>(&mut self, state: &S) -> &mut Self;
    pub fn assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>;
}
```

#### 5. CliTester

Tests CLI entry points and argument parsing.

```rust
pub struct CliTester {
    temp_dir: Option<TempDir>,
}

impl CliTester {
    pub fn new() -> Self;
    pub fn with_temp_dir(mut self) -> Result<Self>;
    pub fn run(&self, args: &[&str]) -> Result<CliOutput>;
    pub fn capture_stdout(&mut self) -> &mut Self;
    pub fn capture_stderr(&mut self) -> &mut Self;
}

pub struct CliOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

**Dependencies:**
- `tempfile` for temp directory management

---

## Integration with ratatui

### Render Testing

```rust
#[test]
fn test_button_rendering() {
    let dsl = TestDsl::new().with_pty().unwrap();
    
    let button = Button::new("Click me");
    let buffer = dsl.render(&button).unwrap();
    
    let expected = Buffer::with_lines(vec![
        "┌─────────┐",
        "│ Click me│",
        "└─────────┘",
    ]);
    
    dsl.assert_buffer_eq(&expected, &buffer).unwrap();
}
```

### Event Injection Testing

```rust
#[tokio::test]
async fn test_input_navigation() {
    let mut dsl = TestDsl::new().with_pty().unwrap();
    let mut pty = dsl.pty_mut().unwrap();
    
    pty.resize(80, 24).unwrap();
    pty.write_input("hello").unwrap();
    
    let output = pty.read_output(Duration::from_millis(100)).unwrap();
    assert!(output.contains("hello"));
}
```

### Snapshot Testing

```rust
#[test]
fn test_session_view_snapshot() {
    let dsl = TestDsl::new().with_pty().unwrap();
    
    let session_view = SessionView::new(&session);
    let buffer = dsl.render(&session_view).unwrap();
    
    let snapshot = load_snapshot("session_view_initial").unwrap();
    let diff = dsl.buffer_diff().diff(&snapshot, &buffer);
    
    if !diff.passed {
        save_snapshot("session_view_initial", &buffer).unwrap();
    }
    
    assert!(diff.passed, "Buffer differs: {:?}", diff.differences);
}
```

---

## File Structure

```
ratatui-testing/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── pty.rs          # PtySimulator implementation
│   ├── diff.rs         # BufferDiff implementation
│   ├── state.rs        # StateTester implementation
│   ├── dsl.rs          # TestDsl implementation
│   ├── cli.rs          # CliTester implementation
│   └── snapshot.rs     # Snapshot management
└── tests/
    ├── pty_tests.rs
    ├── buffer_diff_tests.rs
    ├── state_tests.rs
    ├── dsl_tests.rs
    └── integration_tests.rs
```

---

## Dependencies

```toml
[dependencies]
ratatui = "0.28"
crossterm = { version = "0.28", features = ["events", "mouse"] }
portable-pty = "0.8"
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tempfile = "3.14"
tokio = { version = "1.45", features = ["full"] }

[dev-dependencies]
similar-asserts = "1.5"
```

---

## Acceptance Criteria

### PtySimulator
- [ ] Creates PTY master/slave pair on Unix
- [ ] Writes strings to PTY slave
- [ ] Reads output from PTY master with timeout
- [ ] Resizes PTY window (cols/rows)
- [ ] Injects KeyEvent via crossterm
- [ ] Injects MouseEvent via crossterm

### BufferDiff
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y of differences
- [ ] Supports ignoring foreground color
- [ ] Supports ignoring background color
- [ ] Supports ignoring attributes (bold, italic, etc.)
- [ ] Provides human-readable diff output

### StateTester
- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff

### TestDsl
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

### CliTester
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

### Integration
- [ ] All modules compile together
- [ ] Integration tests pass
- [ ] Works with `cargo test`
- [ ] Cross-platform (Unix primary, Windows best-effort)

---

## Cross-References

| Document | Topic |
|----------|-------|
| [TUI System](./09-tui-system.md) | TUI layout, keybindings, views |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin configuration |
| [Rust Test Implementation Roadmap](./17-rust-test-implementation-roadmap.md) | Overall testing strategy |
| [Crate-by-Crate Test Backlog](./18-crate-by-crate-test-backlog.md) | Testing tasks per crate |
