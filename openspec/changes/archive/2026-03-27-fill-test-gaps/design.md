## Context

The mycode project at `/Users/aaronzh/Documents/GitHub/mycode` contains a Rust port of the TypeScript opencode project at `/Users/aaronzh/Documents/GitHub/opencode`. The Rust port (`rust-opencode-port/`) has:
- Basic CLI with stub implementations
- 172 unit tests passing
- TUI module that exists but isn't fully connected
- LLM providers that attempt API calls but have limited functionality

The target project has 100+ e2e tests in TypeScript using Bun/Effect framework that cannot be run against the Rust implementation.

## Goals / Non-Goals

**Goals:**
1. Achieve feature parity between Rust port and TypeScript target
2. Enable the TypeScript e2e tests to pass against the Rust implementation
3. Complete stub implementations with real functionality
4. Implement async LLM streaming with proper error handling

**Non-Goals:**
1. Convert the entire Rust codebase to TypeScript
2. Match internal implementation details - only match external behavior
3. Achieve 100% test coverage - focus on critical path tests

## Decisions

1. **Test Strategy**: Instead of porting all TypeScript tests to Rust, create a test runner that can execute TypeScript tests against the Rust CLI using process spawning
2. **LLM Integration**: Use async trait for provider implementations to support full streaming
3. **Tool System**: Implement tool execution via command pattern with async runtime
4. **Session Storage**: Continue using JSON file storage (matching target behavior)

## Risks / Trade-offs

- **Risk**: TypeScript e2e tests use Effect framework which doesn't map to Rust → **Mitigation**: Create adapter layer that runs tests against CLI process
- **Risk**: Async streaming in Rust is complex → **Mitigation**: Use tokio with async-trait for clean provider interface
- **Risk**: Many test assertions depend on internal state → **Mitigation**: Expose necessary state via CLI JSON output flags

## Migration Plan

1. First: Complete LLM provider implementations with real streaming
2. Second: Implement tool execution system with all tools
3. Third: Connect agent system to tools and LLM
4. Fourth: Add JSON output flags to CLI for test compatibility
5. Fifth: Run target e2e tests against Rust CLI

## Open Questions

- Should we attempt to run tests inline or via subprocess?
- How to handle tests that require specific file system state?