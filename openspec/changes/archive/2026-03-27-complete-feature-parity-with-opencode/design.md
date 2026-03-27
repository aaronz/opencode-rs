## Context

The `opencode-rs` project is a Rust implementation of the original TypeScript OpenCode project. Currently, the Rust version implements a significant portion of the core functionality, including LLM providers, basic tools, and a CLI. However, there are gaps in subcommand completeness, advanced tool features, and architectural alignment (e.g., PTY management, full MCP support) that prevent it from passing all e2e tests from the TypeScript project.

## Goals / Non-Goals

**Goals:**
- Achieve 1:1 behavioral parity for all CLI subcommands defined in `packages/opencode/src/cli/cmd`.
- Ensure all tools in `crates/tools` support the same arguments and output formats as `packages/opencode/src/tool`.
- Port and execute all remaining e2e tests from the TypeScript project using the Rust binary.
- Implement missing core protocols (ACP, MCP) to their full extent.

**Non-Goals:**
- Replicate TypeScript-specific implementation details (like the Effect framework) unless necessary for behavior.
- Optimize performance beyond the current state unless required for test passing.
- Implement features that are explicitly marked as "deprecated" or "experimental" in the target project unless they are tested.

## Decisions

1. **Subcommand Modularization**: Refactor `crates/cli/src/main.rs` into multiple modules matching the `packages/opencode/src/cli/cmd` structure to improve maintainability and parity tracking.
2. **Process Execution for Tests**: Use a common test harness in Rust that spawns the CLI binary to run against the TypeScript test fixtures.
3. **Trait-based Tool Extensions**: Use Rust traits to implement optional tool features (like context-awareness) to match the flexible tool system in TypeScript.
4. **Shared Schema Definition**: Create a shared `opencode-schema` crate if needed to ensure JSON compatibility for ACP/MCP protocols.

## Risks / Trade-offs

- **PTY Platform Differences**: Implementing PTY management in Rust for both Linux and macOS might have edge cases that differ from the `node-pty` behavior used in TS.
- **Dependency Version Drift**: Keeping Rust dependencies (like LLM SDKs) in sync with TS versions to ensure identical behavior.
- **Complexity of Effect System Porting**: Porting logic that heavily relies on the TypeScript Effect system might be error-prone when translated to Rust's async/await.
