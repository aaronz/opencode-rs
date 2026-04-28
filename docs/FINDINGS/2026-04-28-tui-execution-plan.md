# opencode-rs TUI UX Execution Plan

Date: 2026-04-28
Derived from: `docs/FINDINGS/2026-04-28-tui-ux-audit.md`

## Goal

Move the TUI audit into execution with a safe, high-leverage first slice that addresses P0 UX reliability problems before broader visual refinement.

## Guiding principles

1. Fix reliability before polish.
2. Prefer explicit state over implicit UI inference.
3. Keep user-facing status separate from debug output.
4. Make modal transitions deterministic and testable.
5. Avoid large refactors during the first P0 slice.

## Prioritized workstreams

### Workstream 1: Logging and output isolation (P0)

Objective:

- ensure runtime/debug output does not corrupt the live alternate-screen TUI

Likely files:

- `opencode-rust/crates/logging/src/logger.rs`
- `opencode-rust/crates/logging/src/event.rs`
- `opencode-rust/crates/tui/src/app.rs`
- `opencode-rust/crates/tui/src/server_ws.rs`
- `opencode-rust/crates/tui/src/plugin.rs`
- `opencode-rust/crates/tui/src/shell_handler.rs`

Expected outcome:

- TUI-visible surfaces only render through ratatui
- developer diagnostics route to file or controlled panel/sink
- no uncontrolled stdout/stderr writes during active TUI mode

### Workstream 2: Explicit activity/progress model (P0)

Objective:

- show clear, truthful activity state for long-running operations

Likely files:

- `opencode-rust/crates/tui/src/app.rs`
- `opencode-rust/crates/tui/src/components/status_bar.rs`
- `opencode-rust/crates/tui/src/widgets/indicators.rs`
- `opencode-rust/crates/tui/src/action.rs`

Expected outcome:

- one normalized activity state model
- visible current step, active target, and error/complete status
- reusable across provider validation, tool calls, commands, and tests

### Workstream 3: Provider/API-key/model-selection flow hardening (P0)

Objective:

- make validation success/failure and subsequent transitions deterministic and visible

Likely files:

- `opencode-rust/crates/tui/src/app.rs`
- `opencode-rust/crates/tui/src/dialogs/connect_provider.rs`
- `opencode-rust/crates/tui/src/dialogs/connect_method.rs`
- `opencode-rust/crates/tui/src/dialogs/api_key_input.rs`
- `opencode-rust/crates/tui/src/dialogs/connect_model.rs`
- `opencode-rust/crates/tui/src/dialogs/validation_error_dialog.rs`

Expected outcome:

- validation progress is explicit
- success always advances predictably
- failure always surfaces recovery guidance
- config write/update state is visible enough to build trust

### Workstream 4: UI hierarchy normalization (P1)

Objective:

- improve consistency of layout, spacing, borders, and message hierarchy

Likely files:

- `opencode-rust/crates/tui/src/theme.rs`
- `opencode-rust/crates/tui/src/layout.rs`
- `opencode-rust/crates/tui/src/components/title_bar.rs`
- `opencode-rust/crates/tui/src/components/status_bar.rs`
- `opencode-rust/crates/tui/src/components/input_widget.rs`
- `opencode-rust/crates/tui/src/dialogs/home_view.rs`
- `opencode-rust/crates/tui/src/render/markdown.rs`

## Recommended execution order

### Slice 1: Logging isolation + connection activity state

Why first:

- directly addresses the most trust-damaging P0 issues
- small enough to implement without major architectural churn
- creates reusable status plumbing for later workstreams

Deliverables:

1. identify and route uncontrolled runtime/log output away from the TUI surface
2. introduce a normalized connection/activity status representation
3. surface validation stages in the UI
4. add tests for connection progress/failure state transitions where practical

### Slice 2: Connection-flow transition hardening

Deliverables:

1. tighten validation-complete handling
2. clarify config-save and model-loading handoff
3. ensure failures route cleanly to actionable error dialogs

### Slice 3: Status bar and progress UX expansion

Deliverables:

1. extend activity model beyond connection flow
2. integrate with tool/command/test progress surfaces

### Slice 4: Layout/style consistency pass

Deliverables:

1. normalize spacing and borders
2. improve startup/home hierarchy
3. improve message/tool result rendering

## Risks

1. logging/output issues may originate outside the TUI crate
2. `app.rs` orchestration may resist small changes if state is tightly coupled
3. connection flow may have edge cases hidden in provider-specific branches
4. UI state tests may need new helpers if current tests are too dialog-local

## Verification strategy

1. targeted diagnostics on changed files after each task unit
2. TUI dialog/state tests for provider validation and model selection flows
3. rendering/state tests for visible progress and failure states
4. manual verification that alternate-screen rendering is not corrupted by runtime logging

## First implementation target

Start with Slice 1:

- inspect logging initialization/output routing
- identify current connection progress state representation
- add a first-class connection activity status model
- surface it through the status bar and/or dialog state
- keep the first patch surgical and avoid broad visual redesign
