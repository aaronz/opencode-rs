## 1. TUI Dialog Implementation

- [ ] 1.1 Create `crates/tui/src/dialogs/` module structure
- [ ] 1.2 Implement Settings dialog with tab navigation (General, Keybinds, Models, Providers)
- [ ] 1.3 Implement Model Selection dialog with provider grouping and search
- [ ] 1.4 Implement Provider Management dialog with connection testing
- [ ] 1.5 Implement File Selection dialog with filtering
- [ ] 1.6 Implement Directory Selection dialog
- [ ] 1.7 Implement Release Notes dialog
- [ ] 1.8 Add dialog keybinds to main app (`Ctrl+,` for settings, `Ctrl+M` for models)

## 2. TUI Component Enhancements

- [ ] 2.1 Implement File Tree component with lazy loading
- [ ] 2.2 Add File Tree panel toggle (`Ctrl+Shift+F`)
- [ ] 2.3 Implement Title Bar with session history dropdown
- [ ] 2.4 Add Status Popover system (connection, tokens, context)
- [ ] 2.5 Implement Terminal Panel (`Ctrl+~`)
- [ ] 2.6 Add Prompt Input enhancements (history, multiline)
- [ ] 2.7 Implement virtual scrolling for large lists

## 3. Configuration System Expansion

- [ ] 3.1 Extend Config struct to support all TS settings categories
- [ ] 3.2 Add settings validation logic
- [ ] 3.3 Implement keybind configuration storage
- [ ] 3.4 Add provider settings persistence
- [ ] 3.5 Create settings migration from TS format

## 4. E2E Test Suite - Session Workflows

- [ ] 4.1 Port `e2e/session/create.spec.ts` tests
- [ ] 4.2 Port `e2e/session/list.spec.ts` tests
- [ ] 4.3 Port `e2e/session/resume.spec.ts` tests
- [ ] 4.4 Port `e2e/session/persistence.spec.ts` tests
- [ ] 4.5 Add session forking E2E tests
- [ ] 4.6 Add session sharing E2E tests

## 5. E2E Test Suite - Model Workflows

- [ ] 5.1 Port `e2e/models/list.spec.ts` tests
- [ ] 5.2 Port `e2e/models/switch.spec.ts` tests
- [ ] 5.3 Port `e2e/models/provider.spec.ts` tests
- [ ] 5.4 Add model selection dialog E2E tests
- [ ] 5.5 Add provider connection error E2E tests

## 6. E2E Test Suite - Settings Workflows

- [ ] 6.1 Port `e2e/settings/general.spec.ts` tests
- [ ] 6.2 Port `e2e/settings/keybinds.spec.ts` tests
- [ ] 6.3 Port `e2e/settings/models.spec.ts` tests
- [ ] 6.4 Port `e2e/settings/providers.spec.ts` tests
- [ ] 6.5 Add settings validation E2E tests

## 7. E2E Test Suite - Terminal Workflows

- [ ] 7.1 Port `e2e/terminal/execute.spec.ts` tests
- [ ] 7.2 Port `e2e/terminal/output.spec.ts` tests
- [ ] 7.3 Add long-running command timeout tests
- [ ] 7.4 Add interactive command detection tests

## 8. Integration and Verification

- [ ] 8.1 Run full E2E test suite
- [ ] 8.2 Fix any failing tests
- [ ] 8.3 Verify TUI dialog navigation works end-to-end
- [ ] 8.4 Test settings persistence across restarts
- [ ] 8.5 Verify file tree with large directories
- [ ] 8.6 Final parity check against TS test suite
