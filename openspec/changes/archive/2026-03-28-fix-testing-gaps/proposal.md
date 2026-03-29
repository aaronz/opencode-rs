## Why

The OpenCode project has a test coverage gap of 42%, with 6 packages having zero tests (desktop, plugin, web, script, function, slack) and 8 core modules lacking tests (id, flag, global, env, command, shell, worktree, bun). This leads to potential bugs in critical functionality like AI providers, CLI commands, and storage layers. Fixing these gaps is essential for code quality and preventing regressions.

## What Changes

- Complete test coverage for untested opencode modules: id, flag, global, env, command, shell, worktree
- Add tests for high-priority modules with insufficient coverage: provider (31→15+ tests), cli (52→20+ tests), storage (7→5 tests)
- Add tests to packages without any coverage: desktop, plugin, web, util
- Fix existing test dependency issues that prevent test execution

## Capabilities

### New Capabilities
- `test-id-module`: Add comprehensive tests for Identifier generation in packages/opencode/src/id/id.ts
- `test-flag-module`: Add tests for Feature flags in packages/opencode/src/flag/flag.ts
- `test-global-module`: Add tests for Global path configuration in packages/opencode/src/global/index.ts
- `test-env-module`: Add tests for Environment variable handling in packages/opencode/src/env/index.ts
- `test-provider-module`: Add tests for AI Provider implementations
- `test-cli-module`: Add tests for CLI command handlers
- `test-storage-module`: Add tests for Storage/database operations
- `test-util-package`: Add tests for util package modules
- `test-plugin-package`: Add tests for plugin package
- `fix-test-dependencies`: Fix bun install/node_modules issues preventing test execution

### Modified Capabilities
- (None - new test coverage only)

## Impact

- packages/opencode/src/id/, packages/opencode/src/flag/, packages/opencode/src/global/, packages/opencode/src/env/
- packages/opencode/src/provider/, packages/opencode/src/cli/, packages/opencode/src/storage/
- packages/util/, packages/plugin/
- Testing infrastructure (bun dependencies, node_modules)
