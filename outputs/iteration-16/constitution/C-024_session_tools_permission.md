# Constitution C-024: Session Tools Permission System

**Version**: 1.0  
**Date**: 2026-04-07  
**Iteration**: v16  
**Status**: Adopted

---

## Preamble

This Constitution documents the design decisions and implementation requirements for the `session_load` and `session_save` tools within the OpenCode-RS permission framework.

## Article 1: Scope and Purpose

The session tools provide programmatic access to session storage, enabling:
- Loading session state from persistent storage
- Saving session state to persistent storage
- Integration with the existing permission evaluation system

## Article 2: Permission Model

### Section 2.1: Permission Classification

All tools are classified into three categories based on their potential impact:

| Category | Description | Auto-Approve Scope |
|----------|-------------|-------------------|
| Read Tools | Tools that only read data without modification | `ReadOnly` |
| Safe Tools | Tools with limited side effects | `Restricted` |
| Write Tools | Tools that modify state or execute commands | `Full` |

### Section 2.2: Session Tool Classification

| Tool | Classification | Rationale |
|------|-----------------|-----------|
| `session_load` | **Read Tool** | Only reads session data from storage |
| `session_save` | **Write Tool** | Persists modified session state to storage |

## Article 3: Implementation Requirements

### Section 3.1: Required Permission Checks

All session tools MUST call `check_tool_permission_default(tool_name)` before execution.

```rust
async fn execute(&self, args: Value, _ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
    let permission_check = check_tool_permission_default(self.name());
    if permission_check != ApprovalResult::AutoApprove {
        return Ok(ToolResult::err("Permission denied: requires approval in current scope"));
    }
    // ... tool implementation
}
```

### Section 3.2: Error Handling

When permission is denied:
- Return `ToolResult::err(...)` with descriptive message
- Do NOT return `Err(OpenCodeError::...)` for permission denials
- Log the denial for audit purposes

### Section 3.3: Context Handling

Tools MUST accept `Option<ToolContext>` parameter for future extensibility, even if currently unused.

## Article 4: Integration Points

### Section 4.1: Permission Crate

Location: `crates/permission/`

Exports:
- `check_tool_permission(tool_name: &str, scope: PermissionScope) -> ApprovalResult`
- `check_tool_permission_default(tool_name: &str) -> ApprovalResult`
- `ApprovalResult::AutoApprove | ApprovalResult::RequestApproval | ApprovalResult::Denied`

### Section 4.2: Session Tools Crate

Location: `crates/tools/src/session_tools.rs`

Dependencies:
- `opencode-permission` for permission checks
- `opencode-core` for Session model

## Article 5: Testing Requirements

### Section 5.1: Required Tests

1. **Permission Denied Test**: Verify `ToolResult::err` on permission denial
2. **Invalid UUID Test**: Verify proper error on malformed session ID
3. **Missing Session ID Test**: Verify `Parse` error when session_id absent
4. **Successful Load Test**: Verify session loading when permission granted
5. **Tool Metadata Test**: Verify `name()` and `description()` return correct values

### Section 5.2: Test Classification

Tests MUST verify behavior under both approved and denied permission scopes.

## Article 6: Adoption

This Constitution is effective immediately upon merge to main branch.

---

**Ratified**: 2026-04-07  
**Expires**: Never  
**Amendments**: Requires RFC process
