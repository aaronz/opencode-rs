You are a senior TUI/CLI product designer, terminal UX expert, Rust engineer, and AI coding agent architect.

I am building the TUI for `opencode-rs`, a Rust-based AI coding agent system inspired by tools such as Claude Code and opencode.

Please review the current TUI from a user experience, interaction design, and implementation perspective, then propose and implement improvements.

Do not only focus on visual polish. Think deeply about the full terminal user experience for an AI coding agent.

==================================================
1. Context
==================================================

`opencode-rs` currently has a working TUI, but the experience still feels rough and inconsistent.

Known issues / improvement areas include:

- Content layout is not polished enough.
- Visual hierarchy is weak.
- Styles, spacing, borders, colors, and typography are inconsistent.
- The task-processing state is unclear.
- Users cannot easily understand what the agent is currently doing.
- Progress feedback is insufficient during long-running operations.
- Startup experience and ASCII art are not well designed.
- Dialogs, modals, validation states, loading states, and error states need refinement.
- Logs or debug output may interfere with the TUI display.
- The TUI should feel closer to a professional AI coding tool, not just a raw terminal wrapper.

You may refer to the UX patterns of:

- Claude Code TUI
- opencode TUI
- Other high-quality terminal applications such as Lazygit, Helix, Zellij, k9s, and GitUI

The goal is not to copy them blindly, but to learn from their interaction quality, information architecture, visual hierarchy, keyboard ergonomics, and state feedback.

==================================================
2. Main Objective
==================================================

Improve the TUI design and implementation so that `opencode-rs` becomes:

- Clear
- Fast
- Calm
- Professional
- Keyboard-first
- Easy to understand
- Easy to debug
- Pleasant for long coding sessions
- Robust during async agent/tool/model operations

The TUI should make users always understand:

1. Where they are
2. What they can do next
3. What the agent is doing now
4. Whether the system is waiting, thinking, calling tools, streaming, validating, failed, or completed
5. How to recover from an error
6. How to inspect logs without corrupting the visible TUI

==================================================
3. Review Scope
==================================================

Please review and improve the following areas.

--------------------------------------------------
3.1 Information Architecture
--------------------------------------------------

Analyze whether the TUI should be organized into clear zones such as:

- Header / session context area
- Main conversation or task output area
- Agent activity / progress area
- Input composer
- Status bar
- Help / shortcuts area
- Optional side panel or debug panel

For each zone, define:

- Purpose
- Content
- Priority
- Update frequency
- Whether it should be always visible or conditional
- How it behaves on small terminal sizes

--------------------------------------------------
3.2 Layout and Visual Hierarchy
--------------------------------------------------

Review and improve:

- Screen layout
- Padding and spacing
- Borders
- Alignment
- Wrapping behavior
- Scroll behavior
- Focus indication
- Selected item indication
- Empty states
- Long output rendering
- Code block rendering
- Tool-call rendering
- Error rendering
- Modal/dialog placement

The UI should avoid visual noise but still provide enough structure.

Please ensure the layout works under:

- Large terminals
- Medium terminals
- Small terminals
- Narrow terminal widths
- Split-pane terminal usage

--------------------------------------------------
3.3 Style System
--------------------------------------------------

Design a consistent TUI style system covering:

- Color palette
- Semantic colors
  - normal
  - muted
  - active
  - focused
  - success
  - warning
  - error
  - loading
  - disabled
- Text styles
- Borders
- Separators
- Icons / symbols
- Spinner styles
- Progress indicators
- ASCII art usage
- Theme support, if practical

Avoid hardcoding random styles across the codebase.

Create reusable style helpers or a centralized style module if needed.

--------------------------------------------------
3.4 Startup Experience
--------------------------------------------------

Improve the startup experience.

Consider:

- Whether ASCII art should be displayed
- Whether ASCII art should be compact or adaptive
- Whether it should be disabled in small terminals
- Startup loading states
- First-run experience
- Current project/repo detection
- Provider/model status summary
- Config health check summary
- Helpful next actions after startup

The startup screen should feel professional and useful, not decorative only.

Example startup information may include:

- `opencode-rs`
- Current working directory / detected repo
- Current provider and model
- Config location
- Session mode
- Available commands
- Any warnings or missing setup steps

--------------------------------------------------
3.5 Agent Progress and Task State UX
--------------------------------------------------

This is a critical area.

Improve the user experience for long-running AI coding tasks.

The TUI should clearly show states such as:

- Idle
- Reading context
- Planning
- Calling model
- Streaming response
- Calling tool
- Editing files
- Running command
- Running tests
- Waiting for user confirmation
- Validating result
- Completed
- Failed
- Cancelled

For each state, define:

- Display text
- Visual indicator
- Spinner or progress pattern
- Whether elapsed time is shown
- Whether the active provider/model is shown
- Whether current tool/command/file is shown
- How errors are surfaced
- How cancellation is handled

Do not invent fake progress percentages unless the system can measure them.

Use truthful progress indicators such as:

- Step-based progress
- Current activity label
- Elapsed time
- Streaming token/activity indicator
- Tool execution status
- Test command status
- File operation status

--------------------------------------------------
3.6 Message Rendering
--------------------------------------------------

Improve rendering for:

- User messages
- Assistant messages
- System/status messages
- Tool calls
- Tool results
- File changes
- Diffs
- Errors
- Warnings
- Code blocks
- Markdown
- Lists
- Tables
- Long logs
- Collapsed/expanded sections

The TUI should make AI coding sessions readable and reviewable.

Special attention should be paid to:

- Avoiding wall-of-text output
- Making tool calls understandable
- Showing file edits clearly
- Showing failed commands clearly
- Making retry/recovery obvious
- Keeping logs separate from user-facing conversation output

--------------------------------------------------
3.7 Input Composer UX
--------------------------------------------------

Review and improve the input area.

Consider:

- Multi-line input
- Cursor behavior
- Placeholder text
- Command suggestions
- Slash command discovery
- History navigation
- Keyboard shortcuts
- Submit behavior
- Cancel behavior
- Paste handling
- Large prompt handling
- Attached file/path references
- Context hints

The input composer should support both quick commands and long instructions.

--------------------------------------------------
3.8 Dialogs, Modals, and Flows
--------------------------------------------------

Review and improve key TUI flows such as:

- Provider selection
- API key input
- API key validation
- Model selection
- MCP configuration
- Command palette
- Confirmation dialogs
- Error dialogs
- Help screen
- Settings screen

For each flow, ensure:

- Clear title
- Clear description
- Clear focused control
- Clear success state
- Clear failure state
- Clear escape/cancel behavior
- No hanging loading state
- No logs printed into the middle of the TUI
- Dialog closes or transitions correctly after success
- Next dialog appears when expected

Pay special attention to the provider/API-key/model-selection flow.

Known problem pattern:

- User selects provider.
- User enters API key.
- Validation starts.
- TUI hangs or remains unclear.
- Logs appear inside the TUI screen.
- Validation succeeds but dialog does not close.
- Model-selection dialog does not appear.
- Config is not updated correctly.

Please audit this flow deeply.

--------------------------------------------------
3.9 Error, Logging, and Debug UX
--------------------------------------------------

Improve error and debugging experience.

Requirements:

- Runtime logs must never corrupt the TUI display.
- User-facing errors should be concise and actionable.
- Detailed debug logs should go to log files or a dedicated debug panel.
- The TUI should show where logs are stored.
- Errors should include recovery actions when possible.
- Failed model/provider requests should show enough context:
  - Provider
  - Model
  - Operation
  - HTTP status, if available
  - Error message
  - Config path involved
  - Suggested next step

Design clear separation between:

- User-facing status
- Recoverable validation errors
- Fatal errors
- Debug logs
- Developer diagnostics

--------------------------------------------------
3.10 Keyboard Interaction Model
--------------------------------------------------

Define and improve keyboard behavior.

At minimum, review:

- Enter
- Shift+Enter / Alt+Enter
- Esc
- Ctrl+C
- Ctrl+L
- Ctrl+R
- Tab / Shift+Tab
- Arrow keys
- PageUp / PageDown
- Vim-like navigation if supported
- Command palette shortcut
- Help shortcut

The behavior should be consistent across the whole TUI.

--------------------------------------------------
3.11 Accessibility and Terminal Compatibility
--------------------------------------------------

Consider:

- Low-color terminals
- No-color mode
- High-contrast mode
- Unicode fallback
- Screen readers where practical
- Small terminal fallback
- SSH sessions
- Different terminal emulators
- macOS Terminal, iTerm2, Ghostty, VSCode terminal, Warp, Alacritty, WezTerm

The TUI should degrade gracefully.

--------------------------------------------------
3.12 Implementation Quality
--------------------------------------------------

Review the current TUI implementation and improve code structure.

Focus on:

- Separation between state, rendering, events, effects, and business logic
- Avoiding duplicated style code
- Avoiding scattered magic strings
- Clear component boundaries
- Testable rendering components
- Snapshot tests for screens if practical
- Deterministic state transitions
- Robust async handling
- Clean cancellation handling
- No direct stdout/stderr printing while TUI is active
- Centralized logging strategy

If the project uses Ratatui, Crossterm, Tokio, or similar libraries, follow their best practices.

==================================================
4. Expected Deliverables
==================================================

Please produce the following:

1. TUI UX audit report
   - Current problems
   - Severity
   - User impact
   - Recommended fixes

2. Proposed TUI design specification
   - Layout model
   - Style system
   - State model
   - Interaction model
   - Startup experience
   - Error/logging model

3. Implementation plan
   - Prioritized tasks
   - Files/modules likely to change
   - Refactoring strategy
   - Risk areas
   - Testing strategy

4. Concrete code changes
   - Implement the highest-priority improvements
   - Refactor style/layout/state code where needed
   - Fix obvious UX bugs found during review

5. Test coverage
   - Add or update tests for TUI state transitions
   - Add rendering snapshot tests if the codebase supports them
   - Add integration tests for provider/API-key/model-selection flow where practical
   - Ensure logs do not corrupt the TUI

6. Documentation updates
   - Update user-facing docs if behavior changes
   - Update developer docs for TUI architecture, logging, and testing

==================================================
5. Prioritization
==================================================

Please prioritize improvements in this order:

P0:
- Logs must not corrupt the TUI.
- Long-running operations must show clear current status.
- Provider/API-key/model-selection flow must not hang.
- Validation success/failure must transition correctly.
- Config updates must be visible and reliable.
- Fatal errors must be actionable.

P1:
- Better layout, spacing, visual hierarchy, and message rendering.
- Better startup screen.
- Better status bar.
- Better input composer behavior.
- Better keyboard consistency.

P2:
- Theme system.
- Advanced command palette.
- Collapsible tool results.
- Rich diff rendering.
- Accessibility improvements.
- Snapshot testing polish.

==================================================
6. Design Principles
==================================================

Follow these principles:

- Terminal-first, not web UI copied into terminal.
- Calm interface, no noisy animation.
- Status must be truthful.
- The user should never wonder whether the app is stuck.
- Debug logs must be available but not intrusive.
- Important information should be visible without overwhelming the user.
- Keyboard behavior must be predictable.
- Every async operation needs a clear lifecycle.
- Every modal flow needs success, failure, cancel, and retry states.
- The UI should support long coding sessions.
- The implementation should be maintainable by future coding agents.

==================================================
7. Output Format
==================================================

Please structure your response as:

1. Executive summary
2. Current UX audit
3. Target TUI experience
4. Proposed layout model
5. Proposed style system
6. Agent/task progress state model
7. Startup experience design
8. Dialog/modal flow design
9. Error/logging/debug design
10. Keyboard interaction model
11. Implementation plan
12. Test plan
13. Concrete code changes
14. Remaining risks and follow-up tasks

Be specific. Do not give only generic UX advice. Tie every recommendation back to the current codebase and actual user flows.