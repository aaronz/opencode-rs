# permission.md — Permission Module

## Module Overview

- **Crate**: `opencode-permission`
- **Source**: `crates/permission/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Permission evaluation, approval queues, audit logging, and sensitive path checking.

---

## Crate Layout

```
crates/permission/src/
├── lib.rs              ← Public re-exports, all types
├── [various modules]
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.45", features = ["full"] }

opencode-core = { path = "../core" }
```

**Public exports**:
```rust
pub use crate::audit::{AuditDecision, AuditEntry, AuditLog};
pub use crate::approval_queue::{ApprovalQueue, ApprovalResult, PendingApproval};
pub use crate::evaluator::{FilePermissionResult, PermissionEvaluator};
pub use crate::scope::{AgentPermissionScope, PermissionScope};

// Functions
pub use crate::sensitive::{
    check_sensitive, get_sensitive_reason, is_external_directory,
    is_sensitive_directory, is_sensitive_path, SensitiveCheckResult,
};
pub use crate::check::{check_tool_permission, check_tool_permission_default};
```

---

## Core Types

### AgentPermissionScope

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentPermissionScope {
    None,
    ReadOnly,
    Restricted,
    Full,
}

impl AgentPermissionScope {
    pub fn from_agent_permissions(can_write: bool, can_run_commands: bool) -> Self;
    pub fn intersect(self, other: AgentPermissionScope) -> AgentPermissionScope;
    pub fn can_write_files(&self) -> bool;
    pub fn can_run_commands(&self) -> bool;
}
```

### PermissionScope

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PermissionScope {
    pub read: bool,
    pub edit: bool,
    pub bash: bool,
    pub external_directory: bool,
    pub tool_defaults: HashMap<String, PermissionAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Ask,
    Allow,
    Deny,
}
```

### ApprovalQueue

```rust
pub struct ApprovalQueue {
    scope: PermissionScope,
    pending: Arc<RwLock<Vec<PendingApproval>>>,
}

impl ApprovalQueue {
    pub fn new(scope: PermissionScope) -> Self;
    pub async fn request_approval(&self, req: ApprovalRequest) -> Result<ApprovalResult, PermissionError>;
    pub async fn get_pending(&self) -> Vec<PendingApproval>;
    pub async fn approve(&self, id: &str) -> Result<(), PermissionError>;
    pub async fn deny(&self, id: &str) -> Result<(), PermissionError>;
}

pub struct PendingApproval {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub session_id: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum ApprovalResult {
    AutoApprove,
    RequireApproval,
    Denied,
}
```

### AuditLog

```rust
pub struct AuditLog {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub decision: AuditDecision,
    pub tool: Option<String>,
    pub scope: DecisionScope,
    pub reason: Option<String>,
}

pub enum AuditDecision {
    Allow,
    Deny,
    Ask,
    Approve,
}
```

### PermissionEvaluator

```rust
pub struct PermissionEvaluator {
    scope: PermissionScope,
}

impl PermissionEvaluator {
    pub fn new(scope: PermissionScope) -> Self;
    pub fn evaluate_file_access(&self, path: &Path, operation: FileOperation) -> FilePermissionResult;
    pub fn evaluate_tool_call(&self, tool: &str, args: &serde_json::Value) -> PermissionResult;
}

pub enum FileOperation {
    Read,
    Write,
    Execute,
}

pub enum FilePermissionResult {
    Allowed,
    Denied { reason: String },
    RequiresApproval,
}
```

### Sensitive Path Checking

```rust
pub fn is_sensitive_path(path: &Path) -> bool;
pub fn is_sensitive_directory(path: &Path) -> bool;
pub fn is_external_directory(path: &Path) -> bool;
pub fn check_sensitive(path: &Path) -> SensitiveCheckResult;
pub fn get_sensitive_reason(path: &Path) -> Option<String>;

pub struct SensitiveCheckResult {
    pub is_sensitive: bool,
    pub reason: Option<String>,
    pub suggestion: Option<String>,
}
```

### Tool Permission Checking

```rust
pub fn check_tool_permission(tool: &str, scope: PermissionScope) -> ApprovalResult;
pub fn check_tool_permission_default(tool: &str) -> ApprovalResult;
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-permission` |
|---|---|
| `opencode-server` | `ApprovalQueue`, `AuditLog` in `ServerState` |
| `opencode-tools` | `AgentPermissionScope` in `ToolContext` |
| `opencode-agent` | `AgentPermissionScope` in `RuntimeConfig` |
| `opencode-plugin` | `ApprovalResult`, `PermissionScope` for plugin tool access |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_agent_permission_scope_from_write() {
        assert_eq!(AgentPermissionScope::from_agent_permissions(true, false), AgentPermissionScope::Restricted);
        assert_eq!(AgentPermissionScope::from_agent_permissions(true, true), AgentPermissionScope::Full);
        assert_eq!(AgentPermissionScope::from_agent_permissions(false, false), AgentPermissionScope::ReadOnly);
    }

    #[test]
    fn test_permission_scope_intersection() {
        let full = AgentPermissionScope::Full;
        let readonly = AgentPermissionScope::ReadOnly;
        assert_eq!(full.intersect(readonly), AgentPermissionScope::ReadOnly);
    }

    #[tokio::test]
    async fn test_approval_queue_request() {
        let queue = ApprovalQueue::new(PermissionScope::default());
        let req = ApprovalRequest { tool: "bash".into(), args: serde_json::json!({}), session_id: "s1".into() };
        let result = queue.request_approval(req).await;
        // Depends on tool and scope
    }

    #[test]
    fn test_sensitive_path_detection() {
        assert!(is_sensitive_path(Path::new("/etc/passwd")));
        assert!(!is_sensitive_path(Path::new("/home/user/project/src/main.rs")));
    }

    #[test]
    fn test_audit_log_records_decision() {
        let log = AuditLog::new();
        log.record(AuditEntry { id: Uuid::new_v4(), timestamp: Utc::now(), session_id: Some("s1".into()), decision: AuditDecision::Allow, tool: Some("read".into()), scope: DecisionScope::File, reason: None });
        assert_eq!(log.entries().len(), 1);
    }
}
```
