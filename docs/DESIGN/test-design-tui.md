For Rust TUI projects, especially those based on `ratatui` + `crossterm`, the best design is:

```text
App State / Domain Logic
        ↓
Event Handling / Update Logic
        ↓
View Model / Rendering Model
        ↓
Ratatui Widgets / Frame Rendering
        ↓
Terminal Backend / Crossterm Runtime
```

You should test each layer differently.

---

# 1. Core Principle: Separate TUI Logic from Terminal Runtime

The most important rule:

```text
Do not put business logic directly inside the terminal event loop.
```

Bad design:

```rust
loop {
    if let Event::Key(key) = read()? {
        if key.code == KeyCode::Char('q') {
            break;
        }

        // business logic directly here
        app.items.remove(app.selected);
        terminal.draw(|f| render(f, &app))?;
    }
}
```

Better design:

```rust
pub struct App {
    pub selected: usize,
    pub items: Vec<String>,
    pub should_quit: bool,
}

pub enum Action {
    MoveUp,
    MoveDown,
    DeleteSelected,
    Quit,
}

impl App {
    pub fn update(&mut self, action: Action) {
        match action {
            Action::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Action::MoveDown => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
            }
            Action::DeleteSelected => {
                if !self.items.is_empty() {
                    self.items.remove(self.selected);
                    self.selected = self.selected.min(self.items.len().saturating_sub(1));
                }
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
    }
}
```

Then the event loop only translates terminal input into actions:

```rust
fn key_to_action(key: crossterm::event::KeyEvent) -> Option<Action> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
        KeyCode::Char('d') => Some(Action::DeleteSelected),
        _ => None,
    }
}
```

This makes most TUI behavior testable without a real terminal.

---

# 2. Recommended TUI Test Layers

| Layer                   | What to test                                          | Location                        | Tooling                                    | Speed      |
| ----------------------- | ----------------------------------------------------- | ------------------------------- | ------------------------------------------ | ---------- |
| Domain/state logic      | Selection, filtering, sorting, deletion, mode changes | `src/app.rs` unit tests         | `cargo test`                               | Very fast  |
| Key mapping             | `q`, `j/k`, arrows, enter, esc, shortcuts             | `src/input.rs` unit tests       | `crossterm` event structs                  | Very fast  |
| Update/reducer logic    | Given state + action → new state                      | `src/app.rs` / `src/update.rs`  | table-driven tests                         | Very fast  |
| Widget rendering        | Specific widget renders expected buffer               | `src/ui/*.rs` unit tests        | `ratatui::buffer::Buffer` or `TestBackend` | Fast       |
| Full TUI frame snapshot | Whole screen layout regression                        | `tests/tui_snapshots.rs`        | `ratatui::backend::TestBackend` + `insta`  | Medium     |
| CLI launch smoke        | Binary starts, exits, handles args                    | `tests/cli/*.rs`                | `assert_cmd`                               | Medium     |
| Real terminal behavior  | Raw mode, alternate screen, mouse, resize             | Manual or special ignored tests | `crossterm`, pty tools                     | Slow/flaky |

Ratatui’s `TestBackend` is specifically intended for integration tests of a complete terminal UI, while widget-level tests are often better written directly against buffers. ([Docs.rs][1]) Ratatui’s own testing recipe also recommends combining `TestBackend` with `insta` snapshots for rendered UI regression tests. ([Ratatui][2])

---

# 3. Recommended Project Structure for a Rust TUI App

```text
my-tui/
  Cargo.toml
  src/
    main.rs              # thin runtime entrypoint
    lib.rs
    app.rs               # App state
    action.rs            # Action enum
    input.rs             # key/mouse event -> Action
    update.rs            # state transition logic, optional
    ui/
      mod.rs             # top-level render
      layout.rs
      widgets/
        list.rs
        status_bar.rs
        command_palette.rs
    terminal.rs          # crossterm setup/teardown
    event_loop.rs        # runtime loop

  tests/
    tui_snapshots.rs     # full-frame rendered snapshots
    cli_smoke.rs         # binary launch behavior
    common/
      mod.rs

  snapshots/             # insta snapshots, if configured
  fixtures/
    projects/
    logs/
    configs/
```

The production rule is:

```text
main.rs / terminal.rs / event_loop.rs should be thin.
app.rs / input.rs / update.rs / ui/ should be heavily testable.
```

---

# 4. Unit Test the App State

Most TUI bugs are not terminal bugs. They are state bugs:

* Selection goes out of bounds.
* Empty list crashes.
* Delete changes wrong row.
* Filter resets selection incorrectly.
* Mode transition is wrong.
* Scroll offset becomes invalid.
* Search input handles backspace incorrectly.
* Async result updates the wrong panel.

Example:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
    Help,
}

#[derive(Debug)]
pub struct App {
    pub mode: Mode,
    pub items: Vec<String>,
    pub selected: usize,
    pub search: String,
    pub should_quit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MoveUp,
    MoveDown,
    EnterSearch,
    PushSearchChar(char),
    Backspace,
    ExitMode,
    Quit,
}

impl App {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            mode: Mode::Normal,
            items,
            selected: 0,
            search: String::new(),
            should_quit: false,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Action::MoveDown => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
            }
            Action::EnterSearch => {
                self.mode = Mode::Search;
                self.search.clear();
            }
            Action::PushSearchChar(c) => {
                if self.mode == Mode::Search {
                    self.search.push(c);
                }
            }
            Action::Backspace => {
                if self.mode == Mode::Search {
                    self.search.pop();
                }
            }
            Action::ExitMode => {
                self.mode = Mode::Normal;
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_up_at_top_keeps_selection_at_zero() {
        let mut app = App::new(vec!["a".into(), "b".into()]);

        app.update(Action::MoveUp);

        assert_eq!(app.selected, 0);
    }

    #[test]
    fn move_down_stops_at_last_item() {
        let mut app = App::new(vec!["a".into(), "b".into()]);

        app.update(Action::MoveDown);
        app.update(Action::MoveDown);

        assert_eq!(app.selected, 1);
    }

    #[test]
    fn search_mode_collects_characters() {
        let mut app = App::new(vec!["alpha".into(), "beta".into()]);

        app.update(Action::EnterSearch);
        app.update(Action::PushSearchChar('a'));
        app.update(Action::PushSearchChar('b'));

        assert_eq!(app.mode, Mode::Search);
        assert_eq!(app.search, "ab");
    }

    #[test]
    fn backspace_in_search_mode_removes_last_character() {
        let mut app = App::new(vec![]);

        app.update(Action::EnterSearch);
        app.update(Action::PushSearchChar('x'));
        app.update(Action::Backspace);

        assert_eq!(app.search, "");
    }
}
```

This layer should contain the largest number of TUI tests.

---

# 5. Unit Test Key Mapping

Keyboard mapping should be deterministic and separately testable.

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action::Action;

pub fn key_to_action(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => Some(Action::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char('/'), KeyModifiers::NONE) => Some(Action::EnterSearch),
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(Action::MoveUp),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(Action::MoveDown),
        (KeyCode::Backspace, _) => Some(Action::Backspace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_quits() {
        assert_eq!(
            key_to_action(key(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn ctrl_c_quits() {
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
    }

    #[test]
    fn vim_keys_move_selection() {
        assert_eq!(
            key_to_action(key(KeyCode::Char('j'))),
            Some(Action::MoveDown)
        );

        assert_eq!(
            key_to_action(key(KeyCode::Char('k'))),
            Some(Action::MoveUp)
        );
    }
}
```

This prevents regressions when you add new modes or shortcuts.

---

# 6. Use a Reducer-Style Design for Complex TUIs

For serious TUI apps, I recommend this architecture:

```rust
pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize { width: u16, height: u16 },
    BackendFinished(TaskResult),
}

pub enum Action {
    MoveUp,
    MoveDown,
    OpenHelp,
    CloseHelp,
    StartTask,
    TaskFinished(TaskResult),
    Quit,
}

pub fn event_to_action(app: &App, event: Event) -> Option<Action> {
    // mode-aware input mapping
    todo!()
}

pub fn reduce(app: &mut App, action: Action) -> Option<Effect> {
    // pure state transition plus optional side effect request
    todo!()
}

pub enum Effect {
    RunBackgroundTask,
    SaveFile,
    None,
}
```

This gives you three clean test targets:

```text
Event -> Action
Action + State -> New State
Effect -> runtime side effect
```

That is much easier to test than a giant event loop.

---

# 7. Rendering Tests with Ratatui `TestBackend`

For full-frame rendering, use `ratatui::backend::TestBackend`.

Example:

```rust
use ratatui::{
    backend::TestBackend,
    Terminal,
};

use my_tui::{app::App, ui};

#[test]
fn renders_main_screen() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new(vec![
        "alpha".into(),
        "beta".into(),
        "gamma".into(),
    ]);

    terminal.draw(|frame| {
        ui::render(frame, &app);
    }).unwrap();

    let backend = terminal.backend();

    backend.assert_buffer_lines([
        "┌Items─────────────────────────────────────────────────────────────────────────┐",
        "│> alpha                                                                      │",
        "│  beta                                                                       │",
        "│  gamma                                                                      │",
        "│                                                                             │",
        "└─────────────────────────────────────────────────────────────────────────────┘",
        // remaining lines omitted in real test
    ]);
}
```

This is useful for targeted rendering checks. But don’t assert every cell for every screen unless the layout is very stable.

---

# 8. Snapshot Test Full TUI Screens with `insta`

For real TUI regression testing, snapshots are often more practical than manual buffer assertions. `insta` snapshot tests compare generated values against stored reference snapshots and provide a review workflow. ([Insta Snapshots][3])

Recommended dependency:

```toml
[dev-dependencies]
insta = "1"
ratatui = "..."
```

Example:

```rust
use ratatui::{
    backend::TestBackend,
    Terminal,
};

use my_tui::{app::App, ui};

fn render_app_to_string(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        ui::render(frame, app);
    }).unwrap();

    terminal.backend().buffer_view().to_string()
}

#[test]
fn snapshot_main_screen() {
    let app = App::new(vec![
        "alpha".into(),
        "beta".into(),
        "gamma".into(),
    ]);

    let screen = render_app_to_string(&app, 80, 20);

    insta::assert_snapshot!(screen);
}
```

Run:

```bash
cargo insta test
cargo insta review
```

Recommended snapshot cases:

```text
main_screen_empty
main_screen_with_items
main_screen_selected_second_item
search_mode_empty
search_mode_with_query
help_modal
error_banner
loading_state
small_terminal
wide_terminal
```

---

# 9. Test Widgets Directly When Possible

For reusable widgets, don’t always spin up a full terminal. Render directly into a `Buffer`.

Example:

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

#[test]
fn status_bar_renders_mode_and_hint() {
    let widget = StatusBar {
        mode: "NORMAL",
        hint: "q quit",
    };

    let area = Rect::new(0, 0, 30, 1);
    let mut buffer = Buffer::empty(area);

    widget.render(area, &mut buffer);

    let rendered = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(rendered.contains("NORMAL"));
    assert!(rendered.contains("q quit"));
}
```

Use widget-level tests for:

* Status bars
* Lists
* Tables
* Command palettes
* Help panels
* Error banners
* Progress indicators
* Diff views

Use full-frame snapshots for layout-level regression.

---

# 10. Test Resize Behavior

TUI apps often break on small terminal sizes.

Create explicit size-based snapshot tests:

```rust
#[test]
fn snapshot_small_terminal() {
    let app = App::new(vec!["alpha".into(), "beta".into()]);
    let screen = render_app_to_string(&app, 40, 10);

    insta::assert_snapshot!(screen);
}

#[test]
fn snapshot_wide_terminal() {
    let app = App::new(vec!["alpha".into(), "beta".into()]);
    let screen = render_app_to_string(&app, 120, 30);

    insta::assert_snapshot!(screen);
}
```

Recommended terminal sizes:

```text
40x10   very small
80x24   classic default
120x30  modern wide
160x40  large monitor
```

Important behaviors:

* No panic on small size.
* Text truncates gracefully.
* Modals stay visible.
* Selection remains valid.
* Scroll offset remains valid.
* Layout does not underflow.

---

# 11. Test Scrolling and Selection

TUI list/table components often fail around scroll boundaries.

```rust
#[test]
fn selection_scrolls_when_moving_beyond_visible_area() {
    let mut app = App::new((0..100).map(|i| format!("item-{i}")).collect());

    for _ in 0..20 {
        app.update(Action::MoveDown);
    }

    assert_eq!(app.selected, 20);
    assert!(app.scroll_offset <= app.selected);
}
```

Better invariant:

```rust
#[test]
fn selected_item_is_always_visible() {
    let mut app = App::new((0..100).map(|i| format!("item-{i}")).collect());
    let viewport_height = 10;

    for _ in 0..50 {
        app.update(Action::MoveDown);
        assert!(
            app.selected >= app.scroll_offset
                && app.selected < app.scroll_offset + viewport_height,
            "selected item should remain visible"
        );
    }
}
```

This is also a good target for property-based testing.

---

# 12. Property-Based Tests for TUI State Machines

Use `proptest` for state invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn selection_never_goes_out_of_bounds(actions in proptest::collection::vec(any::<bool>(), 0..200)) {
        let mut app = App::new(vec!["a".into(), "b".into(), "c".into()]);

        for move_down in actions {
            if move_down {
                app.update(Action::MoveDown);
            } else {
                app.update(Action::MoveUp);
            }

            prop_assert!(app.selected < app.items.len());
        }
    }
}
```

Useful TUI invariants:

```text
selected < items.len(), unless list is empty
scroll_offset <= selected
selected item is visible
modal mode can always be exited
search query update never panics
empty list operations never panic
resize never creates invalid layout
background result updates only matching task id
```

---

# 13. Test Async TUI Behavior

Many advanced TUIs have background tasks:

* Loading files
* Watching logs
* Calling APIs
* Running subprocesses
* Streaming model output
* Refreshing dashboard data

Do not test these through the real terminal event loop. Instead model runtime messages:

```rust
pub enum AppEvent {
    User(Action),
    Backend(TaskMessage),
    Tick,
}

pub enum TaskMessage {
    Started { task_id: u64 },
    Progress { task_id: u64, percent: u8 },
    Finished { task_id: u64, result: String },
    Failed { task_id: u64, error: String },
}
```

Test the state update:

```rust
#[test]
fn ignores_stale_task_result() {
    let mut app = App::new(vec![]);
    app.current_task_id = Some(2);

    app.handle_task_message(TaskMessage::Finished {
        task_id: 1,
        result: "old result".into(),
    });

    assert!(app.result.is_none());
}

#[test]
fn applies_current_task_result() {
    let mut app = App::new(vec![]);
    app.current_task_id = Some(2);

    app.handle_task_message(TaskMessage::Finished {
        task_id: 2,
        result: "new result".into(),
    });

    assert_eq!(app.result.as_deref(), Some("new result"));
}
```

For async runtime tests, always use timeout:

```rust
#[tokio::test]
async fn background_loader_sends_finished_message() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        run_background_loader(),
    )
    .await;

    assert!(result.is_ok());
}
```

---

# 14. Testing Mouse Events

Mouse events should also be translated into actions.

```rust
pub enum Action {
    SelectRow(usize),
    OpenContextMenu { x: u16, y: u16 },
}

pub fn mouse_to_action(event: MouseEvent, layout: &LayoutInfo) -> Option<Action> {
    // convert terminal coordinates into app action
    todo!()
}
```

Test coordinate mapping separately:

```rust
#[test]
fn clicking_second_row_selects_second_item() {
    let layout = LayoutInfo {
        list_x: 0,
        list_y: 2,
        list_width: 80,
        row_height: 1,
    };

    let event = mouse_down_at(5, 3);

    assert_eq!(
        mouse_to_action(event, &layout),
        Some(Action::SelectRow(1))
    );
}
```

Avoid testing real mouse capture in default CI unless you have a pseudo-terminal test harness.

---

# 15. Testing Terminal Setup and Teardown

For `crossterm`, keep setup/teardown isolated:

```rust
pub fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    // enter alternate screen, hide cursor, enable mouse
    todo!()
}

pub fn restore_terminal() -> anyhow::Result<()> {
    // show cursor, leave alternate screen, disable raw mode
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
```

Do not run this in normal unit tests. Instead:

* Keep it very thin.
* Manually test it.
* Add an ignored smoke test only if necessary.
* Make sure panic paths restore terminal state.

Pattern:

```rust
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}
```

Test the guard logic without requiring a real terminal by abstracting terminal operations behind a trait if needed.

---

# 16. Snapshot Stabilization Rules

TUI snapshots can become noisy. Normalize these before snapshotting:

| Problem            | Stabilization                                        |
| ------------------ | ---------------------------------------------------- |
| Time               | Inject fake clock                                    |
| Random IDs         | Use deterministic seed                               |
| Absolute paths     | Replace workspace path with `<workspace>`            |
| Terminal width     | Fixed test sizes                                     |
| Colors/styles      | Decide whether to snapshot text only or buffer cells |
| Unicode width      | Test key cases explicitly                            |
| OS path separators | Normalize `\` to `/`                                 |
| Async progress     | Freeze state before render                           |

Example:

```rust
fn normalize_screen(screen: String) -> String {
    screen
        .replace(env!("CARGO_MANIFEST_DIR"), "<crate>")
        .replace('\\', "/")
}
```

Then:

```rust
insta::assert_snapshot!(normalize_screen(screen));
```

---

# 17. Testing Styles and Colors

Text-only snapshots may miss style regressions.

Options:

1. Text snapshot only
   Good for layout/content stability.

2. Buffer-level assertions with style
   Good for selected row, error state, disabled state.

Example:

```rust
#[test]
fn selected_row_uses_highlight_style() {
    let app = App::new_with_selected(vec!["a".into(), "b".into()], 1);
    let buffer = render_to_buffer(&app, 80, 20);

    let cell = buffer.get(2, 3); // example coordinate
    assert_eq!(cell.style().fg, Some(Color::Yellow));
}
```

Use style assertions sparingly. They are useful for:

```text
selected row
error banner
focused panel
disabled action
warning severity
```

---

# 18. CLI Launch Tests for TUI Apps

Even if most behavior is tested headlessly, still add a few binary-level tests.

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag_prints_help_without_entering_tui() {
    let mut cmd = Command::cargo_bin("my-tui").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
```

For TUI mode, provide a non-interactive mode if possible:

```bash
my-tui --check-config
my-tui --print-default-config
my-tui --headless --script tests/fixtures/session.txt
```

A scripted/headless mode is very useful for CI:

```text
tests/fixtures/sessions/basic_navigation.tui
  key:j
  key:j
  key:enter
  assert:selected=2
  key:q
```

This requires you to design a small session runner, but it pays off for complex TUI apps.

---

# 19. Recommended TUI Test Commands

Fast local:

```bash
cargo test -p my-tui app
cargo test -p my-tui input
cargo test -p my-tui update
```

Rendering snapshots:

```bash
cargo insta test
cargo insta review
```

All local tests:

```bash
cargo nextest run -p my-tui
cargo test -p my-tui --doc
```

Ignored terminal smoke tests:

```bash
cargo test -p my-tui -- --ignored
```

With logs:

```bash
RUST_LOG=debug cargo test -p my-tui tui -- --nocapture
```

---

# 20. Recommended CI Strategy for TUI Tests

```text
PR CI:
  - cargo fmt --check
  - cargo clippy --all-targets --all-features
  - cargo nextest run -p my-tui
  - cargo insta test
  - cargo test --doc

OS Matrix:
  - Linux
  - macOS
  - Windows

Nightly:
  - property tests with larger case count
  - terminal smoke tests if available
  - scripted TUI sessions
  - coverage
```

Example GitHub Actions fragment:

```yaml
tui-tests:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    - uses: Swatinem/rust-cache@v2
    - uses: taiki-e/install-action@nextest
    - run: cargo fmt --check
    - run: cargo clippy -p my-tui --all-targets --all-features -- -D warnings
    - run: cargo nextest run -p my-tui
    - run: cargo test -p my-tui --doc
```

For snapshots, you usually want CI to fail on unreviewed snapshot changes:

```yaml
    - run: cargo install cargo-insta
    - run: cargo insta test
```

---

# 21. TUI-Specific Anti-Patterns

| Anti-pattern                          | Consequence                               | Better design                            |
| ------------------------------------- | ----------------------------------------- | ---------------------------------------- |
| Business logic inside terminal loop   | Hard to test, hard to refactor            | `Event -> Action -> update` architecture |
| Only manual testing                   | Regressions in layout and shortcuts       | Unit + snapshot + smoke tests            |
| Snapshotting every tiny visual detail | Brittle snapshots                         | Snapshot key states only                 |
| No small-terminal tests               | Panics or broken layout in real terminals | Add 40x10, 80x24, 120x30 cases           |
| Testing only happy path navigation    | Boundary bugs in selection/scrolling      | Test empty, first, last, overflow cases  |
| Real terminal required in CI          | Flaky or impossible CI                    | Use `TestBackend`                        |
| Async tasks directly mutate UI state  | Race conditions                           | Send typed messages into reducer         |
| Key bindings scattered across code    | Conflicting shortcuts                     | Central `input.rs` mapping tests         |
| No OS matrix                          | Path/terminal behavior breaks on Windows  | Linux/macOS/Windows CI                   |
| Ignoring Unicode width                | Layout bugs for CJK/emoji                 | Add explicit Unicode layout cases        |

---

# 22. Final TUI Testing Blueprint

Recommended final structure:

```text
src/
  app.rs          # state, heavily unit-tested
  action.rs       # Action enum
  input.rs        # KeyEvent / MouseEvent -> Action
  update.rs       # reducer/state transitions
  ui/
    mod.rs        # top-level render
    widgets/      # widget renderers
  terminal.rs     # crossterm setup/teardown
  event_loop.rs   # thin runtime loop
tests/
  tui_snapshots.rs
  cli_smoke.rs
  scripted_sessions.rs
fixtures/
  sessions/
  configs/
snapshots/
```

Recommended test policy:

```text
1. Every state transition gets unit tests.
2. Every key binding gets input mapping tests.
3. Every important screen state gets a snapshot.
4. Every list/table gets boundary tests: empty, first, last, overflow.
5. Every async task communicates through typed messages.
6. Every small terminal size must not panic.
7. The real terminal loop stays thin and mostly smoke-tested.
```

The key idea:

```text
A good Rust TUI test design does not try to automate a real terminal first.
It makes the TUI deterministic, headless, and state-driven first.
Then it tests terminal integration only at the outer edge.
```

[1]: https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html?utm_source=chatgpt.com "TestBackend in ratatui::backend - Rust"
[2]: https://ratatui.rs/recipes/testing/snapshots/?utm_source=chatgpt.com "Testing with insta snapshots"
[3]: https://insta.rs/?utm_source=chatgpt.com "Insta Snapshots"
