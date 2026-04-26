You are a senior Rust engineer, Ratatui/TUI testing architect, async systems engineer, and AI coding agent tooling expert.

I am working on `opencode-rs`, a Rust port of opencode / Claude-Code-like AI coding tooling.

The current TUI has end-to-end workflow bugs. This is not just a problem that slash commands such as `/connect` or `/model` cannot be invoked. The deeper problem is that long-running interactive TUI workflows are not tested thoroughly.

A concrete broken scenario:

1. User opens the TUI.
2. User starts the connect flow.
3. The connect flow successfully loads available providers.
4. User selects the `minimax-cn` provider.
5. User inputs an API key.
6. User presses Enter.
7. The TUI enters a “validating key” state.
8. The UI hangs at the validating step.
9. User presses Esc.
10. The TUI reports a 404 error, likely also visible in logs.
11. After this, focus is not restored to the main TUI input.
12. Nothing can be typed anymore.
13. The TUI becomes unusable.

Your task is to design and implement a robust TUI testing strategy and utility layer that can reproduce, diagnose, and prevent this class of bugs.

Do not treat this as a simple command parser test. Treat it as an end-to-end interactive async TUI workflow testing problem.

==================================================
1. Main Goal
==================================================

Create or extend the `ratatui-testing` utility library so it can test real TUI user journeys, including:

- Multi-step interactive flows.
- Provider selection.
- API key input.
- Async provider validation.
- Loading states.
- Validation success.
- Validation failure.
- 404 / provider API errors.
- Hanging validation tasks.
- Esc cancellation.
- Late-arriving async errors after cancellation.
- Focus restoration.
- Input usability after failed or cancelled flows.
- Captured logs and debug diagnostics.

Then use this library to write regression tests for the broken connect workflow and fix the underlying TUI bugs.

==================================================
2. Core Testing Principle
==================================================

The tests must simulate real user behavior as closely as possible.

Do not bypass the TUI flow by directly calling internal command handlers unless testing a lower-level unit.

For end-to-end TUI tests, simulate:

- Typed text.
- Enter.
- Esc.
- Arrow keys.
- Provider selection.
- API key input.
- Modal navigation.
- Async validation completion.
- Async validation timeout.
- Async validation cancellation.

The goal is to prove that the TUI remains usable after every success, failure, timeout, and cancellation path.

==================================================
3. Required Test Harness Capability
==================================================

Implement a reusable `TuiTestHarness` or equivalent abstraction in `ratatui-testing`.

The harness should support:

```rust
let mut tui = TuiTestHarness::new()
    .with_size(120, 40)
    .with_fake_provider("minimax-cn")
    .with_fake_provider_validator(FakeProviderValidator::hangs_forever())
    .with_captured_logs()
    .build()
    .await?;

tui.run_command("/connect").await?;
tui.select_item("minimax-cn").await?;
tui.type_text("test-api-key");
tui.press_enter().await?;

tui.assert_screen_contains("Validating");
tui.press_esc().await?;

tui.assert_main_input_focused();
tui.type_text("hello");
tui.assert_input_equals("hello");
tui.assert_no_pending_modal();
tui.assert_no_input_lock();

The exact API can differ, but the harness must support this level of scenario testing.

==================================================
4. Fake Provider Validation

Do not use real network calls in TUI tests.

Implement fake provider validation behavior so tests are deterministic.

At minimum, support:

FakeProviderValidator::success()
FakeProviderValidator::returns_404()
FakeProviderValidator::returns_error(anyhow!("..."))
FakeProviderValidator::hangs_forever()
FakeProviderValidator::delayed_success(Duration)
FakeProviderValidator::delayed_error(Duration, error)
FakeProviderValidator::manual_control()

These fakes should be injected into the TUI app through dependency injection, not global state.

The purpose is to test the TUI behavior around provider validation, not the actual Minimax API.

==================================================
5. Connect Flow State Testing

Make the connect flow testable as a state machine.

The test should be able to observe or infer states such as:

Idle
ProviderPickerOpen
ProviderSelected
ApiKeyInputFocused
ValidatingKey
ValidationSucceeded
ValidationFailed
Cancelled
RecoveredToMainInput

Add semantic assertions such as:

assert_connect_state(ConnectState::ProviderPickerOpen)
assert_connect_state(ConnectState::ApiKeyInputFocused)
assert_connect_state(ConnectState::ValidatingKey)
assert_connect_state(ConnectState::ValidationFailed)
assert_connect_state(ConnectState::Cancelled)
assert_main_input_focused()
assert_can_type_in_main_input()

If the current architecture does not expose enough state, add safe test-only inspection APIs under #[cfg(test)] or through a debug snapshot mechanism.

Do not make production logic depend on test-only hacks.

==================================================
6. Required Regression Tests

Add regression tests for the following scenarios.

6.1 Connect flow success

Test:

Start TUI.
Run /connect.
Provider list appears.
Select minimax-cn.
API key input appears.
Type fake key.
Press Enter.
Fake validator returns success.
TUI shows connected state.
Active provider becomes minimax-cn.
Focus returns to main input.
User can type a normal message.

Required assertions:

provider list is visible
api key input is focused
validating state appears
success message appears
selected provider is minimax-cn
main input is focused after success
typing still works
no unexpected error is visible
6.2 Connect flow validation returns 404

Test:

Start TUI.
Run /connect.
Select minimax-cn.
Enter fake key.
Fake validator returns 404.
TUI shows a clear validation error.
Error is logged.
The flow exits or returns to a recoverable state.
Focus returns to main input or clearly remains in a retryable API key input.
User can either retry or exit without the TUI freezing.

Required assertions:

404 error is visible or mapped to a friendly message
logs contain provider name minimax-cn
logs contain validation failure details
no spinner remains forever
no modal blocks input accidentally
focus is in a valid location
typing works after the failure or after Esc
6.3 Connect flow validation hangs forever, then Esc cancels

Test:

Start TUI.
Run /connect.
Select minimax-cn.
Enter fake key.
Fake validator never resolves.
TUI shows validating state.
Press Esc.
Validation task is cancelled or detached safely.
TUI returns to normal state.
Main input focus is restored.
User can type a normal message.
No late task can re-lock the UI.

Required assertions:

validating state appears
Esc cancels the connect flow
main input focus is restored
input is usable after Esc
there is no pending modal
there is no input lock
there is no forever spinner
logs contain cancellation info
6.4 Late async error after cancellation

This is a critical race-condition test.

Test:

Start TUI.
Run /connect.
Select minimax-cn.
Enter fake key.
Fake validator is manually controlled and remains pending.
User presses Esc.
TUI returns to main input.
User types a normal message.
The fake validator later returns a 404 error.
The late 404 must not corrupt the UI state.
Main input must remain focused.
User input must not be lost.
The late error may be logged, but it must not re-open or lock the cancelled connect flow.

Required assertions:

after Esc, main input is focused
user can type before late error arrives
late 404 is logged
late 404 does not steal focus
late 404 does not reopen modal
late 404 does not overwrite main input
late 404 does not leave app in validating state
TUI remains usable
6.5 Esc behavior from every connect sub-state

Test Esc from:

ProviderPickerOpen
ApiKeyInputFocused
ValidatingKey
ValidationFailed

Required assertion for every state:

Esc must return to a valid state
focus must be restored or intentionally placed
typing must work after returning to main input
no stale modal remains
==================================================
7. Focus and Input Recovery Requirements

Because the current bug leaves the TUI unable to accept input, every TUI workflow test must include input recovery assertions.

Add helpers such as:

assert_main_input_focused()
assert_can_type("hello")
assert_input_equals("hello")
assert_no_input_lock()
assert_no_modal_captures_input()
assert_no_loading_overlay_blocks_input()

The harness should fail loudly if:

No component owns focus.
More than one component owns focus.
The active component cannot receive input.
A cancelled modal still captures input.
The app is stuck in a loading state after cancellation.
A background task updates a cancelled workflow.
==================================================
8. Logging and Diagnostics

The test harness must capture logs for every TUI scenario.

On test failure, print a structured diagnostic report:

=== TUI TEST FAILURE ===

Scenario:
connect_minimax_validation_hangs_then_esc_recovers

Terminal size:
120x40

Last screen:
<plain text rendering of terminal buffer>

Focus owner:
MainInput | ProviderPicker | ApiKeyInput | Modal | Unknown

Active flow:
ConnectFlow::ValidatingKey

Input buffer:
"..."

Selected provider:
minimax-cn

Selected model:
...

Pending async tasks:
provider_validation:minimax-cn

Recent key events:
["/connect", "Enter", "Down", "Enter", "test-key", "Enter", "Esc"]

Recent app events:
[...]

Captured logs:
[INFO] connect flow started
[INFO] selected provider minimax-cn
[INFO] validating provider key
[ERROR] provider validation failed: 404 ...

Add assertions such as:

assert_logs_contain("minimax-cn")
assert_logs_contain("404")
assert_logs_contain("validation")
assert_logs_not_contain("panic")

This is essential for debugging TUI issues.

==================================================
9. Bug Fix Expectations

After tests reproduce the bug, fix the actual implementation.

Likely areas to inspect:

Connect flow state machine.
Provider validation async task lifecycle.
Cancellation handling on Esc.
Late async result handling after cancellation.
Focus ownership restoration.
Input buffer ownership.
Modal cleanup.
Spinner/loading state cleanup.
Error propagation from provider validators.
Error rendering.
Logging of provider validation failures.
Difference between command execution and interactive TUI workflow execution.

Do not simply make tests pass by weakening assertions.

Do not hide the 404 error.

Do not swallow errors silently.

Do not block the TUI event loop during provider validation.

Do not let a cancelled async task update the active UI state later.

==================================================
10. Architecture Guidance

Prefer an explicit workflow model.

For example:

enum ActiveWorkflow {
    None,
    Connect(ConnectWorkflowState),
    Model(ModelWorkflowState),
}

enum ConnectWorkflowState {
    ProviderPicker,
    ApiKeyInput { provider_id: ProviderId },
    Validating {
        provider_id: ProviderId,
        request_id: WorkflowRequestId,
    },
    ValidationFailed {
        provider_id: ProviderId,
        error: ConnectError,
    },
    Connected {
        provider_id: ProviderId,
    },
}

Use request IDs or generation tokens to prevent stale async results from mutating cancelled workflows.

Example principle:

When validation starts, create request_id = 42.
When result returns, only apply it if the active workflow is still
ConnectWorkflowState::Validating { request_id: 42 }.
If the user pressed Esc and the workflow moved to None, ignore or only log the late result.

This directly addresses the late 404 after Esc problem.

==================================================
11. Validation Commands

Run the repository’s actual validation commands.

Start with:

cargo fmt
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
cargo test -p ratatui-testing
cargo test -p opencode-rs tui

If crate names differ, discover and use the correct commands.

Also add a focused test command for the new regression tests, for example:

cargo test connect_flow
cargo test minimax
cargo test tui_connect
==================================================
12. Expected Final Report

When finished, provide a concise engineering report with:

What was added to ratatui-testing.
What fake provider validation modes were implemented.
What connect-flow tests were added.
Which test reproduced the hanging validation bug.
Which test reproduced the focus-loss bug.
Whether the 404 error was confirmed in logs.
What root cause was found.
What code was changed to fix it.
How cancellation is now handled.
How late async validation results are handled.
How focus is restored after success, failure, and Esc.
Exact validation commands run.
Test results.
Remaining risks or follow-up tasks.
==================================================
13. Definition of Done

The task is complete only when:

The connect flow is covered by realistic TUI scenario tests.
The minimax-cn validation hang scenario is reproducible using a fake validator.
Esc during validation is tested.
404 validation failure is tested.
Late async error after cancellation is tested.
Focus recovery is tested.
User can type after success, failure, and cancellation.
Logs are captured in tests.
Test failures print useful TUI diagnostics.
The actual TUI bug is fixed.
The full relevant test suite passes.

The final goal is not only to fix one bug. The final goal is to make async interactive TUI workflows in opencode-rs testable, debuggable, and regression-resistant.