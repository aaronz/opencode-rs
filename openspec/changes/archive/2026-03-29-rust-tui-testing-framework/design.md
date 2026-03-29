## Context

Rust TUI applications are becoming increasingly popular (ratatui, zed,ruv). However, the testing ecosystem is fragmented and immature compared to Node.js alternatives. Currently, developers rely on ad-hoc approaches: manual testing, basic assert_cmd for CLI, or experimental PTY libraries. This lack of standardized tooling creates friction in CI/CD pipelines and reduces confidence in TUI application quality.

The target users are Rust developers building terminal applications who need reliable, automated testing at multiple levels: unit, snapshot, integration (PTY), and E2E.

## Goals / Non-Goals

**Goals:**
- Provide a comprehensive, production-ready TUI testing framework for Rust
- Support multiple testing paradigms: unit, snapshot, PTY integration, and E2E CLI
- Leverage Rust's type safety for compile-time test correctness
- Achieve test stability comparable to Playwright but for terminal applications
- Include a fluent DSL for constructing TUI test scenarios

**Non-Goals:**
- GUI testing (TUI only)
- Cross-platform mobile TUI support (focus on desktop terminals)
- Network/async event mocking (separate concern)
- Visual regression testing beyond buffer comparison

## Decisions

### 1. Testing Architecture: Four-Layer Pyramid

**Decision**: Implement a four-layer testing architecture (Unit → Snapshot → PTY → E2E CLI)

**Rationale**: Matches established software testing best practices. Unit tests catch logic bugs fast; snapshot tests verify rendering; PTY tests provide realistic terminal behavior; E2E CLI tests validate user-facing behavior.

**Alternatives considered**:
- Single-layer PTY-only: Too slow for rapid iteration
- Unit-only with mocks: Doesn't catch rendering issues
- Full E2E only: Brittle and slow

### 2. Core Library Selection

**Decision**: Use `ratatui` + `crossterm` for rendering/input, `insta` for snapshots, `assert_cmd` + `predicates` for CLI, `portable-pty` for terminal simulation

**Rationale**:
- `ratatui` is the de facto standard for Rust TUI (formerly tui-rs)
- `crossterm` provides cross-platform terminal handling
- `insta` is the Rust standard for snapshot testing
- `assert_cmd` provides ergonomic CLI testing
- `portable-pty` offers reliable PTY simulation across platforms

**Alternatives considered**:
- `expectrl` for expect-style testing: Good for advanced use cases, but `portable-pty` provides more control
- Custom snapshot solution: Reusing `insta` leverages existing Rust tooling

### 3. Snapshot Testing Approach

**Decision**: Use `ratatui`'s `TestBackend` for complete buffer capture rather than ANSI string comparison

**Rationale**: `TestBackend` provides the full buffer state, avoiding ANSI parsing complexity and enabling precise cell-by-cell comparison. Incremental rendering updates only diffs, so full buffer capture is essential.

**Alternatives considered**:
- ANSI string stripping: Loses color/style information, prone to edge cases
- Manual buffer construction: Too verbose for test authors

### 4. PTY Testing Strategy

**Decision**: Dual-mode PTY testing - basic with `portable-pty`, advanced with `expectrl`

**Rationale**:
- `portable-pty` gives low-level control for precise assertions
- `expectrl` enables expect-style pattern matching for complex scenarios
- Different tools suit different complexity levels

**Alternatives considered**:
- Single library: Neither covers all use cases optimally
- Custom PTY wrapper: Reinventing wheel, maintenance burden

### 5. Architecture Pattern: Input → Update → View Separation

**Decision**: Mandate unidirectional data flow in testable TUI applications

**Rationale**: This pattern (inspired by Elm architecture) makes state transitions predictable and testable. Separating logic (update), rendering (view), and input handling enables:
- Unit testing of state machines without UI
- Snapshot testing of view without real events
- Clear boundaries for mocking

**Alternatives considered**:
- MVC pattern: Too coupled for reliable testing
- Event-driven only: Lacks structure for complex state

## Risks / Trade-offs

### Risk: ANSI Escape Sequence Handling

**Risk**: PTY output contains ANSI escape sequences that complicate assertion matching.

**Mitigation**: Use `crossterm`'s buffer API or strip ANSI codes with `strip_ansi_codes` crate. Prefer `TestBackend` for snapshot tests (no ANSI involved).

### Risk: Incremental Rendering

**Risk**: `ratatui` uses incremental rendering (only updates changed cells), which may cause snapshot tests to miss context.

**Mitigation**: Always use `TestBackend` which captures full buffer state. Document that tests should verify specific regions rather than entire screen when possible.

### Risk: Async Event Handling

**Risk**: TUI applications often use async (tokio), making test timing unpredictable.

**Mitigation**: Provide `#[tokio::test]` async testing utilities. Document that tests should control async sources (timers, network) via test flags.

### Risk: Animation/Spinner Instability

**Risk**: Animated elements cause non-deterministic test output.

**Mitigation**: Provide `TestMode` flag (`APP_TEST=1`) that disables animations, debouncing, and non-deterministic elements. Document test mode requirements.

### Risk: Terminal Size Dependencies

**Risk**: Tests may behave differently across terminal sizes.

**Mitigation**: Standardize on fixed dimensions (80x24) for all tests via `TestBackend::new(80, 24)`. Document that CI should use consistent terminal sizes.

### Trade-off: Test Speed vs. Realism

**Decision**: Unit and snapshot tests run fast (<10ms), PTY tests run slower (~100ms), E2E CLI tests may take seconds.

**Mitigation**: Pyramid architecture encourages appropriate test distribution. Fast tests catch most issues; slow tests validate integration.
