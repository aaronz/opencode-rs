## 1. Advanced Tool Implementation

- [ ] 1.1 Refine the `multiedit` tool in `crates/tools` to support atomic edits (validate all first).
- [ ] 1.2 Implement the `codesearch` tool using AST-aware search patterns.
- [ ] 1.3 Implement the `truncation-dir` tool for large directory management.
- [ ] 1.4 Refine argument validation for all existing tools to match TS schemas.

## 2. TUI Component Richness

- [ ] 2.1 Implement the Session Timeline view in `crates/tui`.
- [ ] 2.2 Add the Fork Session dialog and associated state logic.
- [ ] 2.3 Implement dynamic theme loading from JSON files in the TUI.
- [ ] 2.4 Align TUI message rendering with the TS app's visual style.

## 3. Core Logic Alignment

- [ ] 3.1 Implement token-aware compaction logic in `crates/core`.
- [ ] 3.2 Add paginated message retrieval to the Session storage service in `crates/storage`.
- [ ] 3.3 Refine the `GlobalState` in `crates/core` to match target service architecture.
- [ ] 3.4 Ensure full event bus parity for all internal message types.

## 4. CLI Subcommand Logic

- [ ] 4.1 Implement missing logic for the `layout` subcommand.
- [ ] 4.2 Implement missing logic for the `prompt-input` subcommand.
- [ ] 4.3 Complete the functional implementation of all `session.*` subcommands.
- [ ] 4.4 Finalize the logic for `upgrade`, `uninstall`, and `acp` commands.
- [ ] 4.5 Add the `--json` flag to all subcommands where missing.

## 5. Protocol Parity (ACP/MCP)

- [ ] 5.1 Implement real-time event streaming for ACP agents.
- [ ] 5.2 Add full resource discovery support for MCP servers.
- [ ] 5.3 Complete the routing logic for MCP tool execution.

## 6. End-to-End Testing

- [ ] 6.1 Create the Rust-based e2e test harness that exercises the CLI binary.
- [ ] 6.2 Port and run high-value `packages/app/e2e` cases (home, session, prompt).
- [ ] 6.3 Verify overall functional parity by passing all critical ported tests.

