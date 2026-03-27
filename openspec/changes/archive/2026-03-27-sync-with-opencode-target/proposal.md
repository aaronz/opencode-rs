## Why

The Rust port of OpenCode (`opencode-rs`) has established a solid core architecture and modular CLI, but a detailed gap analysis against the original TypeScript project reveals significant missing functional logic, advanced UI/UX commands, and a complete lack of end-to-end (e2e) testing. This change is necessary to achieve functional equivalence with the target project, ensuring that all feature flows (from complex tool use like `multiedit` to rich TUI interactions like session timelines) are fully implemented and verified against a robust test suite.

## What Changes

1.  **Functional CLI Parity**: Complete the logic for all modularized CLI subcommands, particularly those missing in Rust like `layout`, `prompt-input`, and the extensive `session.*` sub-interface (share, timeline, fork, etc.).
2.  **Advanced Tooling**: Implement missing tool logic for `multiedit` (atomic multi-file changes), `codesearch` (AST-aware searching), and `truncation-dir` (large directory handling).
3.  **TUI Component Richness**: Implement advanced TUI views and components in `crates/tui`, including the session timeline, fork dialogs, and dynamic JSON theme loading to match the TS app experience.
4.  **Core Logic Synchronization**: Refine `crates/core` logic for token-aware message compaction, paginated history retrieval, and full EventBus parity with all TS internal message types.
5.  **ACP/MCP Protocol Full Support**: Finalize all edge cases for the Agent Control Protocol (real-time event streaming) and Model Context Protocol (resource discovery).
6.  **E2E Test Harness**: Implement a new Rust-based end-to-end test suite that exercises the full CLI/Agent/Tool stack, porting high-value test cases from the TS project's `packages/app/e2e`.

## Capabilities

### New Capabilities
- `tui-advanced-navigation`: Real-time session timeline, forking, and message metadata browsing in the TUI.
- `atomic-multiedit`: Capability to apply changes across multiple files or rollback entirely on failure.
- `resource-aware-mcp`: Full discovery and validation of resources from external MCP servers.
- `e2e-test-suite`: A comprehensive integration and end-to-end testing harness for the Rust binary.

### Modified Capabilities
- `cli-subcommand-logic`: Functional implementation of all commands that were previously only modularized stubs.
- `session-data-pagination`: Optimized retrieval of large session histories via offset/limit.
- `llm-usage-tracking`: Precise token counting and usage metadata for all provider responses.

## Impact

- `rust-opencode-port/crates/cli/`: Functional logic for all command modules.
- `rust-opencode-port/crates/tui/`: Implementation of new components and stateful views.
- `rust-opencode-port/crates/tools/`: New logic for `multiedit`, `codesearch`, and `truncation_dir`.
- `rust-opencode-port/crates/storage/`: Addition of paginated retrieval methods.
- `rust-opencode-port/crates/core/`: Compaction and EventBus refinements.
