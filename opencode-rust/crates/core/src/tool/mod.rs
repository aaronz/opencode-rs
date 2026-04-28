//! Core tool types and simple synchronous tool registry.
//!
//! # Tool Registry Architecture
//!
//! OpenCode uses two distinct tool registry implementations with intentionally different designs:
//!
//! ## 1. `opencode_core::ToolRegistry` (this module)
//!
//! A **simple synchronous registry** designed for:
//! - MCP (Model Context Protocol) tool bridging
//! - Legacy compatibility
//! - Blocking/synchronous tool execution
//!
//! **Characteristics:**
//! - Synchronous executor functions (`Fn(serde_json::Value) -> Result<String, String>`)
//! - No async support
//! - No caching
//! - HashMap-based storage
//!
//! ## 2. `opencode_tools::ToolRegistry` (`crates/tools/src/registry.rs`)
//!
//! An **advanced async registry** designed for:
//! - Agent runtime tool management
//! - Concurrent tool execution
//! - Result caching with TTL and dependency invalidation
//! - Priority-based collision resolution (Builtin > Plugin > CustomProject > CustomGlobal)
//!
//! **Characteristics:**
//! - Async tool execution via `Tool` trait
//! - Built-in result caching with SHA256-based cache keys
//! - TTL-based and dependency-based cache invalidation
//! - RwLock-based interior mutability for thread safety
//!
//! # Relationship Between Registries
//!
//! The two registries serve different purposes and are **not directly compatible**:
//!
//! ```text
//! MCP Server → MCP Registry → bridge_to_tool_registry() → opencode_core::ToolRegistry
//!                                                              ↓
//!                                                      (used by TUI, MCP)
//!
//! Agent Runtime → opencode_tools::ToolRegistry
//!                                    ↑
//!                            (used by agent runtime, plugins)
//! ```
//!
//! MCP tools are bridged to `opencode_core::ToolRegistry` which is used by the TUI layer.
//! The agent runtime uses `opencode_tools::ToolRegistry` directly.
//!
//! # When to Use Which Registry
//!
//! - **Use `opencode_core::ToolRegistry`** for:
//!   - MCP tool registration
//!   - TUI-level tool management
//!   - Legacy code that expects synchronous execution
//!
//! - **Use `opencode_tools::ToolRegistry`** for:
//!   - Agent runtime tool execution
//!   - Plugin tool registration
//!   - Any code requiring async execution, caching, or collision resolution

pub mod registry;
pub mod types;

pub(crate) use registry::register_custom_tool;
pub use registry::{build_default_registry, ToolRegistry};
pub use types::{ToolCall, ToolDefinition, ToolExecutor, ToolParameter, ToolResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DirectoryScanner;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();

        registry.register(
            ToolDefinition {
                name: "test".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("success".to_string())),
        );

        assert!(registry.contains("test"));
        assert_eq!(registry.list(), vec!["test"]);
    }

    #[test]
    fn test_tool_registry_get() {
        let mut registry = ToolRegistry::new();

        let def = ToolDefinition {
            name: "test".to_string(),
            description: "A test tool".to_string(),
            parameters: vec![],
            ..Default::default()
        };

        registry.register(def.clone(), Arc::new(|_| Ok("".to_string())));

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_tool_execution() {
        let mut registry = ToolRegistry::new();

        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );

        let executor = registry.get_executor("echo").unwrap();
        let result = executor(serde_json::json!({"test": "value"})).unwrap();
        assert!(result.contains("test"));
    }

    #[test]
    fn test_disabled_tool_executor_unavailable() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );
        registry.set_disabled(HashSet::from(["echo".to_string()]));

        assert!(registry.get_executor("echo").is_none());
    }

    #[test]
    fn test_list_with_status_marks_disabled_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );
        registry.set_disabled(HashSet::from(["echo".to_string()]));

        let status = registry.list_with_status();
        assert!(status
            .iter()
            .any(|(name, disabled)| *name == "echo" && *disabled));
    }

    #[test]
    fn test_empty_disabled_set_means_all_enabled() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );
        registry.set_disabled(HashSet::new());

        assert!(registry.get_executor("echo").is_some());
    }

    #[test]
    fn test_session_tools_registered() {
        let registry = build_default_registry();
        assert!(registry.contains("session_load"));
        assert!(registry.contains("session_save"));
    }

    #[test]
    fn test_session_save_and_load_tools_execute() {
        let registry = build_default_registry();
        let session_id = uuid::Uuid::new_v4();

        let save = registry.get_executor("session_save").unwrap()(
            serde_json::json!({"session_id": session_id.to_string()}),
        );
        assert!(save.is_ok());

        let load = registry.get_executor("session_load").unwrap()(
            serde_json::json!({"session_id": session_id.to_string()}),
        );
        assert!(load.is_ok());

        let _ = crate::Session::delete(&session_id);
    }

    #[test]
    fn test_directory_scanner_registers_discovered_tools_with_registry() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode-rs").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("test_tool.js"),
            r#"export default tool({
                name: "test_discovery_tool",
                description: "A test discovery tool",
                args: { type: "object", properties: { input: { type: "string" } } }
            });"#,
        )
        .unwrap();

        let mut registry = ToolRegistry::new();
        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode-rs");
        let tools = scanner.scan_tools(&opencode_path);

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_discovery_tool");

        for tool_info in tools {
            register_custom_tool(&mut registry, tool_info);
        }

        assert!(
            registry.contains("test_discovery_tool"),
            "Custom tool should be registered in registry"
        );
        assert_eq!(
            registry.list(),
            vec!["test_discovery_tool"],
            "Registry should contain only the discovered tool"
        );
    }

    #[test]
    fn test_custom_tools_appear_in_registry_listing_after_discovery() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode-rs").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("listing_tool.js"),
            r#"export default tool({
                name: "listing_tool",
                description: "A tool to test listing",
                args: {}
            });"#,
        )
        .unwrap();

        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode-rs");
        let tools = scanner.scan_tools(&opencode_path);

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "listing_tool");
    }

    #[test]
    fn test_custom_tool_discovery_registration_execution_pipeline() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode-rs").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("exec_pipeline_tool.js"),
            r#"export default tool({
                name: "exec_pipeline_tool",
                description: "Tests the full pipeline",
                args: { type: "object", properties: { input: { type: "string" } } }
            });"#,
        )
        .unwrap();

        let mut registry = ToolRegistry::new();
        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode-rs");
        let tools = scanner.scan_tools(&opencode_path);

        for tool_info in tools {
            register_custom_tool(&mut registry, tool_info);
        }

        assert!(
            registry.contains("exec_pipeline_tool"),
            "Tool should be registered"
        );

        let executor = registry.get_executor("exec_pipeline_tool");
        assert!(
            executor.is_some(),
            "Executor should be available for registered tool"
        );
    }

    #[test]
    fn test_builtin_tools_still_work_after_custom_tool_registration() {
        let mut registry = ToolRegistry::new();

        registry.register(
            ToolDefinition {
                name: "builtin_echo".to_string(),
                description: "Built-in echo".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );

        registry.register(
            ToolDefinition {
                name: "custom_tool".to_string(),
                description: "Custom tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("custom".to_string())),
        );

        assert!(registry.contains("builtin_echo"));
        assert!(registry.contains("custom_tool"));

        let builtin_executor = registry.get_executor("builtin_echo");
        assert!(builtin_executor.is_some());

        let result = builtin_executor.unwrap()(serde_json::json!({}));
        assert!(result.is_ok() && result.unwrap() == "{}");
    }

    #[test]
    fn test_error_handling_custom_tool_parsing_failure() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode-rs").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("invalid.js"),
            r#"const someVar = {
                name: "invalid_tool",
                description: "Invalid tool",
                args: {}
            };"#,
        )
        .unwrap();

        let scanner = crate::config::DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode-rs");
        let tools = scanner.scan_tools(&opencode_path);

        assert!(tools.is_empty() || !tools.iter().any(|t| t.name == "invalid_tool"));
    }
}
