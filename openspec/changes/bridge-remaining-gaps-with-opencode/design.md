## Context

The Rust port of OpenCode has achieved core functional parity with the TypeScript implementation through the `achieve-full-parity-with-opencode` change. However, detailed comparison reveals that the TUI layer and E2E test coverage remain significantly behind the TS target. The TS project (`/Users/aaronzh/Documents/GitHub/opencode/packages/app/`) has:

- 15+ React-based dialogs for settings, model selection, provider management
- Comprehensive file tree with test coverage
- Status popovers and connection indicators
- 10+ E2E test suites covering session, model, settings, terminal workflows
- Rich terminal integration with history

The Rust TUI (`crates/tui`) currently only has basic chat interface, timeline, and command palette. The E2E harness exists but only covers basic CLI commands.

## Goals / Non-Goals

**Goals:**
- Implement 8+ missing TUI dialogs matching TS functionality
- Add file tree component with navigation and selection
- Create status popovers and connection indicators
- Port all E2E test fixtures from TS `packages/app/e2e/`
- Achieve 90%+ E2E test pass rate for critical workflows
- Maintain consistency with existing Rust architecture patterns

**Non-Goals:**
- Pixel-perfect visual replication of React UI (TUI constraints)
- Re-implementing core logic (already done in previous change)
- Adding new features not present in TS target
- Changing existing working implementations

## Decisions

1. **Dialog Architecture**: Use ratatui's `Clear` widget for modal overlays with centered/popup layouts. Each dialog is a separate module in `crates/tui/src/dialogs/`.
   - *Rationale*: Keeps dialog code isolated and testable
   - *Alternative*: Inline dialogs in main app - rejected for maintainability

2. **Settings Storage**: Extend existing config system in `crates/core/src/config.rs` to support all TS settings options.
   - *Rationale*: Centralized configuration management
   - *Alternative*: Separate settings crate - rejected, config already exists

3. **E2E Test Strategy**: Use Playwright-style patterns with process-based CLI spawning.
   - *Rationale*: Matches TS E2E approach, tests actual binary behavior
   - *Alternative*: Unit tests only - rejected, need integration coverage

4. **File Tree Implementation**: Custom ratatui widget with lazy loading for large directories.
   - *Rationale*: Performance for large codebases
   - *Alternative*: External crate - rejected, need custom behavior matching TS

5. **Theme Consistency**: Extend existing `Theme` system to support dialog-specific color schemes.
   - *Rationale*: Already have theme infrastructure from previous change

## Risks / Trade-offs

- **TUI Complexity**: Adding many dialogs increases binary size and complexity. [Mitigation] → Feature-gate optional dialogs, lazy-load dialog modules
- **E2E Test Flakiness**: Process-based tests can be flaky. [Mitigation] → Use deterministic temp directories, retry logic, proper cleanup
- **Maintenance Burden**: Keeping parity with evolving TS target. [Mitigation] → Document all TS-specific behaviors, version-lock test fixtures
- **Performance**: File tree with large directories. [Mitigation] → Virtual scrolling, directory caching, async loading

## Migration Plan

1. Phase 1: Core dialogs (settings, model selection)
2. Phase 2: File management dialogs
3. Phase 3: Status components and polish
4. Phase 4: E2E test porting
5. Phase 5: Integration and verification

Each phase can be deployed independently. Rollback: Remove dialog modules, revert to basic TUI.
