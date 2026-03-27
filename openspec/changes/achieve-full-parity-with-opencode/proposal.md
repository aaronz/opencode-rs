## Why

The Rust port of OpenCode (`opencode-rs`) has achieved significant structural parity with the original TypeScript implementation, including a modular CLI, a comprehensive set of LLM providers, and core tool implementations. However, a deep gap analysis reveals that while the files and modules are present, many advanced functional behaviors (like atomic multi-file edits, rich TUI views, and full protocol compliance for ACP/MCP) remain either stubbed or partially implemented. Furthermore, there is a massive discrepancy in end-to-end (e2e) test coverage, which is essential to guarantee that the Rust implementation is as robust and predictable as the TypeScript target. Achieving full parity is necessary to ensure 1:1 behavioral equivalence across all user-facing features.

## What Changes

1.  **Refine Advanced Tooling**: Finalize functional logic for complex tools such as `multiedit` (atomic multi-file changes), `truncation_dir` (large directory management), and `apply_patch`.
2.  **TUI Component Parity**: Implement the full suite of TUI views and dialogs from the TS version, including the session timeline, fork dialogs, and dynamic JSON theme management.
3.  **Core Logic Synchronization**: Align `crates/core` logic for token-aware compaction, paginated message storage, and event bus message types with the TypeScript implementation.
4.  **Full Protocol Compliance**: Complete all edge cases and real-time event streaming for the Agent Control Protocol (ACP) and Model Context Protocol (MCP).
5.  **Comprehensive E2E Testing**: Develop a Rust-based e2e test harness that exercises the CLI binary against ported test cases from the TS project's `packages/app/e2e`.
6.  **CLI Command Logic**: Complete the implementation of all modularized commands (e.g., `upgrade`, `uninstall`, `acp`, `github`) which are currently structural stubs.

## Capabilities

### New Capabilities
- `tui-advanced-ui`: Real-time session timeline, message metadata browsing, and themeable UI components.
- `atomic-tool-ops`: Support for atomic multi-file edits and rollback on tool failure.
- `full-protocol-acp-mcp`: Robust implementation of remote agent and resource protocols.
- `e2e-verification-harness`: An automated integration testing suite for the Rust project.

### Modified Capabilities
- `cli-subcommand-logic`: Functional completeness for all CLI operations.
- `llm-provider-metadata`: Enhanced tracking of token usage and provider-specific configuration.
- `session-history-optimization`: Paginated retrieval and optimized compaction of large conversation histories.

## Impact

- `rust-opencode-port/crates/cli/`: Addition of functional logic to all command modules.
- `rust-opencode-port/crates/tui/`: Implementation of stateful components and views.
- `rust-opencode-port/crates/core/`: Logic alignment for session, bus, and compaction.
- `rust-opencode-port/crates/tools/`: Refinement of tool execution and validation.
- `rust-opencode-port/crates/storage/`: Addition of paginated data access methods.
