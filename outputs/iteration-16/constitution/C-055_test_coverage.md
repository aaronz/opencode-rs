# Constitution C-055: Test Coverage Requirements

**Version**: 1.0  
**Date**: 2026-04-07  
**Iteration**: v16  
**Status**: Adopted

---

## Preamble

This Constitution establishes minimum test coverage requirements for the OpenCode-RS project to ensure reliability and enable safe refactoring.

## Article 1: Coverage Targets

### Section 1.1: Overall Targets

| Metric | Target | Current |
|--------|--------|---------|
| Overall Coverage | ≥ 70% | ~50% |
| Core Crates | ≥ 70% | Variable |
| Server Crate | ≥ 60% | ~40% |
| LLM Crate | ≥ 50% | ~30% |
| Tools Crate | ≥ 60% | ~50% |

### Section 1.2: Per-Crate Requirements

| Crate | Min Coverage | Critical Paths |
|-------|--------------|---------------|
| `opencode-core` | 70% | Session, Message, Config, Bus |
| `opencode-server` | 60% | API Endpoints, Route Handlers |
| `opencode-tools` | 60% | Tool Execution, Permission Checks |
| `opencode-llm` | 50% | Provider Logic, Streaming |
| `opencode-permission` | 70% | Evaluation, Queue |
| `opencode-storage` | 60% | Session Persistence |

## Article 2: Test Categories

### Section 2.1: Unit Tests (`#[cfg(test)]`)

Located in same file as implementation:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_operation() { }
}
```

**Coverage Target**: 80% of all functions

### Section 2.2: Integration Tests (`tests/` directory)

Located in `tests/src/`:
- `session_storage_tests.rs`: Session persistence
- `agent_tool_tests.rs`: Tool execution
- `agent_llm_tests.rs`: LLM provider integration

**Coverage Target**: 50% of all modules

### Section 2.3: API Endpoint Tests

**Required Tests**:
```rust
#[actix_web::test]
async fn test_endpoint_success() { }
#[actix_web::test]
async fn test_endpoint_error_case() { }
```

**Coverage Target**: 90% of all endpoints

## Article 3: Test Quality Standards

### Section 3.1: Naming Conventions

Tests MUST be named descriptively:
```rust
// ✅ Good
#[test]
fn test_session_load_returns_error_for_nonexistent_uuid() { }

// ❌ Bad
#[test]
fn test_load() { }
```

### Section 3.2: Assertion Quality

Each test MUST have at least one meaningful assertion:
```rust
// ✅ Good
assert_eq!(result.len(), expected_count);
assert!(result.contains("expected_content"));

// ❌ Bad
assert!(result.is_ok());  // Only checks not error
```

### Section 3.3: Test Isolation

- Tests MUST NOT depend on execution order
- Tests MUST NOT share mutable state
- Tests MUST clean up after themselves (temp files, etc.)

## Article 4: TEST_MAPPING.md

### Section 4.1: Purpose

`TEST_MAPPING.md` tracks the correspondence between TypeScript e2e tests and Rust equivalents.

### Section 4.2: Format

```markdown
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/session/session.test.ts` | `crates/core/src/session.rs` | ✅ Covered |
| `test/tool/grep.test.ts` | `crates/tools/src/grep_tool_test.rs` | ✅ Covered |
```

### Section 4.3: Maintenance

After adding new tests:
1. Update TEST_MAPPING.md with new entries
2. Mark status as "✅ Covered"
3. Document any gaps in issue tracker

## Article 5: Missing Tests (Gap Analysis)

### Section 5.1: Current Gaps

From TEST_MAPPING.md v16:

| Category | Missing | Priority |
|----------|---------|----------|
| Session | 8 tests | High |
| Server | 3 tests | High |
| LSP | 2 tests | Medium |
| Provider | 5 tests | Medium |
| Config | 2 tests | Low |

### Section 5.2: High Priority Tests

1. **Session retry logic** - Test exponential backoff
2. **Session compaction** - Test auto-trigger conditions
3. **Server abort endpoint** - Test session abortion
4. **LSP client communication** - Test JSON-RPC protocol

## Article 6: Test Execution

### Section 6.1: Required Test Commands

```bash
# All tests
cargo test

# Crate-specific
cargo test -p opencode-core
cargo test -p opencode-server
cargo test -p opencode-tools

# With coverage
cargo tarpaulin --output-dir coverage/
```

### Section 6.2: CI Requirements

| Check | Requirement |
|-------|-------------|
| Unit Tests | All must pass |
| Integration Tests | All must pass |
| Coverage | Must meet targets |

## Article 7: Adoption

This Constitution is effective immediately upon merge to main branch.

---

**Ratified**: 2026-04-07  
**Expires**: Never  
**Amendments**: Requires RFC process
