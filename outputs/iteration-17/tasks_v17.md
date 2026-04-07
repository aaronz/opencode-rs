# OpenCode-RS v17 Task List

**Version**: 17  
**Date**: 2026-04-07  
**Status**: Active

---

## P1 Tasks (v17 Release)

### Task 1: FR-022 - Ctrl+C Command Termination
- **FR-ID**: FR-022
- **Description**: Support Ctrl+C to terminate running shell commands
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/shell_handler.rs`
- **Dependencies**: None
- **Acceptance Criteria**: 
  - [x] Ctrl+C interrupts running shell command
  - [x] User receives termination feedback
  - [x] No crash or hang on termination
- **Completion Date**: 2026-04-07

### Task 2: FR-084 - ProgressBar Component
- **FR-ID**: FR-084
- **Description**: Implement Gauge-based progress bar component
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/widgets/indicators.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] ProgressBar widget renders correctly
  - [x] Supports 0-100% progress display
  - [x] Works with Ratatui Gauge component
- **Completion Date**: 2026-04-07

### Task 3: FR-006 - Typewriter Effect
- **FR-ID**: FR-006
- **Description**: Optimize streaming output with typewriter rendering effect
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/components/input_widget.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] Characters appear one at a time during streaming
  - [x] Configurable speed (default 20ms per char)
  - [x] Skippable via user input
- **Completion Date**: 2026-04-07

### Task 4: FR-116 - Diff Style Config
- **FR-ID**: FR-116
- **Description**: Configurable diff display style
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/config.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] `diff_style` config option (side-by-side, unified)
  - [x] Style persists across sessions
  - [x] Works with FR-078 DiffView
- **Completion Date**: 2026-04-07

### Task 5: FR-013 - @ Path Completion
- **FR-ID**: FR-013
- **Description**: Auto-complete file paths when typing `@` references
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/input/completer.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] Tab completion after typing `@`
  - [x] Fuzzy matching for file names
  - [x] Shows file icon and path in suggestions
- **Completion Date**: 2026-04-07

### Task 6: FR-063 - /editor Command
- **FR-ID**: FR-063
- **Description**: Open external editor for composing messages
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/command.rs`, `crates/tui/src/app.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] `/editor` command opens $EDITOR
  - [x] Message content returned to TUI after save
  - [x] Supports vim, nano, vscode --wait
- **Completion Date**: 2026-04-07

### Task 7: FR-064 - /init Command
- **FR-ID**: FR-064
- **Description**: Create or update AGENTS.md in project
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/command.rs`, `crates/tui/src/app.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] `/init` creates AGENTS.md if not exists
  - [x] Updates existing AGENTS.md with new content
  - [x] Works with ProjectInitAgent
- **Completion Date**: 2026-04-07

### Task 8: FR-115 - Custom Keybinds Config
- **FR-ID**: FR-115
- **Description**: User-configurable keyboard shortcuts
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/config.rs`
- **Dependencies**: Constitution C-056 (config system)
- **Acceptance Criteria**:
  - [x] `keybinds` config object support
  - [x] Override default shortcuts
  - [x] Conflict detection for duplicate bindings
- **Completion Date**: 2026-04-07

### Task 9: FR-142 - NDJSON Output Format
- **FR-ID**: FR-142
- **Description**: Streaming JSON output format support
- **Priority**: P1
- **Status**: ✅ Done
- **Files**: `crates/tui/src/cli/args.rs`
- **Dependencies**: None
- **Acceptance Criteria**:
  - [x] `--output-format ndjson` flag works
  - [x] Each line is valid JSON object
  - [x] Includes message, tool calls, status events
- **Completion Date**: 2026-04-07

---

## P2 Tasks (Future Versions)

| Task | FR-ID | Description | Target |
|------|-------|-------------|--------|
| T10 | FR-122 | Custom themes | v18 |
| T11 | - | WASM plugin system | v20 |

---

## Completed Tasks (v17)

| Task | FR-ID | Status | Completion Date |
|------|-------|--------|-----------------|
| T00 | FR-001~FR-005 | ✅ Done | 2026-04-07 |
| T01 | FR-010~FR-012 | ✅ Done | 2026-04-07 |
| T02 | FR-020~FR-021 | ✅ Done | 2026-04-07 |
| T03 | FR-030~FR-062 | ✅ Done | 2026-04-07 |
| T04 | FR-070~FR-100 | ✅ Done | 2026-04-07 |
| T05 | FR-110~FR-114 | ✅ Done | 2026-04-07 |
| T06 | FR-120~FR-121 | ✅ Done | 2026-04-07 |
| T07 | FR-130~FR-132 | ✅ Done | 2026-04-07 |
| T08 | FR-140~FR-141 | ✅ Done | 2026-04-07 |
| T09 | FR-022, FR-084, FR-006, FR-116, FR-013, FR-063, FR-064, FR-115, FR-142 | ✅ Done | 2026-04-07 |

---

## Task Statistics

| Metric | Value |
|--------|-------|
| Total Tasks | 11 |
| Completed | 9 |
| In Progress | 0 |
| Pending | 0 |
| P2/Future | 2 |

---

**Last Updated**: 2026-04-07  
**Status**: All P1 Tasks Complete ✅
