## 1. Advanced Tool Implementation

- [x] 1.1 Refine the `multiedit` tool in `crates/tools` to support atomic edits (validate all first).
- [x] 1.2 Implement the `codesearch` tool using AST-aware search patterns.
- [x] 1.3 Implement the `truncation-dir` tool for large directory management.
- [x] 1.4 Refine argument validation for all existing tools to match TS schemas.

## 2. TUI Component Richness

- [x] 2.1 Implement the Session Timeline view in `crates/tui`.
- [x] 2.2 Add the Fork Session dialog and associated state logic.
- [x] 2.3 Implement dynamic theme loading from JSON files in the TUI.
- [x] 2.4 Align TUI message rendering with the TS app's visual style.

## 3. Core Logic Alignment

- [x] 3.1 Implement token-aware compaction logic in `crates/core`.
- [x] 3.2 Add paginated message retrieval to the Session storage service in `crates/storage`.
- [x] 3.3 Refine the `GlobalState` in `crates/core` to match target service architecture.
- [x] 3.4 Ensure full event bus parity for all internal message types.

## 4. CLI Subcommand Logic

- [x] 4.1 Implement missing logic for the `layout` subcommand.
- [x] 4.2 Implement missing logic for the `prompt-input` subcommand.
- [x] 4.3 Complete the functional implementation of all `session.*` subcommands.
- [x] 4.4 Finalize the logic for `upgrade`, `uninstall`, and `acp` commands.
- [x] 4.5 Add the `--json` flag to all subcommands where missing.

## 5. Protocol Parity (ACP/MCP)

- [x] 5.1 Implement real-time event streaming for ACP agents.
- [x] 5.2 Add full resource discovery support for MCP servers.
- [x] 5.3 Complete the routing logic for MCP tool execution.

## 6. End-to-End Testing

- [x] 6.1 Create the Rust-based e2e test harness that exercises the CLI binary.
- [x] 6.2 Port and run high-value `packages/app/e2e` cases (home, session, prompt).
- [x] 6.3 Verify overall functional parity by passing all critical ported tests.

