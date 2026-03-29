## Context

The OpenCode project currently has ~42% test coverage based on analysis of 550 source files across 13 packages. Key issues identified:
- 6 packages (desktop, plugin, web, script, function, slack) have zero tests
- 8 modules in opencode package lack tests: id, flag, global, env, command, shell, worktree, bun
- High-priority modules are undertested: provider (19%), cli (15%), storage (28%)
- Existing tests cannot run due to bun/node_modules dependency issues

The goal is to increase test coverage and ensure all tests can execute properly.

## Goals / Non-Goals

**Goals:**
- Add tests for all 8 untested opencode modules
- Increase test coverage for provider, cli, storage modules
- Add tests to 6 packages without coverage
- Fix test execution dependency issues
- Ensure new tests follow existing patterns (bun:test, colocated with source)

**Non-Goals:**
- Achieve 100% coverage (unrealistic and diminishing returns)
- Modify production code (tests only)
- Add E2E tests (focus on unit tests)
- Refactor existing working tests

## Decisions

1. **Test Framework**: Use Bun's built-in test runner (bun:test) - matches existing project patterns
2. **Test Location**: Colocate tests next to source files using `.test.ts` suffix - matches existing convention
3. **Test Strategy**: Focus on unit tests for pure functions, integration tests for module interactions
4. **Dependency Fix**: Use workspace symlinks instead of shared node_modules to avoid ENOENT errors

### Alternatives Considered
- Vitest: Rejected - project uses bun:test consistently
- Jest: Rejected - legacy, not used in this project
- Separate test directory: Rejected - existing tests are colocated

## Risks / Trade-offs

- [Risk] Some modules have complex dependencies that make unit testing difficult → [Mitigation] Use mocking or focus on integration tests
- [Risk] Fixing bun dependencies may require reinstalling packages → [Mitigation] Document the fix steps in tasks
- [Risk] Some modules may not be testable without refactoring → [Mitigation] Skip and document as technical debt

## Migration Plan

1. Fix bun/node_modules dependency issues
2. Add tests for id, flag modules (already started)
3. Add tests for global, env modules
4. Add tests for provider, cli, storage modules
5. Add tests for util, plugin packages
6. Run all tests to verify execution

## Open Questions

- Should tests for UI components (app package) be prioritized?
- How to handle tests for modules with heavy external dependencies?
