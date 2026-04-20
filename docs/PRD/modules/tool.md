# tool.md — Tools Module

## Module Overview

- **Crate**: `opencode-tools`
- **Source**: `crates/tools/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Defines the `Tool` trait, `ToolContext`, `ToolResult`, `ToolRegistry` for dynamic tool registration/execution, and implements ~30 concrete tools (read, write, edit, grep, glob, bash, git, lsp, web search, etc.).

---

## Crate Layout

```
crates/tools/src/
├── lib.rs              ← Re-exports, module declarations
├── tool.rs             ← Tool trait, ToolResult, ToolContext
├── registry.rs         ← ToolRegistry (async, RwLock-based, priority)
├── discovery.rs        ← build_default_registry, register_custom_tools
├── schema_validation.rs ← ToolSchema validation
├── codesearch.rs       ← CodeSearchTool
├── multiedit.rs        ← MultiEditTool
├── truncation_dir.rs   ← TruncationDirTool
├── bash.rs             ← BashTool
├── read.rs             ← ReadTool
├── write.rs            ← WriteTool
├── edit.rs             ← EditTool
├── glob.rs             ← GlobTool
├── grep_tool.rs        ← GrepTool
├── lsp_tool.rs         ← LspTool
├── webfetch.rs         ← WebFetchTool
├── web_search.rs       ← WebSearchTool
├── skill.rs            ← SkillTool
├── todowrite.rs        ← TodowriteTool
├── task.rs             ← TaskTool
├── question.rs         ← QuestionTool
├── plan.rs             ← PlanExitTool
├── plan_exit.rs        ← PlanExitTool (duplicate/alias)
├── truncate.rs         ← TruncateTool
├── git_tools.rs        ← GitTools (commit, diff, status, etc.)
├── external_directory.rs ← ExternalDirectoryTool
├── ls.rs               ← LsTool
├── file_tools.rs       ← file-related tools
├── apply_patch.rs      ← ApplyPatchTool
├── batch.rs            ← BatchTool
├── invalid.rs          ← InvalidTool (error tool)
├── grep_tool_test.rs
├── lsp_tool_test.rs
├── read_test.rs
├── session_tools.rs / session_tools_test.rs
├── skill_test.rs
├── write_test.rs
└── formatter_hook.rs
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.45", features = ["full"] }
regex = "1"
reqwest = { version = "0.12" }

opencode-core = { path = "../core" }
opencode-permission = { path = "../permission" }
```

**Public exports from lib.rs**:
```rust
pub use codesearch::CodeSearchTool;
pub use discovery::{build_default_registry, register_custom_tools};
pub use multiedit::MultiEditTool;
pub use registry::{ToolRegistry, ToolSource};
pub use schema_validation::ToolSchema;
pub use tool::sealed;
pub use tool::{Tool, ToolContext, ToolResult};
pub use truncation_dir::TruncationDirTool;
```

---

## Core Types

### ToolResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn ok(content: impl Into<String>) -> Self { ... }
    pub fn err(error: impl Into<String>) -> Self { ... }
    pub fn with_title(mut self, title: impl Into<String>) -> Self { ... }
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self { ... }
}
```

### ToolContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolContext {
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub worktree: Option<String>,
    pub directory: Option<String>,
    pub permission_scope: Option<opencode_permission::AgentPermissionScope>,
}

impl ToolContext {
    pub fn with_permission_scope(
        mut self,
        scope: opencode_permission::AgentPermissionScope,
    ) -> Self { ... }
}
```

### Tool Trait

```rust
pub mod sealed {
    pub trait Sealed {}
}

#[async_trait]
pub trait Tool: Send + Sync + sealed::Sealed {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn clone_tool(&self) -> Box<dyn Tool>;
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError>;

    fn is_safe(&self) -> bool { false }
    fn get_dependencies(&self, _args: &serde_json::Value) -> HashSet<PathBuf> { HashSet::new() }
}
```

---

## ToolRegistry

```rust
// From registry.rs
pub struct ToolRegistry { ... }  // async, RwLock, priority-based dispatch

impl ToolRegistry {
    pub fn new() -> Self;
    pub async fn register(&self, tool: Box<dyn Tool>) -> Result<(), ToolError>;
    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, ToolError>;
    pub async fn get_tool(&self, name: &str) -> Option<Box<dyn Tool>>;
    pub async fn list_tools(&self) -> Vec<String>;
    pub async fn unregister(&self, name: &str) -> Option<Box<dyn Tool>>;
}

pub enum ToolSource {
    BuiltIn,
    Plugin(String),
    Custom,
}
```

---

## Discovery / Default Registry

```rust
// From discovery.rs
pub fn build_default_registry() -> ToolRegistry { ... }
pub fn register_custom_tools(registry: &ToolRegistry) { ... }
```

`build_default_registry()` registers all built-in tools:
- `ReadTool` (name: "read")
- `WriteTool` (name: "write")
- `EditTool` (name: "edit")
- `GlobTool` (name: "glob")
- `GrepTool` (name: "grep")
- `BashTool` (name: "bash")
- `LsTool` (name: "list" / "ls")
- `LspTool` (name: "lsp")
- `WebFetchTool` (name: "webfetch")
- `WebSearchTool` (name: "websearch")
- `SkillTool` (name: "skill")
- `TodowriteTool` (name: "todowrite")
- `TaskTool` (name: "task")
- `QuestionTool` (name: "question")
- `PlanExitTool` (name: "plan_exit")
- `TruncateTool` (name: "truncate")
- `MultiEditTool` (name: "multiedit")
- `CodeSearchTool` (name: "codesearch")
- `TruncationDirTool` (name: "truncation_dir")
- `ApplyPatchTool` (name: "apply_patch")
- `ExternalDirectoryTool` (name: "external_directory")
- `GitTools` (multiple names: "git_commit", "git_diff", "git_status", "git_log", etc.)
- `InvalidTool` (name: "invalid")
- Batch tool(s)
- File tools

---

## Key Tool Implementations

### ReadTool
```rust
pub struct ReadTool;
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read file contents" }
    fn is_safe(&self) -> bool { true }
    async fn execute(&self, args: serde_json::Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError>;
}
```

### BashTool
```rust
pub struct BashTool;
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Execute shell commands" }
    fn is_safe(&self) -> bool { false }  // requires permission check
    async fn execute(&self, args: serde_json::Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError>;
}
```

### GrepTool
```rust
pub struct GrepTool;
impl Tool for GrepTool { ... }
```

### LspTool
```rust
pub struct LspTool;
impl Tool for LspTool { ... }
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-tools` |
|---|---|
| `opencode-agent` | `ToolRegistry`, `ToolContext` to invoke tools in agent loop |
| `opencode-server` | `ToolRegistry` for request handling |
| `opencode-tui` | Tool invocation from UI |
| `opencode-plugin` | `ToolRegistry` + `PluginToolAdapter` to register plugin tools |

**Dependencies of `opencode-tools`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `OpenCodeError`, `Session` |
| `opencode-permission` | `AgentPermissionScope` for permission checks |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_result_ok() {
        let result = ToolResult::ok("file contents");
        assert!(result.success);
        assert_eq!(result.content, "file contents");
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_tool_result_err() {
        let result = ToolResult::err("permission denied");
        assert!(!result.success);
        assert!(result.content.is_empty());
        assert_eq!(result.error, Some("permission denied".to_string()));
    }

    #[tokio::test]
    async fn test_tool_result_with_title() {
        let result = ToolResult::ok("content").with_title("File: README.md");
        assert_eq!(result.title, Some("File: README.md".to_string()));
    }

    #[tokio::test]
    async fn test_tool_context_default() {
        let ctx = ToolContext::default();
        assert!(ctx.session_id.is_empty());
        assert!(ctx.permission_scope.is_none());
    }

    #[tokio::test]
    async fn test_tool_context_with_permission_scope() {
        let ctx = ToolContext::default()
            .with_permission_scope(AgentPermissionScope::ReadOnly);
        assert_eq!(ctx.permission_scope, Some(AgentPermissionScope::ReadOnly));
    }

    #[tokio::test]
    async fn test_registry_execute_built_in_tool() {
        let registry = build_default_registry();
        let result = registry.execute(
            "read",
            serde_json::json!({"path": "/tmp/test.txt"}),
            None,
        ).await;
        // Result depends on whether file exists
    }

    #[tokio::test]
    async fn test_registry_execute_unknown_tool() {
        let registry = build_default_registry();
        let result = registry.execute("nonexistent_tool", serde_json::json!({}), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_list_tools() {
        let registry = build_default_registry();
        let tools = registry.list_tools().await;
        assert!(tools.contains(&"read".to_string()));
        assert!(tools.contains(&"write".to_string()));
        assert!(tools.contains(&"bash".to_string()));
    }

    #[tokio::test]
    async fn test_registry_register_and_execute() {
        let registry = ToolRegistry::new();
        registry.register(Box::new(ReadTool)).await.unwrap();
        let tools = registry.list_tools().await;
        assert!(tools.contains(&"read".to_string()));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::ok("content").with_title("title");
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"title\":\"title\""));
    }

    #[test]
    fn test_tool_result_deserialization() {
        let json = r#"{"success": false, "content": "", "error": "failed"}"#;
        let result: ToolResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert_eq!(result.error, Some("failed".to_string()));
    }
}
```

---

## Usage Example

```rust
use opencode_tools::{ToolRegistry, build_default_registry, ToolContext, ToolResult};
use opencode_permission::AgentPermissionScope;

async fn execute_tool() -> Result<ToolResult, ToolError> {
    let registry = build_default_registry();
    
    let ctx = ToolContext {
        session_id: "session-123".to_string(),
        message_id: "msg-456".to_string(),
        agent: "build".to_string(),
        worktree: Some("/path/to/project".to_string()),
        directory: Some("/path/to/project".to_string()),
        permission_scope: Some(AgentPermissionScope::Full),
    };
    
    let result = registry
        .execute("read", serde_json::json!({"path": "/path/to/file.txt"}), Some(ctx))
        .await?;
    
    println!("Tool result: {}", result.content);
    Ok(result)
}
```
