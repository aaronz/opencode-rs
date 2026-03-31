use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
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
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
        }
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
        self.executors.get(name)
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
        },
        Arc::new(|_args| Err("Web search not yet implemented".to_string())),
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

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();

        registry.register(
            ToolDefinition {
                name: "test".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
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
            },
            Arc::new(|args| Ok(serde_json::to_string(&args).unwrap_or_default())),
        );

        let executor = registry.get_executor("echo").unwrap();
        let result = executor(serde_json::json!({"test": "value"})).unwrap();
        assert!(result.contains("test"));
    }
}
