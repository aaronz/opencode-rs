## 1. CLI Session Undo/Redo Commands

- [x] 1.1 Add undo subcommand to session command with --steps option
- [x] 1.2 Add redo subcommand to session command
- [x] 1.3 Implement undo history tracking in core Session struct
- [x] 1.4 Implement redo history stack
- [x] 1.5 Add persistence for undo/redo history
- [x] 1.6 Write tests for undo/redo functionality

## 2. CLI Session Review Commands

- [x] 2.1 Add review subcommand to session command
- [ ] 2.2 Implement diff generation for session changes
- [ ] 2.3 Add --file filter option for reviewing specific files
- [ ] 2.4 Add --format option for JSON output
- [x] 2.5 Add diff subcommand for single file diffs
- [ ] 2.6 Implement line number display in diffs

## 3. CLI Model Visibility Commands

- [ ] 3.1 Add visibility subcommand to models command
- [ ] 3.2 Implement --hide option to hide models
- [ ] 3.3 Implement --show option to unhide models
- [ ] 3.4 Add --list-hidden option to show hidden models
- [ ] 3.5 Add visibility field to model configuration
- [ ] 3.6 Implement --visibility filter for models list

## 4. E2E Test Infrastructure

- [ ] 4.1 Enhance E2E test fixtures with session state helpers
- [ ] 4.2 Add terminal interaction helpers for E2E tests
- [ ] 4.3 Create mock providers for deterministic testing
- [ ] 4.4 Add file system fixtures for file management tests
- [ ] 4.5 Implement async operation wait helpers

## 5. E2E Session Advanced Tests

- [ ] 5.1 Create e2e_session_undo_redo.rs with undo scenarios
- [ ] 5.2 Create e2e_session_review.rs with review scenarios
- [ ] 5.3 Create e2e_session_persistence.rs with persistence scenarios
- [ ] 5.4 Add tests for undo/redo UI interactions
- [ ] 5.5 Add tests for diff viewing UI

## 6. E2E Terminal Advanced Tests

- [ ] 6.1 Create e2e_terminal_tabs.rs with tab scenarios
- [ ] 6.2 Create e2e_terminal_reconnect.rs with reconnect scenarios
- [ ] 6.3 Create e2e_terminal_init.rs with initialization scenarios
- [ ] 6.4 Add tests for terminal output verification
- [ ] 6.5 Add tests for multi-terminal workflows

## 7. E2E Sidebar Tests

- [ ] 7.1 Create e2e_sidebar_navigation.rs with navigation scenarios
- [ ] 7.2 Create e2e_sidebar_sessions.rs with session link scenarios
- [ ] 7.3 Create e2e_sidebar_popover.rs with popover action scenarios
- [ ] 7.4 Add tests for sidebar toggle functionality
- [ ] 7.5 Add tests for context menu interactions

## 8. E2E Prompt Tests

- [ ] 8.1 Create e2e_prompt_async.rs with async operation scenarios
- [ ] 8.2 Create e2e_prompt_shell.rs with shell command scenarios
- [ ] 8.3 Create e2e_prompt_multiline.rs with multiline input scenarios
- [ ] 8.4 Create e2e_prompt_history.rs with history navigation scenarios
- [ ] 8.5 Create e2e_prompt_mentions.rs with @ mention scenarios
- [ ] 8.6 Add tests for prompt submission and cancellation

## 9. E2E Projects Tests

- [ ] 9.1 Create e2e_projects_management.rs with project CRUD scenarios
- [ ] 9.2 Create e2e_projects_workspace.rs with workspace scenarios
- [ ] 9.3 Create e2e_projects_edit.rs with project editing scenarios
- [ ] 9.4 Add tests for project switching
- [ ] 9.5 Add tests for workspace persistence

## 10. E2E File Management Tests

- [ ] 10.1 Create e2e_file_tree.rs with file tree navigation scenarios
- [ ] 10.2 Create e2e_file_viewer.rs with file viewer scenarios
- [ ] 10.3 Create e2e_file_open.rs with file opening scenarios
- [ ] 10.4 Add tests for drag and drop file operations
- [ ] 10.5 Add tests for file search and filtering

## 11. E2E Command Palette Tests

- [ ] 11.1 Create e2e_command_palette.rs with palette scenarios
- [ ] 11.2 Create e2e_quick_actions.rs with quick action scenarios
- [ ] 11.3 Create e2e_keyboard_shortcuts.rs with shortcut scenarios
- [ ] 11.4 Add tests for command search and filtering
- [ ] 11.5 Add tests for custom keyboard shortcuts

## 12. Integration and Verification

- [ ] 12.1 Run full E2E test suite
- [ ] 12.2 Fix any failing E2E tests
- [ ] 12.3 Verify CLI commands work end-to-end
- [ ] 12.4 Update documentation with new commands
- [ ] 12.5 Final parity check against TS test suite
