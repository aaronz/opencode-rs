## Context

The rust-opencode-port project is a Rust reimplementation of the TypeScript opencode project. To ensure feature parity, we need comprehensive e2e tests. The target project has 100+ test files using Bun/test framework.

## Goals / Non-Goals

**Goals:**
1. Create test infrastructure compatible with Rust's cargo test
2. Port key e2e tests from TypeScript to Rust
3. Verify functionality matches target implementation

**Non-Goals:**
- Porting ALL 100+ test files (focus on core functionality)
- Maintaining exact TypeScript test syntax (use Rust idioms)
- Testing internal implementation details

## Decisions

### Decision 1: Test Framework
Use Rust's built-in `#[test]` attribute with `tokio` for async tests. This matches the existing Rust codebase architecture.

### Decision 2: Test Structure
Organize tests in `tests/` directory per crate, mirroring the TypeScript structure:
- `crates/tools/tests/` - Tool tests
- `crates/core/tests/` - Core functionality tests
- `crates/agent/tests/` - Agent tests

### Decision 3: Fixture Management
Create a `test fixtures` module for temporary directories, similar to TypeScript's `test/fixture/fixture.ts`.

## Risks / Trade-offs

- **Risk**: Some TypeScript tests rely on Bun-specific APIs
  - **Mitigation**: Rewrite using standard Rust/std::process

- **Risk**: Test coverage may be incomplete initially
  - **Mitigation**: Prioritize critical paths (tool execution, session management)
