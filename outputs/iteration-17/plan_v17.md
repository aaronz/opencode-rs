# OpenCode-RS v17 Implementation Plan

**Version**: 17  
**Date**: 2026-04-07  
**Status**: Draft

---

## 1. Overview

This plan covers the remaining implementation tasks for OpenCode-RS v17. Based on the spec analysis, all P0 features are implemented. This plan focuses on P1 priority items that should be completed before v17 release.

## 2. Current Status Summary

| Category | Total FRs | Completed | Pending |
|----------|-----------|-----------|---------|
| P0 Features | 35 | 35 | 0 |
| P1 Features | 9 | 0 | 9 |
| P2 Features | 4 | 0 | 4 |

## 3. P1 Tasks (v17 Release Target)

### 3.1 Typewriter Effect (FR-006)
- **Description**: Optimize streaming output with typewriter rendering effect
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**: `crates/tui/src/components/input_widget.rs`
- **Status**: 🔲 Pending

### 3.2 @ Path Completion (FR-013)
- **Description**: Auto-complete file paths when typing `@` references
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**: `crates/tui/src/input/completer.rs`
- **Status**: 🔲 Pending

### 3.3 Ctrl+C Command Termination (FR-022)
- **Description**: Support Ctrl+C to terminate running shell commands
- **Priority**: P1
- **Estimated Effort**: Low
- **Files Affected**: `crates/tui/src/shell_handler.rs`
- **Status**: 🔲 Pending

### 3.4 `/editor` Command (FR-063)
- **Description**: Open external editor for composing messages
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**: `crates/tui/src/input/editor.rs`, `crates/tui/src/command.rs`
- **Status**: 🔲 Pending

### 3.5 `/init` Command (FR-064)
- **Description**: Create or update AGENTS.md in project
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**: `crates/tui/src/command.rs`
- **Status**: 🔲 Pending

### 3.6 ProgressBar Component (FR-084)
- **Description**: Implement Gauge-based progress bar component
- **Priority**: P1
- **Estimated Effort**: Low
- **Files Affected**: `crates/tui/src/widgets/`
- **Status**: 🔲 Pending

### 3.7 Custom Keybinds Config (FR-115)
- **Description**: User-configurable keyboard shortcuts
- **Priority**: P1
- **Estimated Effort**: High
- **Files Affected**: `crates/tui/src/config.rs`, `crates/tui/src/cli/args.rs`
- **Status**: 🔲 Pending

### 3.8 Diff Style Config (FR-116)
- **Description**: Configurable diff display style
- **Priority**: P1
- **Estimated Effort**: Low
- **Files Affected**: `crates/tui/src/components/diff_view.rs`
- **Status**: 🔲 Pending

### 3.9 NDJSON Output Format (FR-142)
- **Description**: Streaming JSON output format support
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**: `crates/opencode-cli/src/`
- **Status**: 🔲 Pending

## 4. Implementation Order

Recommended implementation sequence based on dependencies and user impact:

1. **FR-022** (Ctrl+C) - Quick win, improves UX immediately
2. **FR-084** (ProgressBar) - Low effort, useful for long operations
3. **FR-006** (Typewriter) - High visibility feature
4. **FR-116** (Diff Style) - Low effort, config improvement
5. **FR-013** (@ Completion) - Improves file reference UX
6. **FR-063** (/editor) - External editor integration
7. **FR-064** (/init) - AGENTS.md management
8. **FR-115** (Keybinds) - High effort, complex config
9. **FR-142** (NDJSON) - Output format option

## 5. P2 Tasks (Future Versions)

| Task | Description | Target Version |
|------|-------------|----------------|
| FR-115 completion | Full keybinds configuration | v18 |
| FR-122 | Custom themes | v18 |
| Plugin system | WASM plugin support | v20 |

## 6. Dependencies

- FR-063 depends on FR-084 (ProgressBar for editor feedback)
- FR-115 requires config system understanding from Constitution C-056

## 7. Testing Requirements

Per Constitution Article 4:
- Unit tests for each new component
- Integration tests for commands
- Update TEST_MAPPING.md with new test cases

---

**Next Review**: After P1 items implementation  
**Blocking Issues**: None identified
