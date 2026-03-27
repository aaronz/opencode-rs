## Why

The rust-opencode-port project needs end-to-end tests to verify functionality matches the target TypeScript implementation. Without tests, we cannot ensure the Rust port behaves identically to the original opencode project.

## What Changes

1. Create a comprehensive test suite for the Rust implementation
2. Port key e2e tests from the TypeScript target project
3. Run tests to verify the implementation works correctly

## Capabilities

### New Capabilities
- `e2e-tool-tests`: End-to-end tests for tool implementations (grep, read, write, edit, bash, skill)
- `e2e-session-tests`: Tests for session management and messaging
- `e2e-project-tests`: Tests for project discovery and worktree handling
- `e2e-config-tests`: Tests for .opencode configuration loading
- `e2e-agent-tests`: Tests for agent system functionality
- `e2e-skill-tests`: Tests for skill loading and execution

### Modified Capabilities
- None. All new test capabilities.

## Impact

- Test infrastructure in rust-opencode-port
- New test files for each module
- Integration with cargo test
