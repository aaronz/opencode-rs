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
//! - Model-specific tool filtering
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

use crate::config::{DirectoryScanner, ToolInfo};
use crate::session::Session;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(default)]
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
    #[serde(default)]
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: Uuid,
    pub tool_name: String,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl ToolResult {
    pub fn success(tool_name: String, result: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            success: true,
            result: Some(result),
            error: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    pub fn failure(tool_name: String, error: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            success: false,
            result: None,
            error: Some(error),
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub type ToolExecutor = Arc<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>;

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    executors: HashMap<String, ToolExecutor>,
    disabled: HashSet<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
            disabled: HashSet::new(),
        }
    }

    pub fn set_disabled(&mut self, tools: HashSet<String>) {
        self.disabled = tools;
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    pub fn requires_approval(&self, name: &str) -> bool {
        self.tools
            .get(name)
            .map(|def| def.requires_approval)
            .unwrap_or(false)
    }

    pub fn register(&mut self, definition: ToolDefinition, executor: ToolExecutor) {
        let name = definition.name.clone();
        self.tools.insert(name.clone(), definition);
        self.executors.insert(name, executor);
    }

    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    pub fn get_executor(&self, name: &str) -> Option<&ToolExecutor> {
        if self.is_disabled(name) {
            return None;
        }
        self.executors.get(name)
    }

    pub fn get_executor_with_status(&self, name: &str) -> Option<(&ToolExecutor, bool)> {
        self.executors
            .get(name)
            .map(|executor| (executor, self.is_disabled(name)))
    }

    pub fn get_all(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn list_with_status(&self) -> Vec<(&String, bool)> {
        self.tools
            .keys()
            .map(|name| (name, self.is_disabled(name)))
            .collect()
    }
}

fn register_custom_tool(registry: &mut ToolRegistry, tool_info: ToolInfo) {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tool_info.content) else {
        tracing::warn!(
            "Failed to parse tool content for {}: not valid JSON",
            tool_info.name
        );
        return;
    };

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&tool_info.name)
        .to_string();

    let description = parsed
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let args = parsed.get("args");
    let parameters = if let Some(obj) = args.and_then(|v| v.as_object()) {
        let props = obj.get("properties").and_then(|v| v.as_object());
        props
            .map(|properties| {
                properties
                    .iter()
                    .map(|(param_name, param_def)| {
                        let required = obj
                            .get("required")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().any(|r| r.as_str() == Some(param_name)))
                            .unwrap_or(false);

                        ToolParameter {
                            name: param_name.clone(),
                            description: param_def
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            required,
                            schema: param_def.clone(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let file_path = tool_info.path.clone();
    let tool_def = ToolDefinition {
        name: name.clone(),
        description,
        parameters,
        ..Default::default()
    };

    let executor = Arc::new(move |args: serde_json::Value| {
        let args_str =
            serde_json::to_string(&args).map_err(|e| format!("Failed to serialize args: {}", e))?;

        let extension = file_path
            .extension()
            .and_then(|e: &std::ffi::OsStr| e.to_str())
            .ok_or_else(|| "Missing file extension".to_string())?;

        let node_cmd = match extension {
            "js" | "ts" | "mjs" | "cjs" => "node",
            _ => {
                return Err(format!(
                    "Unsupported file extension for tool execution: {}",
                    extension
                ));
            }
        };

        let output = std::process::Command::new(node_cmd)
            .arg(&file_path)
            .arg("--args")
            .arg(&args_str)
            .output()
            .map_err(|e| format!("Failed to execute tool: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    });

    registry.register(tool_def, executor);
}

fn register_discovered_custom_tools(registry: &mut ToolRegistry, project_root: Option<PathBuf>) {
    let scanner = DirectoryScanner::new();

    if let Some(ref root) = project_root {
        let opencode_path = root.join(".opencode");
        if opencode_path.exists() {
            let tools = scanner.scan_tools(&opencode_path);
            for tool_info in tools {
                register_custom_tool(registry, tool_info);
            }
        }
    }

    if let Some(global_path) = dirs::config_dir() {
        let global_opencode = global_path.join("opencode");
        if global_opencode.exists() {
            let tools = scanner.scan_tools(&global_opencode);
            for tool_info in tools {
                register_custom_tool(registry, tool_info);
            }
        }
    }
}

pub fn build_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register(
        ToolDefinition {
            name: "read".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file to read".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: file_path")?;
            std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "file_path".to_string(),
                    description: "Path to the file to write".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "content".to_string(),
                    description: "Content to write to the file".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: file_path")?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: content")?;

            if let Some(parent) = std::path::Path::new(file_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directories: {}", e))?;
            }
            std::fs::write(file_path, content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            Ok(format!("Successfully wrote to {}", file_path))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search for patterns in files".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "pattern".to_string(),
                    description: "Regex pattern to search for".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "path".to_string(),
                    description: "Directory or file to search in".to_string(),
                    required: false,
                    schema: serde_json::json!({ "type": "string" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: pattern")?;
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

            let mut matches = Vec::new();

            fn search_dir(
                dir: &std::path::Path,
                pattern: &str,
                matches: &mut Vec<String>,
            ) -> Result<(), String> {
                if !dir.exists() {
                    return Ok(());
                }

                let entries = std::fs::read_dir(dir)
                    .map_err(|e| format!("Failed to read directory: {}", e))?;

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if !name.starts_with('.') && name != "target" && name != "node_modules"
                            {
                                search_dir(&path, pattern, matches)?;
                            }
                        }
                    } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ["rs", "ts", "tsx", "js", "jsx", "py", "go", "java"].contains(&ext) {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                for (i, line) in content.lines().enumerate() {
                                    if line.contains(pattern) {
                                        matches.push(format!(
                                            "{}:{}: {}",
                                            path.display(),
                                            i + 1,
                                            line
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }

            search_dir(std::path::Path::new(path), pattern, &mut matches)?;

            if matches.is_empty() {
                Ok("No matches found".to_string())
            } else {
                Ok(matches.join("\n"))
            }
        }),
    );

    registry.register(
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: vec![ToolParameter {
                name: "command".to_string(),
                description: "Command to execute".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: command")?;

            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(stdout.to_string())
            } else {
                Err(format!("Command failed: {}\n{}", stdout, stderr))
            }
        }),
    );

    registry.register(
        ToolDefinition {
            name: "websearch".to_string(),
            description: "Search the web for information".to_string(),
            parameters: vec![ToolParameter {
                name: "query".to_string(),
                description: "Search query".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|_args| Err("Web search not yet implemented".to_string())),
    );

    registry.register(
        ToolDefinition {
            name: "session_load".to_string(),
            description: "Load a session from local storage".to_string(),
            parameters: vec![ToolParameter {
                name: "session_id".to_string(),
                description: "Session ID to load".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let session_id = args
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: session_id")?;
            let id = uuid::Uuid::parse_str(session_id)
                .map_err(|e| format!("Invalid session_id: {}", e))?;
            let session = Session::load_by_id(&id).map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&session)
                .map_err(|e| format!("Failed to serialize session: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "session_save".to_string(),
            description: "Save current session to local storage".to_string(),
            parameters: vec![ToolParameter {
                name: "session_id".to_string(),
                description: "Optional session ID to persist".to_string(),
                required: false,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let session_id = args.get("session_id").and_then(|v| v.as_str());
            let id = match session_id {
                Some(raw) => {
                    uuid::Uuid::parse_str(raw).map_err(|e| format!("Invalid session_id: {}", e))?
                }
                None => uuid::Uuid::new_v4(),
            };

            let path = Session::session_path(&id);
            let mut session = if path.exists() {
                Session::load(&path).map_err(|e| e.to_string())?
            } else {
                let mut s = Session::new();
                s.id = id;
                s
            };

            session.id = id;
            session.save(&path).map_err(|e| e.to_string())?;

            serde_json::to_string(&serde_json::json!({
                "session_id": id.to_string(),
                "saved": true,
            }))
            .map_err(|e| format!("Failed to serialize response: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "stat".to_string(),
            description: "Get file or directory metadata".to_string(),
            parameters: vec![ToolParameter {
                name: "path".to_string(),
                description: "Path to the file or directory".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: path")?;
            let metadata =
                std::fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;
            let result = serde_json::json!({
                "size": metadata.len(),
                "is_file": metadata.is_file(),
                "is_dir": metadata.is_dir(),
                "readonly": metadata.permissions().readonly(),
            });
            serde_json::to_string_pretty(&result).map_err(|e| format!("Failed to serialize: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "move".to_string(),
            description: "Move or rename a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "source".to_string(),
                    description: "Source path".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "destination".to_string(),
                    description: "Destination path".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let source = args
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: source")?;
            let destination = args
                .get("destination")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: destination")?;
            std::fs::rename(source, destination).map_err(|e| format!("Failed to move: {}", e))?;
            Ok(format!("Moved {} to {}", source, destination))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "delete".to_string(),
            description: "Delete a file or directory".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "Path to delete".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "recursive".to_string(),
                    description: "Delete directories recursively".to_string(),
                    required: false,
                    schema: serde_json::json!({ "type": "boolean" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: path")?;
            let recursive = args
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let metadata =
                std::fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;
            if metadata.is_dir() {
                if recursive {
                    std::fs::remove_dir_all(path)
                        .map_err(|e| format!("Failed to delete directory: {}", e))?;
                } else {
                    std::fs::remove_dir(path)
                        .map_err(|e| format!("Failed to delete directory: {}", e))?;
                }
            } else {
                std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
            }
            Ok(format!("Deleted {}", path))
        }),
    );

    let project_root = find_project_root();
    register_discovered_custom_tools(&mut registry, project_root);

    registry
}

fn find_project_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        let opencode_dir = ancestor.join(".opencode");
        if opencode_dir.exists() && opencode_dir.is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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

        let listed = registry.list_with_status();
        let entry = listed
            .into_iter()
            .find(|(name, _)| *name == "echo")
            .expect("echo should be listed");
        assert!(entry.1);
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
        let listed = registry.list_with_status();
        let entry = listed
            .into_iter()
            .find(|(name, _)| *name == "echo")
            .expect("echo should be listed");
        assert!(!entry.1);
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

        let _ = Session::delete(&session_id);
    }

    #[test]
    fn test_directory_scanner_registers_discovered_tools_with_registry() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode").join("tools");
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
        let opencode_path = temp_dir.path().join(".opencode");
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
        let tools_dir = temp_dir.path().join(".opencode").join("tools");
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

        let mut registry = ToolRegistry::new();
        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode");
        let tools = scanner.scan_tools(&opencode_path);

        for tool_info in tools {
            register_custom_tool(&mut registry, tool_info);
        }

        let listed = registry.list();
        assert!(
            listed.contains(&"listing_tool".to_string()),
            "Discovered tool should appear in listing. Found: {:?}",
            listed
        );
    }

    #[test]
    fn test_custom_tool_discovery_registration_execution_pipeline() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("exec_tool.js"),
            r#"export default tool({
                name: "exec_pipeline_tool",
                description: "Test pipeline",
                args: { type: "object", properties: { message: { type: "string" } } }
            });
            const argsIdx = process.argv.findIndex(a => a === '--args');
            const args = argsIdx >= 0 ? JSON.parse(process.argv[argsIdx + 1] || '{}') : {};
            console.log(args.message || 'default');
            "#,
        )
        .unwrap();

        let mut registry = ToolRegistry::new();
        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode");
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
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("custom.js"),
            r#"export default tool({
                name: "custom_tool",
                description: "A custom tool",
                args: {}
            });"#,
        )
        .unwrap();

        let mut registry = ToolRegistry::new();

        registry.register(
            ToolDefinition {
                name: "builtin_tool".to_string(),
                description: "A built-in tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("builtin result".to_string())),
        );

        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode");
        let tools = scanner.scan_tools(&opencode_path);

        for tool_info in tools {
            register_custom_tool(&mut registry, tool_info);
        }

        assert!(
            registry.contains("builtin_tool"),
            "Built-in tool should still be in registry"
        );
        assert!(
            registry.contains("custom_tool"),
            "Custom tool should be registered"
        );

        let builtin_executor = registry.get_executor("builtin_tool");
        assert!(
            builtin_executor.is_some(),
            "Built-in tool executor should still work"
        );

        let result = builtin_executor.unwrap()(serde_json::json!({}));
        assert!(
            result.is_ok() && result.unwrap() == "builtin result",
            "Built-in tool should execute correctly"
        );
    }

    #[test]
    fn test_error_handling_custom_tool_parsing_failure() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode").join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        fs::write(
            tools_dir.join("invalid.js"),
            r#"const someVar = {
                name: "invalid_tool",
                description: "Invalid tool - no export default"
            };
            export default someVar;
            "#,
        )
        .unwrap();

        let mut registry = ToolRegistry::new();
        let scanner = DirectoryScanner::new();
        let opencode_path = temp_dir.path().join(".opencode");
        let tools = scanner.scan_tools(&opencode_path);

        assert_eq!(
            tools.len(),
            0,
            "Tool without proper export pattern should not be discovered"
        );

        for tool_info in tools {
            register_custom_tool(&mut registry, tool_info);
        }

        assert!(
            !registry.contains("invalid_tool"),
            "Invalid tool should not be registered"
        );
    }
}
