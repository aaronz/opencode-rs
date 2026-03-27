## 1. Advanced Tool Completion

- [ ] 1.1 Refine the `multiedit` tool in `crates/tools` to support atomic multi-file edits and rollback.
- [ ] 1.2 Implement the `truncation_dir` tool for large directory management and summary generation.
- [ ] 1.3 Implement strict JSON schema validation for all tool arguments to ensure TS compatibility.
- [ ] 1.4 Refine the `apply_patch` tool to match the target project's edge-case handling.

## 2. TUI Parity & Rich UI

- [ ] 2.1 Implement the Session Timeline view in `crates/tui` with history navigation.
- [ ] 2.2 Implement the Fork Session dialog and associated state logic in the TUI.
- [ ] 2.3 Implement dynamic theme loading from JSON files and match the TS app's visual style.
- [ ] 2.4 Add message metadata browsing (tokens, duration) to the TUI message view.

## 3. Core & Storage Logic Synchronization

- [ ] 3.1 Implement token-aware compaction logic in `crates/core` matching the TS behavior.
- [ ] 3.2 Add paginated history retrieval (`limit`/`offset`) to the `StorageService` in `crates/storage`.
- [ ] 3.3 Ensure the `EventBus` supports all internal message types and event parity with the TS version.
- [ ] 3.4 Integrate auto-compaction into the agent's prompt generation lifecycle.

## 4. Full Protocol Support (ACP/MCP)

- [ ] 4.1 Implement real-time event streaming for ACP agents in `crates/control-plane`.
- [ ] 4.2 Complete full resource discovery and schema validation for the MCP implementation.
- [ ] 4.3 Finalize functional logic for modularized commands: `upgrade`, `uninstall`, and `acp`.
- [ ] 4.4 Ensure all relevant CLI commands support a uniform `--json` output flag.

## 5. End-to-End Testing Harness

- [ ] 5.1 Develop the Rust process-based test harness for spawning the CLI binary.
- [ ] 5.2 Port and implement core flow test fixtures (session management, model connect) from TS e2e.
- [ ] 5.3 Implement e2e tests for tool execution and error handling parity.
- [ ] 5.4 Verify functional parity by passing all critical ported e2e tests.
