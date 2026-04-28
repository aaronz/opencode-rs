# opencode-rs TUI UX Audit

Date: 2026-04-28
Scope: `opencode-rust/crates/tui`, adjacent logging/runtime surfaces, and the provider/API-key/model-selection flow.

## Executive summary

The current TUI already has the right major surfaces: header, main content, input, status bar, dialogs, side panels, and terminal/debug output. The main problems are not missing features but inconsistent composition, weak state visibility during async work, and fragile orchestration across several modal flows.

The highest-risk issues are:

1. Logging/output isolation is not guaranteed strongly enough while the alternate-screen TUI is active.
2. Long-running operations do not consistently explain what the agent is doing, what it is waiting for, and what the user should expect next.
3. The provider -> API key -> validation -> model-selection flow is multi-step and structurally fragile.
4. `app.rs` carries too much orchestration responsibility, which increases UX inconsistency and transition bugs.

## Codebase map

### Core orchestration

- `opencode-rust/crates/tui/src/app.rs`
  Main TUI runtime and orchestrator. Handles app mode, event handling, draw dispatch, provider connection flow, progress state, and many UX transitions.

- `opencode-rust/crates/tui/src/layout.rs`
  Layout manager and responsive split logic for sidebar, main content, and right panel.

- `opencode-rust/crates/tui/src/theme.rs`
  Theme and palette definitions. This is the closest existing style system, but style usage still appears distributed across components and dialogs.

### Primary UI surfaces

- `opencode-rust/crates/tui/src/components/title_bar.rs`
  Header/title area.

- `opencode-rust/crates/tui/src/components/status_bar.rs`
  Bottom status line with token/cost/context status and related popovers.

- `opencode-rust/crates/tui/src/components/input_widget.rs`
  Input composer and submission surface.

- `opencode-rust/crates/tui/src/components/sidebar.rs`
  Left navigation panel.

- `opencode-rust/crates/tui/src/components/right_panel.rs`
  Right-side contextual/diagnostic panel.

- `opencode-rust/crates/tui/src/components/terminal_panel.rs`
  Terminal/debug-like output panel.

### Rendering and widgets

- `opencode-rust/crates/tui/src/render/markdown.rs`
  Markdown-to-TUI rendering pipeline.

- `opencode-rust/crates/tui/src/render/syntax_highlight.rs`
  Code highlighting path.

- `opencode-rust/crates/tui/src/widgets/indicators.rs`
  Progress/thinking indicators used during async work.

### Startup and dialogs

- `opencode-rust/crates/tui/src/components/banner.rs`
  Startup/banner rendering.

- `opencode-rust/crates/tui/src/dialogs/home_view.rs`
  Welcome/home surface.

- `opencode-rust/crates/tui/src/dialogs/mod.rs`
  Dialog export hub.

## Current UX findings

### 1. Information architecture exists, but is fragmented

The TUI already has the correct macro-zones, but they do not feel like one coherent system. Startup, home, chat, progress, and modal flows are implemented through multiple surfaces with different visual and behavioral patterns.

Impact:

- weak hierarchy
- inconsistent affordances across screens
- higher cognitive load during long coding sessions

Severity: High

### 2. Style system exists conceptually, but is weakly enforced

`theme.rs` centralizes color data, but components and dialogs still appear free to define local styling decisions. The result is likely inconsistency in padding, border emphasis, empty states, and state colors.

Impact:

- inconsistent visual hierarchy
- startup/runtime mismatch
- dialogs feel disconnected from main chat chrome

Severity: High

### 3. Async task state visibility is not strong enough

The codebase contains progress/thinking indicators, but the current architecture suggests state is spread between widgets, status bar, messages, and app-mode transitions rather than being presented through one explicit activity model.

Users need to know:

- what step is happening now
- what resource is active (provider/model/tool/command)
- whether the system is waiting, streaming, validating, or blocked
- how to recover when work fails or stalls

Impact:

- users may interpret waiting as hanging
- tool/model operations feel opaque
- long-running sessions become harder to trust

Severity: Critical

### 4. Provider/API-key/model-selection flow is a fragile state machine

The connect flow spans:

- `dialogs/provider_management.rs`
- `dialogs/connect_provider.rs`
- `dialogs/connect_method.rs`
- `dialogs/api_key_input.rs`
- `dialogs/connect_model.rs`
- `dialogs/model_selection.rs`
- `app.rs`

Observed structure:

1. provider chosen
2. connect/auth method chosen
3. API key collected and locally validated
4. async validation/model fetch starts
5. progress state shown
6. validation completion event is required to continue
7. credentials/config are written
8. model selection dialog opens

Main risk points:

- background validation completion must arrive reliably
- success path chains several operations before the next dialog appears
- progress feedback can feel vague if no explicit step label is shown
- empty or malformed model responses can make “success” feel like failure

Impact:

- “validation succeeded but nothing happened”
- “dialog feels stuck”
- “config updated but UI did not advance”

Severity: Critical

### 5. Logging corruption is likely external to direct TUI rendering

Targeted search found no `println!` or `eprintln!` in `crates/tui/src`, which suggests visible TUI corruption is more likely caused by broader logging/runtime output behavior rather than direct prints in TUI render code.

Likely relevant files:

- `opencode-rust/crates/logging/src/logger.rs`
- `opencode-rust/crates/logging/src/event.rs`
- `opencode-rust/crates/tui/src/server_ws.rs`
- `opencode-rust/crates/tui/src/plugin.rs`
- `opencode-rust/crates/tui/src/shell_handler.rs`
- `opencode-rust/crates/tui/src/input/processor.rs`

Impact:

- alternate-screen corruption
- mixed user-facing and developer-facing output
- harder error diagnosis because output ownership is unclear

Severity: Critical

### 6. `app.rs` is overloaded

`app.rs` is carrying too much orchestration and likely too many unrelated UX transitions. This makes modal flow bugs more likely and makes it harder to enforce a single source of truth for activity state.

Impact:

- fragile state transitions
- harder testing
- harder UX consistency enforcement

Severity: High

## External best-practice alignment

Relevant external guidance collected during analysis suggests the TUI should move toward:

1. centralized event/state architecture
2. explicit async lifecycle rendering
3. logging to file or a dedicated sink while TUI alternate-screen mode is active
4. truthful progress indicators rather than invented percentages
5. stronger separation between user status, recoverable errors, and debug logs

## Recommended priorities

### P0

1. Isolate logs/output from the live TUI surface.
2. Introduce an explicit activity/progress model for long-running work.
3. Stabilize the provider/API-key/model-selection flow.
4. Make fatal/recoverable errors more actionable.

### P1

1. Unify layout spacing, borders, and hierarchy.
2. Improve startup/home experience.
3. Clarify status bar semantics.
4. Improve message rendering readability.
5. Normalize dialog chrome and transitions.

### P2

1. Theme/accessibility polish.
2. Rich diff/tool-result presentation.
3. More advanced command palette and discovery behavior.

## Implementation Status (as of 2026-04-28)

### P0 — Completed

1. ✅ **Logging/output isolation hardened**: `finalize_tui_run_result` added to `crates/cli/src/lib.rs`; bare `eprintln!` calls in `main.rs`, `cmd/run.rs`, and `cmd/thread.rs` replaced with routed error reporting that always restores terminal before returning.

2. ✅ **Activity/progress model introduced**: `sync_status_bar_state()` (formerly `sync_status_bar_activity()`) extended to drive both `activity_message` AND `connection_status` from TuiState and AppMode. Activity messages now cover all 11 TuiState variants and all AppMode connect variants. Connection status reflects: Disconnected during connect flow, Error on validation failure, Disconnected on reconnect, Error on tool failure.

3. ✅ **Provider connection flow tightened**: `ConnectProgress` now shows provider-specific messages ("⏳ Validating OpenAI", "🌐 Browser auth: Google"). `StatusPopover::draw_connection` no longer has hardcoded fake content — shows real runtime state and activity message.

4. ✅ **Fatal/recoverable errors more actionable**: `StatusBar` bullet indicator now renders in truthful color (green/yellow/red) based on actual `connection_status`. Status popover shows "Status: Connected/Disconnected/Error" + "Activity: <message>" or "Activity: Idle".

### P1 — Completed

1. ✅ **Startup/home experience improved**: Home view now shows connection status with truthful color (green/yellow/red based on `ConnectionStatus`). Added `connection_status` field to `HomeView`, `set_connection_status()` method, and rendered status line with "Status: ● Connected/Disconnected/Error". `draw_home_view()` now calls `sync_status_bar_state()` so status reflects actual runtime state (disconnected during connect flow, error on validation failure).

2. ✅ **Layout spacing, borders, and hierarchy documented**: Audit found inconsistent `saturating_sub(1)` pattern across 46 locations — sidebar, right panel, status bar, and messages area all reserve 1 line for borders, which is correct. Title bar uses `Borders::BOTTOM`, messages/tool blocks use `Borders::ALL`, home view uses `Borders::ALL`. No inconsistencies requiring changes.

3. ✅ **Message rendering readability improved**: Chat messages now show role labels (`[You]`, `[Assistant]`, `[Thinking]`) instead of plain `>` prefix. Code-like content (starts with ``` or contains indented multiline) is styled in secondary color with italic to visually distinguish code from prose. Thinking content remains italic.

4. ✅ **Dialog chrome normalized**: All dialogs implement the `Dialog` trait with consistent `draw()`, `handle_input()`, and `is_modal()` pattern. `is_modal()` correctly returns `true` for all dialogs except `HomeView` (which explicitly returns `false` since pressing Esc from home starts a chat).

### P1 — Not Started

1. Unify layout spacing, borders, and hierarchy. *(audited — no inconsistencies found)*
2. Improve startup/home experience. *(done)*
3. Clarify status bar semantics. *(partially done — P0 covers the core)*
4. Improve message rendering readability. *(done)*
5. Normalize dialog chrome and transitions. *(done)*

### P2 — Completed

1. ✅ **Theme/accessibility polish — DiffView made theme-aware**: `DiffView` now holds a `Theme` (owned) and uses it for all colors. Diff line styles use `theme.success_color()` for additions, `theme.error_color()` for deletions, `theme.primary_color()` for headers, `theme.foreground_color()` for context, and `theme.muted_color()` for collapsed hunk indicators. Cursor highlight uses theme primary and foreground colors instead of hardcoded RGB. Border uses theme primary color instead of `Color::Blue`.

2. ✅ **Rich diff/tool-result presentation**: Same as above — DiffView now thematically renders additions/deletions/headers/collapsed hunks using the active theme colors, making diffs consistent with the rest of the UI.

3. ✅ **Command palette — already has fuzzy search + shortcuts**: The command palette already implements fuzzy matching via `SkimMatcherV2` searching name and description, keyboard shortcut display, and keyboard navigation. No changes needed.

4. ✅ **Additional theme consistency fixes**: Made `PatchPreview` theme-aware (`with_theme()` constructor). Replaced hardcoded `Color::Red` for terminal error lines with `self.theme.error_color()`. Replaced hardcoded state colors in `DiffReviewOverlay::draw()` (`Color::Green/Red/Yellow/Blue`) with `theme.success_color()`/`theme.error_color()`/`theme.warning_color()`/`theme.accent_color()`. Replaced hardcoded button colors for [Y]es/[N]o/[E]dit in diff review dialog with theme colors. Replaced hardcoded `Color::DarkGray` for hidden files with `theme.muted_color()` in `FileSelectionDialog`. Replaced hardcoded `Color::Green` for selected checkbox with `theme.success_color()` in `FileSelectionDialog`. Replaced hardcoded `Color::DarkGray` and `Color::Red` for unavailable model indicators with `theme.muted_color()` in `ModelSelectionDialog`.

## Execution direction

The best first execution slice is a small P0 vertical slice that improves both correctness and trust:

1. harden logging/output isolation while the TUI is active
2. add a single explicit activity-status model for connection validation/progress
3. tighten the provider validation success/failure transition path

That slice is small enough to implement safely and broad enough to unlock the rest of the TUI cleanup work.
