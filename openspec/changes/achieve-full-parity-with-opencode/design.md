## Context

The `opencode-rs` project is a Rust port of the original TypeScript OpenCode. While the high-level architecture, providers, and modular CLI structure are established, a gap analysis reveals missing functional logic in advanced tools (e.g., `multiedit`, `truncation_dir`), TUI richness (timeline, themes), and core logic (token-aware compaction). Additionally, there is no end-to-end (e2e) test harness for the Rust binary, making it difficult to guarantee functional parity with the TypeScript version.

## Goals / Non-Goals

**Goals:**
- Implement functional logic for all modularized CLI subcommands.
- Achieve 1:1 behavioral parity for all tools, including identical argument validation and output formatting.
- Implement the full set of TUI views and dialogs from the target project.
- Establish a Rust-based e2e test harness that exercises the compiled CLI binary.
- Align core logic (compaction, event bus) with the service-oriented behavior of the TS version.

**Non-Goals:**
- Rewrite the current Rust architecture or core crates - only extend and refine.
- Replicate internal implementation details (e.g., Effect system) that do not affect external behavior.
- Implement features explicitly marked as "experimental" or "deprecated" in the TS project.

## Decisions

1.  **Modular TUI Architecture**: Map React-style components and hooks from the TS TUI to stateful widgets and shared state logic in `crates/tui`, using `ratatui` as the core rendering library.
2.  **Strict Tool Validation**: Implement JSON schema validation for tool arguments in `crates/tools` to ensure exact compatibility with the TS schemas.
3.  **Process-based E2E Harness**: Develop a test harness in Rust that spawns the CLI binary with controlled environment variables and file systems to run against ported TS test fixtures.
4.  **Service-oriented Core**: Refine the `GlobalState` and `EventBus` in `crates/core` to better reflect the service interactions in the TS version, ensuring consistent state across CLI and TUI.

## Risks / Trade-offs

- **TUI Architectural Friction**: Translating a complex React-based UI to a TUI might lead to significant state management complexity in Rust. [Mitigation] → Use a centralized message-passing architecture within `crates/tui`.
- **PTY Platform Parity**: Ensuring identical terminal interaction behavior across macOS, Linux, and Windows. [Mitigation] → Reference the TS `win32.ts` and `pty.ts` logic closely.
- **E2E Test Complexity**: Creating a robust e2e harness for a CLI can be slow and error-prone. [Mitigation] → Start with high-value core flows (session, model, tool) before expanding.
