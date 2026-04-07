# OpenCode-RS Constitution

**Version**: 1.0  
**Date**: 2026-04-07  
**Status**: Active

---

## Preamble

This document serves as the master constitution for the OpenCode-RS project, establishing fundamental design principles, architectural decisions, and compliance requirements that all contributors must follow.

## Article 1: Source of Authority

This constitution is derived from and supersedes all previous iteration-specific design documents. Individual constitution documents provide detailed specifications for their respective domains.

### Section 1.1: Constitution Hierarchy

| Level | Document | Authority |
|-------|----------|-----------|
| Master | `outputs/.specify/memory/constitution.md` | This file |
| Domain | `outputs/iteration-16/constitution/C-0*.md` | Detailed specs |

### Section 1.2: Incorporation by Reference

The following constitution documents are fully incorporated herein:

| ID | Title | Description |
|----|-------|-------------|
| C-024 | Session Tools Permission | `session_load`/`session_save` permission model |
| C-055 | Test Coverage Requirements | ≥70% coverage targets, TEST_MAPPING.md |
| C-056 | Config JSONC Migration | TOML→JSONC migration, deprecation timeline |
| C-057 | Server API Endpoints | Health, abort, permission reply endpoints |
| C-058 | LSP Capabilities | JSON-RPC protocol, language server detection |

## Article 2: Foundational Principles

### Section 2.1: Code Quality

1. **Type Safety First**: Never suppress type errors with `as any`, `@ts-ignore`, `@ts-expect-error`, or `unsafe`
2. **Error Handling**: Never use empty catch blocks `catch(e) {}`
3. **Testing**: Never delete failing tests to "pass" - fix the underlying issue
4. **Documentation**: Public APIs MUST have doc comments; internal code should be self-documenting

### Section 2.2: Architectural Boundaries

| Boundary | Principle |
|----------|-----------|
| Core ↔ Tools | Core is dependency-free; Tools depend on Core |
| Server ↔ Agent | Server handles HTTP; Agent handles execution |
| Permission | Separate crate (`opencode-permission`) with clear API |
| Storage | Abstracted behind `StorageService` trait |

### Section 2.3: Permission Model

All tools are classified into three categories:

| Category | Auto-Approve | Examples |
|----------|--------------|----------|
| Read | `ReadOnly` scope | `read`, `grep`, `session_load` |
| Safe | `Restricted` scope | `glob`, `ls` |
| Write | `Full` scope | `write`, `bash`, `session_save` |

## Article 3: Implementation Standards

### Section 3.1: Session Tools

**Reference**: C-024

```rust
async fn execute(&self, args: Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
    let permission_check = check_tool_permission_default(self.name());
    if permission_check != ApprovalResult::AutoApprove {
        return Ok(ToolResult::err("Permission denied"));
    }
    // ... implementation
}
```

### Section 3.2: Server Endpoints

**Reference**: C-057

All `/api/*` endpoints:
- Require `x-api-key` header authentication
- Return JSON error responses via `json_error()` helper
- Include `#[actix_web::test]` unit tests

Health endpoint (`/health`):
- Public, no authentication
- Returns `{ "status": "ok", "version": "x.y.z" }`

### Section 3.3: Configuration

**Reference**: C-056

| Priority | Format | Status |
|----------|--------|--------|
| 1 | `.opencode/config.jsonc` | Preferred |
| 2 | `.opencode/config.json` | Supported |
| 3 | `.opencode/config.toml` | **Deprecated** |

### Section 3.4: LSP Integration

**Reference**: C-058

Two LSP modes:
1. **Server Mode**: OpenCode-RS serves LSP to editors (tower_lsp)
2. **Client Mode**: OpenCode-RS spawns external servers (rust-analyzer, tsserver)

All JSON-RPC messages use Content-Length headers.

## Article 4: Testing Requirements

### Section 4.1: Coverage Targets

**Reference**: C-055

| Crate | Minimum Coverage |
|-------|----------------|
| `opencode-core` | 70% |
| `opencode-server` | 60% |
| `opencode-tools` | 60% |
| `opencode-permission` | 70% |
| `opencode-storage` | 60% |
| `opencode-llm` | 50% |

### Section 4.2: Test Mapping

`TEST_MAPPING.md` tracks correspondence between TypeScript e2e tests and Rust equivalents. Maintain this file when adding tests.

### Section 4.3: Required Test Categories

1. **Unit Tests**: In `#[cfg(test)]` modules next to implementation
2. **API Tests**: In `crates/*/src/*_test.rs` files
3. **Integration Tests**: In `tests/src/` directory

## Article 5: Development Workflow

### Section 5.1: Before Submitting PR

- [ ] `cargo test` passes (all crates)
- [ ] `cargo check` passes (no warnings in changed files)
- [ ] New tests added for new functionality
- [ ] TEST_MAPPING.md updated
- [ ] Constitution documents updated if applicable

### Section 5.2: Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

### Section 5.3: Branch Naming

```
<type>/<issue>-<description>
```

Examples:
- `feat/session-tools-permission`
- `fix/health-endpoint`
- `docs/api-constitution`

## Article 6: Technical Debt Management

### Section 6.1: Known Technical Debt

| ID | Description | Risk | Status |
|----|-------------|------|--------|
| T1 | Config format inconsistency | High | Mitigated by C-056 |
| T2 | README outdated | Medium | Pending |
| T3 | Permission routing unclear | Medium | Mitigated by C-024 |
| T4 | TUI permission confirmation | Low | Accepted |
| T5 | auth_layered not integrated | Low | Pending |

### Section 6.2: Deprecation Policy

| Item | Deprecated In | Removed In |
|------|--------------|-----------|
| TOML config | v1.0 | v2.0 |
| Legacy session format | v0.9 | v1.5 |

## Article 7: Amendments

### Section 7.1: Amendment Process

1. Create RFC document in `outputs/rfcs/`
2. Require approval from 2+ senior maintainers
3. Update master constitution and relevant C-XXX documents
4. Announce change in project communication channel

### Section 7.2: Emergency Amendments

Critical security or safety issues may be fixed immediately with notification to maintainers within 48 hours.

## Article 8: Historical Documents

| Document | Description | Status |
|----------|-------------|--------|
| `outputs/iteration-13/` | v13 implementation spec | Superseded |
| `outputs/iteration-14/` | v14 implementation spec | Superseded |
| `outputs/iteration-15/` | v15 implementation spec | Superseded |
| `outputs/iteration-16/` | v16 implementation spec | **Current** |

---

**Ratified**: 2026-04-07  
**Effective**: Immediately upon publication  
**Review**: Annually or upon major version release  
**Amendments**: Requires RFC process per Article 7

---

## Appendix A: Quick Reference

### Permission Check Template

```rust
use opencode_permission::{check_tool_permission_default, ApprovalResult};

async fn my_tool_execute(...) -> Result<ToolResult, OpenCodeError> {
    if check_tool_permission_default("my_tool") != ApprovalResult::AutoApprove {
        return Ok(ToolResult::err("Permission denied"));
    }
    // ... implementation
}
```

### Health Check Implementation

```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

### API Error Response

```rust
json_error(StatusCode::NOT_FOUND, "session_not_found", "Session does not exist")
```
