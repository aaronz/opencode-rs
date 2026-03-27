## Context

The `opencode-rs` project is a comprehensive Rust port of the TypeScript OpenCode project. Currently, the architecture, major providers, and a modular CLI are in place. However, the system lacks the detailed functional implementation of several advanced features (like `multiedit`, TUI timeline views, and theme management) and hasn't yet reached behavioral equivalence with the TS version to pass all end-to-end tests.

## Goals / Non-Goals

**Goals:**
- Complete the functional implementation of all modularized CLI subcommands.
- Implement the full set of TUI components (timeline, dialogs, themes) to match the TS user experience.
- Achieve 1:1 behavioral parity for all tools, ensuring identical argument handling and output formats.
- Align core logic (compaction, event bus) with the TypeScript implementation details.

**Non-Goals:**
- Rewrite Rust core components that already provide functionality - only refine them for parity.
- Replicate TypeScript-specific implementation details (e.g., Effect system) that don't affect external behavior.
- Implement features that are explicitly marked as "experimental" or "deprecated" in the TS project.

## Decisions

1.  **TUI Component Pattern**: Use a component-based approach in `crates/tui` similar to the React-style components in the TS version, mapping them to `ratatui` widgets and stateful components.
2.  **Tool Argument Validation**: Implement strict JSON schema validation for tool arguments in `crates/tools` to ensure exact match with the TypeScript definitions.
3.  **Process-based Test Harness**: Create a Rust test harness that spawns the compiled binary to run against the TypeScript test fixtures, verifying behavioral equivalence.
4.  **Shared State Refinement**: Refine the `GlobalState` in `crates/core` to better reflect the service-oriented architecture of the TS version, ensuring all modules (CLI, TUI, Agent) share the same state logic.

## Risks / Trade-offs

- **TUI Complexity**: Mapping React-style hooks and contexts from the TS TUI to Rust's ownership model and `ratatui` might be complex and lead to architectural friction.
- **PTY Platform Edge Cases**: Ensuring consistent behavior for terminal interactions across macOS, Linux, and Windows (as covered in the TS `win32.ts` and `pty.ts`).
- **Test Alignment Drift**: As the target project evolves, keeping the Rust port in sync will require continuous monitoring of the TS repository.
