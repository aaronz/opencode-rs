## Why

After completing the initial full parity implementation, a detailed comparison between the TypeScript target (`/Users/aaronzh/Documents/GitHub/opencode`) and the Rust port reveals significant gaps in the TUI (Terminal User Interface) and E2E (End-to-End) testing layers. The TS project has 15+ rich UI dialogs (settings, model selection, provider management) and comprehensive E2E test suites covering critical user workflows. The Rust port currently has only basic TUI components and lacks the extensive test coverage needed to guarantee behavioral parity with the TS implementation. Bridging these gaps is essential to achieve true functional equivalence and ensure the Rust port can pass the target project's test cases.

## What Changes

- **TUI Dialog Parity**: Implement 8+ missing TUI dialogs including:
  - Settings dialog (general, keybinds, models, providers)
  - Model selection dialogs (paid/unpaid variants)
  - Provider management dialogs
  - File/directory selection dialogs
  - Release notes dialog
  - Connection status popovers
  
- **TUI Component Enhancements**:
  - File tree component with test coverage
  - Title bar with history navigation
  - Terminal panel integration
  - Status popovers
  - Prompt input enhancements
  
- **E2E Test Suite Expansion**:
  - Port all E2E test fixtures from TS `packages/app/e2e/`
  - Implement session management workflow tests
  - Add model connection tests
  - Add settings/configuration tests
  - Add terminal integration tests
  - Add sidebar and navigation tests

- **Tool Registry Parity**: Ensure all tools have consistent JSON schema validation and error handling matching TS behavior

- **Configuration System**: Expand config handling to support all TS configuration options

## Capabilities

### New Capabilities
- `tui-settings-dialogs`: Settings management UI (general, keybinds, models, providers)
- `tui-model-selection`: Model selection dialogs with paid/unpaid variants
- `tui-file-management`: File tree and file/directory selection dialogs
- `tui-status-components`: Status popovers and connection indicators
- `e2e-session-workflows`: Session management E2E test coverage
- `e2e-model-workflows`: Model connection and provider E2E tests
- `e2e-settings-workflows`: Settings and configuration E2E tests
- `e2e-terminal-workflows`: Terminal integration E2E tests

### Modified Capabilities
- `tui-advanced-ui`: Extend existing timeline/fork with settings dialogs
- `e2e-verification-harness`: Expand harness to support complex UI interactions

## Impact

- **Crates Affected**: `crates/tui`, `crates/cli`, `crates/core`
- **Test Infrastructure**: New E2E tests in `crates/cli/tests/e2e_*.rs`
- **Dependencies**: Additional ratatui widgets, dialog handling
- **Breaking Changes**: None - additive enhancements only
- **API Changes**: New TUI public APIs for dialog management
