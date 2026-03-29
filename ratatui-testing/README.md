# ratatui-testing

A comprehensive testing framework for Rust TUI applications.

## Features

- **Unit Testing** - Test state machines and reducers in isolation
- **PTY Testing** - Simulate real terminal sessions with portable-pty
- **CLI Testing** - Test CLI binaries with assert_cmd
- **DSL** - Fluent builder API for constructing test scenarios
- **Diff Engine** - Compare buffers and generate diffs

## Quick Start

```rust
use ratatui_testing::{StateTester, char_key, AppState, AppEvent};

#[derive(Default, Clone)]
struct MyApp { counter: i32 }
impl AppState for MyApp {}

fn update(state: &mut MyApp, event: AppEvent) -> MyApp {
    if let AppEvent::Key(key) = event {
        if let crossterm::event::KeyCode::Char('j') = key.code {
            state.counter += 1;
        }
    }
    state.clone()
}

#[test]
fn test_counter() {
    let mut tester = StateTester::new(MyApp::default());
    tester.send_event(char_key('j'), &mut update);
    tester.assert_state(&MyApp { counter: 1 });
}
```

## Modules

### state - Unit Testing
```rust
use ratatui_testing::{StateTester, char_key, AppState, AppEvent};
```

### pty - Terminal Simulation
```rust
use ratatui_testing::PtySession;

let mut session = PtySession::spawn_command("myapp", &[])?;
session.write_str("hello")?;
session.expect("Welcome", std::time::Duration::from_secs(5))?;
```

### cli - CLI Testing
```rust
use ratatui_testing::CliTester;

CliTester::cargo_bin("myapp")?
    .arg("--help")
    .assert()
    .success();
```

### dsl - Fluent Test Builder
```rust
use ratatui_testing::dsl::tui;

tui()
    .type_text("hello")
    .press_enter()
    .expect_screen("Welcome")
    .run("myapp");
```

### diff - Buffer Comparison
```rust
use ratatui_testing::BufferDiff;

let result = BufferDiff::compare_content(exp, act, 80, 24);
println!("Similarity: {}%", result.stats.similarity);
```

## Configuration

Set `APP_TEST=1` environment variable to enable test mode:

```rust
use ratatui_testing::TestConfig;

let config = TestConfig::from_env();
assert!(config.is_enabled());
```

## Features

Add to Cargo.toml:
```toml
[dependencies]
ratatui-testing = { features = ["full"] }
```

Or enable individual features:
```toml
ratatui-testing = { features = ["ratatui", "pty", "cli", "insta"] }
```

## License

MIT
