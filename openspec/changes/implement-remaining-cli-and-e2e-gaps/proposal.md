## Why

The Rust port of OpenCode has achieved significant feature parity with the TypeScript target, but several key CLI commands and E2E test scenarios remain unimplemented. These gaps prevent the Rust port from passing the full test suite of the target project and limit its production readiness. This change aims to implement the remaining CLI functionality and comprehensive E2E test coverage to achieve full functional parity.

## What Changes

- **Enhanced CLI Commands**: Implement missing subcommands and options for session management (undo/redo, review), model management (visibility controls), and database operations
- **Comprehensive E2E Test Suite**: Port remaining E2E tests from TypeScript target including session workflows, terminal features, sidebar interactions, prompt behaviors, and project management
- **Test Infrastructure**: Enhance the E2E test framework to support the new test scenarios with proper fixtures, mocks, and assertions
- **Documentation**: Update CLI documentation to reflect new commands and options

## Capabilities

### New Capabilities
- `cli-session-undo-redo`: Session undo and redo command functionality
- `cli-session-review`: Session review and diff viewing capabilities
- `cli-model-visibility`: Model visibility and filtering controls
- `cli-db-management`: Database management and migration commands
- `e2e-session-advanced`: Advanced session workflows (undo/redo, review, persistence)
- `e2e-terminal-advanced`: Terminal tabs, reconnect, and multi-terminal scenarios
- `e2e-sidebar`: Sidebar navigation, session links, and popover actions
- `e2e-prompt`: Prompt behaviors including async, shell, multiline, history, and mentions
- `e2e-projects`: Project and workspace management workflows
- `e2e-file-management`: File tree navigation and file viewer interactions
- `e2e-command-palette`: Command palette and quick action workflows

### Modified Capabilities
- None - this change focuses on new capabilities only

## Impact

- **CLI Crate**: New subcommands and options added to existing command modules
- **Core Crate**: Enhanced session management for undo/redo functionality
- **CLI Tests**: New E2E test files covering all new capabilities
- **Test Infrastructure**: Enhanced test fixtures and helpers in `tests/common.rs`
- **Documentation**: Updated README and command help text
