## Why

The Rust port of OpenCode (`opencode-rs`) aims for full feature parity with the original TypeScript implementation. A detailed gap analysis reveals that while the CLI subcommand surface has high parity, there is a significant discrepancy in test coverage and certain core logic areas (memory usage tracking, bus-effect integration, and provider metadata). Achieving full parity is essential for ensuring that the Rust implementation is as reliable and functionally equivalent as the TypeScript version, allowing it to pass the target project's extensive end-to-end test suite.

## What Changes

1. **Test Porting and Alignment**: Port high-priority TypeScript tests to Rust, specifically those marked as "Not implemented" in `TEST_MAPPING.md`. This includes tests for:
    - Memory leak detection and abort handling.
    - Bus and effect system interactions.
    - Session compaction integration.
    - Tool registry and advanced tool behaviors (e.g., `webfetch`, `apply_patch`).
    - Provider-specific features and transformations.
2. **Logic Refinement**:
    - **Memory Tracking**: Implement explicit memory-use tracking and abort-leak prevention logic in `crates/core` to match the behavior exercised by TS tests.
    - **Provider Metadata**: Enhance `crates/llm` providers to return comprehensive token usage and provider-specific options.
    - **Advanced Tools**: Implement missing logic for complex tools like `multi-edit` and `truncation-dir`.
3. **PTY and Interactive Shell**: Finalize PTY session management to support interactive shell features.
4. **Protocol Completion**: Complete any remaining edge cases for ACP and MCP protocol implementations to ensure full compatibility with remote agents and servers.

## Capabilities

### New Capabilities
- `memory-usage-monitoring`: Logic for tracking and limiting memory usage within sessions and agents.
- `advanced-tool-orchestration`: Support for complex, multi-file tool operations like `multi-edit`.

### Modified Capabilities
- `cli-commands`: refinement of existing subcommands to ensure 1:1 behavioral alignment with TS counterparts.
- `llm-provider-metadata`: Enhanced reporting of token usage and metadata across all providers.
- `test-suite-rust`: A significantly expanded Rust test suite that mirrors the intent and coverage of the TypeScript project.

## Impact

- `rust-opencode-port/crates/core/`: Logic updates for memory management and effect systems.
- `rust-opencode-port/crates/tools/`: implementation of advanced tools.
- `rust-opencode-port/crates/llm/`: Provider metadata enhancements.
- `rust-opencode-port/crates/cli/`: Refactoring for better modularity and parity.
- `rust-opencode-port/TEST_MAPPING.md`: Status updates for ported tests.
