## Context

The Rust port has successfully implemented core functionality including basic CLI commands, TUI components, configuration management, and foundational E2E tests. However, comparing against the TypeScript target project reveals significant gaps:

1. **CLI Gaps**: Missing advanced session commands (undo/redo, review), model visibility controls, and database management
2. **E2E Gaps**: Only 5 E2E test files vs 50+ in the target, missing coverage for sidebar, advanced prompt behaviors, projects, and file management
3. **Test Infrastructure**: Current E2E tests are mostly stubs without full implementation

The architecture follows a modular crate structure with clear separation between CLI, core, TUI, and tools. This design will extend that pattern.

## Goals / Non-Goals

**Goals:**
- Implement missing CLI subcommands for session undo/redo and review
- Add model visibility and filtering controls to the models command
- Create comprehensive E2E test suite covering all major user workflows
- Ensure test infrastructure supports async operations and terminal interactions
- Maintain consistency with existing codebase patterns

**Non-Goals:**
- Reimplementing existing working functionality
- Changing the core architecture or crate structure
- Implementing web interface features (the target's web UI)
- Adding new LLM providers or tool types
- Performance optimizations (focus is on feature parity)

## Decisions

**1. E2E Test Approach: Specification-First**
- Decision: Create E2E tests as executable specifications that define expected behavior
- Rationale: This aligns with the target project's test structure and provides clear acceptance criteria
- Alternative: Unit tests only - rejected because E2E tests catch integration issues

**2. CLI Command Structure: Subcommand Pattern**
- Decision: Use nested subcommands (e.g., `session undo`, `session redo`) rather than flags
- Rationale: Matches the target project's CLI design and provides better discoverability
- Alternative: Flat commands with flags - rejected for consistency with target

**3. Test Data Management: Fixture-Based**
- Decision: Use JSON fixtures for test data (sessions, configs, models)
- Rationale: Easy to maintain, version control, and share between tests
- Alternative: Programmatic test data generation - rejected for clarity

**4. Session Undo/Redo: Command Pattern**
- Decision: Implement undo/redo using a command pattern with history stack
- Rationale: Clean separation of concerns, easy to test, supports serialization
- Alternative: Direct state mutation tracking - rejected for complexity

## Risks / Trade-offs

**Risk**: Large number of E2E tests may slow down CI
→ **Mitigation**: Tests are structured to run in parallel; use cargo test --parallel

**Risk**: Session undo/redo may conflict with compaction
→ **Mitigation**: Clear history on compaction events; document this behavior

**Risk**: Terminal E2E tests may be flaky due to timing
→ **Mitigation**: Use deterministic fixtures; mock terminal where possible

**Trade-off**: Comprehensive E2E tests vs Implementation Speed
→ **Resolution**: Prioritize CLI implementation first, then E2E tests as specifications

## Migration Plan

1. **Phase 1**: Implement CLI enhancements (session undo/redo, model visibility)
2. **Phase 2**: Add database management commands
3. **Phase 3**: Create E2E test infrastructure enhancements
4. **Phase 4**: Implement E2E tests in priority order (session → terminal → sidebar → prompt → projects)
5. **Phase 5**: Documentation and final verification

Rollback: Each phase is independent; can rollback by reverting specific commits

## Open Questions

1. Should undo/redo history persist across sessions? (Target behavior unclear)
2. How should model visibility be stored - in config or separate file?
3. Should E2E tests require a running server or use mocks?
